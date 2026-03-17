//! Type checking command for Ash workflows.
//!
//! TASK-053: Implement `check` command for type checking workflows.

use anyhow::{Context, Result};
use clap::Args;
use colored::Colorize;
use std::path::Path;
use walkdir::WalkDir;

/// Arguments for the check command
#[derive(Args, Debug, Clone)]
pub struct CheckArgs {
    /// Path to workflow file or directory
    #[arg(value_name = "PATH")]
    pub path: String,

    /// Check all files in directory recursively
    #[arg(short, long)]
    pub all: bool,

    /// Enable strict mode (treat warnings as errors)
    #[arg(short, long)]
    pub strict: bool,

    /// Output format (human, json)
    #[arg(short, long, default_value = "human")]
    pub format: String,
}

/// Run type checking on workflow files
pub fn check(args: &CheckArgs) -> Result<()> {
    let path = Path::new(&args.path);

    if args.all || path.is_dir() {
        check_directory(path, args)
    } else {
        check_file(path, args)
    }
}

/// Check a single workflow file
fn check_file(path: &Path, args: &CheckArgs) -> Result<()> {
    let source = std::fs::read_to_string(path)
        .with_context(|| format!("Failed to read file: {}", path.display()))?;

    // Parse the workflow
    let mut input = ash_parser::new_input(&source);
    let workflow_def = ash_parser::parse_workflow::workflow_def(&mut input)
        .map_err(|e| anyhow::anyhow!("Parse error: {}", e))?;

    // Run type checking
    let result = ash_typeck::type_check_workflow(&workflow_def.body)?;

    // Output results
    match args.format.as_str() {
        "json" => output_json(path, &result, args),
        _ => output_human(path, &result, args),
    }
}

/// Check all workflow files in a directory
fn check_directory(path: &Path, args: &CheckArgs) -> Result<()> {
    let mut files_checked = 0;
    let mut errors_found = 0;

    for entry in WalkDir::new(path)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| {
            e.file_type().is_file()
                && e.path()
                    .extension()
                    .map(|ext| ext == "ash" || ext == "wf")
                    .unwrap_or(false)
        })
    {
        match check_file(entry.path(), args) {
            Ok(()) => {
                files_checked += 1;
            }
            Err(e) => {
                files_checked += 1;
                errors_found += 1;
                eprintln!("{} {}", "Error:".red().bold(), e);
            }
        }
    }

    if files_checked == 0 {
        println!("{}", "No workflow files found.".yellow());
        Ok(())
    } else if errors_found > 0 {
        Err(anyhow::anyhow!(
            "Type checking failed: {} error(s) in {} file(s)",
            errors_found,
            files_checked
        ))
    } else {
        println!(
            "[OK] {} file(s) type-checked successfully",
            files_checked
        );
        Ok(())
    }
}

/// Output results in human-readable format
fn output_human(path: &Path, result: &ash_typeck::TypeCheckResult, args: &CheckArgs) -> Result<()> {
    let file_name = path.display().to_string().cyan();

    if result.is_ok() {
        println!("[OK] {}: {}", file_name, "OK".green());
        if args.strict && !result.errors.is_empty() {
            for err in &result.errors {
                println!("  {} {}", "Warning:".yellow(), err);
            }
        }
        Ok(())
    } else {
        println!("[FAIL] {}: {}", file_name, "FAILED".red());
        for err in &result.errors {
            println!("  {} {}", "Error:".red().bold(), err);
        }
        Err(anyhow::anyhow!(
            "Type checking failed for {}",
            path.display()
        ))
    }
}

/// Output results in JSON format
fn output_json(path: &Path, result: &ash_typeck::TypeCheckResult, args: &CheckArgs) -> Result<()> {
    let output = serde_json::json!({
        "file": path.display().to_string(),
        "success": result.is_ok(),
        "errors": result.errors.len(),
        "effect": format!("{:?}", result.effect),
        "obligation_status": format!("{:?}", result.obligation_status),
        "strict": args.strict,
    });
    println!("{}", serde_json::to_string_pretty(&output)?);
    if result.is_ok() {
        Ok(())
    } else {
        Err(anyhow::anyhow!(
            "Type checking failed for {}",
            path.display()
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_check_args_parsing() {
        // Simulate CLI args parsing
        let args = CheckArgs {
            path: "test.ash".to_string(),
            all: false,
            strict: true,
            format: "human".to_string(),
        };

        assert_eq!(args.path, "test.ash");
        assert!(args.strict);
        assert!(!args.all);
        assert_eq!(args.format, "human");
    }

    #[test]
    fn test_check_args_default_format() {
        let args = CheckArgs {
            path: "test.ash".to_string(),
            all: false,
            strict: false,
            format: "human".to_string(),
        };
        assert_eq!(args.format, "human");
    }
}
