# TASK-1783: Close out Phase 174 with broad gates and review

## Status: ✅ Complete

## Description

Close Phase 174 by running focused and broad verification, resolving independent review findings, and reconciling PLAN-174, PLAN-INDEX, task files, specs/docs, and CHANGELOG. This task must not close with accepted blockers unless the user explicitly chooses to defer them and the deferral is recorded as a follow-on.

## Specification Reference

- [PLAN-174: Macro-Aware Tooling, Summary Identity, and Inference Readiness](../PLAN-174-MACRO-AWARE-TOOLING-SUMMARY-IDENTITY-AND-INFERENCE-READINESS.md)
- TASK-1775 through TASK-1782

## Dependencies

- ✅ TASK-1775: Macro-aware tooling audit
- ✅ TASK-1776: Macro-specific symbol/cache model
- ✅ TASK-1777: Macro completion/hover UX
- ✅ TASK-1778: Macro goto/reference boundaries
- ✅ TASK-1779: Callable identity audit
- ✅ TASK-1780: Bounded callable identity inference
- ✅ TASK-1781: Cross-boundary validation
- ✅ TASK-1782: Docs/spec reconciliation

## Deferral / Planned-Feature Reconciliation

| Prior item | Source | Original reason | Prereqs now? | Decision | Gate |
|---|---|---|---|---|---|
| Imported notation propagation | PLAN-170 follow-on | Needs notation summary carriers | Not owned by Phase 174 | Keep deferred | Closeout lists as future owner |
| Generalized mixfix/binder notation | PLAN-170/171 follow-ons | Needs separate grammar/hygiene plan | Not owned by Phase 174 | Keep deferred | Closeout avoids completion language |
| Broad SPEC-098c lowering completion | PLAN-170 follow-on | Too broad | Not owned by Phase 174 | Keep deferred | Closeout lists as future owner |

## Requirements

### Functional Requirements

1. Run all focused Phase 174 tests and prior Phase 173 macro boundary regressions.
2. Run broad parser/typechecker/engine/LSP/workspace/clippy/format/docs gates.
3. Obtain independent review of the Phase 174 diff.
4. Fix or explicitly defer every review finding.
5. Mark all completed task files, PLAN-174, PLAN-INDEX, and CHANGELOG consistently.

### Property Requirements

- Macro metadata remains syntax-phase metadata and cannot grant runtime callability.
- All new inference behavior is backed by unique callable identity proof and negative ambiguity tests.
- Tooling improvements must not weaken parser/engine/typechecker fail-closed boundaries.

## TDD Steps

### Step 1: Run focused verification

Run focused LSP, parser, engine, and typechecker macro tests listed in TASK-1781 and TASK-1780.

### Step 2: Run broad gates

Run the full baseline command list from PLAN-174.

### Step 3: Independent review

Request review of the final diff for overclaiming, boundary leaks, stale docs, and missing negative tests.

### Step 4: Reconcile status surfaces

Patch task files, PLAN-174, PLAN-INDEX, and CHANGELOG only after verification and review are clean or explicitly deferred by the user.

## Dispatch

```yaml
agent: hermes
reasoning: high
max_turns: 20
toolsets: [terminal, file]
```

## Verification

```yaml
strictness: clean
commands:
  - cargo fmt --check
  - cargo test -p ash-parser
  - cargo test -p ash-typeck
  - cargo test -p ash-engine
  - cargo test -p ash-lsp-core
  - cargo check --workspace
  - cargo clippy --workspace --all-targets --all-features -- -D warnings
  - git diff --check
  - python3 tools/docs/validate_orientation_indexes.py --self-test
  - bash scripts/check-docs-gate.sh
checklist:
  - [x] Focused Phase 174 tests pass
  - [x] Broad gates pass
  - [x] Independent review complete
  - [x] Status surfaces and changelog agree
  - [x] Remaining non-goals/follow-ons listed honestly
```

## Dependencies for Next Task

Closeout should recommend the next phase only after review. Likely candidates are imported notation summary carriers, generalized mixfix/binder notation, or broad surface-to-Core lowering completion depending on Phase 174 findings.

## Completion Evidence

- Independent review found and remediation addressed: callable-backed inferred signatures now validate with the same callable environment; macro/function same-name goto and hover contexts are disambiguated; macro cache keys include parameter names; parser negatives cover private, wrong-type, and macro-summary-derived call cases; task dependency status drift was repaired.
- Focused verification passed: `cargo test -p ash-parser --test task_1772_macro_type_inference -- --nocapture` (12 passed) and `cargo test -p ash-lsp-core -- --nocapture` (79 unit tests + LSP integration + doctests passed).
- Broad closeout gate passed: `cargo fmt --check && cargo test -p ash-parser && cargo test -p ash-typeck && cargo test -p ash-engine && cargo test -p ash-lsp-core && cargo check --workspace && cargo clippy --workspace --all-targets --all-features -- -D warnings && git diff --check && python3 tools/docs/validate_orientation_indexes.py --self-test && bash scripts/check-docs-gate.sh`.
