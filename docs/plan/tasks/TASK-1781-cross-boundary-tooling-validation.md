# TASK-1781: Add parser/engine/LSP cross-boundary tooling and inference validation

## Status: ✅ Complete

## Description

Validate Phase 174 as an integrated boundary. The tests must prove macro-aware tooling, cache invalidation, navigation, and bounded callable-identity inference agree across parser, engine/module-loader, typechecker-facing gates, and LSP-facing consumers.

## Specification Reference

- [PLAN-174: Macro-Aware Tooling, Summary Identity, and Inference Readiness](../PLAN-174-MACRO-AWARE-TOOLING-SUMMARY-IDENTITY-AND-INFERENCE-READINESS.md)
- TASK-1776 through TASK-1780

## Dependencies

- ✅ TASK-1776: Macro-specific symbol/cache model
- ✅ TASK-1777: Macro completion/hover UX
- ✅ TASK-1778: Macro goto/reference boundaries
- ✅ TASK-1780: Bounded callable identity inference

## Deferral / Planned-Feature Reconciliation

| Prior item | Source | Original reason | Prereqs now? | Decision | Gate |
|---|---|---|---|---|---|
| Cross-boundary macro metadata agreement | PLAN-173 closeout | Macro carriers were new | Yes | Add Phase 174 tooling/inference integration tests | Parser/engine/LSP tests pass together |
| Runtime callable leakage | Phase 173 negative tests | Macro imports must not become callables | Yes | Retest after tooling changes | Engine negative regression remains green |

## Requirements

### Functional Requirements

1. Add LSP-facing integration tests for macro symbol/cache/completion/hover/goto behavior in `crates/ash-lsp-core`.
2. Add parser/engine tests proving bounded callable-identity inference exports only syntax-phase typed macro summaries.
3. Retain or extend negative tests proving macro summaries do not create runtime callable bindings.
4. Validate that LSP macro presentation and engine/module-loader macro semantics use consistent names and signatures where they overlap.

### Property Requirements

- Every positive visibility/tooling test must have a matching negative leakage or overclaim test.
- LSP metadata must not create or imply engine/runtime authority.
- Imported macro summaries must remain syntax-phase metadata even when LSP can display them.

## TDD Steps

### Step 1: Write RED cross-boundary tests

Add tests that initially fail because one or more consumers still disagree about macro identity, signature, or callable status.

### Step 2: Implement minimal fixes

Patch the owning modules only where required by the failing tests. Do not broaden Phase 174 beyond tooling/identity/inference readiness.

### Step 3: Run focused cross-boundary suites

Run new LSP, parser, and engine tests together.

### Step 4: Run prior macro boundary suites

Rerun TASK-1773 and TASK-1771 focused tests to guard against regressions.

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
  - cargo test -p ash-lsp-core
  - cargo test -p ash-parser --test task_1780_callable_identity_inference -- --nocapture
  - cargo test -p ash-parser --test task_1773_phase_173_boundaries -- --nocapture
  - cargo test -p ash-engine --test task_1773_phase_173_boundaries -- --nocapture
  - cargo test -p ash-parser --test task_1771_typed_macro_checking -- --nocapture
  - cargo test -p ash-engine --test task_1771_typed_macro_boundaries -- --nocapture
  - cargo fmt --check
  - cargo clippy -p ash-parser -p ash-engine -p ash-lsp-core --all-targets --all-features -- -D warnings
checklist:
  - [x] LSP/parser/engine macro identity agree
  - [x] Runtime callable leakage remains rejected
  - [x] Previous Phase 173 boundary tests still pass
```

## Dependencies for Next Task

TASK-1782 uses these validation results when reconciling docs/spec/status language.

## Completion Evidence

- Added/ran parser and LSP regressions for macro cache keys, symbol identity, completion/hover, goto boundaries, and callable-identity inference; existing engine macro boundary tests remain part of closeout gates.
