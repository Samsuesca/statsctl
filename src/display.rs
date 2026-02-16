use crate::correlation::CorrelationMatrix;
use crate::missing::{MissingInfo, MissingPatternReport};
use crate::stats::{CategoricalSummary, DescriptiveStats};
use crate::types::ColumnTypeInfo;
use colored::Colorize;
use tabled::{builder::Builder, settings::Style};

/// Format descriptive statistics as a table.
pub fn format_summary(stats: &[DescriptiveStats]) -> String {
    let mut builder = Builder::new();
    builder.push_record([
        "Variable", "Count", "Mean", "Std", "Min", "Q1", "Median", "Q3", "Max",
    ]);

    for s in stats {
        builder.push_record([
            s.name.clone(),
            s.count.to_string(),
            format_f64(s.mean),
            format_f64(s.std_dev),
            format_f64(s.min),
            format_f64(s.q1),
            format_f64(s.median),
            format_f64(s.q3),
            format_f64(s.max),
        ]);
    }

    let table = builder.build().with(Style::rounded()).to_string();
    table
}

/// Format categorical summaries as a table.
pub fn format_categorical(summaries: &[CategoricalSummary]) -> String {
    let mut builder = Builder::new();
    builder.push_record(["Variable", "Total", "Missing", "Unique", "Top Values"]);

    for s in summaries {
        let top: String = s
            .top_values
            .iter()
            .take(5)
            .map(|(v, c)| format!("{} ({})", v, c))
            .collect::<Vec<_>>()
            .join(", ");

        builder.push_record([
            s.name.clone(),
            s.total.to_string(),
            s.missing.to_string(),
            s.unique.to_string(),
            top,
        ]);
    }

    builder.build().with(Style::rounded()).to_string()
}

/// Format missing data report as a table.
pub fn format_missing(infos: &[MissingInfo]) -> String {
    let mut builder = Builder::new();
    builder.push_record(["Variable", "Missing", "% Missing"]);

    for info in infos {
        builder.push_record([
            info.name.clone(),
            info.missing.to_string(),
            format!("{:.2}%", info.pct),
        ]);
    }

    let mut output = "Missing Data Report:\n".to_string();
    output.push_str(&builder.build().with(Style::rounded()).to_string());
    output
}

/// Format missing data pattern report.
pub fn format_missing_patterns(report: &MissingPatternReport) -> String {
    let mut output = String::new();

    if report.rows_with_missing == 0 {
        output.push_str("No missing data found.\n");
        return output;
    }

    output.push_str(&format!(
        "\n{:.2}% of observations ({}/{}) have at least one missing value\n",
        report.pct_with_missing, report.rows_with_missing, report.total_rows
    ));

    if !report.patterns.is_empty() {
        output.push_str("\nMost common missing patterns:\n");
        let mut builder = Builder::new();
        builder.push_record(["Missing Columns", "Count"]);
        for (cols, count) in &report.patterns {
            builder.push_record([cols.join(", "), count.to_string()]);
        }
        output.push_str(&builder.build().with(Style::rounded()).to_string());
    }

    output
}

/// Format a correlation matrix.
pub fn format_correlation(cm: &CorrelationMatrix) -> String {
    let mut output = "Correlation Matrix (Pearson):\n".to_string();

    // Header row
    let col_width = 8;
    output.push_str(&format!("{:>width$}", "", width = col_width + 1));
    for col in &cm.columns {
        let name = if col.len() > col_width {
            &col[..col_width]
        } else {
            col
        };
        output.push_str(&format!("{:>width$}", name, width = col_width));
    }
    output.push('\n');

    // Data rows
    for (i, row_name) in cm.columns.iter().enumerate() {
        let name = if row_name.len() > col_width {
            &row_name[..col_width]
        } else {
            row_name
        };
        output.push_str(&format!("{:>width$} ", name, width = col_width));
        for j in 0..cm.columns.len() {
            let val = cm.matrix[i][j];
            let formatted = if val.is_nan() {
                format!("{:>width$}", "NaN", width = col_width)
            } else {
                format!("{:>width$.2}", val, width = col_width)
            };

            // Color high correlations
            if i != j && !val.is_nan() {
                if val.abs() >= 0.7 {
                    output.push_str(&formatted.red().bold().to_string());
                } else if val.abs() >= 0.5 {
                    output.push_str(&formatted.yellow().to_string());
                } else {
                    output.push_str(&formatted);
                }
            } else {
                output.push_str(&formatted);
            }
        }
        output.push('\n');
    }

    output
}

/// Format high correlation warnings.
pub fn format_high_correlations(pairs: &[(String, String, f64)], threshold: f64) -> String {
    if pairs.is_empty() {
        return String::new();
    }

    let mut output = format!(
        "\nHigh correlations (|r| > {:.1}):\n",
        threshold
    );

    for (a, b, r) in pairs {
        let arrow = if *r > 0.0 { "↔" } else { "↔" };
        output.push_str(&format!("  - {} {} {}: {:.2}\n", a, arrow, b, r));
    }

    output
}

/// Format column type information as a table.
pub fn format_types(infos: &[ColumnTypeInfo], show_levels: bool) -> String {
    let mut builder = Builder::new();

    if show_levels {
        builder.push_record(["Variable", "Type", "Unique", "Levels"]);
    } else {
        builder.push_record(["Variable", "Type", "Unique"]);
    }

    for info in infos {
        let levels_str = info.levels.join(", ");
        if show_levels {
            builder.push_record([
                info.name.clone(),
                info.col_type.to_string(),
                info.unique_count.to_string(),
                levels_str,
            ]);
        } else {
            builder.push_record([
                info.name.clone(),
                info.col_type.to_string(),
                info.unique_count.to_string(),
            ]);
        }
    }

    let mut output = "Data Types:\n".to_string();
    output.push_str(&builder.build().with(Style::rounded()).to_string());
    output
}

/// Format a comparison between two datasets.
pub fn format_comparison(
    stats1: &[DescriptiveStats],
    stats2: &[DescriptiveStats],
    label1: &str,
    label2: &str,
) -> String {
    let mut builder = Builder::new();
    builder.push_record([
        "Variable",
        &format!("{} Count", label1),
        &format!("{} Count", label2),
        &format!("{} Mean", label1),
        &format!("{} Mean", label2),
        "Diff Mean",
        &format!("{} Std", label1),
        &format!("{} Std", label2),
    ]);

    for s1 in stats1 {
        if let Some(s2) = stats2.iter().find(|s| s.name == s1.name) {
            let diff = if s1.mean.is_nan() || s2.mean.is_nan() {
                "NaN".to_string()
            } else {
                format_f64(s2.mean - s1.mean)
            };

            builder.push_record([
                s1.name.clone(),
                s1.count.to_string(),
                s2.count.to_string(),
                format_f64(s1.mean),
                format_f64(s2.mean),
                diff,
                format_f64(s1.std_dev),
                format_f64(s2.std_dev),
            ]);
        }
    }

    let mut output = format!("Comparison: {} vs {}\n", label1, label2);
    output.push_str(&builder.build().with(Style::rounded()).to_string());
    output
}

/// Format a float for display (reasonable precision).
fn format_f64(val: f64) -> String {
    if val.is_nan() {
        "NaN".to_string()
    } else if val.is_infinite() {
        if val > 0.0 {
            "Inf".to_string()
        } else {
            "-Inf".to_string()
        }
    } else if val.abs() >= 1_000_000.0 {
        format!("{:.2}", val)
    } else if val.abs() >= 100.0 {
        format!("{:.2}", val)
    } else if val.abs() >= 1.0 {
        format!("{:.2}", val)
    } else if val == 0.0 {
        "0.00".to_string()
    } else {
        format!("{:.4}", val)
    }
}

/// Convert output to a specific format for export.
pub fn export_output(content: &str, format: &str) -> String {
    match format {
        "json" => {
            // Wrap as a simple JSON object
            serde_json::json!({ "output": content }).to_string()
        }
        "csv" => {
            // For CSV export, keep as-is (tables are already structured)
            content.to_string()
        }
        _ => {
            // Markdown / plain text
            content.to_string()
        }
    }
}
