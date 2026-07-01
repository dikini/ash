# Phase 174 Macro-Aware Tooling and Summary-Identity Audit

## Scope

This audit satisfies TASK-1775 for PLAN-174. It checks the live parser, LSP, module-loader, and bounded macro-inference seams after Phase 173.

## Current-state findings

### LSP macro surfaces

- `crates/ash-lsp-core/src/completion.rs` offered `Definition::Macro(_)` through the same function-kind branch as ordinary functions. Phase 174 changed this to a syntax-phase macro completion with `CompletionItemKind::SNIPPET` and detail text `syntax-phase macro`.
- `crates/ash-lsp-core/src/db.rs` recorded `Definition::Macro(_)` as internal `SymbolKind::Function`. Phase 174 introduced internal `SymbolKind::Macro` so symbol-index identity no longer conflates macro declarations with runtime callables.
- `crates/ash-lsp-core/src/symbols.rs` emitted macro document/workspace symbols as LSP `FUNCTION`. Because LSP 3.17 has no `MACRO` symbol kind, Phase 174 maps macros to `OPERATOR` with detail `syntax-phase macro`, avoiding ordinary function presentation.
- `crates/ash-lsp-core/src/hover.rs` showed macro parameters but omitted Phase 173 typed signature metadata. Phase 174 hover now derives signature text from `MacroTypeSignatureSummary`.
- `crates/ash-lsp-core/src/goto.rs` resolved by token spelling alone. Phase 174 detects syntax-phase `m!(...)` invocations and macro declaration names, prefers macro declarations only in those syntax-phase contexts, and keeps ordinary calls pointed at same-named functions. Cross-file imported-summary navigation remains out of scope until a real source location is available.

### Cache and summary identity

- Pre-Phase-174 `ParseSummary` tracked only parse success, error count, top-level definition count, module declaration count, and workflow presence.
- That shape could miss same-count macro edits such as `macro id(x: Int) => x;` changing to `macro id(x: Bool) => !x;`.
- Phase 174 adds `macro_count` and `macro_summary_keys`, storing only lightweight syntax-phase data: name, visibility spelling, parameter count, parameter names, typed-signature shape, and a compact template fingerprint. The key intentionally excludes callable authority, rows, contracts, providers, and runtime effects.

### Callable identity and inference seam

- TASK-1772 intentionally left ordinary call templates uninferred because `add(x, 1)` did not prove a unique callable identity.
- Phase 174 adds a bounded local callable type-summary path for fully annotated local `fn` and `builtin fn` definitions in the same definition list.
- Macro summaries remain non-callable metadata and are never treated as callable identity proofs.
- Same-file references remain token-only and do not claim semantic macro/function disambiguation; Phase 174 hardens goto, hover, completion, symbols, and cache identity while leaving semantic references for a later name-resolution-backed LSP phase.
- Ambiguous same-name local callables, wrong arity, type-mismatched arguments, module-qualified calls, unresolved calls, and unannotated macro parameters remain uninferred.

## Task ownership mapping

| Task | Audit decision |
|---|---|
| TASK-1776 | Add internal `SymbolKind::Macro` and macro-sensitive `ParseSummary` keys. |
| TASK-1777 | Present macro completion/hover as syntax-phase and display typed signatures from carriers. |
| TASK-1778 | Keep same-file macro navigation honest; do not overclaim imported-summary navigation. |
| TASK-1779 | Define callable identity proof categories and explicitly exclude `MacroSummary`. |
| TASK-1780 | Implement only the proven local callable-summary case, with fail-closed negative tests. |
| TASK-1781 | Validate parser/LSP/engine agreement with focused tests and existing boundary regressions. |

## Verification evidence

- `cargo test -p ash-lsp-core -- --nocapture`: 77 unit tests + 1 integration test passed.
- `cargo test -p ash-parser --test task_1772_macro_type_inference -- --nocapture`: 9 tests passed.
- `cargo check -p ash-parser -p ash-lsp-core`: passed.
