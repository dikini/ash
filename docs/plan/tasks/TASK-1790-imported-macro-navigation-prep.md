# TASK-1790: Prepare imported macro navigation via summary identities without overclaiming

## Status: 📝 Planned

## Description

Prepare cross-file imported macro navigation by carrying/importing summary identity information in a form LSP can recognize later, while explicitly avoiding full workspace references or runtime callable semantics.

## Specification Reference

- [PLAN-175: Name-Resolution-Backed Semantic Identity for Macros and Tooling](../PLAN-175-NAME-RESOLUTION-BACKED-SEMANTIC-IDENTITY-FOR-MACROS-AND-TOOLING.md)
- [SPEC-095c: Surface AST, Macro Expansion, and Notation](../../spec/SPEC-095c-SURFACE-AST-MACROS-AND-NOTATION.md)
- [SPEC-038: Language Server](../../spec/SPEC-038-LANGUAGE-SERVER.md)
- [SPEC-098c: Surface-to-Core Lowering](../../spec/SPEC-098c-SURFACE-TO-CORE-LOWERING.md)

## Dependencies

- 📝 TASK-1788: LSP summary identity threading
- 📝 TASK-1789: Semantic same-file references

## Deferral / Planned-Feature Reconciliation

| Prior item | Source | Original reason | Prereqs now? | Decision | Gate |
|---|---|---|---|---|---|
| Token-only LSP references | Phase 174 closeout | No resolved semantic identity substrate | This task contributes to substrate | Replace only when resolved identity is proven | Same-name macro/function negative tests |
| Cross-file macro navigation | Phase 174 non-goal | No imported macro identity contract or workspace graph | Summary identity can be designed; graph still absent | Prepare summary-identity boundary only | Unsupported cases return honest no-result |

## Requirements

### Functional Requirements

1. Define imported macro summary identity fields and alias behavior.
2. Add parser/engine tests proving imported macro identity metadata survives summary transport but remains syntax-phase only.
3. Add LSP tests or fixtures for honest imported-summary navigation preparation, such as showing metadata but returning no cross-file location when source mapping is unavailable.
4. Document what would be needed for full cross-file goto/references in a future workspace-graph phase.

### Property Requirements

- Macro identity must remain syntax-phase metadata.
- Macro identity must not imply runtime callability or callable export authority.
- Any reference/navigation behavior must fail closed when identity is ambiguous or unavailable.

## TDD Steps

### Step 1: Write imported summary identity tests

Use engine/module-loader fixtures to prove identity metadata transport and alias behavior.

### Step 2: Implement summary identity fields

Extend macro summary transport with only the compact fields needed for future navigation.

### Step 3: Add honest LSP behavior

Expose metadata or no-result behavior without claiming source locations that are not known.

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
  - cargo test -p ash-engine --test task_1773_phase_173_boundaries -- --nocapture
  - cargo test -p ash-parser imported_macro_identity -- --nocapture
  - cargo test -p ash-lsp-core -- --nocapture
checklist:
  - [ ] Imported summary identity metadata transported
  - [ ] Aliases do not collapse distinct identities incorrectly
  - [ ] No cross-file navigation overclaim
```

## Dependencies for Next Task

This task feeds the following Phase 175 tasks according to the dependency table in PLAN-175.
