use crate::reader::DataFrame;
use crate::stats;

/// Generate an ASCII histogram for a numeric column.
pub fn histogram(df: &DataFrame, col_name: &str, width: usize, height: usize) -> Option<String> {
    let mut values = df.valid_numeric_column(col_name)?;
    if values.is_empty() {
        return Some(format!("{}: No valid numeric data", col_name));
    }

    values.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));

    let n = values.len();
    let min_val = values[0];
    let max_val = *values.last().unwrap();
    let m = stats::mean(&values);
    let med = stats::percentile(&values, 50.0);
    let sd = stats::std_dev(&values);

    // Number of bins using Sturges' rule
    let num_bins = if n > 1 {
        ((n as f64).log2().ceil() as usize + 1).max(5).min(width / 2)
    } else {
        1
    };

    let range = max_val - min_val;
    let bin_width = if range > 0.0 {
        range / num_bins as f64
    } else {
        1.0
    };

    // Count values per bin
    let mut bins = vec![0usize; num_bins];
    for &v in &values {
        let mut idx = ((v - min_val) / bin_width).floor() as usize;
        if idx >= num_bins {
            idx = num_bins - 1;
        }
        bins[idx] += 1;
    }

    let max_count = *bins.iter().max().unwrap_or(&1);

    let mut output = String::new();
    output.push_str(&format!(
        "{}: Distribution (n={})\n\n",
        col_name, n
    ));

    // Draw histogram vertically
    let bar_height = height.min(15);
    for row in (0..bar_height).rev() {
        let threshold = (row as f64 + 0.5) / bar_height as f64 * max_count as f64;
        let label = if row == bar_height - 1 {
            format!("{:>4}", max_count)
        } else if row == 0 {
            format!("{:>4}", 0)
        } else if row == bar_height / 2 {
            format!("{:>4}", max_count / 2)
        } else {
            "    ".to_string()
        };
        output.push_str(&label);
        output.push('|');

        for &count in &bins {
            if count as f64 >= threshold {
                output.push_str("██");
            } else if count as f64 >= threshold - (max_count as f64 / bar_height as f64 / 2.0) {
                output.push_str("▄▄");
            } else {
                output.push_str("  ");
            }
        }
        output.push('\n');
    }

    // X axis
    output.push_str("    └");
    for _ in 0..num_bins {
        output.push_str("──");
    }
    output.push('\n');

    // X axis labels
    output.push_str("     ");
    let label_step = (num_bins / 5).max(1);
    for i in 0..num_bins {
        if i % label_step == 0 {
            let val = min_val + i as f64 * bin_width;
            let label = format_number_short(val);
            output.push_str(&label);
            // Pad to align
            let pad = 2usize.saturating_sub(label.len().saturating_sub(2));
            for _ in 0..pad {
                output.push(' ');
            }
        } else {
            output.push_str("  ");
        }
    }
    output.push('\n');

    output.push('\n');
    output.push_str(&format!(
        "Mean: {:.2} | Median: {:.2} | Std: {:.2}",
        m, med, sd
    ));

    Some(output)
}

/// Generate an ASCII boxplot for a numeric column.
pub fn boxplot(df: &DataFrame, col_name: &str, width: usize) -> Option<String> {
    let mut values = df.valid_numeric_column(col_name)?;
    if values.is_empty() {
        return Some(format!("{}: No valid numeric data", col_name));
    }

    values.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));

    let min_val = values[0];
    let max_val = *values.last().unwrap();
    let q1 = stats::percentile(&values, 25.0);
    let med = stats::percentile(&values, 50.0);
    let q3 = stats::percentile(&values, 75.0);
    let iqr = q3 - q1;

    // Whiskers (capped at 1.5 * IQR)
    let lower_whisker = values
        .iter()
        .copied()
        .find(|&v| v >= q1 - 1.5 * iqr)
        .unwrap_or(min_val);
    let upper_whisker = values
        .iter()
        .rev()
        .copied()
        .find(|&v| v <= q3 + 1.5 * iqr)
        .unwrap_or(max_val);

    // Outliers
    let outliers: Vec<f64> = values
        .iter()
        .copied()
        .filter(|&v| v < lower_whisker || v > upper_whisker)
        .collect();

    let range = max_val - min_val;
    let plot_width = width.min(60).max(20);

    let scale = |v: f64| -> usize {
        if range == 0.0 {
            plot_width / 2
        } else {
            ((v - min_val) / range * (plot_width - 1) as f64).round() as usize
        }
    };

    let mut output = String::new();
    output.push_str(&format!("{}: Boxplot (n={})\n\n", col_name, values.len()));

    // Top line with outliers
    let mut line1 = vec![' '; plot_width];
    for &o in &outliers {
        let pos = scale(o);
        if pos < plot_width {
            line1[pos] = 'o';
        }
    }
    output.push_str("  ");
    output.extend(line1.iter());
    output.push('\n');

    // Boxplot line
    let lw = scale(lower_whisker);
    let uq1 = scale(q1);
    let um = scale(med);
    let uq3 = scale(q3);
    let uw = scale(upper_whisker);

    let mut line2 = vec![' '; plot_width];
    // Whisker lines
    for i in lw..=uw {
        if i < plot_width {
            line2[i] = '─';
        }
    }
    // Box
    for i in uq1..=uq3 {
        if i < plot_width {
            line2[i] = '█';
        }
    }
    // Median
    if um < plot_width {
        line2[um] = '│';
    }
    // Whisker ends
    if lw < plot_width {
        line2[lw] = '├';
    }
    if uw < plot_width {
        line2[uw] = '┤';
    }

    output.push_str("  ");
    output.extend(line2.iter());
    output.push('\n');

    // Scale line
    output.push_str("  ");
    let scale_line = vec!['─'; plot_width];
    output.extend(scale_line.iter());
    output.push('\n');

    // Labels
    output.push_str(&format!(
        "  {:<width$}{}\n",
        format_number_short(min_val),
        format_number_short(max_val),
        width = plot_width - format_number_short(max_val).len()
    ));

    output.push('\n');
    output.push_str(&format!(
        "Min: {:.2} | Q1: {:.2} | Median: {:.2} | Q3: {:.2} | Max: {:.2}",
        min_val, q1, med, q3, max_val
    ));

    if !outliers.is_empty() {
        output.push_str(&format!("\nOutliers: {} values", outliers.len()));
    }

    Some(output)
}

/// Generate an ASCII scatter plot for two numeric columns.
///
/// Uses a pre-computed density map (HashMap) for O(n) point placement instead of
/// the naive O(n*m) approach of recounting density per cell for every point.
pub fn scatter(
    df: &DataFrame,
    x_name: &str,
    y_name: &str,
    width: usize,
    height: usize,
) -> Option<String> {
    let x_all = df.numeric_column(x_name)?;
    let y_all = df.numeric_column(y_name)?;

    // Use only complete pairs
    let pairs: Vec<(f64, f64)> = x_all
        .iter()
        .zip(y_all.iter())
        .filter_map(|(a, b)| match (a, b) {
            (Some(x), Some(y)) => Some((*x, *y)),
            _ => None,
        })
        .collect();

    if pairs.is_empty() {
        return Some(format!(
            "{} vs {}: No complete pairs of data",
            x_name, y_name
        ));
    }

    let x_vals: Vec<f64> = pairs.iter().map(|(x, _)| *x).collect();
    let y_vals: Vec<f64> = pairs.iter().map(|(_, y)| *y).collect();

    let x_min = x_vals.iter().copied().fold(f64::INFINITY, f64::min);
    let x_max = x_vals.iter().copied().fold(f64::NEG_INFINITY, f64::max);
    let y_min = y_vals.iter().copied().fold(f64::INFINITY, f64::min);
    let y_max = y_vals.iter().copied().fold(f64::NEG_INFINITY, f64::max);

    let plot_w = width.min(60).max(20);
    let plot_h = height.min(20).max(8);

    let x_range = if x_max == x_min { 1.0 } else { x_max - x_min };
    let y_range = if y_max == y_min { 1.0 } else { y_max - y_min };

    // Pre-compute density map in a single O(n) pass
    let mut density: std::collections::HashMap<(usize, usize), usize> =
        std::collections::HashMap::new();
    for (x, y) in &pairs {
        let col = ((*x - x_min) / x_range * (plot_w - 1) as f64).round() as usize;
        let row = ((y_max - *y) / y_range * (plot_h - 1) as f64).round() as usize;
        let col = col.min(plot_w - 1);
        let row = row.min(plot_h - 1);
        *density.entry((row, col)).or_insert(0) += 1;
    }

    // Build grid from density map
    let mut grid = vec![vec![' '; plot_w]; plot_h];
    for (&(row, col), &count) in &density {
        grid[row][col] = if count > 3 {
            '●'
        } else if count > 1 {
            '◦'
        } else {
            '·'
        };
    }

    let mut output = String::new();
    output.push_str(&format!(
        "{} vs {} (n={})\n\n",
        y_name,
        x_name,
        pairs.len()
    ));

    for (i, row) in grid.iter().enumerate() {
        let y_val = y_max - (i as f64 / (plot_h - 1) as f64) * y_range;
        if i == 0 || i == plot_h - 1 || i == plot_h / 2 {
            output.push_str(&format!("{:>8.1}│", y_val));
        } else {
            output.push_str("        │");
        }
        output.extend(row.iter());
        output.push('\n');
    }

    output.push_str("        └");
    for _ in 0..plot_w {
        output.push('─');
    }
    output.push('\n');
    output.push_str(&format!(
        "         {:<width$}{:.1}\n",
        format!("{:.1}", x_min),
        x_max,
        width = plot_w - format!("{:.1}", x_max).len()
    ));
    output.push_str(&format!("         {:^width$}\n", x_name, width = plot_w));

    Some(output)
}

/// Format a number in short form.
fn format_number_short(val: f64) -> String {
    if val.abs() >= 1_000_000.0 {
        format!("{:.1}M", val / 1_000_000.0)
    } else if val.abs() >= 1_000.0 {
        format!("{:.1}k", val / 1_000.0)
    } else if val.fract() == 0.0 && val.abs() < 10000.0 {
        format!("{:.0}", val)
    } else {
        format!("{:.1}", val)
    }
}
