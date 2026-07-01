# TASK-1787: Add parser-local resolved macro/callable identity carriers

## Status: ✅ Complete

## Description

Thread parser-local resolved identity through macro invocation and ordinary-call analysis where the current parsed module/file proves a unique declaration. Ambiguous or unavailable identities must remain absent.

## Specification Reference

- [PLAN-175: Name-Resolution-Backed Semantic Identity for Macros and Tooling](../PLAN-175-NAME-RESOLUTION-BACKED-SEMANTIC-IDENTITY-FOR-MACROS-AND-TOOLING.md)
- [SPEC-095c: Surface AST, Macro Expansion, and Notation](../../spec/SPEC-095c-SURFACE-AST-MACROS-AND-NOTATION.md)
- [SPEC-038: Language Server](../../spec/SPEC-038-LANGUAGE-SERVER.md)
- [SPEC-098c: Surface-to-Core Lowering](../../spec/SPEC-098c-SURFACE-TO-CORE-LOWERING.md)

## Dependencies

- 📝 TASK-1786: Canonical macro identity model

## Deferral / Planned-Feature Reconciliation

| Prior item | Source | Original reason | Prereqs now? | Decision | Gate |
|---|---|---|---|---|---|
| Token-only LSP references | Phase 174 closeout | No resolved semantic identity substrate | This task contributes to substrate | Replace only when resolved identity is proven | Same-name macro/function negative tests |
| Cross-file macro navigation | Phase 174 non-goal | No imported macro identity contract or workspace graph | Summary identity can be designed; graph still absent | Prepare summary-identity boundary only | Unsupported cases return honest no-result |

## Requirements

### Functional Requirements

1. Add resolved identity carriers for local macro invocations and ordinary calls without requiring a workspace graph.
2. Resolve same-file `m!(...)` to macro identity and `m()` to callable identity when unique.
3. Keep unresolved, ambiguous, private/imported-unsupported, or module-qualified references identity-free unless TASK-1790 explicitly owns them.
4. Add parser tests for same-name macro/function collisions and ambiguity.

### Property Requirements

- Macro identity must remain syntax-phase metadata.
- Macro identity must not imply runtime callability or callable export authority.
- Any reference/navigation behavior must fail closed when identity is ambiguous or unavailable.

## TDD Steps

### Step 1: Write parser identity tests

Add tests in `crates/ash-parser` proving macro invocations and ordinary calls resolve to different identities.

### Step 2: Implement local resolution

Add a local identity table and lookup path scoped to the parsed module/file.

### Step 3: Run focused parser tests

Verify unresolved and ambiguous cases fail closed.

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
  - cargo test -p ash-parser parser_local_identity -- --nocapture
  - cargo test -p ash-parser --test task_1772_macro_type_inference -- --nocapture
  - cargo fmt --check
checklist:
  - [x] Macro invocation identities resolve locally
  - [x] Ordinary callable identities remain separate
  - [x] Ambiguous/unresolved cases have no fabricated identity
```

## Dependencies for Next Task

This task feeds the following Phase 175 tasks according to the dependency table in PLAN-175.

## Completion Evidence

- Added parser-local macro and callable identity collection/resolution helpers and fail-closed absent-name coverage.
