# TASK-709: Process registry and child environment projection

## Status: ✅ Complete

## Description

Introduce the runtime process registry and component-wise child environment projection API.

## Specification Reference

- SPEC-049
- NOTE-007

## Dependencies

- 📝 TASK-706: prerequisite task
- 📝 TASK-708: prerequisite task

## Requirements

### Functional Requirements

1. Add a process registry keyed by `ProcessId` without replacing `ControlLinkRegistry`.
2. Represent parent/child process relations and terminal result/failure storage.
3. Define `derive_child_env` as component-wise projection rather than monolithic `Context::clone`.
4. Ensure child authority is equal or narrower than parent authority.

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

- ✅ Registry tests cover parent/child identity and terminal-state recording.
- ✅ Projection tests prove no child receives wider authority than parent.
- ✅ Existing workflow execution still compiles and passes targeted smoke tests.
- ✅ `cargo test --all` passes
- ✅ `cargo clippy --all-targets --all-features` passes cleanly
- ✅ `cargo fmt --check` passes

## Completion Notes

- Added `ash_interp::ProcessRegistry` with `ProcessId`-keyed process records, parent/child links, ordered child listing, and write-once `ProcessTerminalState` recording.
- Integrated the process registry into `RuntimeState` beside the existing `ControlLinkRegistry`; workflow control links remain a separate supervision/control authority.
- Added `ash_interp::derive_child_env(...)` and `ChildEnvProjection` as the named component-wise projection boundary for child process contexts.
- Child environment projection snapshots visible lexical bindings, allocates fresh child-local obligation state, preserves hidden runtime policy/Act carriers through `Context`'s internal component projection, records child/parent process identity metadata when supplied, and rejects role-authority widening across capability names, effects, and constraints.
- Added TASK-709 regression/property coverage in `crates/ash-interp/tests/task_709_process_registry.rs`.

## Dependencies for Next Task

This task outputs substrate needed by its direct dependents in PLAN-098.

## Notes

- Preserve existing workflow/control-link behavior unless this task explicitly changes it.
- Keep `Proc<A>` distinct from `Act<A>` and `Workflow`.
- Update `CHANGELOG.md` for implementation/tooling/docs-policy changes made while completing this task.
