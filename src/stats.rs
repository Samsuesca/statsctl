use crate::reader::DataFrame;
use crate::types;
use crate::utils::is_missing;

/// Descriptive statistics for a single numeric column.
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct DescriptiveStats {
    pub name: String,
    pub count: usize,
    pub missing: usize,
    pub mean: f64,
    pub std_dev: f64,
    pub min: f64,
    pub q1: f64,
    pub median: f64,
    pub q3: f64,
    pub max: f64,
}

/// Compute the mean of a slice.
pub fn mean(data: &[f64]) -> f64 {
    if data.is_empty() {
        return f64::NAN;
    }
    data.iter().sum::<f64>() / data.len() as f64
}

/// Compute the sample standard deviation.
pub fn std_dev(data: &[f64]) -> f64 {
    if data.len() < 2 {
        return 0.0;
    }
    let m = mean(data);
    let variance = data.iter().map(|x| (x - m).powi(2)).sum::<f64>() / (data.len() - 1) as f64;
    variance.sqrt()
}

/// Compute a percentile using linear interpolation.
pub fn percentile(sorted: &[f64], p: f64) -> f64 {
    if sorted.is_empty() {
        return f64::NAN;
    }
    if sorted.len() == 1 {
        return sorted[0];
    }
    let n = sorted.len() as f64;
    let index = p / 100.0 * (n - 1.0);
    let lower = index.floor() as usize;
    let upper = index.ceil() as usize;
    if lower == upper {
        sorted[lower]
    } else {
        let frac = index - lower as f64;
        sorted[lower] * (1.0 - frac) + sorted[upper] * frac
    }
}

/// Compute descriptive statistics for a column.
pub fn describe(df: &DataFrame, col_name: &str) -> Option<DescriptiveStats> {
    let all_values = df.numeric_column(col_name)?;
    let missing = all_values.iter().filter(|v| v.is_none()).count();
    let mut values: Vec<f64> = all_values.into_iter().flatten().collect();

    if values.is_empty() {
        return Some(DescriptiveStats {
            name: col_name.to_string(),
            count: 0,
            missing,
            mean: f64::NAN,
            std_dev: f64::NAN,
            min: f64::NAN,
            q1: f64::NAN,
            median: f64::NAN,
            q3: f64::NAN,
            max: f64::NAN,
        });
    }

    values.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));

    Some(DescriptiveStats {
        name: col_name.to_string(),
        count: values.len(),
        missing,
        mean: mean(&values),
        std_dev: std_dev(&values),
        min: values[0],
        q1: percentile(&values, 25.0),
        median: percentile(&values, 50.0),
        q3: percentile(&values, 75.0),
        max: *values.last().unwrap(),
    })
}

/// Compute descriptive statistics for all numeric columns.
pub fn describe_all(df: &DataFrame) -> Vec<DescriptiveStats> {
    let numeric_cols = types::numeric_columns(df);
    numeric_cols
        .iter()
        .filter_map(|col| describe(df, col))
        .collect()
}

/// Compute descriptive statistics for selected columns.
pub fn describe_selected(df: &DataFrame, columns: &[&str]) -> Vec<DescriptiveStats> {
    columns
        .iter()
        .filter_map(|col| describe(df, col))
        .collect()
}

/// Categorical summary: value counts for a column.
#[derive(Debug, Clone)]
pub struct CategoricalSummary {
    pub name: String,
    pub total: usize,
    pub missing: usize,
    pub unique: usize,
    pub top_values: Vec<(String, usize)>,
}

/// Compute categorical summary for a column.
pub fn categorical_summary(df: &DataFrame, col_name: &str) -> Option<CategoricalSummary> {
    let values = df.column(col_name)?;
    let total = values.len();
    let mut missing = 0usize;
    let mut counts: std::collections::HashMap<String, usize> = std::collections::HashMap::new();

    for val in &values {
        let v = val.trim();
        if is_missing(v) {
            missing += 1;
        } else {
            *counts.entry(v.to_string()).or_insert(0) += 1;
        }
    }

    let unique = counts.len();
    let mut top_values: Vec<(String, usize)> = counts.into_iter().collect();
    top_values.sort_by(|a, b| b.1.cmp(&a.1));
    top_values.truncate(10);

    Some(CategoricalSummary {
        name: col_name.to_string(),
        total,
        missing,
        unique,
        top_values,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::reader;

    #[test]
    fn test_mean_basic() {
        assert!((mean(&[1.0, 2.0, 3.0, 4.0, 5.0]) - 3.0).abs() < 1e-10);
    }

    #[test]
    fn test_mean_single() {
        assert!((mean(&[42.0]) - 42.0).abs() < 1e-10);
    }

    #[test]
    fn test_mean_empty() {
        assert!(mean(&[]).is_nan());
    }

    #[test]
    fn test_mean_negative() {
        assert!((mean(&[-2.0, -1.0, 0.0, 1.0, 2.0]) - 0.0).abs() < 1e-10);
    }

    #[test]
    fn test_std_dev_known() {
        // Population {2, 4, 4, 4, 5, 5, 7, 9} has sample std dev ~2.138
        let data = vec![2.0, 4.0, 4.0, 4.0, 5.0, 5.0, 7.0, 9.0];
        let sd = std_dev(&data);
        assert!((sd - 2.13809).abs() < 0.001);
    }

    #[test]
    fn test_std_dev_constant() {
        // All same values: std dev should be 0
        assert!((std_dev(&[5.0, 5.0, 5.0, 5.0]) - 0.0).abs() < 1e-10);
    }

    #[test]
    fn test_std_dev_single() {
        assert!((std_dev(&[7.0]) - 0.0).abs() < 1e-10);
    }

    #[test]
    fn test_std_dev_empty() {
        assert!((std_dev(&[]) - 0.0).abs() < 1e-10);
    }

    #[test]
    fn test_std_dev_two_values() {
        // [0, 10]: mean=5, variance = ((5^2 + 5^2) / 1) = 50, std = sqrt(50) ~ 7.071
        let sd = std_dev(&[0.0, 10.0]);
        assert!((sd - 7.07107).abs() < 0.001);
    }

    #[test]
    fn test_percentile_median_odd() {
        let data = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        assert!((percentile(&data, 50.0) - 3.0).abs() < 1e-10);
    }

    #[test]
    fn test_percentile_median_even() {
        let data = vec![1.0, 2.0, 3.0, 4.0];
        assert!((percentile(&data, 50.0) - 2.5).abs() < 1e-10);
    }

    #[test]
    fn test_percentile_q1_q3() {
        let data = vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0];
        let q1 = percentile(&data, 25.0);
        let q3 = percentile(&data, 75.0);
        assert!((q1 - 2.75).abs() < 1e-10);
        assert!((q3 - 6.25).abs() < 1e-10);
    }

    #[test]
    fn test_percentile_min_max() {
        let data = vec![10.0, 20.0, 30.0];
        assert!((percentile(&data, 0.0) - 10.0).abs() < 1e-10);
        assert!((percentile(&data, 100.0) - 30.0).abs() < 1e-10);
    }

    #[test]
    fn test_percentile_empty() {
        assert!(percentile(&[], 50.0).is_nan());
    }

    #[test]
    fn test_percentile_single() {
        assert!((percentile(&[42.0], 50.0) - 42.0).abs() < 1e-10);
    }

    #[test]
    fn test_describe_with_missing() {
        let df = reader::read_file("tests/data/sample.csv").unwrap();
        let desc = describe(&df, "income").unwrap();
        // income has 2 missing values (Eve row 5 empty, Leo row 12 NA)
        // plus Xavier row 24 also empty
        assert!(desc.missing > 0);
        assert!(desc.count > 0);
        assert!(desc.count + desc.missing == 30);
    }

    #[test]
    fn test_describe_nonexistent_column() {
        let df = reader::read_file("tests/data/sample.csv").unwrap();
        assert!(describe(&df, "nonexistent").is_none());
    }

    #[test]
    fn test_describe_all_returns_numeric_only() {
        let df = reader::read_file("tests/data/sample.csv").unwrap();
        let all = describe_all(&df);
        // Should include numeric columns like id, age, income, score
        let names: Vec<&str> = all.iter().map(|s| s.name.as_str()).collect();
        assert!(names.contains(&"age"));
        assert!(names.contains(&"income"));
        assert!(names.contains(&"score"));
        // Should NOT include categorical columns
        assert!(!names.contains(&"name"));
        assert!(!names.contains(&"city"));
    }

    #[test]
    fn test_categorical_summary() {
        let df = reader::read_file("tests/data/sample.csv").unwrap();
        let summary = categorical_summary(&df, "city").unwrap();
        assert!(summary.unique > 0);
        assert_eq!(summary.total, 30);
        // city has some missing values (Uma row 21, Ben row 28)
        assert!(summary.missing >= 1);
    }
}
