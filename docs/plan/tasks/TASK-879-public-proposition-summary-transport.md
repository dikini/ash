# TASK-879: Public proposition summary transport

## Status: 🟡 Ready

## Description

Export/import public proposition requirements and optional evidence through V5 semantic summaries while preserving private-dependency opacity.

## Specification Reference

- [SPEC-064: Constraint and Proposition Layer](../../spec/SPEC-064-CONSTRAINT-PROPOSITION-LAYER.md)
- [PLAN-112: Constraint and Proposition Layer](../PLAN-112-CONSTRAINT-PROPOSITION-LAYER.md)
- [DESIGN-034 §16.8](../../design/DESIGN-034-TOTAL-TYPE-COMPUTATION.md#168-spec-h-constraintproposition-layer)

## Dependencies

- Depends on TASK-878 completion

## Files / Ownership

- Modify: `crates/ash-core/src/semantic_summary.rs`
- Modify: `crates/ash-typeck/src/type_env.rs`
- Modify: `crates/ash-engine/src/lib.rs`
- Modify: `crates/ash-engine/src/module_loader.rs`
- Test: `crates/ash-core/tests/task_879_proposition_summary_schema.rs`
- Test: `crates/ash-typeck/tests/task_879_proposition_summary_import.rs`
- Test: `crates/ash-engine/tests/task_879_proposition_summary_transport.rs`
- Audit rows: H-AUD-CORE-03, H-AUD-TYPECK-07, H-AUD-ENGINE-01, H-AUD-ENGINE-02, H-SUM-01, H-SUM-02, H-SUM-03, H-SUM-04, H-FORCE-07, H-RISK-04, H-RISK-05

## TASK-872 Binding Notes

- Core owns V5 schema/version validation; TypeEnv owns export/import revalidation and private-leak checks; engine only transports `ModuleSemanticSummary` payloads.
- Proposition payloads in V4-or-older summaries must be rejected before partial registration.
- Public summaries may transport requirements even if proof evidence export is deferred, but must not expose private helper type functions/domains/families/predicates.

## Requirements

### Functional Requirements

1. Export public proposition requirements that appear in public signatures or public type-computation summaries.
2. Use V5 summaries for proposition facts; reject V4-or-older proposition payloads.
3. Revalidate imported proposition requirements before using them.
4. Reject private helper type functions, private domains, private families, or private predicates in public proposition summaries.
5. Transport requirements even if proof evidence export is deferred; emit explicit deferred evidence diagnostics when needed.

### Non-Goals

- Do not implement type-function inversion, injectivity, unrestricted SMT/proof search, HKT, holes, or partial type-constructor application.
- Do not merge type-level propositions with runtime workflow/capability/provider constraints.
- Do not move semantic proposition ownership into `ash-parser` or `ash-engine`.
- Preserve SPEC-035/SPEC-063 associated-type behavior unless the task explicitly assigns compatibility coverage.

## TDD / Execution Steps

### Step 1

- Write V5 serde/version/cache tests.

### Step 2

- Write engine transport tests for named/glob/pub-use paths identified by the audit.

### Step 3

- Write TypeEnv import revalidation tests.

### Step 4

- Write private-leakage and malformed-version tests.

### Independent Verification

Dispatch an independent review/verification subagent with this task file, SPEC-064, PLAN-112, changed files, and any task-owned audit/test evidence. Do not mark this task complete until findings are fixed and verification passes.

## Completion Checklist

- [ ] Focused core/typeck/engine tests pass.
- [ ] V4-with-proposition facts is rejected before partial registration.
- [ ] Private dependency leakage is fail-closed.

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
  - test -f crates/ash-core/tests/task_879_proposition_summary_schema.rs
  - cargo test -p ash-core --test task_879_proposition_summary_schema -- --list | grep -q task_879_
  - cargo test -p ash-core --test task_879_proposition_summary_schema
  - test -f crates/ash-typeck/tests/task_879_proposition_summary_import.rs
  - cargo test -p ash-typeck --test task_879_proposition_summary_import -- --list | grep -q task_879_
  - cargo test -p ash-typeck --test task_879_proposition_summary_import
  - test -f crates/ash-engine/tests/task_879_proposition_summary_transport.rs
  - cargo test -p ash-engine --test task_879_proposition_summary_transport -- --list | grep -q task_879_
  - cargo test -p ash-engine --test task_879_proposition_summary_transport
checklist:
  - "[ ] Task requirements are satisfied"
  - "[ ] Focused verification is recorded"
  - "[ ] Status docs and CHANGELOG.md are updated if release-facing docs changed"
```

## Dependencies for Next Task

This task outputs:
- Phase 116 artifact/surface owned by TASK-879 for downstream tasks.
