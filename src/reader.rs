use crate::utils::is_missing;
use anyhow::{bail, Context, Result};
use std::io::{self, Read};

/// Represents a parsed dataset with headers and rows of string values.
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct DataFrame {
    pub headers: Vec<String>,
    pub rows: Vec<Vec<String>>,
}

#[allow(dead_code)]
impl DataFrame {
    /// Returns the number of rows.
    pub fn nrows(&self) -> usize {
        self.rows.len()
    }

    /// Returns the number of columns.
    pub fn ncols(&self) -> usize {
        self.headers.len()
    }

    /// Returns the index of a column by name.
    pub fn col_index(&self, name: &str) -> Option<usize> {
        self.headers.iter().position(|h| h == name)
    }

    /// Extracts a column as a vector of string references.
    pub fn column(&self, name: &str) -> Option<Vec<&str>> {
        let idx = self.col_index(name)?;
        Some(self.rows.iter().map(|row| row[idx].as_str()).collect())
    }

    /// Extracts a column as numeric values, skipping non-parseable entries.
    pub fn numeric_column(&self, name: &str) -> Option<Vec<Option<f64>>> {
        let idx = self.col_index(name)?;
        Some(
            self.rows
                .iter()
                .map(|row| {
                    let val = row[idx].trim();
                    if is_missing(val) {
                        None
                    } else {
                        val.parse::<f64>().ok()
                    }
                })
                .collect(),
        )
    }

    /// Returns only the non-None numeric values for a column.
    pub fn valid_numeric_column(&self, name: &str) -> Option<Vec<f64>> {
        self.numeric_column(name)
            .map(|col| col.into_iter().flatten().collect())
    }

    /// Filter to only specific columns.
    pub fn select_columns(&self, names: &[&str]) -> DataFrame {
        let indices: Vec<usize> = names
            .iter()
            .filter_map(|n| self.col_index(n))
            .collect();
        let headers: Vec<String> = indices.iter().map(|&i| self.headers[i].clone()).collect();
        let rows: Vec<Vec<String>> = self
            .rows
            .iter()
            .map(|row| indices.iter().map(|&i| row[i].clone()).collect())
            .collect();
        DataFrame { headers, rows }
    }
}

/// Detects the delimiter (comma or tab) by inspecting the first line.
fn detect_delimiter(first_line: &str) -> u8 {
    let tab_count = first_line.chars().filter(|&c| c == '\t').count();
    let comma_count = first_line.chars().filter(|&c| c == ',').count();
    if tab_count > comma_count {
        b'\t'
    } else {
        b','
    }
}

/// Parse CSV/TSV content from a string buffer into a DataFrame.
fn parse_csv(content: &str) -> Result<DataFrame> {
    let first_line = content.lines().next().unwrap_or("");
    if first_line.trim().is_empty() {
        bail!("Input data is empty");
    }

    let delimiter = detect_delimiter(first_line);

    let mut rdr = csv::ReaderBuilder::new()
        .delimiter(delimiter)
        .flexible(true)
        .has_headers(true)
        .from_reader(content.as_bytes());

    let headers: Vec<String> = rdr
        .headers()
        .context("Cannot read headers")?
        .iter()
        .map(|h| h.trim().to_string())
        .collect();

    if headers.is_empty() {
        bail!("No columns found in input");
    }

    let mut rows: Vec<Vec<String>> = Vec::new();
    for result in rdr.records() {
        let record = result
            .with_context(|| format!("Error reading row {}", rows.len() + 1))?;
        let mut row: Vec<String> = record.iter().map(|f| f.trim().to_string()).collect();
        // Pad short rows with empty strings
        while row.len() < headers.len() {
            row.push(String::new());
        }
        // Truncate long rows
        row.truncate(headers.len());
        rows.push(row);
    }

    Ok(DataFrame { headers, rows })
}

/// Reads a CSV/TSV file into a DataFrame.
///
/// The file is read once into memory and then parsed, avoiding a double file open.
pub fn read_file(path: &str) -> Result<DataFrame> {
    let content = std::fs::read_to_string(path)
        .with_context(|| format!("Cannot open file '{}'", path))?;

    if content.trim().is_empty() {
        bail!("File '{}' is empty", path);
    }

    parse_csv(&content)
        .with_context(|| format!("Failed to parse '{}'", path))
}

/// Reads from stdin into a DataFrame.
pub fn read_stdin() -> Result<DataFrame> {
    let stdin = io::stdin();
    let mut input = String::new();
    stdin
        .lock()
        .read_to_string(&mut input)
        .context("Cannot read stdin")?;

    if input.trim().is_empty() {
        bail!("No data received from stdin");
    }

    parse_csv(&input).context("Failed to parse stdin input")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_delimiter_comma() {
        assert_eq!(detect_delimiter("a,b,c"), b',');
    }

    #[test]
    fn test_detect_delimiter_tab() {
        assert_eq!(detect_delimiter("a\tb\tc"), b'\t');
    }

    #[test]
    fn test_detect_delimiter_mixed_more_tabs() {
        assert_eq!(detect_delimiter("a,b\tc\td\te"), b'\t');
    }

    #[test]
    fn test_detect_delimiter_empty() {
        assert_eq!(detect_delimiter(""), b',');
    }

    #[test]
    fn test_parse_csv_basic() {
        let data = "name,age,score\nAlice,25,85\nBob,34,72\n";
        let df = parse_csv(data).unwrap();
        assert_eq!(df.headers, vec!["name", "age", "score"]);
        assert_eq!(df.nrows(), 2);
        assert_eq!(df.ncols(), 3);
    }

    #[test]
    fn test_parse_csv_tsv() {
        let data = "name\tage\tscore\nAlice\t25\t85\nBob\t34\t72\n";
        let df = parse_csv(data).unwrap();
        assert_eq!(df.headers, vec!["name", "age", "score"]);
        assert_eq!(df.nrows(), 2);
    }

    #[test]
    fn test_parse_csv_empty() {
        let data = "";
        assert!(parse_csv(data).is_err());
    }

    #[test]
    fn test_parse_csv_headers_only() {
        let data = "name,age,score\n";
        let df = parse_csv(data).unwrap();
        assert_eq!(df.nrows(), 0);
        assert_eq!(df.ncols(), 3);
    }

    #[test]
    fn test_dataframe_column() {
        let data = "name,age\nAlice,25\nBob,34\n";
        let df = parse_csv(data).unwrap();
        let col = df.column("name").unwrap();
        assert_eq!(col, vec!["Alice", "Bob"]);
    }

    #[test]
    fn test_dataframe_numeric_column() {
        let data = "name,age\nAlice,25\nBob,NA\nCarol,30\n";
        let df = parse_csv(data).unwrap();
        let col = df.numeric_column("age").unwrap();
        assert_eq!(col, vec![Some(25.0), None, Some(30.0)]);
    }

    #[test]
    fn test_dataframe_valid_numeric_column() {
        let data = "name,age\nAlice,25\nBob,NA\nCarol,30\n";
        let df = parse_csv(data).unwrap();
        let col = df.valid_numeric_column("age").unwrap();
        assert_eq!(col, vec![25.0, 30.0]);
    }

    #[test]
    fn test_dataframe_missing_column() {
        let data = "name,age\nAlice,25\n";
        let df = parse_csv(data).unwrap();
        assert!(df.column("nonexistent").is_none());
    }

    #[test]
    fn test_read_file_real() {
        let df = read_file("tests/data/sample.csv").unwrap();
        assert_eq!(df.ncols(), 8);
        assert_eq!(df.nrows(), 30);
    }

    #[test]
    fn test_read_file_nonexistent() {
        assert!(read_file("nonexistent.csv").is_err());
    }

    #[test]
    fn test_short_row_padding() {
        let data = "a,b,c\n1,2\n4,5,6\n";
        let df = parse_csv(data).unwrap();
        assert_eq!(df.rows[0].len(), 3);
        assert_eq!(df.rows[0][2], "");
    }
}
