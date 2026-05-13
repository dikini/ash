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
- Test: exact ash-typeck test target bound by TASK-872

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
  - |
    python3 - <<'PY'
    raise SystemExit('TASK-872 must replace this intentional verification guard with exact non-zero focused test commands before implementation can be verified')
    PY
checklist:
  - "[ ] Task requirements are satisfied"
  - "[ ] Focused verification is recorded"
  - "[ ] Status docs and CHANGELOG.md are updated if release-facing docs changed"
```

## Dependencies for Next Task

This task outputs:
- Phase 116 artifact/surface owned by TASK-877 for downstream tasks.
