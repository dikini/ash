# TASK-1777: Implement macro-aware completion and hover/signature presentation

## Status: 📝 Planned

## Description

Update LSP completion and hover so macros are presented as syntax-phase macros, not runtime functions. Hover should show macro parameters and typed macro signatures when available, while completion should use a macro-specific kind/label/detail that does not imply ordinary callable authority.

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
| Completion presents macros as functions | Live `completion.rs` | No macro-specific LSP model | Yes after TASK-1776 | Change completion kind/detail/insert text honestly | Regression asserts macro completion is macro-specific |
| Hover omits typed macro signatures | Phase 173 typed carriers are new | Tooling not updated yet | Yes | Show syntax-phase signature when source has it | Regression checks typed signature text |

## Requirements

### Functional Requirements

1. Update `crates/ash-lsp-core/src/completion.rs` so `Definition::Macro` completions use macro-specific metadata and do not share the ordinary function branch.
2. Update `crates/ash-lsp-core/src/hover.rs` so macro hover displays `macro name(params)` plus typed parameter/result annotations when available.
3. Add hover/completion regressions for untyped macros, typed macros, and ordinary functions to prove the distinction.
4. Keep notation completions as operator-like and ordinary functions as functions.

### Property Requirements

- Macro UI text must say macro/syntax-phase, not callable/function authority.
- Typed signature display must be derived from `MacroTypeSignatureSummary`, not string reparsing of source snippets.
- Completion insert text must not introduce unsupported macro call syntax.

## TDD Steps

### Step 1: Write failing completion tests

Add tests showing a macro completion has macro-specific kind/detail and an ordinary `fn` still has function kind.

### Step 2: Write failing hover tests

Add tests for `macro id(x: Int) -> Int => x;` showing hover includes parameter/result type metadata and syntax-phase wording.

### Step 3: Implement completion and hover changes

Refactor only the macro branches in `completion.rs` and `hover.rs`; avoid drive-by context-aware completion work.

### Step 4: Verify focused tests

Run the new `ash-lsp-core` completion and hover tests.

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
  - cargo test -p ash-lsp-core completion::tests
  - cargo test -p ash-lsp-core hover::tests
  - cargo test -p ash-lsp-core
  - cargo fmt --check
  - cargo clippy -p ash-lsp-core --all-targets --all-features -- -D warnings
checklist:
  - [ ] Macro completions are macro-specific
  - [ ] Macro hover shows typed signatures when present
  - [ ] Ordinary function completion/hover behavior remains intact
```

## Dependencies for Next Task

TASK-1778 depends on the same macro-specific symbol vocabulary and should reuse the helper formatting introduced here where useful.
