# TASK-720: Write the normative [SPEC-052](../../spec/SPEC-052-CAPABILITY-INTERFACES-AND-IMPLEMENTATIONS.md) capability interface, implementation, and binding contract promoted from [NOTE-009](../../notes/NOTE-009-CAPABILITY-INTERFACES-IMPLEMENTATIONS-AND-INTERNAL-AUTHORITY.md).

## Status: ✅ Complete

## Task Type

Spec hardening

## Description

Write the normative [SPEC-052](../../spec/SPEC-052-CAPABILITY-INTERFACES-AND-IMPLEMENTATIONS.md) capability interface, implementation, and binding contract promoted from [NOTE-009](../../notes/NOTE-009-CAPABILITY-INTERFACES-IMPLEMENTATIONS-AND-INTERNAL-AUTHORITY.md).

## Specification Reference

- [NOTE-009](../../notes/NOTE-009-CAPABILITY-INTERFACES-IMPLEMENTATIONS-AND-INTERNAL-AUTHORITY.md)
- [SPEC-052](../../spec/SPEC-052-CAPABILITY-INTERFACES-AND-IMPLEMENTATIONS.md)

## Dependencies

- None

## Requirements

### Functional Requirements

1. Define capability interface terminology and constraints.
2. Define capability implementation recipe semantics.
3. Define capability binding and late-binding semantics.
4. Map existing `pub capability` to the new explicit model without breaking compatibility.

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

- [x] Requirements above are satisfied by [SPEC-052](../../spec/SPEC-052-CAPABILITY-INTERFACES-AND-IMPLEMENTATIONS.md) sections 1-13.
- [x] New docs checks cover the task-owned behavior: Phase 100 source-link and cross-reference sweep verified linked references for [NOTE-009](../../notes/NOTE-009-CAPABILITY-INTERFACES-IMPLEMENTATIONS-AND-INTERNAL-AUTHORITY.md), [SPEC-052](../../spec/SPEC-052-CAPABILITY-INTERFACES-AND-IMPLEMENTATIONS.md), and [TASK-720](TASK-720-write-spec-052-capability-interface-implementation-contract.md).
- [x] Existing public behavior remains compatible: this task is docs/spec-only and does not change parser/typechecker/runtime behavior.
- [x] [CHANGELOG.md](../../../CHANGELOG.md) is updated for the spec/plan packet.
- [x] [PLAN-INDEX.md](../PLAN-INDEX.md) status is updated only because the docs/spec contract exists and was audited.

## Completion Evidence

- Created [SPEC-052](../../spec/SPEC-052-CAPABILITY-INTERFACES-AND-IMPLEMENTATIONS.md) as the normative draft owner for stateless capability interfaces, implementation recipes, binding-time selection, module visibility, conformance, derived/adapted implementations, and runtime invocation boundaries.
- Verified [SPEC-052](../../spec/SPEC-052-CAPABILITY-INTERFACES-AND-IMPLEMENTATIONS.md) preserves compatibility with existing `pub capability` / Rust `CapabilityProvider` behavior and explicitly defers parser/type/runtime implementation to Phases 101-104.
- Linked this task, [NOTE-009](../../notes/NOTE-009-CAPABILITY-INTERFACES-IMPLEMENTATIONS-AND-INTERNAL-AUTHORITY.md), and [SPEC-052](../../spec/SPEC-052-CAPABILITY-INTERFACES-AND-IMPLEMENTATIONS.md) from the Phase 100 planning surfaces so future implementers can navigate to source artifacts directly.

## Dependencies for Next Task

This task outputs:

- Spec file created at [`docs/spec/SPEC-052-CAPABILITY-INTERFACES-AND-IMPLEMENTATIONS.md`](../../spec/SPEC-052-CAPABILITY-INTERFACES-AND-IMPLEMENTATIONS.md).

## Notes

- Preserve existing `pub capability` and Rust `CapabilityProvider` compatibility unless the task explicitly owns a migration.
- Do not allow ambient capability/resource lookup to bypass explicit admission.
- Do not manufacture external authority from Ash-defined code.
- Keep resource handles environment-owned unless a later spec explicitly introduces first-class `ResourceRef<T>` values.
