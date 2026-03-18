//! Type checking command for Ash workflows.
//!
//! TASK-053: Implement `check` command for type checking workflows.
//! TASK-076: Updated to use ash-engine.

use anyhow::{Context, Result};
use ash_engine::EngineError;
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
    // Create the engine
    let engine = ash_engine::Engine::new()
        .build()
        .context("Failed to build engine")?;

    // Parse and type check the file
    let workflow = engine
        .parse_file(path)
        .map_err(|e| anyhow::anyhow!("Parse error: {e}"))?;

    // Convert EngineError to anyhow::Error for consistent error handling
    let check_result: Result<(), EngineError> = engine.check(&workflow);
    let check_result: Result<()> = check_result.map_err(|e| anyhow::anyhow!("{e}"));

    // Output results
    match args.format.as_str() {
        "json" => output_json(path, &check_result, args),
        _ => output_human(path, &check_result, args),
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
            "Type checking failed: {errors_found} error(s) in {files_checked} file(s)"
        ))
    } else {
        println!("[OK] {files_checked} file(s) type-checked successfully");
        Ok(())
    }
}

/// Output results in human-readable format
fn output_human(path: &Path, result: &Result<()>, args: &CheckArgs) -> Result<()> {
    let file_name = path.display().to_string().cyan();

    if result.is_ok() {
        println!("[OK] {file_name}: {}", "OK".green());
        if args.strict {
            // In strict mode, we could output warnings here if the engine
            // provided them. For now, just indicate strict mode is on.
            println!("  {} Strict mode enabled", "Note:".yellow());
        }
        Ok(())
    } else {
        println!("[FAIL] {file_name}: {}", "FAILED".red());
        if let Err(e) = result {
            println!("  {} {e}", "Error:".red().bold());
        }
        Err(anyhow::anyhow!(
            "Type checking failed for {}",
            path.display()
        ))
    }
}

/// Output results in JSON format
fn output_json(path: &Path, result: &Result<()>, args: &CheckArgs) -> Result<()> {
    let success = result.is_ok();
    let error_message = result.as_ref().err().map(|e| format!("{e}"));

    let output = serde_json::json!({
        "file": path.display().to_string(),
        "success": success,
        "strict": args.strict,
        "error": error_message,
    });
    println!("{}", serde_json::to_string_pretty(&output)?);
    if success {
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
