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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::reader;

    #[test]
    fn test_perfect_positive_correlation() {
        // x and y are identical => r = 1.0
        let x: Vec<Option<f64>> = vec![Some(1.0), Some(2.0), Some(3.0), Some(4.0), Some(5.0)];
        let y: Vec<Option<f64>> = vec![Some(1.0), Some(2.0), Some(3.0), Some(4.0), Some(5.0)];
        let r = pearson_correlation(&x, &y);
        assert!((r - 1.0).abs() < 1e-10);
    }

    #[test]
    fn test_perfect_negative_correlation() {
        // x goes up, y goes down => r = -1.0
        let x: Vec<Option<f64>> = vec![Some(1.0), Some(2.0), Some(3.0), Some(4.0), Some(5.0)];
        let y: Vec<Option<f64>> = vec![Some(5.0), Some(4.0), Some(3.0), Some(2.0), Some(1.0)];
        let r = pearson_correlation(&x, &y);
        assert!((r - (-1.0)).abs() < 1e-10);
    }

    #[test]
    fn test_zero_correlation() {
        // Orthogonal pattern: no linear relationship
        let x: Vec<Option<f64>> = vec![Some(1.0), Some(0.0), Some(-1.0), Some(0.0)];
        let y: Vec<Option<f64>> = vec![Some(0.0), Some(1.0), Some(0.0), Some(-1.0)];
        let r = pearson_correlation(&x, &y);
        assert!(r.abs() < 1e-10);
    }

    #[test]
    fn test_correlation_with_missing() {
        // Only complete pairs should be used
        let x: Vec<Option<f64>> = vec![Some(1.0), None, Some(3.0), Some(4.0), Some(5.0)];
        let y: Vec<Option<f64>> = vec![Some(2.0), Some(4.0), None, Some(8.0), Some(10.0)];
        // Complete pairs: (1,2), (4,8), (5,10) => perfect correlation
        let r = pearson_correlation(&x, &y);
        assert!((r - 1.0).abs() < 1e-10);
    }

    #[test]
    fn test_correlation_too_few_pairs() {
        let x: Vec<Option<f64>> = vec![Some(1.0), None, None];
        let y: Vec<Option<f64>> = vec![None, Some(2.0), None];
        // 0 complete pairs
        let r = pearson_correlation(&x, &y);
        assert!(r.is_nan());
    }

    #[test]
    fn test_correlation_constant_variable() {
        // One variable is constant => r is NaN (zero variance)
        let x: Vec<Option<f64>> = vec![Some(1.0), Some(2.0), Some(3.0)];
        let y: Vec<Option<f64>> = vec![Some(5.0), Some(5.0), Some(5.0)];
        let r = pearson_correlation(&x, &y);
        assert!(r.is_nan());
    }

    #[test]
    fn test_correlation_known_value() {
        // Known dataset: x = [1,2,3,4,5], y = [2,4,5,4,5]
        // Expected r ~ 0.7746
        let x: Vec<Option<f64>> = vec![Some(1.0), Some(2.0), Some(3.0), Some(4.0), Some(5.0)];
        let y: Vec<Option<f64>> = vec![Some(2.0), Some(4.0), Some(5.0), Some(4.0), Some(5.0)];
        let r = pearson_correlation(&x, &y);
        assert!((r - 0.7746).abs() < 0.01);
    }

    #[test]
    fn test_diagonal_is_one() {
        let df = reader::read_file("tests/data/sample.csv").unwrap();
        let cm = correlation_matrix(&df, None);
        for i in 0..cm.columns.len() {
            assert!((cm.matrix[i][i] - 1.0).abs() < 1e-10);
        }
    }

    #[test]
    fn test_matrix_is_symmetric() {
        let df = reader::read_file("tests/data/sample.csv").unwrap();
        let cm = correlation_matrix(&df, None);
        let n = cm.columns.len();
        for i in 0..n {
            for j in 0..n {
                let diff = (cm.matrix[i][j] - cm.matrix[j][i]).abs();
                assert!(diff < 1e-10 || (cm.matrix[i][j].is_nan() && cm.matrix[j][i].is_nan()));
            }
        }
    }

    #[test]
    fn test_selected_columns() {
        let df = reader::read_file("tests/data/sample.csv").unwrap();
        let cm = correlation_matrix(&df, Some(&["age", "income"]));
        assert_eq!(cm.columns.len(), 2);
        assert_eq!(cm.columns[0], "age");
        assert_eq!(cm.columns[1], "income");
    }

    #[test]
    fn test_high_correlations_filter() {
        let df = reader::read_file("tests/data/sample.csv").unwrap();
        let cm = correlation_matrix(&df, None);

        // With threshold 0.0, should find some pairs
        let high = high_correlations(&cm, 0.0);
        assert!(!high.is_empty());

        // With threshold 1.0, should find none (no perfect correlations between different cols)
        let perfect = high_correlations(&cm, 1.0);
        assert!(perfect.is_empty());
    }

    #[test]
    fn test_high_correlations_sorted_by_abs() {
        let df = reader::read_file("tests/data/sample.csv").unwrap();
        let cm = correlation_matrix(&df, None);
        let high = high_correlations(&cm, 0.0);
        // Verify sorted by descending absolute value
        for i in 1..high.len() {
            assert!(high[i - 1].2.abs() >= high[i].2.abs());
        }
    }

    #[test]
    fn test_nonexistent_column_filtered() {
        let df = reader::read_file("tests/data/sample.csv").unwrap();
        let cm = correlation_matrix(&df, Some(&["age", "nonexistent_col"]));
        // Only "age" should remain
        assert_eq!(cm.columns.len(), 1);
        assert_eq!(cm.columns[0], "age");
    }
}
