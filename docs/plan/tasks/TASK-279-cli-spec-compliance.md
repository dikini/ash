# TASK-279: Align CLI Surface with SPEC-005

## Status: 📝 Planned

## Description

Align the shipped CLI surface with SPEC-005. Current CLI diverges from the specification with missing flags, incorrect exit codes, and unsupported options. Failures are flattened into exit code 1 instead of preserving parse/type/I/O distinctions.

## Specification Reference

- SPEC-005: CLI Specification

## Dependencies

- ✅ TASK-053: CLI check command
- ✅ TASK-054: CLI run command
- ✅ TASK-055: CLI trace command
- ✅ TASK-056: CLI repl command
- ✅ TASK-278: CLI --input functional

## Requirements

### Functional Requirements

1. Add missing global flags: `--quiet`, `--color auto|always|never`, repeatable `-v`
2. Add `check --policy-check` and short flags
3. Add `run --format`, `--dry-run`, `--timeout`, `--capability`
4. Add `trace --sign`, `--export`, `--provn`, `--cypher`
5. Add `repl --init`, `--config`, `--capability`
6. Implement proper exit codes per SPEC-005 Section 4
7. `main()` must preserve error distinctions

### Missing CLI Items

**Global Flags:**
- `--quiet` / `-q` - Suppress non-error output
- `--color auto|always|never` - Control color output
- `-v`, `-vv`, `-vvv` - Verbose levels (repeatable)

**`ash check`:**
- `--policy-check` - Enable policy verification
- `-s` short for `--strict`
- `-f` short for `--format`

**`ash run`:**
- `--format json|text` - Output format
- `--dry-run` - Validate without executing
- `--timeout SECONDS` - Execution timeout
- `--capability NAME` - Grant capability

**`ash trace`:**
- `--sign` - Cryptographically sign trace
- `--export FORMAT` - Export format
- `--provn` - PROV-N format output
- `--cypher` - Cypher graph output

**`ash repl`:**
- `--init FILE` - Initialize with file
- `--config FILE` - Config file path
- `--capability NAME` - Grant capability

### Exit Codes (SPEC-005 Section 4)

| Code | Meaning |
|------|---------|
| 0 | Success |
| 1 | General error |
| 2 | Parse error |
| 3 | Type error |
| 4 | Verification failure |
| 5 | Runtime error |
| 6 | I/O error |
| 7 | Timeout |
| 127 | Command not found |

### Current State (Broken)

**File:** `crates/ash-cli/src/main.rs`

```rust
fn main() {
    let args = Cli::parse();
    
    let result = match args.command {
        Commands::Check(cmd) => check::execute(cmd),
        Commands::Run(cmd) => run::execute(cmd),
        // ...
    };
    
    // All failures exit with code 1!
    if let Err(e) = result {
        eprintln!("Error: {}", e);
        std::process::exit(1); // Wrong! Should preserve error type
    }
}
```

### Target State (Fixed)

**File:** `crates/ash-cli/src/main.rs`

```rust
use std::process::ExitCode;

fn main() -> ExitCode {
    let args = Cli::parse();
    
    match execute_command(args) {
        Ok(_) => ExitCode::SUCCESS,
        Err(e) => {
            if !args.quiet {
                eprintln!("Error: {}", e);
            }
            e.exit_code()
        }
    }
}

// Error type with exit code mapping
#[derive(Debug)]
pub enum CliError {
    ParseError { .. },      // exit code 2
    TypeError { .. },       // exit code 3
    VerificationError { .. }, // exit code 4
    RuntimeError { .. },    // exit code 5
    IoError { .. },         // exit code 6
    Timeout { .. },         // exit code 7
}

impl CliError {
    pub fn exit_code(&self) -> ExitCode {
        match self {
            CliError::ParseError { .. } => ExitCode::from(2),
            CliError::TypeError { .. } => ExitCode::from(3),
            CliError::VerificationError { .. } => ExitCode::from(4),
            CliError::RuntimeError { .. } => ExitCode::from(5),
            CliError::IoError { .. } => ExitCode::from(6),
            CliError::Timeout { .. } => ExitCode::from(7),
            _ => ExitCode::from(1),
        }
    }
}
```

## TDD Steps

### Step 1: Write Tests (Red)

**File:** `crates/ash-cli/tests/cli_spec_compliance_test.rs`

```rust
//! Tests for CLI SPEC-005 compliance

use std::process::Command;
use tempfile::TempDir;
use std::fs;

#[test]
fn test_exit_code_parse_error() {
    let temp = TempDir::new().unwrap();
    let bad_syntax = temp.path().join("bad.ash");
    fs::write(&bad_syntax, "workflow { bad syntax }").unwrap();
    
    let output = Command::new("cargo")
        .args(&["run", "--bin", "ash", "--", "run"])
        .arg(&bad_syntax)
        .output()
        .unwrap();
    
    assert_eq!(output.status.code(), Some(2)); // Parse error code
}

#[test]
fn test_exit_code_type_error() {
    let temp = TempDir::new().unwrap();
    let bad_types = temp.path().join("bad.ash");
    fs::write(&bad_types, r#"
        workflow test {
            let x = "string" + 42;  // Type error
        }
    "#).unwrap();
    
    let output = Command::new("cargo")
        .args(&["run", "--bin", "ash", "--", "check"])
        .arg(&bad_types)
        .output()
        .unwrap();
    
    assert_eq!(output.status.code(), Some(3)); // Type error code
}

#[test]
fn test_quiet_flag_suppresses_output() {
    let temp = TempDir::new().unwrap();
    let workflow = temp.path().join("test.ash");
    fs::write(&workflow, "workflow test() { decide 42 }").unwrap();
    
    let output = Command::new("cargo")
        .args(&["run", "--bin", "ash", "--", "run", "--quiet"])
        .arg(&workflow)
        .output()
        .unwrap();
    
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.is_empty() || stdout == "42\n");
}

#[test]
fn test_verbose_flag_levels() {
    // Test that -v, -vv, -vvv are accepted
    for flag in &["-v", "-vv", "-vvv"] {
        let output = Command::new("cargo")
            .args(&["run", "--bin", "ash", "--", flag, "--help"])
            .output()
            .unwrap();
        
        assert!(output.status.success(), "Flag {} failed", flag);
    }
}

#[test]
fn test_run_format_flag() {
    let temp = TempDir::new().unwrap();
    let workflow = temp.path().join("test.ash");
    fs::write(&workflow, "workflow test() { decide { \"key\": \"value\" } }").unwrap();
    
    // Test JSON output
    let output = Command::new("cargo")
        .args(&["run", "--bin", "ash", "--", "run", "--format", "json"])
        .arg(&workflow)
        .output()
        .unwrap();
    
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("{")); // Valid JSON output
}

#[test]
fn test_color_flag() {
    // Test --color never works
    let output = Command::new("cargo")
        .args(&["run", "--bin", "ash", "--", "--color", "never", "--help"])
        .output()
        .unwrap();
    
    assert!(output.status.success());
}
```

### Step 2: Define Error Types with Exit Codes

**File:** `crates/ash-cli/src/error.rs`

```rust
use std::process::ExitCode;

#[derive(Debug, Error)]
pub enum CliError {
    #[error("parse error: {message}")]
    ParseError {
        message: String,
        span: Option<Span>,
    },
    
    #[error("type error: {message}")]
    TypeError {
        message: String,
        span: Option<Span>,
    },
    
    #[error("verification failed: {message}")]
    VerificationError {
        message: String,
        details: Vec<String>,
    },
    
    #[error("runtime error: {message}")]
    RuntimeError {
        message: String,
    },
    
    #[error("I/O error: {message}")]
    IoError {
        message: String,
        path: Option<PathBuf>,
    },
    
    #[error("timeout after {seconds}s")]
    Timeout {
        seconds: u64,
    },
    
    #[error("unknown command: {name}")]
    UnknownCommand {
        name: String,
    },
}

impl CliError {
    pub fn exit_code(&self) -> ExitCode {
        use CliError::*;
        ExitCode::from(match self {
            ParseError { .. } => 2,
            TypeError { .. } => 3,
            VerificationError { .. } => 4,
            RuntimeError { .. } => 5,
            IoError { .. } => 6,
            Timeout { .. } => 7,
            UnknownCommand { .. } => 127,
            _ => 1,
        })
    }
}
```

### Step 3: Update CLI Args Structure

**File:** `crates/ash-cli/src/args.rs`

```rust
use clap::{Parser, Subcommand, Args, ValueEnum};

#[derive(Parser)]
#[command(name = "ash")]
#[command(about = "Ash workflow language interpreter")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
    
    /// Suppress non-error output
    #[arg(long, short, global = true)]
    pub quiet: bool,
    
    /// Control color output
    #[arg(long, value_enum, default_value = "auto", global = true)]
    pub color: ColorOption,
    
    /// Increase verbosity (repeatable)
    #[arg(short, action = clap::ArgAction::Count, global = true)]
    pub verbose: u8,
}

#[derive(ValueEnum, Clone, Debug)]
pub enum ColorOption {
    Auto,
    Always,
    Never,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Type-check a workflow
    Check(CheckArgs),
    
    /// Run a workflow
    Run(RunArgs),
    
    /// Trace workflow execution
    Trace(TraceArgs),
    
    /// Start interactive REPL
    Repl(ReplArgs),
    // ...
}

#[derive(Args)]
pub struct CheckArgs {
    /// File to check
    pub file: PathBuf,
    
    /// Enable strict mode
    #[arg(long, short)]
    pub strict: bool,
    
    /// Output format
    #[arg(long, short, value_enum, default_value = "text")]
    pub format: OutputFormat,
    
    /// Enable policy verification
    #[arg(long)]
    pub policy_check: bool,
}

#[derive(Args)]
pub struct RunArgs {
    /// File to run
    pub file: PathBuf,
    
    /// Input JSON file
    #[arg(long, short)]
    pub input: Option<PathBuf>,
    
    /// Output format
    #[arg(long, value_enum, default_value = "text")]
    pub format: OutputFormat,
    
    /// Validate without executing
    #[arg(long)]
    pub dry_run: bool,
    
    /// Execution timeout in seconds
    #[arg(long)]
    pub timeout: Option<u64>,
    
    /// Grant capability (repeatable)
    #[arg(long)]
    pub capability: Vec<String>,
}

#[derive(Args)]
pub struct TraceArgs {
    /// File to trace
    pub file: PathBuf,
    
    /// Input JSON file
    #[arg(long, short)]
    pub input: Option<PathBuf>,
    
    /// Cryptographically sign trace
    #[arg(long)]
    pub sign: bool,
    
    /// Export format
    #[arg(long, value_enum)]
    pub export: Option<ExportFormat>,
    
    /// Output in PROV-N format
    #[arg(long)]
    pub provn: bool,
    
    /// Output Cypher graph
    #[arg(long)]
    pub cypher: bool,
}

#[derive(Args)]
pub struct ReplArgs {
    /// Initialize with file
    #[arg(long)]
    pub init: Option<PathBuf>,
    
    /// Config file path
    #[arg(long)]
    pub config: Option<PathBuf>,
    
    /// Grant capability (repeatable)
    #[arg(long)]
    pub capability: Vec<String>,
}

#[derive(ValueEnum, Clone, Debug)]
pub enum OutputFormat {
    Text,
    Json,
}

#[derive(ValueEnum, Clone, Debug)]
pub enum ExportFormat {
    Json,
    Provn,
    Cypher,
}
```

### Step 4: Update Main Function

**File:** `crates/ash-cli/src/main.rs`

```rust
use std::process::ExitCode;

fn main() -> ExitCode {
    let cli = Cli::parse();
    
    // Set up logging based on verbosity
    init_logging(cli.verbose);
    
    // Set up color output
    init_color(cli.color);
    
    let result = match cli.command {
        Commands::Check(args) => commands::check::execute(args),
        Commands::Run(args) => commands::run::execute(args),
        Commands::Trace(args) => commands::trace::execute(args),
        Commands::Repl(args) => commands::repl::execute(args),
        Commands::Fmt(args) => commands::fmt::execute(args),
        Commands::Dot(args) => commands::dot::execute(args),
        Commands::Lsp(args) => commands::lsp::execute(args),
    };
    
    match result {
        Ok(code) => code,
        Err(e) => {
            if !cli.quiet {
                eprintln!("{}: {}", "error".red().bold(), e);
                
                // Show help for common errors
                if let CliError::UnknownCommand { .. } = &e {
                    eprintln!("\nRun 'ash --help' for available commands.");
                }
            }
            e.exit_code()
        }
    }
}
```

## Verification Steps

- [ ] `cargo test -p ash-cli --test cli_spec_compliance_test` passes
- [ ] `cargo test -p ash-cli` all tests pass
- [ ] Manual verification of exit codes
- [ ] Manual verification of global flags
- [ ] `cargo clippy --all-targets --all-features` clean
- [ ] `cargo fmt --check` clean

## Dependencies for Next Task

This task outputs:
- SPEC-005 compliant CLI
- Proper exit codes
- Missing flags implemented

Required by:
- All CLI-dependent workflows

## Notes

**Critical Issue**: Users rely on exit codes for scripting. Current behavior breaks automation.

**Design Decision**: Use std::process::ExitCode for portable exit code handling.

**Edge Cases**:
- Multiple error types - use most specific
- Quiet mode still shows errors
- Color mode respects terminal capabilities
