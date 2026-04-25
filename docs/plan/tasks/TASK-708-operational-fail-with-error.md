# TASK-708: Operational fail and with_error

## Status: ✅ Complete

## Description

Implement `fail` as operational bottom and `with_error` as scoped operational failure handling.

## Specification Reference

- SPEC-050
- SPEC-004

## Dependencies

- 📝 TASK-706: prerequisite task

## Requirements

### Functional Requirements

1. ✅ Add explicit surface/core carriers for `fail` and `with_error` or document why a different representation is required before coding.
2. ✅ Type `fail e` as bottom-compatible in expression/workflow positions covered by SPEC-050.
3. ✅ Implement scoped dynamic handling that catches operational failures, not ordinary `Result` values.
4. ✅ Preserve lower failure identity/cause when handlers reinterpret failures.

### Property Requirements (proptest)

```rust
// Add property-based tests for identity preservation, handle linearity,
// failure aggregation, environment projection, or typing invariants where
// this task manipulates those semantics.
```

## TDD Steps

### Step 1: Write Tests (Red)

✅ Added failing tests before implementation:

- `crates/ash-parser/tests/task_708_fail_with_error.rs`
- `crates/ash-typeck/tests/task_708_operational_bottom.rs`
- `crates/ash-interp/tests/task_708_operational_fail.rs`

### Step 2: Implement (Green)

✅ Implemented the minimal cross-layer expression slice while preserving the semantic tower split:

- parser surface and core `Expr::Fail` / `Expr::WithError` carriers
- parser syntax for `fail payload` and `with_error { body } handle { arms }`
- lowering to core carriers
- bottom-compatible type checking for `fail`
- handler/body type unification for `with_error`
- synchronous and async interpreter handling via `EvalError::OperationalFailure`

### Step 3: Integration (Green)

✅ Wired through affected Ash layers without collapsing Act, Proc, or Workflow boundaries. `with_error` catches only the dedicated operational failure carrier and does not catch ordinary Ash domain values such as `Err` variants.

### Step 4: Property Tests (Verify)

Not added in this slice. TASK-708 established focused regression tests for the cross-layer carrier, typing, dynamic handling, domain-error separation, and cause-preservation invariants; broader failure algebra/process aggregation proptests belong with later process-observation tasks.

## Verification Steps

- [x] Failing tests cover bottom typing and branch unification.
- [x] Handlers catch failures raised in dynamic scope and do not catch ordinary `Err` values.
- [x] Existing `panic` behavior is explicitly separated from `fail`; `panic` remains a parser-only surface expression rejected by lowering, while `fail` lowers to an operational runtime carrier.
- [x] `cargo test --all` passes
- [x] `cargo clippy --all-targets --all-features` passes cleanly
- [x] `cargo fmt --check` passes

Focused verification run during implementation:

```text
cargo check --workspace
cargo test -p ash-parser --test task_708_fail_with_error -- --nocapture
cargo test -p ash-typeck --test task_708_operational_bottom -- --nocapture
cargo test -p ash-interp --test task_708_operational_fail -- --nocapture
cargo fmt --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all
```

## Dependencies for Next Task

This task outputs substrate needed by its direct dependents in PLAN-098.

## Notes

- Preserve existing workflow/control-link behavior unless this task explicitly changes it.
- Keep `Proc<A>` distinct from `Act<A>` and `Workflow`.
- Update `CHANGELOG.md` for implementation/tooling/docs-policy changes made while completing this task.
