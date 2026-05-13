# TASK-875: TypeEnv proposition environment

## Status: 🟡 Ready

## Description

Add TypeEnv proposition environment, canonical lowering, source/generator provenance, and obligation tracking without solving beyond classification.

## Specification Reference

- [SPEC-064: Constraint and Proposition Layer](../../spec/SPEC-064-CONSTRAINT-PROPOSITION-LAYER.md)
- [PLAN-112: Constraint and Proposition Layer](../PLAN-112-CONSTRAINT-PROPOSITION-LAYER.md)
- [DESIGN-034 §16.8](../../design/DESIGN-034-TOTAL-TYPE-COMPUTATION.md#168-spec-h-constraintproposition-layer)

## Dependencies

- Depends on TASK-874 completion

## Files / Ownership

- Modify: `crates/ash-typeck/src/type_env.rs`
- Modify: `crates/ash-typeck/src/error.rs` and `diagnostic.rs` if new environment errors are needed
- Test: exact ash-typeck test target bound by TASK-872

## Requirements

### Functional Requirements

1. Lower raw source proposition clauses to core canonical propositions.
2. Record generated obligations with source anchors and owning checking site.
3. Track assumed facts separately from required-to-discharge obligations.
4. Preserve interface-bound facts from existing where-bound/impl evidence as proposition inputs.
5. Reject or defer unknown predicates with typed reasons, not strings.
6. Do not discharge propositions in this task except for trivial registry/classification checks.

### Non-Goals

- Do not implement type-function inversion, injectivity, unrestricted SMT/proof search, HKT, holes, or partial type-constructor application.
- Do not merge type-level propositions with runtime workflow/capability/provider constraints.
- Do not move semantic proposition ownership into `ash-parser` or `ash-engine`.
- Preserve SPEC-035/SPEC-063 associated-type behavior unless the task explicitly assigns compatibility coverage.

## TDD / Execution Steps

### Step 1

- Write failing tests for canonical lowering of equality/disequality/interface/named predicate propositions.

### Step 2

- Write tests proving generated obligations retain source anchors and owner site IDs.

### Step 3

- Implement TypeEnv storage and lowering APIs.

### Step 4

- Verify no equality/disequality solver decisions are made before TASK-876.

### Independent Verification

Dispatch an independent review/verification subagent with this task file, SPEC-064, PLAN-112, changed files, and any task-owned audit/test evidence. Do not mark this task complete until findings are fixed and verification passes.

## Completion Checklist

- [ ] Focused ash-typeck tests pass.
- [ ] Every proposition has source/generator provenance.
- [ ] Unknown predicate handling returns typed deferred/error information.

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
- Phase 116 artifact/surface owned by TASK-875 for downstream tasks.
