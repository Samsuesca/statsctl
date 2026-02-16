use crate::reader::DataFrame;
use crate::stats;
use crate::types;

/// A correlation matrix result.
#[derive(Debug, Clone)]
pub struct CorrelationMatrix {
    pub columns: Vec<String>,
    pub matrix: Vec<Vec<f64>>,
}

/// Compute Pearson correlation between two slices.
/// Both slices must have the same length. Uses pairwise complete observations.
fn pearson_correlation(x_all: &[Option<f64>], y_all: &[Option<f64>]) -> f64 {
    assert_eq!(x_all.len(), y_all.len());

    // Use only observations where both x and y are present
    let pairs: Vec<(f64, f64)> = x_all
        .iter()
        .zip(y_all.iter())
        .filter_map(|(a, b)| match (a, b) {
            (Some(x), Some(y)) => Some((*x, *y)),
            _ => None,
        })
        .collect();

    let n = pairs.len();
    if n < 2 {
        return f64::NAN;
    }

    let x_vals: Vec<f64> = pairs.iter().map(|(x, _)| *x).collect();
    let y_vals: Vec<f64> = pairs.iter().map(|(_, y)| *y).collect();

    let mean_x = stats::mean(&x_vals);
    let mean_y = stats::mean(&y_vals);

    let mut cov = 0.0;
    let mut var_x = 0.0;
    let mut var_y = 0.0;

    for (x, y) in &pairs {
        let dx = x - mean_x;
        let dy = y - mean_y;
        cov += dx * dy;
        var_x += dx * dx;
        var_y += dy * dy;
    }

    if var_x == 0.0 || var_y == 0.0 {
        return f64::NAN;
    }

    cov / (var_x.sqrt() * var_y.sqrt())
}

/// Compute the correlation matrix for all numeric columns.
pub fn correlation_matrix(df: &DataFrame, columns: Option<&[&str]>) -> CorrelationMatrix {
    let col_names: Vec<String> = match columns {
        Some(cols) => cols
            .iter()
            .filter(|c| {
                // Verify the column exists and is numeric
                df.numeric_column(c).is_some()
            })
            .map(|c| c.to_string())
            .collect(),
        None => types::numeric_columns(df),
    };

    let n = col_names.len();
    let mut matrix = vec![vec![0.0f64; n]; n];

    // Pre-compute numeric columns (with Option values for pairwise completeness)
    let data: Vec<Vec<Option<f64>>> = col_names
        .iter()
        .map(|col| df.numeric_column(col).unwrap_or_default())
        .collect();

    for i in 0..n {
        matrix[i][i] = 1.0;
        for j in (i + 1)..n {
            let r = pearson_correlation(&data[i], &data[j]);
            matrix[i][j] = r;
            matrix[j][i] = r;
        }
    }

    CorrelationMatrix {
        columns: col_names,
        matrix,
    }
}

/// Find high correlations above a threshold.
pub fn high_correlations(cm: &CorrelationMatrix, threshold: f64) -> Vec<(String, String, f64)> {
    let mut result = Vec::new();
    let n = cm.columns.len();

    for i in 0..n {
        for j in (i + 1)..n {
            let r = cm.matrix[i][j];
            if r.abs() >= threshold && !r.is_nan() {
                result.push((cm.columns[i].clone(), cm.columns[j].clone(), r));
            }
        }
    }

    result.sort_by(|a, b| b.2.abs().partial_cmp(&a.2.abs()).unwrap_or(std::cmp::Ordering::Equal));
    result
}
