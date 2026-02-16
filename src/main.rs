mod correlation;
mod display;
mod missing;
mod plot;
mod reader;
mod stats;
mod types;
pub mod utils;

use anyhow::{bail, Context, Result};
use clap::{Parser, Subcommand};
use std::fs;
use std::process;

#[derive(Parser)]
#[command(
    name = "statsctl",
    about = "Quick statistical analysis CLI for CSV/TSV data files",
    version,
    author = "Angel Samuel Suesca Ríos <suescapsam@gmail.com>",
    after_help = "\
Common workflows:
  Quick overview:      statsctl summary data.csv
  Specific columns:    statsctl summary data.csv --vars age,income
  All columns:         statsctl summary data.csv --all
  Missing analysis:    statsctl missing data.csv --patterns
  Visualize:           statsctl plot data.csv --var age --type histogram
  Correlations:        statsctl correlation data.csv --min 0.7
  Compare datasets:    statsctl compare train.csv test.csv
  Export markdown:     statsctl summary data.csv -o report.md
  Pipe from stdin:     cat data.csv | statsctl summary --stdin"
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Descriptive statistics for numeric columns
    #[command(long_about = "\
Compute descriptive statistics (count, mean, std, min, Q1, median, Q3, max) for \
numeric columns in a CSV/TSV file. Use --all to also include categorical summaries.

Examples:
  statsctl summary data.csv
      Describe all numeric columns in data.csv

  statsctl summary data.csv --vars age,income,score
      Describe only the specified columns

  statsctl summary data.csv --all
      Include categorical variable summaries (top values, unique counts)

  statsctl summary data.csv -o report.md
      Export the summary table to a Markdown file

  cat data.csv | statsctl summary --stdin
      Read data from a piped command via stdin")]
    Summary {
        /// Path to the CSV/TSV file
        file: Option<String>,

        /// Comma-separated list of column names
        #[arg(long)]
        vars: Option<String>,

        /// Include all columns (numeric + categorical)
        #[arg(long)]
        all: bool,

        /// Output file path (supports .md, .json, .csv)
        #[arg(long, short)]
        output: Option<String>,

        /// Read from stdin
        #[arg(long)]
        stdin: bool,
    },

    /// Missing data analysis
    #[command(long_about = "\
Analyze missing data across all columns, showing counts and percentages. \
Use --patterns to reveal co-occurrence patterns of missingness.

Examples:
  statsctl missing data.csv
      Show missing counts for every column

  statsctl missing data.csv --only-missing
      Show only columns that have at least one missing value

  statsctl missing data.csv --patterns
      Show which columns tend to be missing together (co-occurrence patterns)

  statsctl missing data.csv --patterns -o missing_report.md
      Export the full missing data report to Markdown

  statsctl missing survey_responses.tsv --only-missing --patterns
      Combine filters: only missing columns with pattern analysis")]
    Missing {
        /// Path to the CSV/TSV file
        file: String,

        /// Show only columns with missing values
        #[arg(long)]
        only_missing: bool,

        /// Show missing data patterns
        #[arg(long)]
        patterns: bool,

        /// Output file path
        #[arg(long, short)]
        output: Option<String>,
    },

    /// Correlation matrix for numeric variables
    #[command(long_about = "\
Compute the Pearson correlation matrix for numeric columns using pairwise \
complete observations. Highlights high correlations with color coding.

Examples:
  statsctl correlation data.csv
      Full correlation matrix for all numeric columns

  statsctl correlation data.csv --vars age,income,score
      Correlation matrix for selected columns only

  statsctl correlation data.csv --min 0.7
      Highlight pairs with |r| >= 0.7

  statsctl correlation data.csv --min 0.3 -o corr.json
      Export correlations as JSON with a lower threshold

  statsctl correlation wide_dataset.csv --vars x1,x2,x3,x4,x5
      Focused correlation analysis on a subset of features")]
    Correlation {
        /// Path to the CSV/TSV file
        file: String,

        /// Comma-separated list of column names
        #[arg(long)]
        vars: Option<String>,

        /// Minimum correlation threshold to highlight
        #[arg(long, default_value = "0.5")]
        min: f64,

        /// Output file path
        #[arg(long, short)]
        output: Option<String>,
    },

    /// Quick ASCII plots
    #[command(long_about = "\
Generate ASCII-art visualizations directly in the terminal. Supports histograms, \
boxplots, and scatter plots for quick exploratory data analysis.

Examples:
  statsctl plot data.csv --var age --type histogram
      Histogram of the age column

  statsctl plot data.csv --var income --type boxplot
      Boxplot showing quartiles and outliers for income

  statsctl plot data.csv --vars age,income --type scatter
      Scatter plot of age (x) vs income (y)

  statsctl plot data.csv --var score --type hist -o plot.txt
      Save a histogram to a text file

  statsctl plot data.csv --var income --type box
      Shorthand: 'hist' and 'box' are accepted aliases")]
    Plot {
        /// Path to the CSV/TSV file
        file: String,

        /// Column name (for histogram, boxplot)
        #[arg(long)]
        var: Option<String>,

        /// Comma-separated column names (for scatter: x,y)
        #[arg(long)]
        vars: Option<String>,

        /// Plot type: histogram, boxplot, scatter
        #[arg(long = "type", default_value = "histogram")]
        plot_type: String,

        /// Output file path
        #[arg(long, short)]
        output: Option<String>,
    },

    /// Infer and display data types
    #[command(long_about = "\
Analyze each column and infer its data type (Numeric, Boolean, or Categorical). \
Optionally display the unique levels for categorical and boolean columns.

Examples:
  statsctl types data.csv
      Show inferred type and unique count for every column

  statsctl types data.csv --show-levels
      Also display the distinct values for categorical/boolean columns

  statsctl types survey.tsv
      Works with tab-separated files as well")]
    Types {
        /// Path to the CSV/TSV file
        file: String,

        /// Show unique values / levels for categorical variables
        #[arg(long)]
        show_levels: bool,
    },

    /// Compare two datasets
    #[command(long_about = "\
Side-by-side comparison of descriptive statistics and missing data between two \
CSV/TSV files. Useful for comparing train/test splits, before/after transformations, \
or different time periods.

Examples:
  statsctl compare train.csv test.csv
      Compare all numeric columns between two files

  statsctl compare train.csv test.csv --vars age,income
      Compare only specific columns

  statsctl compare before.csv after.csv -o comparison.md
      Export the comparison report to Markdown

  statsctl compare 2023_data.csv 2024_data.csv --vars revenue,users
      Compare specific metrics across yearly snapshots")]
    Compare {
        /// First file path
        file1: String,

        /// Second file path
        file2: String,

        /// Comma-separated list of column names to compare
        #[arg(long)]
        vars: Option<String>,

        /// Output file path
        #[arg(long, short)]
        output: Option<String>,
    },
}

fn main() {
    let cli = Cli::parse();

    let result = match cli.command {
        Commands::Summary {
            file,
            vars,
            all,
            output,
            stdin,
        } => cmd_summary(file, vars, all, output, stdin),
        Commands::Missing {
            file,
            only_missing,
            patterns,
            output,
        } => cmd_missing(&file, only_missing, patterns, output),
        Commands::Correlation {
            file,
            vars,
            min,
            output,
        } => cmd_correlation(&file, vars, min, output),
        Commands::Plot {
            file,
            var,
            vars,
            plot_type,
            output,
        } => cmd_plot(&file, var, vars, &plot_type, output),
        Commands::Types { file, show_levels } => cmd_types(&file, show_levels),
        Commands::Compare {
            file1,
            file2,
            vars,
            output,
        } => cmd_compare(&file1, &file2, vars, output),
    };

    if let Err(e) = result {
        eprintln!("Error: {:#}", e);
        process::exit(1);
    }
}

fn load_data(file: Option<&str>, stdin: bool) -> Result<reader::DataFrame> {
    if stdin {
        reader::read_stdin()
    } else {
        match file {
            Some(path) => reader::read_file(path),
            None => bail!("No file specified. Use --stdin to read from stdin."),
        }
    }
}

fn write_output(content: &str, output: Option<&str>) -> Result<()> {
    match output {
        Some(path) => {
            // Determine format from extension
            let format = if path.ends_with(".json") {
                "json"
            } else if path.ends_with(".csv") {
                "csv"
            } else {
                "md"
            };
            let exported = display::export_output(content, format);
            fs::write(path, &exported)
                .with_context(|| format!("Cannot write to '{}'", path))?;
            println!("Output written to: {}", path);
            Ok(())
        }
        None => {
            println!("{}", content);
            Ok(())
        }
    }
}

fn parse_vars(vars: &Option<String>) -> Option<Vec<String>> {
    vars.as_ref().map(|v| {
        v.split(',')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect()
    })
}

fn cmd_summary(
    file: Option<String>,
    vars: Option<String>,
    all: bool,
    output: Option<String>,
    stdin: bool,
) -> Result<()> {
    let df = load_data(file.as_deref(), stdin)?;
    let selected = parse_vars(&vars);

    let numeric_stats = if let Some(ref cols) = selected {
        let col_refs: Vec<&str> = cols.iter().map(|s| s.as_str()).collect();
        stats::describe_selected(&df, &col_refs)
    } else {
        stats::describe_all(&df)
    };

    let mut result = String::new();

    if !numeric_stats.is_empty() {
        result.push_str(&display::format_summary(&numeric_stats));
    }

    if all {
        // Also show categorical summaries
        let type_infos = types::infer_types(&df);
        let cat_cols: Vec<String> = type_infos
            .iter()
            .filter(|t| t.col_type != types::ColumnType::Numeric)
            .map(|t| t.name.clone())
            .collect();

        if !cat_cols.is_empty() {
            let cat_summaries: Vec<stats::CategoricalSummary> = cat_cols
                .iter()
                .filter_map(|col| stats::categorical_summary(&df, col))
                .collect();

            if !cat_summaries.is_empty() {
                result.push_str("\n\nCategorical Variables:\n");
                result.push_str(&display::format_categorical(&cat_summaries));
            }
        }
    }

    if result.is_empty() {
        result = "No numeric columns found in the dataset.".to_string();
    }

    write_output(&result, output.as_deref())
}

fn cmd_missing(
    file: &str,
    only_missing_flag: bool,
    patterns: bool,
    output: Option<String>,
) -> Result<()> {
    let df = reader::read_file(file)?;
    let infos = missing::analyze(&df);

    let mut result = String::new();

    if only_missing_flag {
        let filtered = missing::only_missing(&infos);
        if filtered.is_empty() {
            result.push_str("No missing data found.");
        } else {
            let owned: Vec<missing::MissingInfo> = filtered.into_iter().cloned().collect();
            result.push_str(&display::format_missing(&owned));
        }
    } else {
        result.push_str(&display::format_missing(&infos));
    }

    if patterns {
        let pattern_report = missing::missing_patterns(&df);
        result.push_str(&display::format_missing_patterns(&pattern_report));
    } else {
        // Show summary even without --patterns
        let total = df.nrows();
        let rows_with_any_missing = df
            .rows
            .iter()
            .filter(|row| row.iter().any(|v| utils::is_missing(v)))
            .count();

        if rows_with_any_missing > 0 && total > 0 {
            let pct = (rows_with_any_missing as f64 / total as f64) * 100.0;
            result.push_str(&format!(
                "\n{:.2}% of observations have at least one missing value",
                pct
            ));
        }
    }

    write_output(&result, output.as_deref())
}

fn cmd_correlation(
    file: &str,
    vars: Option<String>,
    min_threshold: f64,
    output: Option<String>,
) -> Result<()> {
    let df = reader::read_file(file)?;
    let selected = parse_vars(&vars);

    let cm = if let Some(ref cols) = selected {
        let col_refs: Vec<&str> = cols.iter().map(|s| s.as_str()).collect();
        correlation::correlation_matrix(&df, Some(&col_refs))
    } else {
        correlation::correlation_matrix(&df, None)
    };

    if cm.columns.is_empty() {
        bail!("No numeric columns found for correlation analysis.");
    }

    let mut result = display::format_correlation(&cm);

    let high = correlation::high_correlations(&cm, min_threshold);
    result.push_str(&display::format_high_correlations(&high, min_threshold));

    write_output(&result, output.as_deref())
}

fn cmd_plot(
    file: &str,
    var: Option<String>,
    vars: Option<String>,
    plot_type: &str,
    output: Option<String>,
) -> Result<()> {
    let df = reader::read_file(file)?;

    let result = match plot_type {
        "histogram" | "hist" => {
            let col = var
                .or_else(|| vars.as_ref().and_then(|v| v.split(',').next().map(|s| s.trim().to_string())))
                .context("Please specify a column with --var")?;
            plot::histogram(&df, &col, 50, 12)
                .with_context(|| format!("Cannot create histogram for column '{}'", col))?
        }
        "boxplot" | "box" => {
            let col = var
                .or_else(|| vars.as_ref().and_then(|v| v.split(',').next().map(|s| s.trim().to_string())))
                .context("Please specify a column with --var")?;
            plot::boxplot(&df, &col, 50)
                .with_context(|| format!("Cannot create boxplot for column '{}'", col))?
        }
        "scatter" => {
            let cols = vars.context("Please specify two columns with --vars x,y")?;
            let parts: Vec<&str> = cols.split(',').map(|s| s.trim()).collect();
            if parts.len() < 2 {
                bail!("Scatter plot requires two columns: --vars x,y");
            }
            plot::scatter(&df, parts[0], parts[1], 50, 15)
                .with_context(|| format!(
                    "Cannot create scatter plot for columns '{}' and '{}'",
                    parts[0], parts[1]
                ))?
        }
        _ => {
            bail!(
                "Unknown plot type '{}'. Use: histogram, boxplot, scatter",
                plot_type
            );
        }
    };

    write_output(&result, output.as_deref())
}

fn cmd_types(file: &str, show_levels: bool) -> Result<()> {
    let df = reader::read_file(file)?;
    let type_infos = types::infer_types(&df);
    let result = display::format_types(&type_infos, show_levels);
    println!("{}", result);
    Ok(())
}

fn cmd_compare(
    file1: &str,
    file2: &str,
    vars: Option<String>,
    output: Option<String>,
) -> Result<()> {
    let df1 = reader::read_file(file1)?;
    let df2 = reader::read_file(file2)?;

    let selected = parse_vars(&vars);

    let stats1 = if let Some(ref cols) = selected {
        let col_refs: Vec<&str> = cols.iter().map(|s| s.as_str()).collect();
        stats::describe_selected(&df1, &col_refs)
    } else {
        stats::describe_all(&df1)
    };

    let stats2 = if let Some(ref cols) = selected {
        let col_refs: Vec<&str> = cols.iter().map(|s| s.as_str()).collect();
        stats::describe_selected(&df2, &col_refs)
    } else {
        stats::describe_all(&df2)
    };

    // Extract filename for labels
    let label1 = std::path::Path::new(file1)
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or(file1);
    let label2 = std::path::Path::new(file2)
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or(file2);

    let mut result = display::format_comparison(&stats1, &stats2, label1, label2);

    // Also compare missing data
    let missing1 = missing::analyze(&df1);
    let missing2 = missing::analyze(&df2);

    result.push_str("\n\nMissing Data Comparison:\n");
    let mut builder = tabled::builder::Builder::new();
    builder.push_record([
        "Variable",
        &format!("{} Missing", label1),
        &format!("{} Missing", label2),
        "Diff",
    ]);

    for m1 in &missing1 {
        if let Some(m2) = missing2.iter().find(|m| m.name == m1.name) {
            let diff = m2.missing as i64 - m1.missing as i64;
            let diff_str = if diff > 0 {
                format!("+{}", diff)
            } else {
                diff.to_string()
            };
            builder.push_record([
                m1.name.clone(),
                format!("{} ({:.1}%)", m1.missing, m1.pct),
                format!("{} ({:.1}%)", m2.missing, m2.pct),
                diff_str,
            ]);
        }
    }

    result.push_str(
        &builder
            .build()
            .with(tabled::settings::Style::rounded())
            .to_string(),
    );

    write_output(&result, output.as_deref())
}
