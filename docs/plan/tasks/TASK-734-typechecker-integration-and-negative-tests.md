# TASK-734: Add static-semantics integration and negative tests across interface, impl, resource, and binding checks.

## Status: ✅ Complete

## Task Type

Verification

## Description

Add static-semantics integration and negative tests across interface, impl, resource, and binding checks.

## Specification Reference

- SPEC-052
- SPEC-053

## Dependencies

- ✅ TASK-729: prerequisite task
- ✅ TASK-730: prerequisite task
- ✅ TASK-731: prerequisite task
- ✅ TASK-732: prerequisite task
- ✅ TASK-733: prerequisite task

## Requirements

### Functional Requirements

1. Add end-to-end check tests for valid interface/impl/resource/binding packets.
2. Add negative tests for wrong implementation target, missing dependency, wrong operation type, and authority widening.
3. Add imported-module tests for interface and implementation metadata.
4. Run focused and workspace typechecker gates.

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

## Dependencies for Next Task

This task outputs:

- Phase 102 static semantics have regression coverage.

## Notes

- Preserve existing `pub capability` and Rust `CapabilityProvider` compatibility unless the task explicitly owns a migration.
- Do not allow ambient capability/resource lookup to bypass explicit admission.
- Do not manufacture external authority from Ash-defined code.
- Keep resource handles environment-owned unless a later spec explicitly introduces first-class `ResourceRef<T>` values.
