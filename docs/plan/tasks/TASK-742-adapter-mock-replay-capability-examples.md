# TASK-742: Add adapter, mock, and replay examples proving test/prod substitution and derived capability patterns.

## Status: ✅ Complete

## Task Type

Examples/DX

## Description

Add adapter, mock, and replay examples proving test/prod substitution and derived capability patterns.

## Specification Reference

- SPEC-052
- SPEC-053

## Dependencies

- ✅ TASK-741: prerequisite task

## Requirements

### Functional Requirements

1. Create a mock/internal KV implementation example.
2. Create a logging or caching adapter example.
3. Create a replay/record sketch or executable pilot if substrate supports it.
4. Add `ash check` and `ash run` coverage where executable.

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

### Completion Evidence

- Added `examples/06-capability-implementations/` with `ash check`-covered packets for:
  - mock/internal KV implementation,
  - logging/cache adapter over an inner `KeyValue` dependency,
  - recording/replay sketch over replay-log authority.
- Added top-level and local example README coverage that states the current boundary honestly: source examples are checkable declaration packets, while executable operation-body behavior is covered by runtime API tests until source-level `ash run` lowering/admission is implemented.
- Added CLI conformance coverage in `crates/ash-cli/tests/task_742_capability_examples_conformance.rs`.
- Added executable runtime API coverage in `crates/ash-interp/tests/task_742_capability_examples.rs` for host/mock substitution, adapter invocation of an inner capability dependency, and a recording-envelope pilot without claiming persistent replay.
- Verified focused behavior with:
  - `cargo test -p ash-cli --test task_742_capability_examples_conformance`
  - `cargo test -p ash-interp --test task_742_capability_examples`
  - `cargo clippy -p ash-interp --test task_742_capability_examples --all-features -- -D warnings`
  - `cargo clippy -p ash-cli --test task_742_capability_examples_conformance --all-features -- -D warnings`
  - `cargo run --quiet --bin ash -- check` for each Phase 104 source example.
- Independent review found the implementation broadly compliant after fixing misleading source `Unit` bodies to return `()`.

## Dependencies for Next Task

This task outputs:

- Users have concrete examples of the NOTE-009 DX model.

## Notes

- Preserve existing `pub capability` and Rust `CapabilityProvider` compatibility unless the task explicitly owns a migration.
- Do not allow ambient capability/resource lookup to bypass explicit admission.
- Do not manufacture external authority from Ash-defined code.
- Keep resource handles environment-owned unless a later spec explicitly introduces first-class `ResourceRef<T>` values.
