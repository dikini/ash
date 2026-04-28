# TASK-741: Execute Ash-defined capability implementation operation bodies through the effectful runtime path.

## Status: ✅ Complete

## Task Type

Runtime Semantics

## Description

Execute Ash-defined capability implementation operation bodies through the effectful runtime path.

## Specification Reference

- SPEC-052
- SPEC-053
- SPEC-047

## Dependencies

- ✅ TASK-740: prerequisite task

## Requirements

### Functional Requirements

1. Lower or register implementation operation bodies as callable effectful bodies.
2. Execute operation bodies with dependency bindings and resource access in scope.
3. Preserve operational failure attribution and provenance.
4. Keep pure functions unable to invoke capability bindings directly.

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

- Added runtime registration for `ImplementationOperationBody` values keyed by capability implementation and operation.
- Routed effectful `invoke(provider, action, args)` captures through admitted implementation bindings when explicitly admitted in the runtime context.
- Evaluates Ash-defined operation bodies with parameter, config dependency, and admitted capability dependency aliases in scope while keeping resource dependencies authority-only and non-first-class.
- Preserves existing host-provider binding behavior and explicit-admission enforcement.
- Converts missing bodies, arity mismatches, dependency-context failures, and body evaluation failures into operational failures with implementation/binding context.
- Added focused regression and proptest coverage in `crates/ash-interp/tests/task_741_ash_defined_capability_implementation_execution.rs`, including nested implementation dependency alias invocation and non-exposure of resource dependencies as pure variables.
- Verified focused behavior with:
  - `cargo test -p ash-interp --test task_741_ash_defined_capability_implementation_execution`
  - `cargo clippy -p ash-interp --test task_741_ash_defined_capability_implementation_execution --all-features -- -D warnings`
- Independent review found no TASK-741 spec/security/semantic blockers.

## Dependencies for Next Task

This task outputs:

- Ash-defined capability implementations can satisfy interfaces at runtime.

## Notes

- Preserve existing `pub capability` and Rust `CapabilityProvider` compatibility unless the task explicitly owns a migration.
- Do not allow ambient capability/resource lookup to bypass explicit admission.
- Do not manufacture external authority from Ash-defined code.
- Keep resource handles environment-owned unless a later spec explicitly introduces first-class `ResourceRef<T>` values.
