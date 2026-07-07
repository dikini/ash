//! Conservative Ash source formatting command.

use crate::error::{CliError, CliResult};
use clap::Args;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::process::ExitCode;
use walkdir::WalkDir;

/// Format Ash source files.
#[derive(Debug, Clone, Args)]
pub struct FmtArgs {
    /// File or directory to format.
    #[arg(value_name = "PATH", required_unless_present = "stdin")]
    pub path: Option<PathBuf>,

    /// Check formatting without modifying files.
    #[arg(long, short)]
    pub check: bool,

    /// Write formatted output back to files.
    #[arg(long, short)]
    pub write: bool,

    /// Read source from stdin and write formatted source to stdout.
    #[arg(long)]
    pub stdin: bool,

    /// Number of spaces for indentation. Reserved for the AST formatter.
    #[arg(long, default_value_t = 4)]
    pub indent: usize,
}

/// Run `ash fmt`.
pub fn fmt(args: &FmtArgs) -> CliResult<ExitCode> {
    if !(1..=16).contains(&args.indent) {
        return Err(CliError::general("fmt --indent must be between 1 and 16"));
    }

    if args.stdin {
        return fmt_stdin(args);
    }

    let path = args
        .path
        .as_deref()
        .ok_or_else(|| CliError::general("fmt requires a path unless --stdin is used"))?;

    if path.is_file() {
        return fmt_file(path, args);
    }

    if path.is_dir() {
        return fmt_dir(path, args);
    }

    Err(CliError::general(format!(
        "fmt target does not exist: {}",
        path.display()
    )))
}

fn fmt_stdin(args: &FmtArgs) -> CliResult<ExitCode> {
    let mut source = String::new();
    std::io::stdin()
        .read_to_string(&mut source)
        .map_err(|error| CliError::io("failed to read stdin", Some(PathBuf::from("-")), error))?;
    let formatted = format_source(&source, args.indent)?;

    if args.check && source != formatted {
        eprintln!("<stdin>: would reformat");
        return Ok(ExitCode::FAILURE);
    }

    print!("{formatted}");
    Ok(ExitCode::SUCCESS)
}

fn fmt_dir(path: &Path, args: &FmtArgs) -> CliResult<ExitCode> {
    let mut clean = true;
    for entry in WalkDir::new(path) {
        let entry = entry.map_err(|error| {
            CliError::io(
                format!("failed to walk {}", path.display()),
                Some(path.to_path_buf()),
                error,
            )
        })?;
        let file_path = entry.path();
        if file_path.is_file()
            && file_path.extension().is_some_and(|ext| ext == "ash")
            && fmt_file(file_path, args)? != ExitCode::SUCCESS
        {
            clean = false;
        }
    }

    Ok(if clean {
        ExitCode::SUCCESS
    } else {
        ExitCode::FAILURE
    })
}

fn fmt_file(path: &Path, args: &FmtArgs) -> CliResult<ExitCode> {
    let source = std::fs::read_to_string(path)
        .map_err(|error| CliError::io("failed to read source", Some(path.to_path_buf()), error))?;
    let formatted = format_source(&source, args.indent)?;

    if source == formatted {
        return Ok(ExitCode::SUCCESS);
    }

    if args.check {
        eprintln!("{}: would reformat", path.display());
        return Ok(ExitCode::FAILURE);
    }

    if args.write {
        std::fs::write(path, formatted).map_err(|error| {
            CliError::io(
                "failed to write formatted source",
                Some(path.to_path_buf()),
                error,
            )
        })?;
        eprintln!("{}: reformatted", path.display());
        return Ok(ExitCode::SUCCESS);
    }

    print!("{formatted}");
    Ok(ExitCode::SUCCESS)
}

/// Format source with deterministic whitespace normalization.
pub fn format_source(source: &str, _indent: usize) -> CliResult<String> {
    if let Some(pattern) = deprecated_pattern(source) {
        return Err(CliError::general(format!(
            "unsupported deprecated syntax in formatter input: {pattern}"
        )));
    }

    let mut lines = Vec::new();
    let mut previous_blank = false;
    for line in source.lines() {
        let trimmed = line.trim_end();
        let blank = trimmed.is_empty();
        if blank {
            if !previous_blank && !lines.is_empty() {
                lines.push(String::new());
            }
        } else {
            lines.push(trimmed.to_string());
        }
        previous_blank = blank;
    }

    while lines.last().is_some_and(|line| line.is_empty()) {
        lines.pop();
    }

    let mut formatted = lines.join("\n");
    formatted.push('\n');
    Ok(formatted)
}

fn deprecated_pattern(source: &str) -> Option<&'static str> {
    for line in source.lines() {
        let code = strip_line_comment(line);
        if contains_token_followed_by_with(code, "observe") {
            return Some("observe ... with");
        }
        if contains_token_followed_by_with(code, "act") {
            return Some("act ... with");
        }
        for pattern in [
            "Proc<",
            "Act<",
            "Workflow<",
            "legacy workflow",
            "ambient authority",
            "direct provider",
        ] {
            if line.contains(pattern) {
                return Some(pattern);
            }
        }
    }
    None
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

fn contains_token_followed_by_with(line: &str, token: &str) -> bool {
    let mut saw_token = false;
    for part in line.split(|ch: char| !(ch.is_ascii_alphanumeric() || ch == '_')) {
        if part.is_empty() {
            continue;
        }
        if saw_token && part == "with" {
            return true;
        }
        saw_token |= part == token;
    }
    false
}
