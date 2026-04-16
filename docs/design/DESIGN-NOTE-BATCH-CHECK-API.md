# Design Note: Batch `ash check` API vs Per-File Subprocess

## Status: Draft (Decision D2)

## 1. Question

For example-syntax conformance, should the spec processor:

- **(A)** Spawn `ash check <file>` as a subprocess for every `.ash` file?
- **(B)** Call a Rust engine API that checks multiple files in one invocation?
- **(C)** Use a hybrid: subprocess for integration-level testing, engine API for batch internal validation?

## 2. Analysis

### 2.1 Why not (B) engine API first

A batch `engine.check_files(paths)` API does not exist today. Designing and implementing it would require:
- Multi-file type-checker context sharing
- Cross-module dependency graph caching
- Error collection and attribution per file

This is a large substrate task (estimated 12–20h) that blocks the spec processor indefinitely. It also conflates two concerns: *making the processor work* and *optimizing the processor*.

### 2.2 Why not (C) hybrid

A hybrid approach creates two code paths with divergent error formats and behavior. The spec processor would need to normalize engine-API errors and subprocess errors into a single representation. This adds complexity before the core validation logic is proven.

### 2.3 Resolution: (A) Per-file subprocess, with a clear migration path

The spec processor **initially uses `std::process::run("ash", ["check", path])` for each file**.

**Rationale:**
- It works immediately once `std::process` is available.
- It exercises the real CLI surface, catching CLI-specific bugs that an engine API would hide.
- It produces the same error format users see, making processor output directly actionable.
- Performance is acceptable for the MVP (the example directory is < 50 files).

**Migration path:**
- Track A6 (example conformance) is explicitly labeled as *subprocess-based*.
- A future task `TASK-NNN-batch-check-api` is added to PLAN-INDEX as a **performance optimization**, not a prerequisite.
- When the batch API exists, the processor's `capability_boundary.ash` gains a `batch_check_api` flag. The processor switches internally.

## 3. Interface contract

```ash
// Spec processor internal helper
fn check_example(path: String) -> Result<(), ExampleError> {
    let output = process::run("ash", ["check", path]);
    if output.status == 0 {
        Ok(())
    } else {
        Err(ExampleError { path, stderr: output.stderr })
    }
}
```

## 4. Error handling

- If `ash check` exits non-zero, the processor captures stdout/stderr verbatim.
- If `ash check` is not found in `PATH`, the processor emits a `ToolingGap` finding.
- Timeout is handled by the `Process` capability provider (see DESIGN-NOTE-PROCESS-EFFECT.md).

## 5. Decision

**Adopt option (A).** The spec processor uses per-file subprocess invocation for example conformance. A batch engine API is an explicit future optimization, not a blocker.

This unblocks:
- Task A6 (example-syntax conformance)
- Gate D2 merge point
- End-to-end integration testing of the processor
