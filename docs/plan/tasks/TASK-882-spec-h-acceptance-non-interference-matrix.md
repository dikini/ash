# TASK-882: SPEC-H acceptance/non-interference matrix

## Status: 🟡 Ready

## Description

Create the Phase 116 acceptance/non-interference matrix and focused aggregator evidence for every SPEC-064 §12 row.

## Specification Reference

- [SPEC-064: Constraint and Proposition Layer](../../spec/SPEC-064-CONSTRAINT-PROPOSITION-LAYER.md)
- [PLAN-112: Constraint and Proposition Layer](../PLAN-112-CONSTRAINT-PROPOSITION-LAYER.md)
- [DESIGN-034 §16.8](../../design/DESIGN-034-TOTAL-TYPE-COMPUTATION.md#168-spec-h-constraintproposition-layer)

## Dependencies

- Depends on TASK-881 completion

## Files / Ownership

- Create: `docs/plan/audits/TASK-882-proposition-acceptance-matrix.md`
- Modify/add focused aggregator tests bound by TASK-872
- Modify: task/status docs if evidence changes

## Requirements

### Functional Requirements

1. Map every SPEC-064 §12 row H1 through H12 to exact focused test evidence; H3 and other deferred-behavior rows still require focused tests proving the expected diagnostic/deferred outcome.
2. Include command, test count, expected result, actual result, and owning earlier task for every row.
3. Add focused aggregator tests when a row lacks evidence.
4. Run non-interference suites for SPEC-035 and SPEC-057 through SPEC-063 named by TASK-872.

### Non-Goals

- Do not implement type-function inversion, injectivity, unrestricted SMT/proof search, HKT, holes, or partial type-constructor application.
- Do not merge type-level propositions with runtime workflow/capability/provider constraints.
- Do not move semantic proposition ownership into `ash-parser` or `ash-engine`.
- Preserve SPEC-035/SPEC-063 associated-type behavior unless the task explicitly assigns compatibility coverage.

## TDD / Execution Steps

### Step 1

- Create acceptance matrix artifact.

### Step 2

- Run all focused tests with non-zero guards.

### Step 3

- Patch missing evidence with focused tests; a documentation-only deferral is allowed only after SPEC-064 and PLAN-112 are explicitly amended to narrow the acceptance row.

### Step 4

- Record exact commands and outcomes.

### Independent Verification

Dispatch an independent review/verification subagent with this task file, SPEC-064, PLAN-112, changed files, and any task-owned audit/test evidence. Do not mark this task complete until findings are fixed and verification passes.

## Completion Checklist

- [ ] Matrix artifact exists and covers H1-H12.
- [ ] All required focused evidence passes.
- [ ] No zero-test pass is accepted.

## Dispatch

```yaml
agent: hermes
reasoning: low
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
- Phase 116 artifact/surface owned by TASK-882 for downstream tasks.
