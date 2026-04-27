# TASK-733: Resolve capability binding names through module/admission metadata instead of ambient lexical lookup.

## Status: 📝 Planned

## Task Type

Type System/Engine

## Description

Resolve capability binding names through module/admission metadata instead of ambient lexical lookup.

## Specification Reference

- SPEC-052
- SPEC-053
- SPEC-009
- SPEC-012

## Dependencies

- ✅ TASK-729: prerequisite task
- ✅ TASK-730: prerequisite task
- ✅ TASK-731: prerequisite task
- ✅ TASK-732: prerequisite task

## Requirements

### Functional Requirements

1. Build a capability binding resolution context from module metadata and workflow/process headers.
2. Resolve `binding.operation(...)` capability calls to interface operation signatures.
3. Preserve existing module-owned provider/action resolution compatibility.
4. Reject ambient unadmitted capability binding use.

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

- [ ] Requirements above are satisfied.
- [ ] New tests or docs checks cover the task-owned behavior.
- [ ] Existing public behavior remains compatible unless the spec explicitly says otherwise.
- [ ] CHANGELOG.md is updated for implementation/tooling/docs-policy changes.
- [ ] PLAN-INDEX.md status is updated only when the task is actually complete.

## Dependencies for Next Task

This task outputs:

- Capability calls resolve through explicit admitted bindings.

## Notes

- Preserve existing `pub capability` and Rust `CapabilityProvider` compatibility unless the task explicitly owns a migration.
- Do not allow ambient capability/resource lookup to bypass explicit admission.
- Do not manufacture external authority from Ash-defined code.
- Keep resource handles environment-owned unless a later spec explicitly introduces first-class `ResourceRef<T>` values.
