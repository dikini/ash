//! Ash Lint - Custom lints for Ash workflow language
//!
//! Detects common issues beyond what clippy catches:
//! - OODA loop violations
//! - Missing provenance tracking
//! - Potential policy conflicts

use anyhow::Result;
use clap::Parser;
use colored::Colorize;
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "ash-lint")]
#[command(about = "Lint Ash workflow files")]
struct Args {
    /// Files or directories to lint
    #[arg(required = true)]
    paths: Vec<PathBuf>,
    
    /// Exit with error on warnings
    #[arg(short, long)]
    deny_warnings: bool,
    
    /// Output format
    #[arg(short, long, value_enum, default_value = "human")]
    format: OutputFormat,
    
    /// Disable specific lint rules
    #[arg(long, value_delimiter = ',')]
    allow: Vec<String>,
}

#[derive(Clone, Copy, Debug, clap::ValueEnum)]
enum OutputFormat {
    Human,
    Json,
    Github,
}

#[derive(Debug, Clone)]
struct LintDiagnostic {
    rule: String,
    severity: Severity,
    message: String,
    file: PathBuf,
    line: usize,
    column: usize,
    suggestion: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Severity {
    Error,
    Warning,
    Info,
}

impl std::fmt::Display for Severity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Severity::Error => write!(f, "{}", "error".red().bold()),
            Severity::Warning => write!(f, "{}", "warning".yellow().bold()),
            Severity::Info => write!(f, "{}", "info".blue()),
        }
    }
}

fn main() -> Result<()> {
    let args = Args::parse();
    
    let mut diagnostics = Vec::new();
    
    for path in &args.paths {
        if path.is_file() {
            lint_file(path, &args, &mut diagnostics)?;
        } else if path.is_dir() {
            lint_directory(path, &args, &mut diagnostics)?;
        }
    }
    
    // Output results
    match args.format {
        OutputFormat::Human => output_human(&diagnostics),
        OutputFormat::Json => output_json(&diagnostics)?,
        OutputFormat::Github => output_github(&diagnostics),
    }
    
    // Exit code
    let has_errors = diagnostics.iter().any(|d| d.severity == Severity::Error);
    let has_warnings = diagnostics.iter().any(|d| d.severity == Severity::Warning);
    
    if has_errors || (args.deny_warnings && has_warnings) {
        std::process::exit(1);
    }
    
    Ok(())
}

fn lint_file(path: &PathBuf, _args: &Args, diagnostics: &mut Vec<LintDiagnostic>) -> Result<()> {
    // Placeholder - actual implementation will parse and lint
    // For now, just show structure
    
    let content = std::fs::read_to_string(path)?;
    
    // Check for OODA violations
    if content.contains("observe") && !content.contains("decide") {
        diagnostics.push(LintDiagnostic {
            rule: "ooda-missing-decide".to_string(),
            severity: Severity::Warning,
            message: "OBSERVE without explicit DECIDE step".to_string(),
            file: path.clone(),
            line: 1,
            column: 1,
            suggestion: Some("Consider adding a DECIDE step after OBSERVE".to_string()),
        });
    }
    
    // Check for unused observations
    if content.contains("observe") && !content.contains("orient") {
        diagnostics.push(LintDiagnostic {
            rule: "ooda-missing-orient".to_string(),
            severity: Severity::Warning,
            message: "OBSERVE result never used in ORIENT".to_string(),
            file: path.clone(),
            line: 1,
            column: 1,
            suggestion: Some("Consider processing observations in ORIENT".to_string()),
        });
    }
    
    Ok(())
}

fn lint_directory(path: &PathBuf, args: &Args, diagnostics: &mut Vec<LintDiagnostic>) -> Result<()> {
    for entry in walkdir::WalkDir::new(path)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_file())
        .filter(|e| e.path().extension().map(|ext| ext == "ash").unwrap_or(false))
    {
        lint_file(&entry.path().to_path_buf(), args, diagnostics)?;
    }
    Ok(())
}

fn output_human(diagnostics: &[LintDiagnostic]) {
    if diagnostics.is_empty() {
        println!("{}", "✓ No issues found".green());
        return;
    }
    
    for diag in diagnostics {
        println!(
            "{}: {} [{}]",
            diag.severity,
            diag.message,
            diag.rule.dimmed()
        );
        println!(
            "  {}:{}:{}",
            diag.file.display(),
            diag.line,
            diag.column
        );
        if let Some(suggestion) = &diag.suggestion {
            println!("  {}: {}", "help".cyan(), suggestion);
        }
        println!();
    }
    
    let errors = diagnostics.iter().filter(|d| d.severity == Severity::Error).count();
    let warnings = diagnostics.iter().filter(|d| d.severity == Severity::Warning).count();
    
    if errors > 0 {
        println!("{}: {} errors, {} warnings", "failed".red().bold(), errors, warnings);
    } else {
        println!("{}: {} warnings", "completed".yellow().bold(), warnings);
    }
}

fn output_json(diagnostics: &[LintDiagnostic]) -> Result<()> {
    println!("{}", serde_json::to_string_pretty(diagnostics)?);
    Ok(())
}

fn output_github(diagnostics: &[LintDiagnostic]) {
    // GitHub Actions annotation format
    // ::error file={file},line={line},col={col}::{message}
    for diag in diagnostics {
        let level = match diag.severity {
            Severity::Error => "error",
            Severity::Warning => "warning",
            Severity::Info => "notice",
        };
        println!(
            "::{level} file={file},line={line},col={col}::{message} [{rule}]",
            level = level,
            file = diag.file.display(),
            line = diag.line,
            col = diag.column,
            message = diag.message,
            rule = diag.rule
        );
    }
}
