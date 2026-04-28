# TASK-730: Check capability implementation recipes against their target interfaces.

## Status: ✅ Complete

## Task Type

Type System

## Description

Check capability implementation recipes against their target interfaces.

## Specification Reference

- SPEC-052
- SPEC-003
- SPEC-047

## Dependencies

- ✅ TASK-729: prerequisite task

## Requirements

### Functional Requirements

1. Verify every interface operation is implemented exactly once.
2. Verify modes, arity, parameter types, and return type match.
3. Type-check operation bodies in effectful contexts with declared dependencies only.
4. Reject implementations that use undeclared dependencies or ordinary pure authority.

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

## Completion Notes

- Added `TypeEnv` capability-implementation registration and lookup APIs for
  TASK-730: `register_capability_implementation`,
  `lookup_capability_implementation`, and `has_capability_implementation`.
- Preserved static implementation metadata as `CapabilityImplementationInfo`,
  including target interface, declared dependencies, operation modes, parameter
  names/types, and return types.
- Enforced exact operation coverage against the target capability interface:
  missing, extra, and duplicate implementation operations are rejected before
  registration.
- Validated operation mode, arity, parameter types, return type, and operation
  body type against the target interface operation signature.
- Type-checks operation bodies in an effectful body environment derived from the
  operation mode (`observe` → epistemic, `execute` → operational) so nested
  function values use the effectful `Fun` classification rather than pure `Fn`.
- Restricts implementation-body value scope to operation parameters plus declared
  `Config` dependencies only. Ambient variables, same-program helper functions,
  builtins, direct `invoke(...)`, and non-`Config` dependencies are not exposed as
  implementation-body authority.
- Rejects duplicate dependency names and dependency names that collide with
  operation parameter names.
- Wired `type_check_program_in_env` to register
  `Definition::CapabilityImplementation` declarations after capability
  interfaces and before ordinary function signatures/workflow checking, keeping
  same-program pure helpers out of implementation-body scope.
- Added focused tests in
  `crates/ash-typeck/tests/task_730_capability_implementation_conformance.rs`,
  including RED evidence for missing APIs and review-driven regressions for
  effectful body typing, ambient authority rejection, direct `invoke(...)`
  rejection, dependency duplicate/shadowing rejection, Config-only body values,
  child environment inheritance/isolation, and program-level registration.
- Verification completed with focused TASK-730 tests, `cargo test -p ash-typeck`,
  `cargo clippy -p ash-typeck --all-targets --all-features -- -D warnings`,
  `cargo fmt --check`, and `git diff --check`.
- Scope boundary: this task does not implement resource type registries,
  workflow `owns`/`uses` typechecking, authority provenance validation, module
  binding resolution, runtime admission, or runtime dispatch; those remain owned
  by later Phase 102/103 tasks.

## Dependencies for Next Task

This task outputs:

- Malformed implementations fail with precise diagnostics.

## Notes

- Preserve existing `pub capability` and Rust `CapabilityProvider` compatibility unless the task explicitly owns a migration.
- Do not allow ambient capability/resource lookup to bypass explicit admission.
- Do not manufacture external authority from Ash-defined code.
- Keep resource handles environment-owned unless a later spec explicitly introduces first-class `ResourceRef<T>` values.
