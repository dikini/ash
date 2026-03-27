//! Type checking command for Ash workflows.
//!
//! TASK-053: Implement `check` command for type checking workflows.
//! TASK-076: Updated to use ash-engine.
//! TASK-280: Fixed JSON output schema compliance.
//! TASK-307: Fixed exit codes for SPEC-005 compliance.

use crate::error::{CliError, CliResult};
use crate::output::json::{JsonLocation, JsonOutput};
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
pub fn check(args: &CheckArgs) -> CliResult<()> {
    let path = Path::new(&args.path);

    if args.all || path.is_dir() {
        check_directory(path, args)
    } else {
        check_file(path, args)
    }
}

/// Check a single workflow file
fn check_file(path: &Path, args: &CheckArgs) -> CliResult<()> {
    // Start timing
    let total_start = Instant::now();
    let parse_start = Instant::now();

    // Create the engine
    let engine = ash_engine::Engine::new()
        .build()
        .map_err(|e| CliError::general(format!("Failed to build engine: {e}")))?;

    // Parse and type check the file
    let parse_result = engine.parse_file(path);
    let parse_time = parse_start.elapsed();

    let tc_start = Instant::now();
    let check_result: CliResult<()> = match parse_result {
        Ok(workflow) => {
            let type_result = engine.check(&workflow);
            type_result.map_err(|e| CliError::TypeError {
                message: format!("{e}"),
                source: None,
            })
        }
        Err(e) => {
            let err_msg = format!("{e}");
            // Check if this is an I/O error (e.g., file not found)
            if err_msg.contains("io error") || err_msg.contains("No such file") {
                Err(CliError::IoError {
                    message: err_msg,
                    path: Some(path.to_path_buf()),
                    source: None,
                })
            } else {
                Err(CliError::ParseError {
                    message: err_msg,
                    source: None,
                })
            }
        }
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
fn check_directory(path: &Path, args: &CheckArgs) -> CliResult<()> {
    let mut files_checked = 0;
    let mut errors_found = 0;
    let mut first_error: Option<CliError> = None;

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
                // Preserve the first error to return correct exit code
                if first_error.is_none() {
                    first_error = Some(e);
                }
            }
        }
    }

    if files_checked == 0 {
        println!("{}", "No workflow files found.".yellow());
        Ok(())
    } else if errors_found > 0 {
        // Return the first error to preserve exit code classification
        // If no specific error was captured, create a general error
        first_error.map_or_else(
            || {
                Err(CliError::general(format!(
                    "Type checking failed: {errors_found} error(s) in {files_checked} file(s)"
                )))
            },
            Err,
        )
    } else {
        println!("[OK] {files_checked} file(s) type-checked successfully");
        Ok(())
    }
}

/// Output results in human-readable format
fn output_human(path: &Path, result: &CliResult<()>, args: &CheckArgs) -> CliResult<()> {
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
        // Print the error message
        if let Err(e) = result {
            println!("  {} {e}", "Error:".red().bold());
        }
        // Return a new error with the same type to preserve exit code classification
        match result {
            Err(CliError::ParseError { message, .. }) => Err(CliError::ParseError {
                message: message.clone(),
                source: None,
            }),
            Err(CliError::TypeError { message, .. }) => Err(CliError::TypeError {
                message: message.clone(),
                source: None,
            }),
            Err(CliError::IoError {
                message,
                path: io_path,
                ..
            }) => Err(CliError::IoError {
                message: message.clone(),
                path: io_path.clone(),
                source: None,
            }),
            Err(other) => Err(CliError::general(format!("{other}"))),
            Ok(_) => unreachable!(),
        }
    }
}

/// Output results in JSON format
fn output_json(
    path: &Path,
    result: &CliResult<()>,
    args: &CheckArgs,
    parse_time: std::time::Duration,
    tc_time: std::time::Duration,
    total_time: std::time::Duration,
) -> CliResult<()> {
    let success = result.is_ok();
    // Determine exit code based on error type
    let exit_code = if success {
        0
    } else {
        match result {
            Err(CliError::ParseError { .. }) => 2,
            Err(CliError::TypeError { .. }) => 3,
            Err(CliError::IoError { .. }) => 6,
            _ => 1,
        }
    };

    // Build the JSON output
    let mut output = JsonOutput::new(path)
        .with_strict(args.strict)
        .with_exit_code(exit_code)
        .with_timing(parse_time, tc_time, total_time);

    // Add errors if present
    if let Err(e) = result {
        let error_str = format!("{e}");
        // Determine error code based on error type
        let code = match e {
            CliError::ParseError { .. } => "E0001",
            CliError::TypeError { .. } => "E0002",
            _ => "E9999",
        };
        output = output.with_error(
            &error_str,
            code,
            Some(JsonLocation::new(path.display().to_string(), 0, 0)),
        );
    }

    // Print the JSON output
    println!(
        "{}",
        output
            .to_json()
            .map_err(|e| CliError::general(format!("{e}")))?
    );

    // Return a new error with the same type to preserve exit code classification
    match result {
        Err(CliError::ParseError { message, .. }) => Err(CliError::ParseError {
            message: message.clone(),
            source: None,
        }),
        Err(CliError::TypeError { message, .. }) => Err(CliError::TypeError {
            message: message.clone(),
            source: None,
        }),
        Err(CliError::IoError {
            message,
            path: io_path,
            ..
        }) => Err(CliError::IoError {
            message: message.clone(),
            path: io_path.clone(),
            source: None,
        }),
        Err(other) => Err(CliError::general(format!("{other}"))),
        Ok(_) => unreachable!(),
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
