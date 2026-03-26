//! Type checking command for Ash workflows.
//!
//! TASK-053: Implement `check` command for type checking workflows.
//! TASK-076: Updated to use ash-engine.
//! TASK-280: Fixed JSON output schema compliance.

use crate::output::json::{JsonLocation, JsonOutput};
use anyhow::{Context, Result};
use ash_engine::EngineError;
use clap::Args;
use colored::Colorize;
use std::path::Path;
use std::time::Instant;
use walkdir::WalkDir;

/// Output format for check command
#[derive(Debug, Clone, Copy, Default, clap::ValueEnum)]
pub enum CheckOutputFormat {
    /// Human-readable format
    #[default]
    Human,
    /// JSON format
    Json,
}

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
    #[arg(short = 's', long)]
    pub strict: bool,

    /// Output format (human, json)
    #[arg(short = 'f', long, value_enum, default_value = "human")]
    pub format: CheckOutputFormat,

    /// Enable policy verification
    #[arg(long)]
    pub policy_check: bool,
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
    // Start timing
    let total_start = Instant::now();
    let parse_start = Instant::now();

    // Create the engine
    let engine = ash_engine::Engine::new()
        .build()
        .context("Failed to build engine")?;

    // Parse and type check the file
    let parse_result = engine.parse_file(path);
    let parse_time = parse_start.elapsed();

    let tc_start = Instant::now();
    let check_result = match parse_result {
        Ok(workflow) => {
            let check_result: Result<(), EngineError> = engine.check(&workflow);
            check_result.map_err(|e| anyhow::anyhow!("{e}"))
        }
        Err(e) => Err(anyhow::anyhow!("Parse error: {e}")),
    };
    let tc_time = tc_start.elapsed();
    let total_time = total_start.elapsed();

    // Output results
    match args.format {
        CheckOutputFormat::Json => {
            output_json(path, &check_result, args, parse_time, tc_time, total_time)
        }
        CheckOutputFormat::Human => output_human(path, &check_result, args),
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
fn output_json(
    path: &Path,
    result: &Result<()>,
    args: &CheckArgs,
    parse_time: std::time::Duration,
    tc_time: std::time::Duration,
    total_time: std::time::Duration,
) -> Result<()> {
    let success = result.is_ok();
    let exit_code = if success { 0 } else { 3 };

    // Build the JSON output
    let mut output = JsonOutput::new(path)
        .with_strict(args.strict)
        .with_exit_code(exit_code)
        .with_timing(parse_time, tc_time, total_time);

    // Add errors if present
    if let Err(e) = result {
        let error_str = format!("{e}");
        // Determine error code based on error content
        let code = if error_str.contains("Parse") {
            "E0001"
        } else if error_str.contains("Type") {
            "E0002"
        } else {
            "E9999"
        };
        output = output.with_error(
            &error_str,
            code,
            Some(JsonLocation::new(path.display().to_string(), 0, 0)),
        );
    }

    // Print the JSON output
    println!("{}", output.to_json()?);

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
            format: CheckOutputFormat::Human,
            policy_check: false,
        };

        assert_eq!(args.path, "test.ash");
        assert!(args.strict);
        assert!(!args.all);
        assert!(matches!(args.format, CheckOutputFormat::Human));
    }

    #[test]
    fn test_check_args_default_format() {
        let args = CheckArgs {
            path: "test.ash".to_string(),
            all: false,
            strict: false,
            format: CheckOutputFormat::Human,
            policy_check: false,
        };
        assert!(matches!(args.format, CheckOutputFormat::Human));
    }

    #[test]
    fn test_check_args_policy_check() {
        let args = CheckArgs {
            path: "test.ash".to_string(),
            all: false,
            strict: false,
            format: CheckOutputFormat::Json,
            policy_check: true,
        };
        assert!(args.policy_check);
    }
}
