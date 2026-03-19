# TASK-053: ash check Command

## Status: ✅ Complete

## Objective

Implement the `ash check` command for type checking and linting Ash workflow files.

## Test Strategy (TDD)

### Unit Tests

```rust
#[test]
fn test_check_valid_file() {
    let result = CheckCommand::run(&["test_data/valid.ash"]);
    assert!(result.is_ok());
}

#[test]
fn test_check_invalid_file() {
    let result = CheckCommand::run(&["test_data/type_error.ash"]);
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(err.to_string().contains("Type mismatch"));
}

#[test]
fn test_check_json_output() {
    let output = CheckCommand::run_with_output(&["--format", "json", "test.ash"]).unwrap();
    let parsed: serde_json::Value = serde_json::from_str(&output).unwrap();
    assert!(parsed.get("diagnostics").is_some());
}
```

### Integration Tests

```rust
#[test]
fn test_check_all_directory() {
    // Create temp directory with multiple .ash files
    // Run: ash check --all <dir>
    // Verify all files are checked
}

#[test]
fn test_check_strict_mode() {
    // File with warnings but no errors
    // Without --strict: exit code 0
    // With --strict: exit code 1
}
```

## Implementation Notes

### Command Structure

```rust
use clap::Parser;

#[derive(Parser)]
pub struct CheckArgs {
    /// Files or directories to check
    #[arg(required = true)]
    pub paths: Vec<PathBuf>,
    
    /// Check all workflows in directory
    #[arg(long)]
    pub all: bool,
    
    /// Treat warnings as errors
    #[arg(long)]
    pub strict: bool,
    
    /// Output format
    #[arg(long, value_enum, default_value = "human")]
    pub format: OutputFormat,
    
    /// Enable policy conflict detection
    #[arg(long)]
    pub policy_check: bool,
}

#[derive(Clone, Copy, Debug, Default, ValueEnum)]
pub enum OutputFormat {
    #[default]
    Human,
    Json,
    Short,
}
```

### Implementation Steps

1. **Parse arguments** using clap
2. **Discover files** (single file or directory with --all)
3. **Parse each file** using ash-parser
4. **Type check** using ash-typeck
5. **Run lints** using ash-lint
6. **Format output** based on --format
7. **Exit with appropriate code**

### Error Formatting

```rust
pub fn format_diagnostic_human(diag: &Diagnostic) -> String {
    format!(
        "{}: {}\n  --> {}:{}:{}\n   |\n{} | {}\n   | {}\n   |\n   = {}: {}\n",
        diag.severity,
        diag.message,
        diag.file,
        diag.line,
        diag.column,
        diag.line,
        diag.code_line,
        "^".repeat(diag.code_line.len()),
        diag.category,
        diag.explanation
    )
}
```

## Completion Criteria

- [ ] Can check single .ash file
- [ ] Can check directory with --all
- [ ] Human-readable output format
- [ ] JSON output format
- [ ] Short output format
- [ ] Exit code 0 on success
- [ ] Exit code 1 on type errors
- [ ] Exit code 2 on parse errors
- [ ] --strict treats warnings as errors
- [ ] --policy-check enables SMT checking (when implemented)
- [ ] Tests pass
- [ ] Documentation updated

## Dependencies

- TASK-012: Parser core (for parsing)
- TASK-018: Type representation (for type checking)
- TASK-024b: SMT integration (for --policy-check)

## Estimation

6 hours

## Related

- SPEC-005-CLI.md: CLI Specification
- ash-lint crate: For linting integration
