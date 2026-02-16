use crate::reader::DataFrame;

/// Inferred type for a column.
#[derive(Debug, Clone, PartialEq)]
pub enum ColumnType {
    Numeric,
    Boolean,
    Categorical,
}

impl std::fmt::Display for ColumnType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ColumnType::Numeric => write!(f, "Numeric"),
            ColumnType::Boolean => write!(f, "Boolean"),
            ColumnType::Categorical => write!(f, "Categorical"),
        }
    }
}

/// Information about a column's type.
#[derive(Debug, Clone)]
pub struct ColumnTypeInfo {
    pub name: String,
    pub col_type: ColumnType,
    pub unique_count: usize,
    pub levels: Vec<String>,
}

/// Returns true if a value looks like a missing/null value.
fn is_missing(val: &str) -> bool {
    let v = val.trim();
    v.is_empty()
        || v == "NA"
        || v == "na"
        || v == "N/A"
        || v == "null"
        || v == "NULL"
        || v == "."
        || v == "NaN"
        || v == "nan"
}

/// Returns true if all non-missing values look boolean.
fn is_boolean(values: &[&str]) -> bool {
    let non_missing: Vec<&str> = values.iter().copied().filter(|v| !is_missing(v)).collect();
    if non_missing.is_empty() {
        return false;
    }
    non_missing.iter().all(|v| {
        let lower = v.to_lowercase();
        lower == "true"
            || lower == "false"
            || lower == "yes"
            || lower == "no"
            || lower == "1"
            || lower == "0"
    })
}

/// Returns true if most non-missing values can be parsed as numbers.
fn is_numeric(values: &[&str]) -> bool {
    let non_missing: Vec<&str> = values.iter().copied().filter(|v| !is_missing(v)).collect();
    if non_missing.is_empty() {
        return false;
    }
    let parseable = non_missing.iter().filter(|v| v.parse::<f64>().is_ok()).count();
    // Consider numeric if >= 80% of non-missing values parse as numbers
    (parseable as f64 / non_missing.len() as f64) >= 0.8
}

/// Infer the type of each column in the DataFrame.
pub fn infer_types(df: &DataFrame) -> Vec<ColumnTypeInfo> {
    let mut results = Vec::new();

    for header in &df.headers {
        if let Some(values) = df.column(header) {
            let non_missing: Vec<&str> = values.iter().copied().filter(|v| !is_missing(v)).collect();

            // Count unique non-missing values
            let mut unique_set: Vec<String> = non_missing.iter().map(|v| v.to_string()).collect();
            unique_set.sort();
            unique_set.dedup();
            let unique_count = unique_set.len();

            let col_type = if is_boolean(&values) {
                ColumnType::Boolean
            } else if is_numeric(&values) {
                ColumnType::Numeric
            } else {
                ColumnType::Categorical
            };

            let levels = if col_type == ColumnType::Categorical || col_type == ColumnType::Boolean {
                if unique_count <= 20 {
                    unique_set.clone()
                } else {
                    vec![format!("({} unique)", unique_count)]
                }
            } else {
                vec!["-".to_string()]
            };

            results.push(ColumnTypeInfo {
                name: header.clone(),
                col_type,
                unique_count,
                levels,
            });
        }
    }

    results
}

/// Returns the names of columns that are numeric.
pub fn numeric_columns(df: &DataFrame) -> Vec<String> {
    infer_types(df)
        .into_iter()
        .filter(|info| info.col_type == ColumnType::Numeric)
        .map(|info| info.name)
        .collect()
}
