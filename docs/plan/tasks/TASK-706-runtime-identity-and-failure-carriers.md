# TASK-706: Runtime identity and failure carriers

## Status: ✅ Complete

## Description

Add the foundational identity and structured failure carriers needed by process and workflow runtime semantics.

## Specification Reference

- SPEC-049
- SPEC-050
- SPEC-051

## Dependencies

- 📝 TASK-705: prerequisite task

## Requirements

### Functional Requirements

1. Define `RunId`, `ProcessId`, and internal `BranchId` newtypes in the appropriate core/runtime module.
2. Define process lifecycle/terminal-state carriers for admitting, running, yielded, succeeded, failed, and cancelled states.
3. Define structured operational/process failure carriers that preserve tower/entity identity and lower causes.
4. Define skeleton workflow boundary carrier types only where needed for dependency stability; do not wire admission yet.

### Property Requirements (proptest)

```rust
// Add property-based tests for identity preservation, handle linearity,
// failure aggregation, environment projection, or typing invariants where
// this task manipulates those semantics.
```

## TDD Steps

### Step 1: Write Tests (Red)

Add failing tests that capture the target PLAN-098 behavior before implementation.

### Step 2: Implement (Green)

Implement the minimal change set needed to satisfy the tests while preserving the semantic tower split.

### Step 3: Integration (Green)

Wire the feature through all affected Ash layers honestly; do not collapse Act, Proc, and Workflow boundaries.

### Step 4: Property Tests (Verify)

Add or extend proptests for algebraic, typing, runtime identity, failure, or ordering invariants where appropriate.

## Verification Steps

- [x] Identity newtypes are serializable/debuggable where existing ID types are.
- [x] Failure carriers preserve source process/workflow identity in unit tests.
- [x] No public API treats `ControlLink` as `P<A>`.
- [x] `cargo test --all` passes
- [x] `cargo clippy --all-targets --all-features -- -D warnings` passes cleanly
- [x] `cargo fmt --check` passes

## Completion Notes

- Added `ash_core::runtime` carrier substrate with `RunId`, `ProcessId`, crate-internal `BranchId`, `LexicalFrameId`, `EffectScopeId`, lifecycle/terminal-state carriers, structured operational/process failure carriers, and skeleton workflow failure/report carriers.
- Kept `BranchId` internal to the runtime module and kept existing `ControlLink` semantics distinct from future affine `P<A>` handles.
- Deferred runtime admission, Proc/P type registration, process scheduling, operational `fail`/`with_error`, and workflow admission/report wiring to later Phase 98 tasks.

## Verification Evidence

- `cargo fmt --check`
- `cargo clippy --all-targets --all-features -- -D warnings`
- `cargo test --all`

## Dependencies for Next Task

This task outputs substrate needed by its direct dependents in PLAN-098.

## Notes

- Preserve existing workflow/control-link behavior unless this task explicitly changes it.
- Keep `Proc<A>` distinct from `Act<A>` and `Workflow`.
- Update `CHANGELOG.md` for implementation/tooling/docs-policy changes made while completing this task.
