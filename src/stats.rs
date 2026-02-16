use crate::reader::DataFrame;
use crate::types;

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
        if v.is_empty() || v == "NA" || v == "na" || v == "N/A" || v == "null" || v == "NULL"
            || v == "." || v == "NaN" || v == "nan"
        {
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
