# TASK-727: Export and import capability interface, implementation, and resource metadata through the module pipeline.

## Status: ✅ Complete

## Task Type

Module System

## Description

Export and import capability interface, implementation, and resource metadata through the module pipeline.

## Specification Reference

- SPEC-009
- SPEC-012
- SPEC-052
- SPEC-053

## Dependencies

- ✅ TASK-724: prerequisite task
- ✅ TASK-725: prerequisite task
- ✅ TASK-726: prerequisite task

## Requirements

### Functional Requirements

1. Extend module export metadata for capability interfaces.
2. Extend module export metadata for capability implementations.
3. Extend module export metadata for resource types.
4. Honor existing visibility and external import rules for the new definition kinds.

### Property Requirements (proptest)

```rust
// Add property-based tests for parser round-trips, conformance invariants,
// authority non-widening, resource identity preservation, or split/join
// behavior where this task introduces executable semantics.
// Docs-only tasks must instead include a corpus consistency sweep.
```

## TDD Steps

### Step 1: Write Tests or Corpus Checks (Red)

For implementation tasks, add failing tests before code changes. For docs/planning tasks, add or run corpus checks that fail or report missing references before updating docs.

### Step 2: Implement or Write Docs (Green)

Make the minimal focused change required by this task while preserving the Ash semantic tower:

```text
Pure < Effectful / Act < Proc < Workflow
```

### Step 3: Integration (Green)

Wire only the layer owned by this task. Do not silently implement downstream runtime behavior from later tasks.

### Step 4: Verification

Required verification for this task class:

- Parser/type/runtime tasks: focused crate tests plus affected integration tests.
- Docs/planning tasks: `git diff --check` plus cross-reference sweep for changed docs.
- All code tasks: `cargo fmt --check`, `cargo clippy --workspace --all-targets --all-features -- -D warnings`, and `cargo test --workspace` before completion.

## Verification Steps

- [x] Requirements above are satisfied.
- [x] New tests or docs checks cover the task-owned behavior.
- [x] Existing public behavior remains compatible unless the spec explicitly says otherwise.
- [x] CHANGELOG.md is updated for implementation/tooling/docs-policy changes.
- [x] PLAN-INDEX.md status is updated only when the task is actually complete.

## Completion Evidence

- Module metadata for capability/resource definitions is implemented as Phase 101 parser/module substrate only.
- Module export/import metadata now carries capability interfaces, capability implementations, and resource types alongside legacy capabilities/roles/policies/functions.
- Visibility-aware metadata preserves public/private definition kinds through the parser module surfaces without executing capability implementation bodies.
- Downstream LSP/lint/typeck role surfaces were updated to handle the expanded non-executable definition set exhaustively.
- Focused verification: `cargo test -p ash-parser --test phase_101_module_metadata`.
- Final closeout verification also runs `cargo test --workspace --all-targets --all-features`, `cargo clippy --workspace --all-targets --all-features -- -D warnings`, `cargo fmt --check`, `cargo doc --workspace --no-deps`, and `git diff --check`.

## Dependencies for Next Task

This task outputs:

- Downstream type checking receives complete imported metadata.

## Notes

- Preserve existing `pub capability` and Rust `CapabilityProvider` compatibility unless the task explicitly owns a migration.
- Do not allow ambient capability/resource lookup to bypass explicit admission.
- Do not manufacture external authority from Ash-defined code.
- Keep resource handles environment-owned unless a later spec explicitly introduces first-class `ResourceRef<T>` values.
