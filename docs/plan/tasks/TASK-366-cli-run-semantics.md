# TASK-366: Redefine `ash run` Entry-Point Semantics

## Status: ⛔ Pending broader `ash run` migration and CLI alignment

## Description

**CRITICAL: This is NOT introducing `ash run` from scratch.** The command surface and its
current CLI execution slice are already landed in `ash-cli`; this task remains the downstream
migration that redefines the normal `ash run` path around the designated-entry bootstrap.

This task **redefines** the existing `ash run` command to use the new entry-point bootstrap
(TASK-363c) instead of the current direct parse/check/execute path.

**VALIDATION GATE - REQUIRED BEFORE IMPLEMENTATION:**

1. **Verify S57-2 (CLI policy)**: ✅ Complete - confirms command semantics
2. **Verify existing `ash run`**: Review current implementation in `main.rs`/`run.rs`
3. **Confirm migration path**: Ensure new bootstrap is drop-in replacement

## Current State

This is currently a **partial, narrow migration** rather than a full run-path cutover.

- **Already migrated:** detected entry sources in the current bootstrap-aware slice already route
    through the designated-entry bootstrap path.
- **Still on the older path:** the broader normal `ash run` execution flow and its full CLI
    behavior alignment are not migrated yet.

The remaining downstream work is to:

1. Preserve the existing command surface and flag behavior
2. Replace the normal execution path with TASK-363c entry bootstrap semantics
3. Thread CLI arguments into the runtime-owned `Args` capability path when that migration lands
4. Keep dry-run, tracing, timeout, and error-reporting behavior aligned with the migrated entry path

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

Migrate the current `ash run` handler from direct workflow execution to:

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

| Aspect | Current landed slice | TASK-366 target |
|--------|----------------------|-----------------|
| Entry point | Direct `engine.run_file()` / traced parse-check-execute path | Canonical `main` workflow bootstrap |
| Return type | Existing workflow execution semantics | Exact `Result<(), RuntimeError>` contract |
| Args passing | No runtime-owned `Args` injection yet | `cap Args` runtime injection |
| Stdlib | No narrow runtime-entry stdlib bootstrap on normal `ash run` | Runtime entry stdlib roots loaded before bootstrap |

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

## Current Boundary

- **Already landed:** the baseline `ash run` CLI command, flag parsing, output handling, and the
    current partial bootstrap-aware slice for detected entry sources
- **Still pending downstream work:** migrating the broader normal `ash run` execution path to the
    designated entry bootstrap contract without regressing CLI-specific behavior or observable
    behavior alignment

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
- [ ] Existing landed `ash run` command structure preserved
- [ ] Uses TASK-363c bootstrap
- [ ] Args passed correctly
- [ ] Exit codes propagated
- [ ] Tests pass

## Est. Hours: 2-3
