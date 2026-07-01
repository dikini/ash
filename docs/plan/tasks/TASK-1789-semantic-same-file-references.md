# TASK-1789: Replace token-only same-file references with semantic macro/function splitting

## Status: ✅ Complete

## Description

Replace lexical same-file reference scans with semantic reference splitting for supported same-file macro and callable identities. Unsupported identity cases must remain honest no-result/limited-result behavior.

## Specification Reference

- [PLAN-175: Name-Resolution-Backed Semantic Identity for Macros and Tooling](../PLAN-175-NAME-RESOLUTION-BACKED-SEMANTIC-IDENTITY-FOR-MACROS-AND-TOOLING.md)
- [SPEC-095c: Surface AST, Macro Expansion, and Notation](../../spec/SPEC-095c-SURFACE-AST-MACROS-AND-NOTATION.md)
- [SPEC-038: Language Server](../../spec/SPEC-038-LANGUAGE-SERVER.md)
- [SPEC-098c: Surface-to-Core Lowering](../../spec/SPEC-098c-SURFACE-TO-CORE-LOWERING.md)

## Dependencies

- 📝 TASK-1788: LSP summary identity threading

## Deferral / Planned-Feature Reconciliation

| Prior item | Source | Original reason | Prereqs now? | Decision | Gate |
|---|---|---|---|---|---|
| Token-only LSP references | Phase 174 closeout | No resolved semantic identity substrate | This task contributes to substrate | Replace only when resolved identity is proven | Same-name macro/function negative tests |
| Cross-file macro navigation | Phase 174 non-goal | No imported macro identity contract or workspace graph | Summary identity can be designed; graph still absent | Prepare summary-identity boundary only | Unsupported cases return honest no-result |

## Requirements

### Functional Requirements

1. Update `crates/ash-lsp-core/src/goto.rs` reference helpers to group references by resolved identity instead of token spelling where identity is available.
2. Add tests where `macro id(x)` and `fn id()` coexist and references for each do not cross-contaminate.
3. Preserve existing lexical behavior only as a documented fallback for identity-free tokens, if the audit accepts a fallback.
4. Document limitations in code comments and task evidence.

### Property Requirements

- Macro identity must remain syntax-phase metadata.
- Macro identity must not imply runtime callability or callable export authority.
- Any reference/navigation behavior must fail closed when identity is ambiguous or unavailable.

## TDD Steps

### Step 1: Write failing reference tests

Add LSP tests for same-name macro/function declarations, macro invocations, ordinary calls, and declarations.

### Step 2: Implement semantic splitting

Use resolved identity keys from TASK-1788 to filter references.

### Step 3: Verify fallback honesty

Add unsupported/ambiguous cases that do not overclaim semantic references.

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
  - cargo test -p ash-lsp-core goto::tests::test_find_references -- --nocapture
  - cargo test -p ash-lsp-core -- --nocapture
  - cargo fmt --check
checklist:
  - [x] Same-file references split macro/function identities
  - [x] Ambiguous cases do not fabricate semantic refs
  - [x] Existing non-macro reference tests still pass
```

## Dependencies for Next Task

This task feeds the following Phase 175 tasks according to the dependency table in PLAN-175.

## Completion Evidence

- Updated same-file references to semantically split `m!(...)` macro uses from `m()` callable uses when identity is proven.
