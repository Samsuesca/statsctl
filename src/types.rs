use crate::reader::DataFrame;
use crate::utils::is_missing;

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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::reader;

    #[test]
    fn test_is_boolean_true_false() {
        assert!(is_boolean(&["true", "false", "true", "false"]));
    }

    #[test]
    fn test_is_boolean_yes_no() {
        assert!(is_boolean(&["yes", "no", "YES", "NO"]));
    }

    #[test]
    fn test_is_boolean_01() {
        assert!(is_boolean(&["0", "1", "1", "0"]));
    }

    #[test]
    fn test_is_boolean_with_missing() {
        assert!(is_boolean(&["true", "NA", "false", ""]));
    }

    #[test]
    fn test_is_boolean_mixed_not_bool() {
        assert!(!is_boolean(&["true", "maybe", "false"]));
    }

    #[test]
    fn test_is_boolean_empty() {
        assert!(!is_boolean(&[]));
    }

    #[test]
    fn test_is_boolean_all_missing() {
        assert!(!is_boolean(&["NA", "", "null"]));
    }

    #[test]
    fn test_is_numeric_integers() {
        assert!(is_numeric(&["1", "2", "3", "100"]));
    }

    #[test]
    fn test_is_numeric_floats() {
        assert!(is_numeric(&["1.5", "2.7", "3.14"]));
    }

    #[test]
    fn test_is_numeric_with_missing() {
        assert!(is_numeric(&["1", "NA", "3", ""]));
    }

    #[test]
    fn test_is_numeric_mostly_numeric() {
        // 80% threshold: 4 out of 5 non-missing are numeric
        assert!(is_numeric(&["1", "2", "3", "4", "hello"]));
    }

    #[test]
    fn test_is_numeric_text() {
        assert!(!is_numeric(&["hello", "world", "foo"]));
    }

    #[test]
    fn test_is_numeric_all_missing() {
        assert!(!is_numeric(&["NA", "", "null"]));
    }

    #[test]
    fn test_infer_types_sample_csv() {
        let df = reader::read_file("tests/data/sample.csv").unwrap();
        let types = infer_types(&df);

        let find = |name: &str| -> &ColumnTypeInfo {
            types.iter().find(|t| t.name == name).unwrap()
        };

        assert_eq!(find("age").col_type, ColumnType::Numeric);
        assert_eq!(find("income").col_type, ColumnType::Numeric);
        assert_eq!(find("score").col_type, ColumnType::Numeric);
        assert_eq!(find("name").col_type, ColumnType::Categorical);
        assert_eq!(find("city").col_type, ColumnType::Categorical);
        assert_eq!(find("gender").col_type, ColumnType::Categorical);
        assert_eq!(find("employed").col_type, ColumnType::Boolean);
    }

    #[test]
    fn test_numeric_columns() {
        let df = reader::read_file("tests/data/sample.csv").unwrap();
        let nums = numeric_columns(&df);
        assert!(nums.contains(&"age".to_string()));
        assert!(nums.contains(&"income".to_string()));
        assert!(nums.contains(&"score".to_string()));
        assert!(!nums.contains(&"name".to_string()));
        assert!(!nums.contains(&"employed".to_string()));
    }

    #[test]
    fn test_column_type_display() {
        assert_eq!(format!("{}", ColumnType::Numeric), "Numeric");
        assert_eq!(format!("{}", ColumnType::Boolean), "Boolean");
        assert_eq!(format!("{}", ColumnType::Categorical), "Categorical");
    }
}
