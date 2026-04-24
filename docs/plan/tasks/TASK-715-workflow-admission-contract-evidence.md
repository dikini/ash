# TASK-715: Workflow admission and contract evidence

## Status: 🟡 Ready

## Description

Implement workflow admission checks, requires evidence, and ensures evidence plumbing without completion-time ensures evaluation.

## Specification Reference

- SPEC-051
- SPEC-019
- SPEC-022

## Dependencies

- 📝 TASK-714: prerequisite task

## Requirements

### Functional Requirements

1. Add an admission API above interpreter execution, likely in `ash-engine`, that creates/accepts workflow identity and `RunId`.
2. Resolve role/capability/provider/policy availability at admission and snapshot admitted context.
3. Evaluate `requires` predicates at admission/call boundary and record evidence.
4. Define and carry ensures evidence schema/plumbing for completion-time evaluation by TASK-716; do not evaluate ensures in this task.

### Property Requirements (proptest)

```rust
// Add property-based tests for identity preservation, handle linearity,
// failure aggregation, environment projection, or typing invariants where
// this task manipulates those semantics.
```

## TDD Steps

### Step 1: Write Tests (Red)

Add failing tests that capture the target PLAN-098 behavior before implementation.

### Step 2: Implement (Green)

Implement the minimal change set needed to satisfy the tests while preserving the semantic tower split.

### Step 3: Integration (Green)

Wire the feature through all affected Ash layers honestly; do not collapse Act, Proc, and Workflow boundaries.

### Step 4: Property Tests (Verify)

Add or extend proptests for algebraic, typing, runtime identity, failure, or ordering invariants where appropriate.

## Verification Steps

- [ ] Role/capability admission failures map to structured workflow failures.
- [ ] Requires failure prevents body execution and records evidence.
- [ ] Ensures evidence schema/plumbing is available for TASK-716 without performing completion-time evaluation in this task.
- [ ] `cargo test --all` passes
- [ ] `cargo clippy --all-targets --all-features` passes cleanly
- [ ] `cargo fmt --check` passes

## Dependencies for Next Task

This task outputs substrate needed by its direct dependents in PLAN-098.

## Notes

- Preserve existing workflow/control-link behavior unless this task explicitly changes it.
- Keep `Proc<A>` distinct from `Act<A>` and `Workflow`.
- Update `CHANGELOG.md` for implementation/tooling/docs-policy changes made while completing this task.
