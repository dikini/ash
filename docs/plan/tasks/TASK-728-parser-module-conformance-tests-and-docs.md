# TASK-728: Add parser/module conformance coverage for the new definition syntax and metadata transport.

## Status: ✅ Complete

## Task Type

Verification

## Description

Add parser/module conformance coverage for the new definition syntax and metadata transport.

## Specification Reference

- SPEC-052
- SPEC-053

## Dependencies

- ✅ TASK-724: prerequisite task
- ✅ TASK-725: prerequisite task
- ✅ TASK-726: prerequisite task
- ✅ TASK-727: prerequisite task

## Requirements

### Functional Requirements

1. Add positive parser tests for interfaces, impls, resources, owns, and uses clauses.
2. Add negative parser tests for malformed headers and duplicate operations.
3. Add module import/export tests for public/private visibility.
4. Update examples/docs to mark the syntax as planned if not fully executable yet.

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

- Parser/module conformance tests and docs is implemented as Phase 101 parser/module substrate only.
- Added focused Phase 101 conformance tests for capability interfaces, implementations, resources, workflow `owns`/`uses`, and module metadata transport.
- Added `docs/reference/phase-101-capability-resource-parser-substrate.md` documenting the syntax as parser/module substrate, not executable semantics.
- Updated stale parser regression expectations so visibility-qualified capability metadata is asserted as supported rather than rejected.
- Focused verification: `cargo test --workspace --all-targets --all-features`.
- Final closeout verification also runs `cargo test --workspace --all-targets --all-features`, `cargo clippy --workspace --all-targets --all-features -- -D warnings`, `cargo fmt --check`, `cargo doc --workspace --no-deps`, and `git diff --check`.

## Dependencies for Next Task

This task outputs:

- Phase 101 parser/module substrate has regression coverage.

## Notes

- Preserve existing `pub capability` and Rust `CapabilityProvider` compatibility unless the task explicitly owns a migration.
- Do not allow ambient capability/resource lookup to bypass explicit admission.
- Do not manufacture external authority from Ash-defined code.
- Keep resource handles environment-owned unless a later spec explicitly introduces first-class `ResourceRef<T>` values.
