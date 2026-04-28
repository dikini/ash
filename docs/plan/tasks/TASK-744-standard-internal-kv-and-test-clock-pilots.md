# TASK-744: Add standard internal KV and test-clock pilot resources to validate internal authority creation.

## Status: ✅ Complete

## Task Type

Stdlib/Pilot

## Description

Add standard internal KV and test-clock pilot resources to validate internal authority creation.

## Specification Reference

- SPEC-052
- SPEC-053

## Dependencies

- ✅ TASK-741: prerequisite task
- ✅ TASK-742: prerequisite task
- ✅ TASK-743: prerequisite task

## Requirements

### Functional Requirements

1. Add a WorkflowKV resource pilot.
2. Add a FrozenClock/TestClock resource pilot if time provider boundaries allow it.
3. Bind pilot resources to capability interfaces through Ash-defined implementations.
4. Add deterministic tests showing substitution from host-backed to internal implementations.

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

- Added `ash-interp` runtime API pilots for standard internal `WorkflowKV` and `FrozenClock` resources.
- Pilot admission creates internal runtime-owned resources, deterministic Ash-defined implementation bodies, and implementation-backed bindings whose authority derives from explicit resource dependencies.
- Tests prove host-backed to internal WorkflowKV substitution, explicit admission boundaries, deterministic FrozenClock execution without a host time provider, and rejection of pre-registered internal-body collisions.
- This remains an honest runtime API pilot: mutable first-class KV resource state and source-level `ash run` lowering/admission are intentionally not claimed by this task.

## Dependencies for Next Task

This task outputs:

- Internal authority model is proven with at least one useful stdlib/test pilot.

## Notes

- Preserve existing `pub capability` and Rust `CapabilityProvider` compatibility unless the task explicitly owns a migration.
- Do not allow ambient capability/resource lookup to bypass explicit admission.
- Do not manufacture external authority from Ash-defined code.
- Keep resource handles environment-owned unless a later spec explicitly introduces first-class `ResourceRef<T>` values.
