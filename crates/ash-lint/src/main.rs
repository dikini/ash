//! Ash Lint CLI — Thin wrapper around the `ash_lint` library.

use anyhow::Result;
use ash_lint::{LintCode, LintConfig, LintDiagnostic, RuleLevel, lint_source};
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

    /// Disable specific lint rules (accepts rule IDs or legacy aliases)
    #[arg(long, value_delimiter = ',')]
    allow: Vec<String>,
}

#[derive(Clone, Copy, Debug, clap::ValueEnum)]
enum OutputFormat {
    Human,
    Json,
    Github,
}

fn main() -> Result<()> {
    let args = Args::parse();

    let mut config = LintConfig::default();

    // Map legacy CLI flags to LintConfig rules
    for rule in &args.allow {
        let code = resolve_alias(rule);
        config
            .rules
            .insert(LintCode(code.to_string()), RuleLevel::Allow);
    }
    if args.deny_warnings {
        for level in config.rules.values_mut() {
            if *level == RuleLevel::Warn {
                *level = RuleLevel::Deny;
            }
        }
    }

    let mut diagnostics = Vec::new();

    for path in &args.paths {
        if path.is_file() {
            lint_file(path, &config, &mut diagnostics)?;
        } else if path.is_dir() {
            lint_directory(path, &config, &mut diagnostics)?;
        }
    }

    // Output results
    match args.format {
        OutputFormat::Human => output_human(&diagnostics),
        OutputFormat::Json => output_json(&diagnostics)?,
        OutputFormat::Github => output_github(&diagnostics),
    }

    // Exit code
    let has_errors = diagnostics
        .iter()
        .any(|d| d.severity == ash_lint::LintSeverity::Error);
    let has_warnings = diagnostics
        .iter()
        .any(|d| d.severity == ash_lint::LintSeverity::Warning);

    if has_errors || (args.deny_warnings && has_warnings) {
        std::process::exit(1);
    }

    Ok(())
}

/// Resolve legacy rule aliases to canonical lint codes.
fn resolve_alias(rule: &str) -> &str {
    match rule {
        "ooda-missing-decide" => "L001",
        "ooda-missing-orient" => "L002",
        other => other,
    }
}

fn lint_file(
    path: &PathBuf,
    config: &LintConfig,
    diagnostics: &mut Vec<LintDiagnostic>,
) -> Result<()> {
    let source = std::fs::read_to_string(path)?;
    let file_diags = lint_source(&source, config);
    diagnostics.extend(file_diags);
    Ok(())
}

fn lint_directory(
    path: &PathBuf,
    config: &LintConfig,
    diagnostics: &mut Vec<LintDiagnostic>,
) -> Result<()> {
    for entry in walkdir::WalkDir::new(path)
        .into_iter()
        .filter_map(std::result::Result::ok)
        .filter(|e| e.file_type().is_file())
        .filter(|e| e.path().extension().is_some_and(|ext| ext == "ash"))
    {
        lint_file(&entry.path().to_path_buf(), config, diagnostics)?;
    }
    Ok(())
}

fn output_human(diagnostics: &[LintDiagnostic]) {
    if diagnostics.is_empty() {
        println!("{}", "✓ No issues found".green());
        return;
    }

    for diag in diagnostics {
        let severity_str = match diag.severity {
            ash_lint::LintSeverity::Error => "error".red().bold().to_string(),
            ash_lint::LintSeverity::Warning => "warning".yellow().bold().to_string(),
            ash_lint::LintSeverity::Information => "info".blue().to_string(),
            ash_lint::LintSeverity::Hint => "hint".dimmed().to_string(),
        };
        println!(
            "{}: {} [{}]",
            severity_str,
            diag.message,
            diag.code.0.dimmed()
        );
        println!(
            "  {}:{}:{}",
            diag.span.start, diag.span.line, diag.span.column
        );
        for fix in &diag.fixes {
            println!("  {}: {}", "help".cyan(), fix.description);
        }
        println!();
    }

    let errors = diagnostics
        .iter()
        .filter(|d| d.severity == ash_lint::LintSeverity::Error)
        .count();
    let warnings = diagnostics
        .iter()
        .filter(|d| d.severity == ash_lint::LintSeverity::Warning)
        .count();

    if errors > 0 {
        println!(
            "{}: {} errors, {} warnings",
            "failed".red().bold(),
            errors,
            warnings
        );
    } else {
        println!("{}: {} warnings", "completed".yellow().bold(), warnings);
    }
}

#[allow(clippy::unnecessary_wraps)]
fn output_json(diagnostics: &[LintDiagnostic]) -> Result<()> {
    // Serialize using the public types (serde feature may not be enabled,
    // so build a simple JSON manually if needed).
    #[cfg(feature = "serde")]
    {
        println!("{}", serde_json::to_string_pretty(diagnostics)?);
    }
    #[cfg(not(feature = "serde"))]
    {
        // Minimal JSON output without serde
        println!("[");
        for (i, d) in diagnostics.iter().enumerate() {
            if i > 0 {
                println!(",");
            }
            print!(
                "  {{\"code\": \"{}\", \"message\": \"{}\", \"line\": {}, \"column\": {}}}",
                d.code.0,
                d.message.replace('"', "\\\""),
                d.span.line,
                d.span.column
            );
        }
        println!("\n]");
    }
    Ok(())
}

fn output_github(diagnostics: &[LintDiagnostic]) {
    for diag in diagnostics {
        let level = match diag.severity {
            ash_lint::LintSeverity::Error => "error",
            ash_lint::LintSeverity::Warning => "warning",
            ash_lint::LintSeverity::Information | ash_lint::LintSeverity::Hint => "notice",
        };
        println!(
            "::{level} line={line},col={col}::{message} [{code}]",
            level = level,
            line = diag.span.line,
            col = diag.span.column,
            message = diag.message,
            code = diag.code.0
        );
    }
}
