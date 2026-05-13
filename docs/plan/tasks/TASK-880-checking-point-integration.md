# TASK-880: Checking-point integration

## Status: 🟡 Ready

## Description

Integrate proposition generation/discharge at audited checking points without accidental inversion, meta-solving, or parser-owned semantics.

## Specification Reference

- [SPEC-064: Constraint and Proposition Layer](../../spec/SPEC-064-CONSTRAINT-PROPOSITION-LAYER.md)
- [PLAN-112: Constraint and Proposition Layer](../PLAN-112-CONSTRAINT-PROPOSITION-LAYER.md)
- [DESIGN-034 §16.8](../../design/DESIGN-034-TOTAL-TYPE-COMPUTATION.md#168-spec-h-constraintproposition-layer)

## Dependencies

- Depends on TASK-879 completion

## Files / Ownership

- Modify: `crates/ash-typeck/src/type_env.rs`
- Inspect: `crates/ash-typeck/src/normalizer.rs`; consume existing public helper APIs from `type_env.rs` only, and do not add new reduction behavior here
- Modify: `crates/ash-typeck/src/error.rs`, `diagnostic.rs` as needed for required-discharge failures
- Modify: `crates/ash-engine/src/lib.rs` for imported V5 summary handoff into TypeEnv; the engine remains transport-only and must not solve propositions
- Test: `crates/ash-typeck/tests/task_880_proposition_checking_points.rs`
- Test: `crates/ash-engine/tests/task_880_proposition_public_integration.rs`
- Audit rows: H-FORCE-08, H-RISK-01, H-RISK-02, H-RISK-03, H-RISK-04, H-RISK-05, H-AUD-TYPECK-01, H-AUD-TYPECK-04, H-AUD-TYPECK-07, H-AUD-ENGINE-01

## TASK-872 Binding Notes

- Generate and discharge obligations only at audited public signature/type-function/impl/fn/builtin/imported-summary checking points.
- Required-discharge sites must reject refuted or deferred propositions with diagnostics; assumption sites must be explicit.
- No proposition path may mutate substitutions for open type-function/associated-family outputs or move semantics into parser/engine.

## Requirements

### Functional Requirements

1. Generate proposition obligations at audited source sites.
2. Discharge required propositions using the conservative solver.
3. Treat deferred propositions as errors only where the checking context requires discharge.
4. Allow assumptions only in contexts explicitly classified by SPEC-064/TASK-872.
5. Verify no current inference metas are solved by proposition equality except through pre-existing top-level typechecking paths.

### Non-Goals

- Do not implement type-function inversion, injectivity, unrestricted SMT/proof search, HKT, holes, or partial type-constructor application.
- Do not merge type-level propositions with runtime workflow/capability/provider constraints.
- Do not move semantic proposition ownership into `ash-parser` or `ash-engine`.
- Preserve SPEC-035/SPEC-063 associated-type behavior unless the task explicitly assigns compatibility coverage.

## TDD / Execution Steps

### Step 1

- Write integration tests for public signatures/type functions/impl surfaces selected by the audit.

### Step 2

- Write negative no-inversion tests at actual checking entrypoints.

### Step 3

- Wire proposition discharge into the selected checking points.

### Step 4

- Run focused non-interference suites from SPEC-057 through SPEC-063 named by TASK-872.

### Independent Verification

Dispatch an independent review/verification subagent with this task file, SPEC-064, PLAN-112, changed files, and any task-owned audit/test evidence. Do not mark this task complete until findings are fixed and verification passes.

## Completion Checklist

- [ ] Focused integration tests pass.
- [ ] Deferred propositions surface user-facing diagnostics at required-discharge sites.
- [ ] No proposition path mutates substitutions for open type-function arguments.

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
  - test -f crates/ash-typeck/tests/task_880_proposition_checking_points.rs
  - cargo test -p ash-typeck --test task_880_proposition_checking_points -- --list | grep -q task_880_
  - cargo test -p ash-typeck --test task_880_proposition_checking_points
  - test -f crates/ash-engine/tests/task_880_proposition_public_integration.rs
  - cargo test -p ash-engine --test task_880_proposition_public_integration -- --list | grep -q task_880_
  - cargo test -p ash-engine --test task_880_proposition_public_integration
checklist:
  - "[ ] Task requirements are satisfied"
  - "[ ] Focused verification is recorded"
  - "[ ] Status docs and CHANGELOG.md are updated if release-facing docs changed"
```

## Dependencies for Next Task

This task outputs:
- Phase 116 artifact/surface owned by TASK-880 for downstream tasks.
