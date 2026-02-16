/// Check if a value represents a missing value.
///
/// Recognizes common missing/null representations found in CSV/TSV data files,
/// including NA, null, NaN, empty strings, dots, dashes, and None.
pub fn is_missing(val: &str) -> bool {
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
        || v == "n/a"
        || v == "-"
        || v == "None"
        || v == "none"
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_missing_values() {
        assert!(is_missing(""));
        assert!(is_missing("NA"));
        assert!(is_missing("na"));
        assert!(is_missing("N/A"));
        assert!(is_missing("n/a"));
        assert!(is_missing("null"));
        assert!(is_missing("NULL"));
        assert!(is_missing("."));
        assert!(is_missing("NaN"));
        assert!(is_missing("nan"));
        assert!(is_missing("-"));
        assert!(is_missing("None"));
        assert!(is_missing("none"));
    }

    #[test]
    fn test_missing_with_whitespace() {
        assert!(is_missing("  "));
        assert!(is_missing("  NA  "));
        assert!(is_missing("\tNaN\t"));
    }

    #[test]
    fn test_not_missing() {
        assert!(!is_missing("0"));
        assert!(!is_missing("hello"));
        assert!(!is_missing("123"));
        assert!(!is_missing("3.14"));
        assert!(!is_missing("true"));
        assert!(!is_missing("N/A value"));
    }
}
