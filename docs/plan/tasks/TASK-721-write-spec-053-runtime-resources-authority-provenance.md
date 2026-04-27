# TASK-721: Write the normative [SPEC-053](../../spec/SPEC-053-RUNTIME-RESOURCES-AND-AUTHORITY-PROVENANCE.md) resource, resource-instance, binding, and authority provenance contract promoted from [NOTE-009](../../notes/NOTE-009-CAPABILITY-INTERFACES-IMPLEMENTATIONS-AND-INTERNAL-AUTHORITY.md).

## Status: ✅ Complete

## Task Type

Spec hardening

## Description

Write the normative [SPEC-053](../../spec/SPEC-053-RUNTIME-RESOURCES-AND-AUTHORITY-PROVENANCE.md) resource, resource-instance, binding, and authority provenance contract promoted from [NOTE-009](../../notes/NOTE-009-CAPABILITY-INTERFACES-IMPLEMENTATIONS-AND-INTERNAL-AUTHORITY.md).

## Specification Reference

- [NOTE-009](../../notes/NOTE-009-CAPABILITY-INTERFACES-IMPLEMENTATIONS-AND-INTERNAL-AUTHORITY.md)
- [SPEC-053](../../spec/SPEC-053-RUNTIME-RESOURCES-AND-AUTHORITY-PROVENANCE.md)

## Dependencies

- ✅ [TASK-720](TASK-720-write-spec-052-capability-interface-implementation-contract.md): prerequisite task

## Requirements

### Functional Requirements

1. Define resource type/requirement/allocation/instance/binding terms.
2. Define host, internal, and derived authority provenance.
3. Define split/join/share/move resource policy requirements for Proc boundaries.
4. Define lifecycle, provenance, and failure requirements.

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

- [x] Requirements above are satisfied by [SPEC-053](../../spec/SPEC-053-RUNTIME-RESOURCES-AND-AUTHORITY-PROVENANCE.md) sections 1-13.
- [x] New docs checks cover the task-owned behavior: Phase 100 source-link and cross-reference sweep verified linked references for [NOTE-009](../../notes/NOTE-009-CAPABILITY-INTERFACES-IMPLEMENTATIONS-AND-INTERNAL-AUTHORITY.md), [SPEC-053](../../spec/SPEC-053-RUNTIME-RESOURCES-AND-AUTHORITY-PROVENANCE.md), and [TASK-721](TASK-721-write-spec-053-runtime-resources-authority-provenance.md).
- [x] Existing public behavior remains compatible: this task is docs/spec-only and does not change parser/typechecker/runtime behavior.
- [x] [CHANGELOG.md](../../../CHANGELOG.md) is updated for the spec/plan packet.
- [x] [PLAN-INDEX.md](../PLAN-INDEX.md) status is updated only because the docs/spec contract exists and was audited.

## Completion Evidence

- Created [SPEC-053](../../spec/SPEC-053-RUNTIME-RESOURCES-AND-AUTHORITY-PROVENANCE.md) as the normative draft owner for resource types, resource requirements, allocation/admission sites, resource instances/bindings, host/internal/derived authority provenance, split/join policy, lifecycle, and resource-backed failure/provenance obligations.
- Verified [SPEC-053](../../spec/SPEC-053-RUNTIME-RESOURCES-AND-AUTHORITY-PROVENANCE.md) explicitly separates Ash-owned internal authority from host/external authority and does not claim that Ash can manufacture external authority.
- Linked this task, [NOTE-009](../../notes/NOTE-009-CAPABILITY-INTERFACES-IMPLEMENTATIONS-AND-INTERNAL-AUTHORITY.md), and [SPEC-053](../../spec/SPEC-053-RUNTIME-RESOURCES-AND-AUTHORITY-PROVENANCE.md) from the Phase 100 planning surfaces so future implementers can navigate to source artifacts directly.

## Dependencies for Next Task

This task outputs:

- Spec file created at [`docs/spec/SPEC-053-RUNTIME-RESOURCES-AND-AUTHORITY-PROVENANCE.md`](../../spec/SPEC-053-RUNTIME-RESOURCES-AND-AUTHORITY-PROVENANCE.md).

## Notes

- Preserve existing `pub capability` and Rust `CapabilityProvider` compatibility unless the task explicitly owns a migration.
- Do not allow ambient capability/resource lookup to bypass explicit admission.
- Do not manufacture external authority from Ash-defined code.
- Keep resource handles environment-owned unless a later spec explicitly introduces first-class `ResourceRef<T>` values.
