# TASK-1791: Validate identity threading does not make macros runtime-callable

## Status: ✅ Complete

## Description

Add cross-boundary positive and negative regressions proving semantic identity threading improves tooling without turning macros or imported macro summaries into runtime callable bindings.

## Specification Reference

- [PLAN-175: Name-Resolution-Backed Semantic Identity for Macros and Tooling](../PLAN-175-NAME-RESOLUTION-BACKED-SEMANTIC-IDENTITY-FOR-MACROS-AND-TOOLING.md)
- [SPEC-095c: Surface AST, Macro Expansion, and Notation](../../spec/SPEC-095c-SURFACE-AST-MACROS-AND-NOTATION.md)
- [SPEC-038: Language Server](../../spec/SPEC-038-LANGUAGE-SERVER.md)
- [SPEC-098c: Surface-to-Core Lowering](../../spec/SPEC-098c-SURFACE-TO-CORE-LOWERING.md)

## Dependencies

- 📝 TASK-1787: Parser-local resolved identities
- 📝 TASK-1790: Imported macro navigation preparation

## Deferral / Planned-Feature Reconciliation

| Prior item | Source | Original reason | Prereqs now? | Decision | Gate |
|---|---|---|---|---|---|
| Token-only LSP references | Phase 174 closeout | No resolved semantic identity substrate | This task contributes to substrate | Replace only when resolved identity is proven | Same-name macro/function negative tests |
| Cross-file macro navigation | Phase 174 non-goal | No imported macro identity contract or workspace graph | Summary identity can be designed; graph still absent | Prepare summary-identity boundary only | Unsupported cases return honest no-result |

## Requirements

### Functional Requirements

1. Add parser regressions for macro identity versus callable identity in expansion/inference paths.
2. Add engine/module-loader regressions proving imported macro identity metadata does not create callable export/import authority.
3. Add LSP regressions proving identity improves navigation/references without changing runtime semantics.
4. Check sibling call paths for macro summary leakage into callable environments.

### Property Requirements

- Macro identity must remain syntax-phase metadata.
- Macro identity must not imply runtime callability or callable export authority.
- Any reference/navigation behavior must fail closed when identity is ambiguous or unavailable.

## TDD Steps

### Step 1: Write cross-boundary tests

Add parser, engine, and LSP tests before any final implementation patches.

### Step 2: Fix leaks found by tests

Patch parser/LSP/engine code paths so macro identity remains syntax-phase only.

### Step 3: Run broad focused suite

Run parser, engine, LSP, and typechecker-adjacent tests.

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
  - cargo test -p ash-parser --test task_1772_macro_type_inference -- --nocapture
  - cargo test -p ash-engine --test task_1773_phase_173_boundaries -- --nocapture
  - cargo test -p ash-lsp-core -- --nocapture
  - cargo check -p ash-parser -p ash-engine -p ash-lsp-core
checklist:
  - [x] Macro identities do not enter callable envs
  - [x] Imported macro identities do not become callable exports
  - [x] LSP identity behavior has negative leakage tests
```

## Dependencies for Next Task

This task feeds the following Phase 175 tasks according to the dependency table in PLAN-175.

## Completion Evidence

- Validated non-callability with parser and engine negative tests, including existing Phase 173/174 macro-summary leakage regressions.
