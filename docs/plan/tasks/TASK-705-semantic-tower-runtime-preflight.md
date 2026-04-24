# TASK-705: Semantic tower runtime preflight

## Status: 🟡 Ready

## Description

Perform the implementation preflight for PLAN-098 before any runtime code is changed.

## Specification Reference

- SPEC-047
- SPEC-048
- SPEC-049
- SPEC-050
- SPEC-051

## Dependencies

- None

## Requirements

### Functional Requirements

1. Confirm Phase 97 Act tasks required by the selected slice are complete or explicitly not required.
2. Inventory live parser/typechecker/runtime surfaces that changed since PLAN-098 was written.
3. Check for stale `yield`, `panic`, `Workflow::Spawn`, and `ControlLink` behavior that could be confused with PLAN-098 semantics.
4. Record any semantic fork as a plan/task amendment instead of implementation.

### Property Requirements (proptest)

No property tests are expected for this preflight-only task. If preflight discovers implementation drift, create or amend the downstream implementation task that should own the tests.

## Preflight Steps

### Step 1: Inspect Live Code

Re-run prerequisite discovery against the current branch before choosing an implementation task.

### Step 2: Verify Phase 97 Dependencies

Check whether the selected PLAN-098 slice depends on completed Act substrate work; if it does, verify the relevant Phase 97 tasks first.

### Step 3: Document Drift or Forks

If code/spec drift or a semantic fork is found, amend PLAN-098 and affected task files before implementation.

### Step 4: No Runtime Implementation

Do not implement runtime behavior in this preflight task. Its output is verified readiness or an explicit plan/task amendment.

## Verification Steps

- [ ] No runtime implementation is performed in this task.
- [ ] PLAN-098 dependency assumptions are validated against live code.
- [ ] Any changed prerequisite order is documented in PLAN-INDEX and task dependencies.
- [ ] `cargo test --all` passes
- [ ] `cargo clippy --all-targets --all-features` passes cleanly
- [ ] `cargo fmt --check` passes

## Dependencies for Next Task

This task outputs substrate needed by its direct dependents in PLAN-098.

## Notes

- Preserve existing workflow/control-link behavior unless this task explicitly changes it.
- Keep `Proc<A>` distinct from `Act<A>` and `Workflow`.
- Update `CHANGELOG.md` for implementation/tooling/docs-policy changes made while completing this task.
