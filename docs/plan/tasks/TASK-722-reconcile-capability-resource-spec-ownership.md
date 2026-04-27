# TASK-722: Reconcile adjacent specs and indices with [SPEC-052](../../spec/SPEC-052-CAPABILITY-INTERFACES-AND-IMPLEMENTATIONS.md)/[SPEC-053](../../spec/SPEC-053-RUNTIME-RESOURCES-AND-AUTHORITY-PROVENANCE.md) as the normative owners for [NOTE-009](../../notes/NOTE-009-CAPABILITY-INTERFACES-IMPLEMENTATIONS-AND-INTERNAL-AUTHORITY.md) concepts.

## Status: ✅ Complete

## Task Type

Docs/Planning

## Description

Reconcile adjacent specs and indices with [SPEC-052](../../spec/SPEC-052-CAPABILITY-INTERFACES-AND-IMPLEMENTATIONS.md)/[SPEC-053](../../spec/SPEC-053-RUNTIME-RESOURCES-AND-AUTHORITY-PROVENANCE.md) as the normative owners for [NOTE-009](../../notes/NOTE-009-CAPABILITY-INTERFACES-IMPLEMENTATIONS-AND-INTERNAL-AUTHORITY.md) concepts.

## Specification Reference

- [SPEC-017](../../spec/SPEC-017-CAPABILITY-INTEGRATION.md)
- [SPEC-048](../../spec/SPEC-048-PROC-LIBRARY.md)
- [SPEC-049](../../spec/SPEC-049-PROCESS-RUNTIME-SEMANTICS.md)
- [SPEC-051](../../spec/SPEC-051-WORKFLOW-SEMANTICS.md)
- [SPEC-052](../../spec/SPEC-052-CAPABILITY-INTERFACES-AND-IMPLEMENTATIONS.md)
- [SPEC-053](../../spec/SPEC-053-RUNTIME-RESOURCES-AND-AUTHORITY-PROVENANCE.md)

## Dependencies

- ✅ [TASK-720](TASK-720-write-spec-052-capability-interface-implementation-contract.md): prerequisite task
- ✅ [TASK-721](TASK-721-write-spec-053-runtime-resources-authority-provenance.md): prerequisite task

## Requirements

### Functional Requirements

1. Update `docs/spec/README.md` with [SPEC-052](../../spec/SPEC-052-CAPABILITY-INTERFACES-AND-IMPLEMENTATIONS.md) and [SPEC-053](../../spec/SPEC-053-RUNTIME-RESOURCES-AND-AUTHORITY-PROVENANCE.md) rows.
2. Update PLAN-INDEX with Phase 100 through Phase 104.
3. Add a [CHANGELOG.md](../../../CHANGELOG.md) entry for the spec/plan packet.
4. Avoid claiming implementation of runtime/resource semantics in this docs-only task.

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

- [x] Requirements above are satisfied by [docs/spec/README.md](../../spec/README.md), [PLAN-INDEX.md](../PLAN-INDEX.md), [PLAN-100](../PLAN-100-CAPABILITY-INTERFACES-RESOURCES.md), and [CHANGELOG.md](../../../CHANGELOG.md).
- [x] New docs checks cover the task-owned behavior: Phase 100 source-link and cross-reference sweep verified links for [TASK-720](TASK-720-write-spec-052-capability-interface-implementation-contract.md) through [TASK-723](TASK-723-phase-100-closeout-audit.md), [SPEC-052](../../spec/SPEC-052-CAPABILITY-INTERFACES-AND-IMPLEMENTATIONS.md)/[SPEC-053](../../spec/SPEC-053-RUNTIME-RESOURCES-AND-AUTHORITY-PROVENANCE.md), [NOTE-009](../../notes/NOTE-009-CAPABILITY-INTERFACES-IMPLEMENTATIONS-AND-INTERNAL-AUTHORITY.md), and [PLAN-100](../PLAN-100-CAPABILITY-INTERFACES-RESOURCES.md).
- [x] Existing public behavior remains compatible: this task is docs/planning-only and does not change parser/typechecker/runtime behavior.
- [x] [CHANGELOG.md](../../../CHANGELOG.md) is updated for the spec/plan packet.
- [x] [PLAN-INDEX.md](../PLAN-INDEX.md) status is updated only because the docs/spec ownership split and planned implementation phases are present.

## Completion Evidence

- Verified [docs/spec/README.md](../../spec/README.md) contains [SPEC-052](../../spec/SPEC-052-CAPABILITY-INTERFACES-AND-IMPLEMENTATIONS.md) and [SPEC-053](../../spec/SPEC-053-RUNTIME-RESOURCES-AND-AUTHORITY-PROVENANCE.md) rows.
- Verified [PLAN-INDEX.md](../PLAN-INDEX.md) contains Phase 100 through Phase 104 with Phase 100 complete and implementation Phases 101-104 still planned.
- Verified [CHANGELOG.md](../../../CHANGELOG.md) records the [SPEC-052](../../spec/SPEC-052-CAPABILITY-INTERFACES-AND-IMPLEMENTATIONS.md)/[SPEC-053](../../spec/SPEC-053-RUNTIME-RESOURCES-AND-AUTHORITY-PROVENANCE.md)/[PLAN-100](../PLAN-100-CAPABILITY-INTERFACES-RESOURCES.md)/[TASK-720](TASK-720-write-spec-052-capability-interface-implementation-contract.md)..[TASK-745](TASK-745-capability-resource-final-docs-examples-verification.md) planning packet.
- Remediated source links in Phase 100 planning/task/spec surfaces while preserving implementation status semantics.

## Dependencies for Next Task

This task outputs:

- Spec index, plan index, and changelog reference the new packet with source links for the Phase 100 artifacts.

## Notes

- Preserve existing `pub capability` and Rust `CapabilityProvider` compatibility unless the task explicitly owns a migration.
- Do not allow ambient capability/resource lookup to bypass explicit admission.
- Do not manufacture external authority from Ash-defined code.
- Keep resource handles environment-owned unless a later spec explicitly introduces first-class `ResourceRef<T>` values.
