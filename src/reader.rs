use std::fs::File;
use std::io::{self, BufRead, BufReader, Read};

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
                    if val.is_empty() || val == "NA" || val == "na" || val == "N/A"
                        || val == "null" || val == "NULL" || val == "."
                        || val == "NaN" || val == "nan"
                    {
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

/// Reads a CSV/TSV file into a DataFrame.
pub fn read_file(path: &str) -> Result<DataFrame, String> {
    let file = File::open(path).map_err(|e| format!("Cannot open file '{}': {}", path, e))?;
    let mut buf_reader = BufReader::new(file);

    // Read first line to detect delimiter
    let mut first_line = String::new();
    buf_reader
        .read_line(&mut first_line)
        .map_err(|e| format!("Cannot read file '{}': {}", path, e))?;

    if first_line.trim().is_empty() {
        return Err(format!("File '{}' is empty", path));
    }

    let delimiter = detect_delimiter(&first_line);

    // Re-open file for csv reader
    let file = File::open(path).map_err(|e| format!("Cannot open file '{}': {}", path, e))?;
    let mut rdr = csv::ReaderBuilder::new()
        .delimiter(delimiter)
        .flexible(true)
        .has_headers(true)
        .from_reader(file);

    let headers: Vec<String> = rdr
        .headers()
        .map_err(|e| format!("Cannot read headers: {}", e))?
        .iter()
        .map(|h| h.trim().to_string())
        .collect();

    if headers.is_empty() {
        return Err("No columns found in file".to_string());
    }

    let mut rows: Vec<Vec<String>> = Vec::new();
    for result in rdr.records() {
        let record = result.map_err(|e| format!("Error reading row {}: {}", rows.len() + 1, e))?;
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

/// Reads from stdin into a DataFrame.
pub fn read_stdin() -> Result<DataFrame, String> {
    let stdin = io::stdin();
    let mut input = String::new();
    stdin
        .lock()
        .read_to_string(&mut input)
        .map_err(|e| format!("Cannot read stdin: {}", e))?;

    if input.trim().is_empty() {
        return Err("No data received from stdin".to_string());
    }

    let delimiter = detect_delimiter(input.lines().next().unwrap_or(""));

    let mut rdr = csv::ReaderBuilder::new()
        .delimiter(delimiter)
        .flexible(true)
        .has_headers(true)
        .from_reader(input.as_bytes());

    let headers: Vec<String> = rdr
        .headers()
        .map_err(|e| format!("Cannot read headers: {}", e))?
        .iter()
        .map(|h| h.trim().to_string())
        .collect();

    let mut rows: Vec<Vec<String>> = Vec::new();
    for result in rdr.records() {
        let record = result.map_err(|e| format!("Error reading row: {}", e))?;
        let mut row: Vec<String> = record.iter().map(|f| f.trim().to_string()).collect();
        while row.len() < headers.len() {
            row.push(String::new());
        }
        row.truncate(headers.len());
        rows.push(row);
    }

    Ok(DataFrame { headers, rows })
}
