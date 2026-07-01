# Phase 175 Semantic Identity Surface Audit

Status: complete for TASK-1785

Scope: parser macro/callable identity carriers, LSP symbol/goto/reference paths, ParseSummary cache identity, and engine/module-loader macro summary transport.

## Current identity surfaces

| Surface | Current carrier | Classification | Finding | Owner |
|---|---|---|---|---|
| `crates/ash-parser/src/surface.rs::Definition::Macro` | `MacroDef { visibility, name, params, typed_signature, body, span }` | declaration identity input | Local macro declarations have enough source-local evidence to derive a syntax-phase identity, but no explicit comparable identity carrier. | TASK-1786, TASK-1787 |
| `crates/ash-parser/src/surface.rs::MacroSummary` | module path, exported name, params, input/output kind, fingerprint, typed signature, origin span | imported-summary identity input | Public macro summaries preserve export metadata but do not carry a canonical identity key distinct from callable summaries. | TASK-1786, TASK-1790 |
| `crates/ash-parser/src/surface.rs::LocalMacroEntry` | local/imported expansion row keyed by local name | use-site expansion identity input | Expansion can resolve `m!(...)` to a local row, but the row has no stable identity for LSP/reference comparison. | TASK-1787 |
| `crates/ash-parser/src/surface.rs::CallableTypeSummary` | private name + parameter/return types + ambiguity bit | parser-local callable type evidence | This is inference evidence only, not a public callable identity. Phase 175 may add a separate identity shape; it must not treat macro summaries as callables. | TASK-1786, TASK-1787 |
| `crates/ash-lsp-core/src/db.rs::MacroSummaryKey` | name, visibility, params, typed signature, template hash | cache identity | Phase 174 cache keys invalidate macro presentation edits but do not carry canonical declaration/use identities. | TASK-1788 |
| `crates/ash-lsp-core/src/db.rs::SymbolIndex` | symbol names/kinds and definition locations | LSP declaration index | The index distinguishes `SymbolKind::Macro`, but definitions remain name/location pairs without semantic identity. | TASK-1788 |
| `crates/ash-lsp-core/src/goto.rs` | context-sensitive token checks | navigation identity approximation | Goto splits `m!(...)` and `m()` for same-name macro/function declarations, but references are still lexical token scans. | TASK-1789 |
| `crates/ash-engine/src/module_loader.rs` | `macro_summaries` + `macro_templates` maps | imported summary/template transport | Imported macros are syntax-phase templates keyed by exported name and optional local alias. Navigation can safely prepare summary identity, but no workspace source graph exists. | TASK-1790 |

## Risks and decisions

1. **Macro identity is syntax-phase only.** It may identify a declaration for expansion/tooling, but it must not be accepted as an ordinary callable identity, effect row, provider authority, contract, or runtime binding.
2. **Local same-file identity is provable now.** Same parsed module/file declarations and uses can be compared by declaration span/name/kind.
3. **Imported identity is summary-level only.** Imported macro summaries can preserve origin module/name plus local alias, but Phase 175 must not claim full cross-file references without workspace graph/source-root support.
4. **Callable identity must be distinct.** Ordinary `fn`/`builtin fn` identities are callable declarations. Macro summaries and macro declarations are not callable identities even if they have typed syntax signatures.
5. **Ambiguity fails closed.** Duplicate, unresolved, private unsupported, module-qualified unsupported, or identity-free use sites should produce no semantic reference set rather than a token-only overclaim for macro/function collisions.

## Implementation ownership

- TASK-1786: add canonical `MacroDeclarationIdentity` and callable-boundary types in parser-facing surface metadata.
- TASK-1787: add parser-local identity resolution helpers for macro invocations and ordinary calls where a single local declaration is provable.
- TASK-1788: thread compact identity keys through LSP summaries and symbol indexes.
- TASK-1789: use resolved same-file identity to split references for same-name macro/function declarations and uses.
- TASK-1790: carry imported macro summary identity/alias metadata, but keep cross-file navigation preparation honest and limited.
- TASK-1791: validate non-callability and no overclaiming across parser, engine, and LSP tests.

## Audit conclusion

The plan remains valid. No downstream task reordering is required. The first implementation slice should add explicit syntax-phase macro identity and separate callable identity keys, then let LSP references consume only same-file identities that are proven by declaration context or invocation/call syntax.
