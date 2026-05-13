# TASK-881: Proposition diagnostics

## Status: 🟡 Ready

## Description

Add structured diagnostics for unsupported propositions, neutral/rigid blockers, no-inversion boundaries, malformed summaries, and private leaks.

## Specification Reference

- [SPEC-064: Constraint and Proposition Layer](../../spec/SPEC-064-CONSTRAINT-PROPOSITION-LAYER.md)
- [PLAN-112: Constraint and Proposition Layer](../PLAN-112-CONSTRAINT-PROPOSITION-LAYER.md)
- [DESIGN-034 §16.8](../../design/DESIGN-034-TOTAL-TYPE-COMPUTATION.md#168-spec-h-constraintproposition-layer)

## Dependencies

- Depends on TASK-880 completion

## Files / Ownership

- Modify: `crates/ash-typeck/src/error.rs`
- Modify: `crates/ash-typeck/src/diagnostic.rs`
- Modify parser diagnostics if TASK-872 assigns unsupported-surface errors there
- Test: exact diagnostic targets bound by TASK-872

## Requirements

### Functional Requirements

1. Add stable diagnostic codes for every SPEC-064 §11 family.
2. Ensure diagnostics include span/source anchor, expected/found proposition shape, solver rule/deferred reason, and likely fix.
3. Make no-inversion diagnostics explicitly say Ash will not solve type-function or associated-family inputs from outputs.
4. Keep malformed summary and private leak diagnostics fail-closed.

### Non-Goals

- Do not implement type-function inversion, injectivity, unrestricted SMT/proof search, HKT, holes, or partial type-constructor application.
- Do not merge type-level propositions with runtime workflow/capability/provider constraints.
- Do not move semantic proposition ownership into `ash-parser` or `ash-engine`.
- Preserve SPEC-035/SPEC-063 associated-type behavior unless the task explicitly assigns compatibility coverage.

## TDD / Execution Steps

### Step 1

- Write focused diagnostic tests asserting code, severity, span, and key message tokens.

### Step 2

- Wire errors to diagnostics.

### Step 3

- Verify unsupported named predicate, neutral equality, rigid projection, disequality-open, malformed V5, and private-leak cases.

### Independent Verification

Dispatch an independent review/verification subagent with this task file, SPEC-064, PLAN-112, changed files, and any task-owned audit/test evidence. Do not mark this task complete until findings are fixed and verification passes.

## Completion Checklist

- [ ] Focused diagnostic tests pass.
- [ ] Every SPEC-064 §11 diagnostic family has coverage.
- [ ] Messages avoid claiming unsupported proof search succeeded.

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
- Phase 116 artifact/surface owned by TASK-881 for downstream tasks.
