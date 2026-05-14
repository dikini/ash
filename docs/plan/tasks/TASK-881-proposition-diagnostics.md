# TASK-881: Proposition diagnostics

## Status: ✅ Complete

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
- Modify: `crates/ash-parser/src/parse_module.rs` and `crates/ash-parser/src/parse_type_def.rs` only for parser-owned unsupported-surface/malformed proposition diagnostics
- Test: `crates/ash-typeck/tests/task_881_proposition_diagnostics.rs`
- Test: `crates/ash-parser/tests/task_881_proposition_parse_diagnostics.rs`
- Audit rows: H-AUD-TYPECK-06, H-AUD-PARSE-03, H-AUD-PARSE-04, H-FORCE-09, H-RISK-04

## TASK-872 Binding Notes

- Typeck diagnostics must distinguish unsupported propositions, unknown predicates, deferred predicates, neutral/rigid blockers, no-inversion boundaries, malformed V5 summaries, and private leaks.
- Parser diagnostics are limited to syntax/unsupported-surface errors; semantic proposition errors remain in TypeEnv diagnostics.
- No diagnostic may imply Ash performed type-function inversion, arbitrary proof search, or associated-family output solving.

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

- [x] Focused diagnostic tests pass.
- [x] Every SPEC-064 §11 diagnostic family has coverage.
- [x] Messages avoid claiming unsupported proof search succeeded.

## Completion Evidence

- Added stable proposition diagnostic routing for SPEC-064 §11 families E166 and E168 through E177.
- Added parser-owned E168 coverage through `parse_surface_file(...)` for disabled type-alias proposition tails while preserving generic E001 for malformed workflow equality, workflow-body `where`, and legacy impl `where` parse failures.
- Added live TypeEnv required-discharge diagnostics for E169 through E174, including neutral equality E170, rigid projection equality E171, open/neutral disequality E172, equality-refuted disequality E173, and missing interface evidence E174.
- Added fail-closed summary diagnostics E175/E176 for malformed pre-V5 proposition payloads and private predicate leaks.
- Kept E177 as a stable reserved no-inversion family while avoiding spurious E177 masking for precise live solver causes.
- Verification: `cargo test -p ash-parser --test task_881_proposition_parse_diagnostics`; `cargo test -p ash-typeck --test task_881_proposition_diagnostics`; `cargo test -p ash-typeck --test task_878_named_predicate_registration`; `cargo test -p ash-typeck --test task_879_proposition_summary_import`; `cargo fmt --check`; `git diff --check`; `cargo check --workspace`; `cargo clippy -p ash-parser -p ash-typeck --all-targets --all-features -- -D warnings`.
- Independent review verdict: PASS after remediation.

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
  - test -f crates/ash-typeck/tests/task_881_proposition_diagnostics.rs
  - cargo test -p ash-typeck --test task_881_proposition_diagnostics -- --list | grep -q task_881_
  - cargo test -p ash-typeck --test task_881_proposition_diagnostics
  - test -f crates/ash-parser/tests/task_881_proposition_parse_diagnostics.rs
  - cargo test -p ash-parser --test task_881_proposition_parse_diagnostics -- --list | grep -q task_881_
  - cargo test -p ash-parser --test task_881_proposition_parse_diagnostics
checklist:
  - "[x] Task requirements are satisfied"
  - "[x] Focused verification is recorded"
  - "[x] Status docs and CHANGELOG.md are updated if release-facing docs changed"
```

## Dependencies for Next Task

This task outputs:
- Phase 116 artifact/surface owned by TASK-881 for downstream tasks.
