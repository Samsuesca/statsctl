# statsctl

> Quick statistical analysis CLI for CSV/TSV data files

![macOS](https://img.shields.io/badge/macOS-Apple_Silicon-blue)
![Rust](https://img.shields.io/badge/Rust-1.70+-orange)
![License](https://img.shields.io/badge/license-MIT-green)

**statsctl** is a lightweight command-line tool for rapid exploratory data analysis (EDA) before diving into Stata, R, or Python. Perfect for quick summaries, missing data analysis, and basic visualizations.

---

## Features

- **Descriptive Statistics**: Mean, median, std dev, min/max, quartiles
- **Missing Data Analysis**: Count and percentage of missing values per variable
- **Correlation Matrix**: Pearson correlations between numeric variables
- **Quick Plots**: Histograms, boxplots, scatter plots in terminal (ASCII art)
- **Data Type Detection**: Automatic identification of numeric/categorical variables
- **Export**: Results to CSV, JSON, or Markdown tables

---

## Installation

```bash
# Clone and build
git clone https://github.com/Samsuesca/statsctl.git
cd statsctl
cargo build --release
cargo install --path .
```

---

## Usage

### Summary Statistics

```bash
# Basic summary of all numeric columns
statsctl summary data.csv

# Summary of specific columns
statsctl summary data.csv --vars age,income,score

# Include categorical variables
statsctl summary data.csv --all

# Export to file
statsctl summary data.csv --output results.md
```

**Output:**
```
┌──────────┬───────┬──────────┬────────┬────────┬────────┬────────┬────────┐
│ Variable │ Count │ Mean     │ Std    │ Min    │ Q1     │ Median │ Max    │
├──────────┼───────┼──────────┼────────┼────────┼────────┼────────┼────────┤
│ age      │ 1000  │ 34.5     │ 12.3   │ 18     │ 25     │ 33     │ 65     │
│ income   │ 998   │ 45231.12 │ 8932.4 │ 12000  │ 38000  │ 44500  │ 95000  │
│ score    │ 1000  │ 78.3     │ 15.2   │ 0      │ 68     │ 80     │ 100    │
└──────────┴───────┴──────────┴────────┴────────┴────────┴────────┴────────┘
```

### Missing Data Analysis

```bash
# Check missing values
statsctl missing data.csv

# Show only variables with missings
statsctl missing data.csv --only-missing

# Detailed report with patterns
statsctl missing data.csv --patterns
```

**Output:**
```
Missing Data Report:
┌──────────────┬───────────┬─────────────┐
│ Variable     │ Missing   │ % Missing   │
├──────────────┼───────────┼─────────────┤
│ age          │ 0         │ 0.00%       │
│ income       │ 2         │ 0.20%       │
│ phone        │ 45        │ 4.50%       │
│ address      │ 123       │ 12.30%      │
└──────────────┴───────────┴─────────────┘

⚠️  12.30% of observations have at least one missing value
```

### Correlation Matrix

```bash
# Correlation matrix for all numeric variables
statsctl correlation data.csv

# Specific variables
statsctl correlation data.csv --vars age,income,score

# Only show correlations above threshold
statsctl correlation data.csv --min 0.5
```

**Output:**
```
Correlation Matrix (Pearson):
         age    income   score
age      1.00   0.34     -0.12
income   0.34   1.00     0.67
score    -0.12  0.67     1.00

⚠️  High correlations (|r| > 0.5):
  - income ↔ score: 0.67
```

### Quick Plots

```bash
# Histogram (ASCII art in terminal)
statsctl plot data.csv --var age --type histogram

# Boxplot
statsctl plot data.csv --var income --type boxplot

# Scatter plot (bivariate)
statsctl plot data.csv --vars age,income --type scatter

# Save plot to file (Unicode/ASCII art)
statsctl plot data.csv --var age --type histogram --output age_dist.txt
```

**Example Histogram:**
```
age: Distribution (n=1000)

 120│         ▄▄▄
    │       ▄▄███▄
  80│      ▄██████▄
    │    ▄▄████████▄
  40│   ▄███████████▄
    │ ▄▄██████████████▄
   0└─────────────────────
    18  25  33  45  55  65

Mean: 34.5 | Median: 33.0 | Std: 12.3
```

### Data Type Detection

```bash
# Infer data types
statsctl types data.csv

# Show unique values for categorical variables
statsctl types data.csv --show-levels
```

**Output:**
```
Data Types:
┌──────────────┬────────────┬──────────────┬─────────────────┐
│ Variable     │ Type       │ Unique       │ Levels          │
├──────────────┼────────────┼──────────────┼─────────────────┤
│ age          │ Numeric    │ 48           │ -               │
│ gender       │ Categorical│ 3            │ M, F, Other     │
│ city         │ Categorical│ 125          │ (125 unique)    │
│ income       │ Numeric    │ 989          │ -               │
│ employed     │ Boolean    │ 2            │ true, false     │
└──────────────┴────────────┴──────────────┴─────────────────┘
```

### Compare Datasets

```bash
# Compare two datasets (useful for before/after data cleaning)
statsctl compare raw.csv processed.csv

# Compare specific columns
statsctl compare raw.csv processed.csv --vars age,income
```

---

## Command Reference

| Command | Description | Options |
|---------|-------------|---------|
| `summary` | Descriptive statistics | `--vars`, `--all`, `--output` |
| `missing` | Missing data analysis | `--only-missing`, `--patterns` |
| `correlation` | Correlation matrix | `--vars`, `--min`, `--method` |
| `plot` | Quick plots (ASCII) | `--var`, `--vars`, `--type`, `--output` |
| `types` | Infer data types | `--show-levels` |
| `compare` | Compare two datasets | `--vars` |

---

## Use Cases

### Research Workflow (Stata/R Integration)

```bash
# Before running Stata scripts
statsctl summary data/raw/EDIF_2022.csv --output docs/data_summary.md
statsctl missing data/raw/EDIF_2022.csv --patterns

# Quick EDA before complex analysis
statsctl correlation data/processed/panel_data.csv --vars income,debt,score

# Validate processed data
statsctl compare data/raw/original.csv data/processed/cleaned.csv
```

### Data Cleaning Pipeline

```bash
#!/bin/bash
# Quick validation before Stata pipeline

echo "📊 Validating raw data..."
statsctl missing data/raw/*.csv --only-missing

if [ $? -ne 0 ]; then
  echo "⚠️  Missing data issues detected!"
  exit 1
fi

echo "✓ Data validated, proceeding with Stata..."
stata-mp -b do code/Stata/S00_Master.do
```

---

## Technical Stack

**Language**: Rust 2021 edition

**Dependencies**:
- `clap` - CLI argument parsing
- `csv` / `polars` - Data frame operations (choose one)
- `statrs` - Statistical functions
- `textplots` - ASCII plots
- `colored` - Terminal colors
- `tabled` - Table formatting
- `serde` / `serde_json` - Data serialization

---

## Architecture

```
src/
├── main.rs           # CLI entry point
├── reader.rs         # CSV/TSV parsing
├── stats.rs          # Statistical functions (mean, median, etc.)
├── missing.rs        # Missing data analysis
├── correlation.rs    # Correlation matrix
├── plot.rs           # ASCII plotting
├── types.rs          # Type inference
└── display.rs        # Formatted output
```

---

## Implementation Notes

### Data Frame Library Choice

**Option 1: Polars** (Recommended)
- Blazing fast (Rust-native)
- Handles large datasets (28GB Clean Slate dataset)
- Similar API to pandas
- Built-in statistical functions

**Option 2: CSV + Manual Processing**
- Lighter weight
- More control
- Sufficient for smaller datasets

### Statistical Accuracy

- Use `statrs` crate for accurate statistical computations
- Handle edge cases (empty data, single value, all missing)
- Verify against R/Stata outputs for correctness

---

## Platform Support

| Platform | Support |
|----------|---------|
| macOS (Apple Silicon) | ✅ Full support |
| macOS (Intel) | ✅ Full support |
| Linux | ✅ Full support |
| Windows | ✅ Full support |

---

## Roadmap

- [ ] Support for Excel files (.xlsx)
- [ ] Shapiro-Wilk normality test
- [ ] Outlier detection (IQR, Z-score methods)
- [ ] Group-by statistics (like `dplyr::group_by`)
- [ ] Time series summary (autocorrelation, seasonality)
- [ ] Export plots to PNG (via plotters crate)

---

## License

MIT License

---

## Author

**Angel Samuel Suesca Ríos**
suescapsam@gmail.com

---

## Integration with Other Tools

```bash
# Pipe to other tools
statsctl summary data.csv | grep -i "income"

# Use in data pipelines
cat data.csv | statsctl summary --stdin

# Export for Stata documentation
statsctl summary data.csv --output summary.md
# Include in LaTeX documents
```

---

**Perfect for**: Quick EDA before Stata/R scripts, data validation, teaching statistics, reproducible research.
