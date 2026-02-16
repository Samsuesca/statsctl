use crate::reader::DataFrame;
use crate::utils::is_missing;

/// Missing data info for one column.
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct MissingInfo {
    pub name: String,
    pub missing: usize,
    pub total: usize,
    pub pct: f64,
}

/// Analyze missing data for all columns.
pub fn analyze(df: &DataFrame) -> Vec<MissingInfo> {
    let total = df.nrows();
    df.headers
        .iter()
        .map(|header| {
            let missing = df
                .column(header)
                .map(|vals| vals.iter().filter(|v| is_missing(v)).count())
                .unwrap_or(0);
            let pct = if total > 0 {
                (missing as f64 / total as f64) * 100.0
            } else {
                0.0
            };
            MissingInfo {
                name: header.clone(),
                missing,
                total,
                pct,
            }
        })
        .collect()
}

/// Returns only columns that have missing values.
pub fn only_missing(infos: &[MissingInfo]) -> Vec<&MissingInfo> {
    infos.iter().filter(|info| info.missing > 0).collect()
}

/// Analyze missing data patterns (which rows have missing values in which columns).
pub fn missing_patterns(df: &DataFrame) -> MissingPatternReport {
    let total = df.nrows();
    let mut rows_with_missing = 0usize;
    let mut pattern_counts: std::collections::HashMap<String, usize> =
        std::collections::HashMap::new();

    for row in &df.rows {
        let pattern: String = row
            .iter()
            .map(|v| if is_missing(v) { '1' } else { '0' })
            .collect();

        if pattern.contains('1') {
            rows_with_missing += 1;
        }
        *pattern_counts.entry(pattern).or_insert(0) += 1;
    }

    let mut patterns: Vec<(String, usize)> = pattern_counts.into_iter().collect();
    patterns.sort_by(|a, b| b.1.cmp(&a.1));

    // Convert bit patterns to column name patterns
    let named_patterns: Vec<(Vec<String>, usize)> = patterns
        .iter()
        .filter(|(p, _)| p.contains('1'))
        .take(10)
        .map(|(pattern, count)| {
            let cols: Vec<String> = pattern
                .chars()
                .enumerate()
                .filter(|(_, c)| *c == '1')
                .filter_map(|(i, _)| df.headers.get(i).cloned())
                .collect();
            (cols, *count)
        })
        .collect();

    MissingPatternReport {
        total_rows: total,
        rows_with_missing,
        pct_with_missing: if total > 0 {
            (rows_with_missing as f64 / total as f64) * 100.0
        } else {
            0.0
        },
        patterns: named_patterns,
    }
}

/// Report on missing data patterns.
#[derive(Debug)]
pub struct MissingPatternReport {
    pub total_rows: usize,
    pub rows_with_missing: usize,
    pub pct_with_missing: f64,
    pub patterns: Vec<(Vec<String>, usize)>,
}
