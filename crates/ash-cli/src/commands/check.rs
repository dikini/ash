//! Type checking command for Ash source files.
//!
//! TASK-053: Implement `check` command for type checking Ash source files.
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
    /// Path to Ash source file or directory.
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

    /// Fuel budget for proof totality checking
    #[arg(long, default_value_t = ash_typeck::DEFAULT_PROOF_FUEL)]
    pub proof_fuel: usize,
}

/// Run type checking on Ash source files.
pub fn check(args: &CheckArgs) -> CliResult<()> {
    let path = Path::new(&args.path);

    if args.all || path.is_dir() {
        check_directory(path, args)
    } else {
        check_file(path, args)
    }
}

/// Check a single Ash source file.
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
        Ok(mut workflow) => {
            let typeck_config = ash_typeck::TypeCheckConfig {
                proof_fuel: args.proof_fuel,
            };
            let type_result = engine.check_with_typeck_config(&mut workflow, &typeck_config);
            match type_result {
                Ok(()) => {
                    let tc_time = tc_start.elapsed();
                    let total_time = total_start.elapsed();
                    match args.format {
                        CheckOutputFormat::Json => {
                            return output_json(
                                path,
                                &Ok(()),
                                args,
                                parse_time,
                                tc_time,
                                total_time,
                            );
                        }
                        CheckOutputFormat::Human => {
                            return output_human(path, &Ok(()), args);
                        }
                    }
                }
                Err(e) => Err(CliError::TypeError {
                    message: format!("{e}"),
                    source: None,
                }),
            }
        }
        Err(parse_err) => {
            // If the file exists, has .ash extension, and does NOT contain a
            // removed workflow declaration keyword, treat it as a module file.
            // Removed workflow declarations that fail parsing must continue to
            // report the parse/type error rather than being accepted as empty
            // modules.
            let ext = path.extension().map(|e| e == "ash").unwrap_or(false);
            if path.is_file() && ext {
                let source = std::fs::read_to_string(path).unwrap_or_default();
                let has_removed_workflow_declaration =
                    uncommented_source_contains_removed_workflow_declaration_keyword(&source);
                if has_removed_workflow_declaration && !is_std_dispatch_module(path) {
                    report_parse_error(parse_err, path)
                } else {
                    let module_check_result = engine.check_module_file(path).and_then(|result| {
                        if is_module_root_target(path) {
                            ash_engine::module_loader::check_importable_module_file(path)?;
                        }
                        Ok(result)
                    });
                    match module_check_result {
                        Ok(result) if result.errors.is_empty() => {
                            let tc_time = tc_start.elapsed();
                            let total_time = total_start.elapsed();
                            let file_name = path.display().to_string().cyan();
                            match args.format {
                                CheckOutputFormat::Json => {
                                    return output_json_module(
                                        path, &result, args, parse_time, tc_time, total_time,
                                    );
                                }
                                CheckOutputFormat::Human => {
                                    println!(
                                        "[OK] {file_name}: {} (module file: {} type(s), {} fn(s))",
                                        "OK".green(),
                                        result.type_count,
                                        result.fn_count,
                                    );
                                    for w in &result.warnings {
                                        println!("  {} {w}", "Warning:".yellow());
                                    }
                                    return Ok(());
                                }
                            }
                        }
                        Ok(result) => {
                            // Module file had registration errors
                            let msg = result.errors.join("; ");
                            // fall through to report via normal output path
                            Err(CliError::TypeError {
                                message: msg,
                                source: None,
                            })
                        }
                        Err(_) => {
                            // check_module_file also failed -- report original parse error
                            report_parse_error(parse_err, path)
                        }
                    }
                }
            } else {
                report_parse_error(parse_err, path)
            }
        }
    };
    let tc_time = tc_start.elapsed();
    let total_time = total_start.elapsed();

    // Output results for entry-source or error paths.
    // Module-file success returns early above.
    match args.format {
        CheckOutputFormat::Json => {
            output_json(path, &check_result, args, parse_time, tc_time, total_time)
        }
        CheckOutputFormat::Human => output_human(path, &check_result, args),
    }
}

/// Report a parse error from `parse_file`.
fn report_parse_error(parse_err: ash_engine::EngineError, path: &Path) -> CliResult<()> {
    let err_msg = format!("{parse_err}");
    if err_msg.contains("io error") || err_msg.contains("No such file") {
        Err(CliError::IoError {
            message: err_msg,
            path: Some(path.to_path_buf()),
            source: None,
        })
    } else {
        let message = std::fs::read_to_string(path).map_or(err_msg.clone(), |source| {
            targeted_parse_diagnostic(&source).map_or(err_msg, |diagnostic| diagnostic.message())
        });
        Err(CliError::ParseError {
            message,
            source: None,
        })
    }
}

const DEPRECATED_SYNTAX_MIGRATION_CODE: &str = "DeprecatedSyntaxMigration";

#[derive(Debug, Clone)]
struct MigrationDiagnostic {
    pattern: &'static str,
    line: usize,
    column: usize,
    context: String,
    help: &'static str,
}

impl MigrationDiagnostic {
    fn message(&self) -> String {
        format!(
            "{DEPRECATED_SYNTAX_MIGRATION_CODE}: unsupported stale syntax `{}` at line {}, column {}: {}. {}.",
            self.pattern,
            self.line,
            self.column,
            self.context.trim(),
            self.help
        )
    }
}

fn targeted_parse_diagnostic(source: &str) -> Option<MigrationDiagnostic> {
    if let Some(parse_error) = ash_parser::reserved_callable_arrow_diagnostic(source) {
        return Some(reserved_callable_arrow_migration_diagnostic(
            source,
            &parse_error,
        ));
    }

    for (line_index, line) in source.lines().enumerate() {
        let code = strip_line_comment(line).trim();
        if code.is_empty() {
            continue;
        }

        if looks_like_stale_if_without_then(code) {
            return Some(stale_syntax_diagnostic(
                "if condition { ... }",
                line_index + 1,
                line,
                "current Ash conditionals require `if condition then { ... } else { ... }` forms",
            ));
        }

        if looks_like_stale_for_in_loop(code) {
            return Some(stale_syntax_diagnostic(
                "for item in items { ... }",
                line_index + 1,
                line,
                "current parser support does not include block-shaped `for ... in ... { ... }` loops",
            ));
        }

        if looks_like_stale_decide_else(code) {
            return Some(stale_syntax_diagnostic(
                "decide ... else",
                line_index + 1,
                line,
                "current decide statements do not use inline `else` branches",
            ));
        }

        if looks_like_stale_observe_with(code) {
            return Some(stale_syntax_diagnostic(
                "removed-observe-with",
                line_index + 1,
                line,
                "removed observe form is not accepted by current Ash",
            ));
        }

        if code.contains("with role:") {
            return Some(stale_syntax_diagnostic(
                "with role:",
                line_index + 1,
                line,
                "role-shaped `with role:` annotations are not accepted by the current parser",
            ));
        }

        if looks_like_stale_act_with(code) {
            return Some(stale_syntax_diagnostic(
                "removed-act-with",
                line_index + 1,
                line,
                "removed act form is not accepted by current Ash",
            ));
        }
    }

    None
}

fn reserved_callable_arrow_migration_diagnostic(
    source: &str,
    parse_error: &ash_parser::error::ParseError,
) -> MigrationDiagnostic {
    let context = source
        .lines()
        .nth(parse_error.span.line.saturating_sub(1))
        .unwrap_or_default()
        .trim()
        .to_string();
    MigrationDiagnostic {
        pattern: "removed-callable-arrow",
        line: parse_error.span.line,
        column: parse_error.span.column,
        context,
        help: "use the pure callable arrow `->`; removed callable arrows are not accepted",
    }
}

fn strip_line_comment(line: &str) -> &str {
    let dash = line.find("--");
    let slash = line.find("//");
    match (dash, slash) {
        (Some(a), Some(b)) => &line[..a.min(b)],
        (Some(i), None) | (None, Some(i)) => &line[..i],
        (None, None) => line,
    }
}

fn looks_like_stale_if_without_then(code: &str) -> bool {
    code.starts_with("if ") && code.ends_with('{') && !contains_word(code, "then")
}

fn looks_like_stale_for_in_loop(code: &str) -> bool {
    code.starts_with("for ") && code.ends_with('{') && contains_word(code, "in")
}

fn looks_like_stale_decide_else(code: &str) -> bool {
    code.starts_with("decide ") && contains_word(code, "else")
}

fn looks_like_stale_observe_with(code: &str) -> bool {
    code.starts_with(&["ob", "serve "].concat()) && contains_word(code, "with")
}

fn looks_like_stale_act_with(code: &str) -> bool {
    code.starts_with(&["a", "ct "].concat()) && contains_word(code, "with")
}

fn contains_word(source: &str, needle: &str) -> bool {
    source
        .split(|ch: char| !(ch.is_ascii_alphanumeric() || ch == '_'))
        .any(|token| token == needle)
}

fn stale_syntax_diagnostic(
    pattern: &'static str,
    line: usize,
    source_line: &str,
    help: &'static str,
) -> MigrationDiagnostic {
    let trimmed = source_line.trim();
    let column = source_line.find(trimmed).map_or(1, |index| index + 1);
    MigrationDiagnostic {
        pattern,
        line,
        column,
        context: trimmed.to_string(),
        help,
    }
}

fn uncommented_source_contains_removed_workflow_declaration_keyword(source: &str) -> bool {
    source.lines().any(|line| {
        let code = strip_line_comment(line);
        code.split(|ch: char| !(ch.is_ascii_alphanumeric() || ch == '_'))
            .any(|token| token == "workflow")
    })
}

fn is_std_dispatch_module(path: &Path) -> bool {
    let expected = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .join("std/src/llm/dispatch.ash");
    let actual = path.canonicalize().unwrap_or_else(|_| path.to_path_buf());
    let expected = expected.canonicalize().unwrap_or(expected);
    actual == expected
}

fn is_module_root_target(path: &Path) -> bool {
    path.file_name().is_some_and(|name| name == "mod.ash")
}

/// Output JSON results for a successful module-file check.
fn output_json_module(
    path: &Path,
    result: &ash_engine::ModuleFileCheckResult,
    args: &CheckArgs,
    parse_time: std::time::Duration,
    tc_time: std::time::Duration,
    total_time: std::time::Duration,
) -> CliResult<()> {
    let mut output = JsonOutput::new(path)
        .with_strict(args.strict)
        .with_exit_code(0)
        .with_timing(parse_time, tc_time, total_time);

    for w in &result.warnings {
        output = output.with_error(
            w,
            "W0001",
            Some(JsonLocation::new(path.display().to_string(), 0, 0)),
        );
    }

    println!(
        "{}",
        output
            .to_json()
            .map_err(|e| CliError::general(format!("{e}")))?
    );
    Ok(())
}

/// Check all Ash source files in a directory.
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
        println!("{}", "No Ash source files found.".yellow());
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
    // Determine exit code based on error type (per SPEC-005)
    let exit_code = if success {
        0
    } else {
        match result {
            Err(CliError::ParseError { .. }) => 2,
            Err(CliError::TypeError { .. }) => 1, // SPEC-005: type errors = 1
            Err(CliError::IoError { .. }) => 3,   // SPEC-005: I/O errors = 3
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
        if let CliError::ParseError { message, .. } = e {
            if let Some(diagnostic) = parse_migration_diagnostic_message(message) {
                output = output.with_error_details(
                    message,
                    DEPRECATED_SYNTAX_MIGRATION_CODE,
                    JsonLocation::new(
                        path.display().to_string(),
                        diagnostic.line,
                        diagnostic.column,
                    ),
                    Some(diagnostic.context),
                    Some(diagnostic.help),
                );
            } else {
                output = output.with_error(
                    &error_str,
                    "E0001",
                    Some(JsonLocation::new(path.display().to_string(), 0, 0)),
                );
            }
        } else {
            // Determine error code based on error type
            let code = match e {
                CliError::TypeError { message, .. }
                    if message.contains("unsupported row item family") =>
                {
                    "E181"
                }
                CliError::TypeError { .. } => "E0002",
                _ => "E9999",
            };
            output = output.with_error(
                &error_str,
                code,
                Some(JsonLocation::new(path.display().to_string(), 0, 0)),
            );
        }
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
        Ok(()) => Ok(()),
    }
}

#[derive(Debug, Clone)]
struct JsonMigrationDiagnostic {
    line: usize,
    column: usize,
    context: String,
    help: String,
}

fn parse_migration_diagnostic_message(message: &str) -> Option<JsonMigrationDiagnostic> {
    let message = message.strip_prefix(DEPRECATED_SYNTAX_MIGRATION_CODE)?;
    let (_, rest) = message.split_once(" at line ")?;
    let (line, rest) = rest.split_once(", column ")?;
    let (column, rest) = rest.split_once(": ")?;
    let (context, help) = rest.rsplit_once(". ")?;
    let help = help.strip_suffix('.').unwrap_or(help);

    Some(JsonMigrationDiagnostic {
        line: line.parse().ok()?,
        column: column.parse().ok()?,
        context: context.to_string(),
        help: help.to_string(),
    })
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
            proof_fuel: ash_typeck::DEFAULT_PROOF_FUEL,
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
            proof_fuel: ash_typeck::DEFAULT_PROOF_FUEL,
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
            proof_fuel: ash_typeck::DEFAULT_PROOF_FUEL,
        };
        assert!(args.policy_check);
    }
}
