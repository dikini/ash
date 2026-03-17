# TASK-054: ash run Command

## Status: 🟢 Complete

## Objective

Implement the `ash run` command for executing Ash workflows.

## Test Strategy (TDD)

```rust
#[test]
fn test_run_simple_workflow() {
    let result = RunCommand::run(&["test_data/hello.ash"]);
    assert!(result.is_ok());
}

#[test]
fn test_run_with_input() {
    let result = RunCommand::run(&[
        "--input", "test_data/input.json",
        "test_data/process.ash"
    ]);
    assert!(result.is_ok());
}

#[test]
fn test_run_dry_run() {
    // Should validate but not execute
    let result = RunCommand::run(&["--dry-run", "test_data/valid.ash"]);
    assert!(result.is_ok());
}

#[test]
fn test_run_timeout() {
    let start = Instant::now();
    let result = RunCommand::run(&["--timeout", "1", "test_data/infinite.ash"]);
    let elapsed = start.elapsed();
    
    assert!(result.is_err() || elapsed < Duration::from_secs(2));
}
```

## Implementation Notes

```rust
#[derive(Parser)]
pub struct RunArgs {
    /// Workflow file to run
    #[arg(required = true)]
    pub workflow: PathBuf,
    
    /// JSON input file
    #[arg(short, long)]
    pub input: Option<PathBuf>,
    
    /// Output file (default: stdout)
    #[arg(short, long)]
    pub output: Option<PathBuf>,
    
    /// Output format
    #[arg(short, long, value_enum, default_value = "json")]
    pub format: OutputFormat,
    
    /// Validate only, don't execute
    #[arg(long)]
    pub dry_run: bool,
    
    /// Maximum execution time in seconds
    #[arg(long)]
    pub timeout: Option<u64>,
    
    /// Enable provenance tracing
    #[arg(long)]
    pub trace: bool,
    
    /// Capability bindings
    #[arg(long, value_parser = parse_capability_binding)]
    pub capability: Vec<(String, String)>,
}
```

### Execution Flow

1. Parse workflow file
2. Type check
3. Bind capabilities (from args or config)
4. Parse input (if provided)
5. Execute with interpreter
6. Format and output result
7. Handle timeout/cancellation

## Completion Criteria

- [ ] Can run single workflow file
- [ ] Accepts JSON input via --input
- [ ] Accepts inline input
- [ ] Outputs to file or stdout
- [ ] Multiple output formats (json, yaml, raw)
- [ ] --dry-run validates without executing
- [ ] --timeout enforces time limit
- [ ] --trace enables provenance capture
- [ ] --capability binds providers
- [ ] Tests pass
- [ ] Documentation updated

## Dependencies

- TASK-053: ash check (for validation)
- TASK-035: Capability provider trait
- TASK-036: Runtime policy evaluation

## Estimation

8 hours
