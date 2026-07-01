# TASK-1778: Harden macro goto-definition, symbols, and references without callable overclaiming

## Status: 📝 Planned

## Description

Make goto-definition, document symbols, workspace/same-file symbols, and reference-style scans macro-aware without pretending that macro summaries are ordinary runtime callables. This task should make local macro navigation useful and explicitly document what imported-summary navigation does and does not support.

## Specification Reference

- [PLAN-174: Macro-Aware Tooling, Summary Identity, and Inference Readiness](../PLAN-174-MACRO-AWARE-TOOLING-SUMMARY-IDENTITY-AND-INFERENCE-READINESS.md)
- TASK-1775 audit artifact
- TASK-1776 macro symbol/cache model

## Dependencies

- 📝 TASK-1775: Macro-aware tooling audit
- 📝 TASK-1776: Macro-specific symbol/cache model

## Deferral / Planned-Feature Reconciliation

| Prior item | Source | Original reason | Prereqs now? | Decision | Gate |
|---|---|---|---|---|---|
| Macro document symbols are function-like | Live LSP code | No macro-specific symbol model | Yes after TASK-1776 | Use macro-specific symbol kind | Symbol tests distinguish macro/function |
| References are token scans | Phase 173 audit | Cross-file model not complete | Partial | Keep same-file honest; no broad cross-file claim | Negative imported/private overclaim test |

## Requirements

### Functional Requirements

1. Update `crates/ash-lsp-core/src/symbols.rs` and `crates/ash-lsp-core/src/db.rs` document-symbol construction to use the macro-specific symbol kind.
2. Update `crates/ash-lsp-core/src/goto.rs` so local macro names resolve to macro declarations without sharing ordinary callable semantics.
3. Add or update reference tests to cover macro declaration uses and avoid matching unrelated function names where possible.
4. Document imported macro summary navigation limits in code comments or docs if cross-file support is not implemented.

### Property Requirements

- Same-file local macro navigation must be precise enough for declarations and invocations.
- Imported macro summaries must not be reported as callable function definitions unless a real source location is available.
- Private template helpers must not become navigable public callables.

## TDD Steps

### Step 1: Write failing symbol tests

Add tests proving `macro m(x) => x;` appears as a macro symbol and `fn m()` appears as a function symbol.

### Step 2: Write failing goto/reference tests

Add tests for goto from `m!(1)` to `macro m(x) => x;`, plus a negative test that `fn m()` is not selected as the macro target.

### Step 3: Implement navigation changes

Update the smallest LSP modules needed by the tests. Keep cross-file behavior explicitly out of scope unless TASK-1775 proves the data is already available.

### Step 4: Verify LSP crate

Run focused LSP tests and the whole `ash-lsp-core` crate tests.

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
  - cargo test -p ash-lsp-core goto::tests
  - cargo test -p ash-lsp-core symbols::tests
  - cargo test -p ash-lsp-core db::tests
  - cargo test -p ash-lsp-core
  - cargo fmt --check
  - cargo clippy -p ash-lsp-core --all-targets --all-features -- -D warnings
checklist:
  - [ ] Macro document symbols use macro-specific identity
  - [ ] Goto from macro invocation resolves to macro declaration where supported
  - [ ] Tests prevent macro/function identity confusion
```

## Dependencies for Next Task

This task completes the core LSP-facing macro identity work needed before cross-boundary validation in TASK-1781.
