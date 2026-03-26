# TASK-293: Align CLI Surface with SPEC-005

## Status: 📝 Planned

## Description

Fix the shipped CLI surface which materially diverges from SPEC-005. Error classes are flattened too aggressively. Missing or mismatched items include --quiet, --color auto|always|never, repeatable -v, multiple command flags, and distinct exit codes. Engine::run/run_file also collapse parse/type/io into ExecError::ExecutionFailed, conflicting with SPEC-021's requirement that observable failure classes remain distinct.

## Specification Reference

- SPEC-005: CLI Specification
- SPEC-021: Observable Behavior Specification

## Dependencies

- ✅ TASK-053: CLI check command
- ✅ TASK-054: CLI run command
- ✅ TASK-076: CLI engine integration

## Critical File Locations

- `crates/ash-cli/src/main.rs:26` - missing --quiet, --color flags
- `crates/ash-cli/src/main.rs:83` - exit codes not distinct
- `crates/ash-cli/src/commands/check.rs:15` - error class flattening
- `crates/ash-cli/src/commands/trace.rs:18` - error class flattening
- `crates/ash-cli/src/commands/repl.rs:10` - error class flattening
- `crates/ash-engine/src/error.rs:8` - ExecutionFailed collapse

## Requirements

### Functional Requirements

1. Add missing `--quiet` flag
2. Add missing `--color auto|always|never` flag
3. Support repeatable `-v` for verbosity levels
4. Implement distinct exit codes per error class
5. Preserve parse/type/io error distinction in Engine
6. Align all command surfaces with SPEC-005

### Current State (Broken)

**File:** `crates/ash-cli/src/main.rs:26`

```rust
#[derive(Parser)]
#[command(name = "ash")]
struct Cli {
    // MISSING: --quiet flag
    // MISSING: --color flag
    // MISSING: repeatable -v support
    
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Check { ... },
    Run { ... },
    Trace { ... },
    Repl { ... },
}
```

**File:** `crates/ash-engine/src/error.rs:8`

```rust
#[derive(Error, Debug)]
pub enum EngineError {
    // WRONG: All errors collapsed into one variant
    #[error("Execution failed: {0}")]
    ExecutionFailed(String),  // Collapses parse/type/io errors
    
    // MISSING: Distinct error variants
    // ParseError(...)
    // TypeError(...)
    // IoError(...)
}
```

Problems:
1. Missing CLI flags from SPEC-005
2. Exit codes not per SPEC-021
3. Error classes flattened
4. Observable behavior contracts broken

### Target State (Fixed)

```rust
// crates/ash-cli/src/main.rs

#[derive(Parser)]
#[command(name = "ash")]
#[command(version)]
struct Cli {
    /// Silence all output
    #[arg(long, global = true)]
    quiet: bool,
    
    /// Control colored output
    #[arg(long, value_enum, default_value = "auto", global = true)]
    color: ColorChoice,
    
    /// Increase verbosity (repeatable)
    #[arg(short, long, action = clap::ArgAction::Count, global = true)]
    verbose: u8,
    
    #[command(subcommand)]
    command: Commands,
}

#[derive(ValueEnum, Clone, Debug)]
enum ColorChoice {
    Auto,
    Always,
    Never,
}

#[derive(Subcommand)]
enum Commands {
    Check(CheckArgs),
    Run(RunArgs),
    Trace(TraceArgs),
    Repl(ReplArgs),
}

// crates/ash-engine/src/error.rs

#[derive(Error, Debug)]
pub enum EngineError {
    #[error("Parse error: {0}")]
    Parse(ParseError),
    
    #[error("Type error: {0}")]
    Type(TypeError),
    
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    
    #[error("Execution error: {0}")]
    Execution(ExecutionError),
    
    #[error("Capability error: {0}")]
    Capability(CapabilityError),
}

impl EngineError {
    /// Get exit code per SPEC-021
    pub fn exit_code(&self) -> ExitCode {
        match self {
            EngineError::Parse(_) => ExitCode::from(2),
            EngineError::Type(_) => ExitCode::from(3),
            EngineError::Io(_) => ExitCode::from(4),
            EngineError::Execution(_) => ExitCode::from(5),
            EngineError::Capability(_) => ExitCode::from(6),
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
use std::fs;
use tempfile::TempDir;

#[test]
fn test_quiet_flag_silences_output() {
    let temp = TempDir::new().unwrap();
    let workflow = r#"workflow test { act print("hello") }"#;
    let path = temp.path().join("test.ash");
    fs::write(&path, workflow).unwrap();
    
    let output = Command::new("cargo")
        .args(&["run", "--", "--quiet", "run", path.to_str().unwrap()])
        .current_dir("/home/dikini/Projects/ash")
        .output()
        .unwrap();
    
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.is_empty(), "Expected no output with --quiet");
}

#[test]
fn test_color_always_produces_ansi_codes() {
    let temp = TempDir::new().unwrap();
    let workflow = r#"workflow test { act print("hello") }"#;
    let path = temp.path().join("test.ash");
    fs::write(&path, workflow).unwrap();
    
    let output = Command::new("cargo")
        .args(&["run", "--", "--color", "always", "run", path.to_str().unwrap()])
        .current_dir("/home/dikini/Projects/ash")
        .output()
        .unwrap();
    
    let stderr = String::from_utf8_lossy(&output.stderr);
    // Should contain ANSI escape codes
    assert!(stderr.contains("\x1b["), "Expected ANSI codes with --color always");
}

#[test]
fn test_color_never_no_ansi_codes() {
    let temp = TempDir::new().unwrap();
    let workflow = r#"workflow test { act print("hello") }"#;
    let path = temp.path().join("test.ash");
    fs::write(&path, workflow).unwrap();
    
    let output = Command::new("cargo")
        .args(&["run", "--", "--color", "never", "run", path.to_str().unwrap()])
        .current_dir("/home/dikini/Projects/ash")
        .output()
        .unwrap();
    
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(!stderr.contains("\x1b["), "Expected no ANSI codes with --color never");
}

#[test]
fn test_repeatable_verbose() {
    let temp = TempDir::new().unwrap();
    let workflow = r#"workflow test { act print("hello") }"#;
    let path = temp.path().join("test.ash");
    fs::write(&path, workflow).unwrap();
    
    // Single -v
    let output1 = Command::new("cargo")
        .args(&["run", "--", "-v", "run", path.to_str().unwrap()])
        .current_dir("/home/dikini/Projects/ash")
        .output()
        .unwrap();
    
    // Multiple -vvv
    let output3 = Command::new("cargo")
        .args(&["run", "--", "-vvv", "run", path.to_str().unwrap()])
        .current_dir("/home/dikini/Projects/ash")
        .output()
        .unwrap();
    
    // More verbose should produce more output
    assert!(output3.stdout.len() >= output1.stdout.len());
}

#[test]
fn test_parse_error_exit_code() {
    let temp = TempDir::new().unwrap();
    let workflow = r#"workflow test { invalid syntax!!! }"#;
    let path = temp.path().join("test.ash");
    fs::write(&path, workflow).unwrap();
    
    let output = Command::new("cargo")
        .args(&["run", "--", "run", path.to_str().unwrap()])
        .current_dir("/home/dikini/Projects/ash")
        .output()
        .unwrap();
    
    // Parse errors should exit with code 2
    assert_eq!(output.status.code(), Some(2));
}

#[test]
fn test_type_error_exit_code() {
    let temp = TempDir::new().unwrap();
    let workflow = r#"
        workflow test {
            let x: Int = "string";  // Type error
        }
    "#;
    let path = temp.path().join("test.ash");
    fs::write(&path, workflow).unwrap();
    
    let output = Command::new("cargo")
        .args(&["run", "--", "run", path.to_str().unwrap()])
        .current_dir("/home/dikini/Projects/ash")
        .output()
        .unwrap();
    
    // Type errors should exit with code 3
    assert_eq!(output.status.code(), Some(3));
}

#[test]
fn test_io_error_exit_code() {
    // Try to run a non-existent file
    let output = Command::new("cargo")
        .args(&["run", "--", "run", "/nonexistent/path.ash"])
        .current_dir("/home/dikini/Projects/ash")
        .output()
        .unwrap();
    
    // IO errors should exit with code 4
    assert_eq!(output.status.code(), Some(4));
}

#[test]
fn test_success_exit_code() {
    let temp = TempDir::new().unwrap();
    let workflow = r#"workflow test { act print("hello") }"#;
    let path = temp.path().join("test.ash");
    fs::write(&path, workflow).unwrap();
    
    let output = Command::new("cargo")
        .args(&["run", "--", "run", path.to_str().unwrap()])
        .current_dir("/home/dikini/Projects/ash")
        .output()
        .unwrap();
    
    // Success should exit with code 0
    assert_eq!(output.status.code(), Some(0));
}

#[test]
fn test_global_flags_on_all_commands() {
    let temp = TempDir::new().unwrap();
    let workflow = r#"workflow test { act print("hello") }"#;
    let path = temp.path().join("test.ash");
    fs::write(&path, workflow).unwrap();
    
    // Test each command accepts global flags
    for cmd in &["check", "run", "trace"] {
        let output = Command::new("cargo")
            .args(&["run", "--", "--quiet", "--color", "never", *cmd, path.to_str().unwrap()])
            .current_dir("/home/dikini/Projects/ash")
            .output()
            .unwrap();
        
        // Should parse successfully (may fail for other reasons)
        assert!(!String::from_utf8_lossy(&output.stderr).contains("unexpected argument"));
    }
}
```

### Step 2: Add Global CLI Flags

**File:** `crates/ash-cli/src/main.rs`

```rust
use clap::{Parser, ValueEnum};

#[derive(Parser)]
#[command(name = "ash")]
#[command(about = "Ash workflow language CLI")]
#[command(version)]
pub struct Cli {
    /// Silence all output except errors
    #[arg(long, global = true)]
    pub quiet: bool,
    
    /// When to use colored output
    #[arg(long, value_enum, default_value = "auto", global = true)]
    pub color: ColorChoice,
    
    /// Increase verbosity (can be repeated)
    #[arg(short, long, action = clap::ArgAction::Count, global = true)]
    pub verbose: u8,
    
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(ValueEnum, Clone, Debug, Copy)]
pub enum ColorChoice {
    Auto,
    Always,
    Never,
}

impl ColorChoice {
    pub fn should_colorize(&self) -> bool {
        match self {
            ColorChoice::Always => true,
            ColorChoice::Never => false,
            ColorChoice::Auto => atty::is(atty::Stream::Stderr),
        }
    }
}
```

### Step 3: Implement Distinct Error Types

**File:** `crates/ash-engine/src/error.rs`

```rust
use thiserror::Error;
use std::process::ExitCode;

#[derive(Error, Debug)]
pub enum EngineError {
    #[error("Parse error:\n{0}")]
    Parse(#[from] ParseError),
    
    #[error("Type error:\n{0}")]
    Type(#[from] TypeError),
    
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    
    #[error("Execution error:\n{0}")]
    Execution(#[from] ExecutionError),
    
    #[error("Capability error:\n{0}")]
    Capability(#[from] CapabilityError),
}

impl EngineError {
    /// Get the SPEC-021 compliant exit code
    pub fn exit_code(&self) -> ExitCode {
        match self {
            EngineError::Parse(_) => ExitCode::from(2),
            EngineError::Type(_) => ExitCode::from(3),
            EngineError::Io(_) => ExitCode::from(4),
            EngineError::Execution(_) => ExitCode::from(5),
            EngineError::Capability(_) => ExitCode::from(6),
        }
    }
    
    /// Get error category for logging
    pub fn category(&self) -> &'static str {
        match self {
            EngineError::Parse(_) => "parse",
            EngineError::Type(_) => "type",
            EngineError::Io(_) => "io",
            EngineError::Execution(_) => "execution",
            EngineError::Capability(_) => "capability",
        }
    }
}
```

### Step 4: Update Command Handlers

**File:** `crates/ash-cli/src/commands/run.rs`

```rust
use std::process::ExitCode;
use crate::main::ColorChoice;

pub fn execute(args: RunArgs, global: &GlobalArgs) -> Result<(), CliError> {
    // Setup output styling based on color choice
    let use_color = global.color.should_colorize();
    
    if !global.quiet {
        eprintln!("{} Running workflow: {}", 
            if use_color { "▸".green() } else { "▸" },
            args.file.display()
        );
    }
    
    // ... execution logic ...
    
    match result {
        Ok(_) => Ok(()),
        Err(e) => {
            // Preserve error type, don't collapse
            report_error(&e, use_color)?;
            std::process::exit(e.exit_code());
        }
    }
}
```

## Verification Steps

- [ ] `cargo test -p ash-cli --test cli_spec_compliance_test` passes
- [ ] `--quiet` flag works on all commands
- [ ] `--color` flag works with auto/always/never
- [ ] `-v` repeatable for verbosity levels
- [ ] Distinct exit codes per error type
- [ ] `cargo clippy --all-targets --all-features` clean
- [ ] `cargo fmt --check` clean

## Dependencies for Next Task

This task outputs:
- SPEC-005 compliant CLI surface
- SPEC-021 compliant exit codes
- Distinct error class preservation

Required by:
- Scripting and automation
- CI/CD integration

## Notes

**Critical Issue**: CLI doesn't follow its own specification.

**Risk Assessment**: Medium-High - affects scripting and observability.

**Implementation Strategy**:
1. First: Add missing global flags
2. Second: Implement distinct error types
3. Third: Add exit code mapping
4. Fourth: Update all commands
