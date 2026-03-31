# TASK-366: Redefine `ash run` Entry-Point Semantics

## Status: ⛔ Blocked

## Description

**CRITICAL: This is NOT implementing `ash run` - it already exists in `main.rs:52-77` and `run.rs:23-113`.**

This task **redefines** the `ash run` command to use the new entry-point bootstrap (TASK-363c) instead of current behavior.

**VALIDATION GATE - REQUIRED BEFORE IMPLEMENTATION:**

1. **Verify S57-2 (CLI policy)**: ✅ Complete - confirms command semantics
2. **Verify existing `ash run`**: Review current implementation in `main.rs`/`run.rs`
3. **Confirm migration path**: Ensure new bootstrap is drop-in replacement

## Current State

`ash run` exists but likely uses old execution model. Need to:

1. Identify current `ash run` implementation
2. Determine what needs changing
3. Migrate to TASK-363c bootstrap

## Redefinition Scope

Update `ash run` to:

```bash
# Usage (unchanged or clarified)
ash run <file> [-- <args>...]

# Examples
ash run hello.ash
ash run myapp.ash -- --flag value
```

## Implementation

Replace current `ash run` handler with:

```rust
// In run.rs or main.rs
fn cmd_run(file: &Path, args: Vec<String>) -> i32 {
    // Set up args for injection (future capability creation)
    std::env::set_var("ASH_ARGS", args.join("\0"));
    
    // Use new bootstrap
    match bootstrap(file) {  // TASK-363c
        Ok(exit_code) => exit_code,
        Err(e) => {
            eprintln!("error: {}", e);
            1
        }
    }
}
```

## Changes from Current

| Aspect | Current (assumed) | New |
|--------|-------------------|-----|
| Entry point | ? | `main` workflow |
| Return type | ? | `Result<(), RuntimeError>` |
| Args passing | ? | `cap Args` injection |
| Stdlib | ? | standard-library root modules loaded |

## TDD Steps

### Test 1: Command Still Works

```rust
// Verify `ash run` executes
let output = Command::new("ash")
    .args(["run", "test.ash"])
    .output()
    .expect("command runs");
assert!(output.status.code().is_some());
```

### Test 2: New Semantics Active

```rust
// Test.ash contains `main` workflow
let output = Command::new("ash")
    .args(["run", "test.ash"])
    .output()?;

// If test.ash has `Ok(())`, exits 0
assert_eq!(output.status.code(), Some(0));
```

## Dependencies

- S57-2: CLI policy
- TASK-363c: Bootstrap ready
- Existing `ash run` command structure

## Migration Notes

- **Preserve**: CLI argument parsing
- **Replace**: Execution engine call with bootstrap
- **Update**: Help text if needed

## Spec Citations

| Aspect | Spec |
|--------|------|
| CLI syntax | SPEC-005 after S57-2 |
| Entry contract | SPEC-003/022 after S57-6 |

## Acceptance Criteria

- [ ] S57-2 shows ✅ Complete (VALIDATION GATE)
- [ ] Existing `ash run` command structure preserved
- [ ] Uses TASK-363c bootstrap
- [ ] Args passed correctly
- [ ] Exit codes propagated
- [ ] Tests pass

## Est. Hours: 2-3
