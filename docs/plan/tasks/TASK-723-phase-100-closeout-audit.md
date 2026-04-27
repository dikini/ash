# TASK-723: Close out Phase 100 by verifying the [NOTE-009](../../notes/NOTE-009-CAPABILITY-INTERFACES-IMPLEMENTATIONS-AND-INTERNAL-AUTHORITY.md) promotion packet is internally consistent and ready to drive implementation phases.

## Status: ✅ Complete

## Task Type

Docs/Planning

## Description

Close out Phase 100 by verifying the [NOTE-009](../../notes/NOTE-009-CAPABILITY-INTERFACES-IMPLEMENTATIONS-AND-INTERNAL-AUTHORITY.md) promotion packet is internally consistent and ready to drive implementation phases.

## Specification Reference

- [NOTE-009](../../notes/NOTE-009-CAPABILITY-INTERFACES-IMPLEMENTATIONS-AND-INTERNAL-AUTHORITY.md)
- [SPEC-052](../../spec/SPEC-052-CAPABILITY-INTERFACES-AND-IMPLEMENTATIONS.md)
- [SPEC-053](../../spec/SPEC-053-RUNTIME-RESOURCES-AND-AUTHORITY-PROVENANCE.md)
- [PLAN-100](../PLAN-100-CAPABILITY-INTERFACES-RESOURCES.md)

## Dependencies

- ✅ [TASK-720](TASK-720-write-spec-052-capability-interface-implementation-contract.md): prerequisite task
- ✅ [TASK-721](TASK-721-write-spec-053-runtime-resources-authority-provenance.md): prerequisite task
- ✅ [TASK-722](TASK-722-reconcile-capability-resource-spec-ownership.md): prerequisite task

## Requirements

### Functional Requirements

1. Verify every [NOTE-009](../../notes/NOTE-009-CAPABILITY-INTERFACES-IMPLEMENTATIONS-AND-INTERNAL-AUTHORITY.md) core concept is either specified or explicitly deferred.
2. Verify implementation phases map each public spec requirement to a task.
3. Run documentation consistency checks for new files and changed indices.
4. Record Phase 100 complete while later implementation phases remain planned.

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

- [x] Requirements above are satisfied by the Phase 100 closeout review recorded below.
- [x] New docs checks cover the task-owned behavior: Phase 100 closeout checked [NOTE-009](../../notes/NOTE-009-CAPABILITY-INTERFACES-IMPLEMENTATIONS-AND-INTERNAL-AUTHORITY.md) concept coverage, implementation-phase mapping, local markdown links, table syntax, and source-reference links.
- [x] Existing public behavior remains compatible: this task is docs/planning-only and does not change parser/typechecker/runtime behavior.
- [x] [CHANGELOG.md](../../../CHANGELOG.md) is updated for the spec/plan packet.
- [x] [PLAN-INDEX.md](../PLAN-INDEX.md) status is updated only because Phase 100 docs/spec/planning work is complete; Phases 101-104 remain planned.

## Completion Evidence

- [NOTE-009](../../notes/NOTE-009-CAPABILITY-INTERFACES-IMPLEMENTATIONS-AND-INTERNAL-AUTHORITY.md) concept coverage: [SPEC-052](../../spec/SPEC-052-CAPABILITY-INTERFACES-AND-IMPLEMENTATIONS.md) owns capability interfaces, implementations, bindings, adapters, module visibility, and invocation boundaries; [SPEC-053](../../spec/SPEC-053-RUNTIME-RESOURCES-AND-AUTHORITY-PROVENANCE.md) owns resource types, resource instances/bindings, internal authority, authority provenance, lifecycle, and split/join policy.
- Implementation mapping: [PLAN-100](../PLAN-100-CAPABILITY-INTERFACES-RESOURCES.md) and [PLAN-INDEX.md](../PLAN-INDEX.md) sequence implementation into Phases 101-104, leaving parser/module metadata, static semantics, runtime resources/bindings, and Ash-defined capability implementation execution planned rather than claimed complete.
- Remediation performed during closeout: linked source references for Phase 100 task/spec/note/plan artifacts and checked task verification boxes with evidence instead of leaving completed task files with unchecked verification criteria.
- Remaining risk: [SPEC-052](../../spec/SPEC-052-CAPABILITY-INTERFACES-AND-IMPLEMENTATIONS.md) and [SPEC-053](../../spec/SPEC-053-RUNTIME-RESOURCES-AND-AUTHORITY-PROVENANCE.md) are normative draft specs; concrete parser/typechecker/runtime semantics remain future implementation work in TASK-724 through TASK-745.

## Dependencies for Next Task

This task outputs:

- Phase 100 is complete; implementation [Phases 101-104](../PLAN-100-CAPABILITY-INTERFACES-RESOURCES.md) remain planned.

## Notes

- Preserve existing `pub capability` and Rust `CapabilityProvider` compatibility unless the task explicitly owns a migration.
- Do not allow ambient capability/resource lookup to bypass explicit admission.
- Do not manufacture external authority from Ash-defined code.
- Keep resource handles environment-owned unless a later spec explicitly introduces first-class `ResourceRef<T>` values.
