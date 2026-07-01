# TASK-1788: Thread resolved identities through LSP parse and symbol summaries

## Status: ✅ Complete

## Description

Expose compact resolved macro/callable identity keys to LSP parse summaries, symbol indexes, hover/goto/reference consumers, and cache invalidation without retaining full AST payloads.

## Specification Reference

- [PLAN-175: Name-Resolution-Backed Semantic Identity for Macros and Tooling](../PLAN-175-NAME-RESOLUTION-BACKED-SEMANTIC-IDENTITY-FOR-MACROS-AND-TOOLING.md)
- [SPEC-095c: Surface AST, Macro Expansion, and Notation](../../spec/SPEC-095c-SURFACE-AST-MACROS-AND-NOTATION.md)
- [SPEC-038: Language Server](../../spec/SPEC-038-LANGUAGE-SERVER.md)
- [SPEC-098c: Surface-to-Core Lowering](../../spec/SPEC-098c-SURFACE-TO-CORE-LOWERING.md)

## Dependencies

- 📝 TASK-1787: Parser-local resolved identities

## Deferral / Planned-Feature Reconciliation

| Prior item | Source | Original reason | Prereqs now? | Decision | Gate |
|---|---|---|---|---|---|
| Token-only LSP references | Phase 174 closeout | No resolved semantic identity substrate | This task contributes to substrate | Replace only when resolved identity is proven | Same-name macro/function negative tests |
| Cross-file macro navigation | Phase 174 non-goal | No imported macro identity contract or workspace graph | Summary identity can be designed; graph still absent | Prepare summary-identity boundary only | Unsupported cases return honest no-result |

## Requirements

### Functional Requirements

1. Extend `crates/ash-lsp-core/src/db.rs` summary/index structures with compact identity keys.
2. Thread identity through `symbols.rs`, `hover.rs`, and `goto.rs` where already proven by parser-local analysis.
3. Ensure identity changes invalidate relevant LSP cache summaries.
4. Add tests that same spelling but different identity changes are observable.

### Property Requirements

- Macro identity must remain syntax-phase metadata.
- Macro identity must not imply runtime callability or callable export authority.
- Any reference/navigation behavior must fail closed when identity is ambiguous or unavailable.

## TDD Steps

### Step 1: Write LSP summary identity tests

Add db/symbol tests showing identity keys distinguish macro and callable declarations.

### Step 2: Implement summary threading

Add compact identity fields and update construction paths.

### Step 3: Verify cache behavior

Extend parse summary update tests to include identity-significant edits.

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
  - cargo test -p ash-lsp-core db::tests::test_parse_summary -- --nocapture
  - cargo test -p ash-lsp-core symbols::tests -- --nocapture
  - cargo fmt --check
checklist:
  - [x] LSP summaries carry compact identities
  - [x] Cache invalidates on identity-significant edits
  - [x] No full AST retained in cache summary
```

## Dependencies for Next Task

This task feeds the following Phase 175 tasks according to the dependency table in PLAN-175.

## Completion Evidence

- Added compact LSP `SymbolIdentityKey` carriers to parse macro keys and symbol indexes without storing full ASTs.
