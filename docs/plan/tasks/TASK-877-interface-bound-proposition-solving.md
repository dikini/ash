# TASK-877: Interface-bound proposition solving

## Status: 🟡 Ready

## Description

Solve interface-bound propositions from existing TypeEnv evidence without broadening impl search or associated-family selection.

## Specification Reference

- [SPEC-064: Constraint and Proposition Layer](../../spec/SPEC-064-CONSTRAINT-PROPOSITION-LAYER.md)
- [PLAN-112: Constraint and Proposition Layer](../PLAN-112-CONSTRAINT-PROPOSITION-LAYER.md)
- [DESIGN-034 §16.8](../../design/DESIGN-034-TOTAL-TYPE-COMPUTATION.md#168-spec-h-constraintproposition-layer)

## Dependencies

- Depends on TASK-876 completion

## Files / Ownership

- Modify: `crates/ash-typeck/src/type_env.rs`
- Modify: `crates/ash-typeck/src/error.rs`, `diagnostic.rs` as needed
- Test: `crates/ash-typeck/tests/task_877_interface_bound_propositions.rs`
- Regression: `crates/ash-typeck/tests/task_864_rigid_where_bound_projection.rs`
- Audit rows: H-AUD-TYPECK-01, H-AUD-TYPECK-02, H-AUD-TYPECK-03, H-FORCE-05, H-RISK-02

## TASK-872 Binding Notes

- Solve interface-bound propositions only from exact in-scope generic where-bound evidence or already-selected concrete impl evidence.
- Keep associated-family equation selection and rigid where-bound projection behavior separate from proposition-bound solving.
- Missing evidence must reject/defer without broad arbitrary impl search.

## Requirements

### Functional Requirements

1. Satisfy interface-bound propositions for known selected concrete impls.
2. Satisfy interface-bound propositions for exact in-scope generic where-bound evidence.
3. Reject/defer missing evidence without searching arbitrary impl candidates.
4. Keep where-bound evidence separate from associated-family equation selection.
5. Preserve Phase 115 rigid where-bound projection behavior.

### Non-Goals

- Do not implement type-function inversion, injectivity, unrestricted SMT/proof search, HKT, holes, or partial type-constructor application.
- Do not merge type-level propositions with runtime workflow/capability/provider constraints.
- Do not move semantic proposition ownership into `ash-parser` or `ash-engine`.
- Preserve SPEC-035/SPEC-063 associated-type behavior unless the task explicitly assigns compatibility coverage.

## TDD / Execution Steps

### Step 1

- Write tests for known impl evidence and generic where-bound evidence.

### Step 2

- Write tests for missing bound evidence and no broad search.

### Step 3

- Implement interface-bound solver path.

### Step 4

- Run SPEC-063 rigid where-bound regressions.

### Independent Verification

Dispatch an independent review/verification subagent with this task file, SPEC-064, PLAN-112, changed files, and any task-owned audit/test evidence. Do not mark this task complete until findings are fixed and verification passes.

## Completion Checklist

- [ ] Focused interface-bound tests pass.
- [ ] No associated-family equation is selected solely because a bound exists.
- [ ] Existing impl lookup behavior remains unchanged.

## Dispatch

```yaml
agent: hermes
reasoning: medium
max_turns: 15
toolsets: [terminal, file]
```

## Verification

```yaml
strictness: clean
commands:
  - cargo fmt --check
  - git diff --check
  - cargo check --workspace
  - test -f crates/ash-typeck/tests/task_877_interface_bound_propositions.rs
  - cargo test -p ash-typeck --test task_877_interface_bound_propositions -- --list | grep -q task_877_
  - cargo test -p ash-typeck --test task_877_interface_bound_propositions
  - cargo test -p ash-typeck --test task_864_rigid_where_bound_projection
checklist:
  - "[ ] Task requirements are satisfied"
  - "[ ] Focused verification is recorded"
  - "[ ] Status docs and CHANGELOG.md are updated if release-facing docs changed"
```

## Dependencies for Next Task

This task outputs:
- Phase 116 artifact/surface owned by TASK-877 for downstream tasks.
