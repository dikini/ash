# Phase 173 Macro-System Expansion Audit

## Status

TASK-1761 audit artifact for PLAN-173. This audit records the live Phase 172 macro implementation seams and decides how Phase 173 should sequence imported/exported macro summaries, token-tree carriers, hygienic binder-introducing macros, and typed macro checking.

## Current carrier map

### Parser-owned macro declarations

`MacroDef` is parser-owned syntax metadata:

- `crates/ash-parser/src/surface.rs:92` adds `Definition::Macro(MacroDef)`.
- `crates/ash-parser/src/surface.rs:134` defines `MacroDef { visibility, name, params, body, span }`.
- `crates/ash-parser/src/parse_module.rs:367` parses `visibility? macro name(params) => expr;`.

There is no exported macro summary carrier, typed macro signature field, token-tree body field, or binder hygiene metadata. `visibility` is retained on the parsed declaration, but `LocalMacroEntry` does not retain it after local registry collection.

### Parsed invocation carriers

`MacroInvocation` is a fail-closed expression carrier:

- `crates/ash-parser/src/surface.rs:1767` defines `MacroDelimiter::{Paren, Bracket, Brace}`.
- `crates/ash-parser/src/surface.rs:1778` defines `MacroInvocation { name, delimiter, raw_body, args, span }`.
- `crates/ash-parser/src/parse_expr.rs:1408` accepts only an unqualified identifier followed by `!` and an immediate `(`, `[`, or `{`.
- `crates/ash-parser/src/parse_expr.rs:1470` parses structured expression arguments only for parenthesized invocations; bracketed and braced invocations keep `args: None`.

The live `raw_body` carrier is not a delimiter-preserving token tree. It is a raw substring captured by a same-delimiter depth scan, so it does not honestly model mixed delimiter nesting, comments, string escapes, or token spans.

### Local expansion and expanded-surface boundary

Local macro execution is parser-first and module-local:

- `crates/ash-parser/src/surface.rs:2015` builds a local macro table from a module's own definitions.
- `crates/ash-parser/src/surface.rs:2032` collects only `Definition::Macro`, rejects duplicate local names, and drops parsed visibility in `LocalMacroEntry`.
- `crates/ash-parser/src/surface.rs:2158` expands macros before notation/operator-section elaboration, then rejects leftover macro invocations before the expanded-surface boundary.
- `crates/ash-parser/src/surface.rs:2189` builds separate local tables for top-level and inline-module scopes; there is no import/export summary path.
- `crates/ash-parser/src/surface.rs:2547` expands `Expr::MacroInvocation` during expression traversal.
- `crates/ash-parser/src/surface.rs:2876` enforces local resolution, parenthesized delimiter, structured args, arity, template whitelist, depth limit, origin sidecars, nested macro expansion, and notation re-entry.
- `crates/ash-parser/src/surface.rs:3108` rejects binder-like templates, blocks, matches, `if`, `if-let`, `with_error`, `fail`, act/do blocks, comprehensions, and free variables.
- `crates/ash-parser/src/surface.rs:4003` finds leftover macro invocations at the expanded boundary while intentionally skipping macro declaration bodies.
- `crates/ash-parser/src/lower.rs:1865` rejects direct Core lowering of any `Expr::MacroInvocation`.

The boundary split is important: expanded-surface scanning ignores macro declaration bodies because declarations are syntax metadata, while general expression visitors and lower-validation paths can still see declaration bodies when validating all expression-bearing definitions.

### Engine and module-loader boundaries

Engine/module-loader paths depend on parser-owned expansion:

- `crates/ash-engine/src/lib.rs:1489` validates expanded surface before `Engine::check_module_file` acceptance.
- `crates/ash-engine/src/module_loader.rs:242` has no macro slot in `ModuleExports`.
- `crates/ash-engine/src/module_loader.rs:2644` parses and expands provider modules before collecting exports.
- `crates/ash-engine/src/module_loader.rs:2746` accepts public callables from expanded definitions, not from macro declarations.
- `crates/ash-engine/src/module_loader.rs:2968` and `crates/ash-engine/src/module_loader.rs:3067` merge only existing type, constructor, callable, type-function, associated-family, interface, and child-module surfaces.

Export acceptance is intentionally macro-blind today. That remains correct until Phase 173 adds explicit syntax-phase macro summary carriers.

### Typechecker boundaries

Typechecking assumes callers have already crossed the expanded-surface boundary:

- `crates/ash-typeck/src/check_expr/mod.rs:73` rejects raw macro invocations in expression checking.
- `crates/ash-typeck/src/lib.rs:588` rejects macro invocations in interface-call validation.
- `crates/ash-typeck/src/lib.rs:1881` rejects macro invocations in function precondition validation.
- `crates/ash-typeck/src/lib.rs:2607` exposes program-level entry points that operate on the provided parsed module; they do not own macro expansion.

Several auxiliary passes currently skip macro carriers rather than rejecting them directly: do-notation diagnostics, purity, capability checking, name collection, and proof visitors. Current ordering makes that safe because high-level routes expand/reject before typecheck and `check_expr` rejects if a carrier leaks. If Phase 173 introduces typechecker-owned macro validation or runs auxiliary passes independently on parsed surfaces, these skips become a fail-open risk.

### LSP-facing consumers

LSP consumers see parsed macro declarations but not imported macro summaries or typed/hygiene metadata:

- `crates/ash-lsp-core/src/completion.rs:62` offers `Definition::Macro` names as completions; `crates/ash-lsp-core/src/completion.rs:104` classifies them as `CompletionItemKind::FUNCTION`.
- `crates/ash-lsp-core/src/goto.rs:33` resolves macro names to declarations, while references are raw same-file token scans.
- `crates/ash-lsp-core/src/hover.rs:278` displays `macro name(params)` with a "Parser-first expression macro declaration" label.
- `crates/ash-lsp-core/src/symbols.rs:116` and `crates/ash-lsp-core/src/db.rs:356` classify macros as function-like symbols.
- `crates/ash-lsp-core/src/db.rs:40` uses a parse summary that does not include macro declaration details, so same-shape macro edits can be invisible to cache validation.

Phase 173 must keep LSP macro presentation syntax-phase-specific and avoid implying callable status.

## Existing regression coverage

The Phase 172 tests prove the current conservative surface:

- `crates/ash-parser/tests/task_1754_macro_declaration_parse.rs` covers declaration shape, parenthesized structured args, non-executable bracket/brace carriers, and qualified-path rejection.
- `crates/ash-parser/tests/task_1755_macro_registry_scope.rs` covers local registry behavior, unsupported bracket/brace execution, and macro declarations crossing the expansion boundary as syntax metadata only.
- `crates/ash-parser/tests/task_1756_expression_macro_expansion.rs` covers supported local expression macro expansion and rejection of binder-like/fail templates.
- `crates/ash-parser/tests/task_1757_macro_origin_hygiene.rs` covers nested macro origin parentage and free-template-variable rejection.
- `crates/ash-parser/tests/task_1758_macro_lowering_boundaries.rs` covers high-level lowering expansion and direct expanded-gate rejection of injected raw macro carriers.
- `crates/ash-engine/tests/task_1758_macro_execution_boundaries.rs` covers high-level engine checking and callable import/export without macro import activation.
- `crates/ash-engine/tests/task_1755_macro_registry_scope.rs` covers imported macros remaining inactive.
- `crates/ash-engine/tests/task_1749_cross_boundary_hygiene_validation.rs` covers raw-carrier rejection at engine/typecheck boundaries.

## Split-risk classification

| Track | Live prerequisite state | Risk | Decision |
|---|---|---|---|
| Macro summaries and imported/exported activation | Parsed `MacroDef.visibility` exists, but exports have no macro slot and local entries drop visibility. | High if activation is added before explicit syntax-phase summary carriers, because macro declarations could be mistaken for callables or raw reparsed source. | Go for summary-carrier/export collection first; split activation into the later task after positive import tests and negative callable-leakage tests exist. |
| Token-tree / bracket / brace parsing | `MacroDelimiter` and `raw_body` exist; bracket/brace invocations parse as non-executable carriers with no structured args. | High if bracket/brace execution uses raw substrings; spans, nested delimiters, comments, and string escapes are not represented honestly. | Go for token-tree carrier design first; split bracket/brace parsing and execution behind delimiter/span preservation tests. |
| Bounded token-tree expansion/reparse | Macro output today is parsed expression templates, not token trees. | High if token-tree output reparses through ad hoc strings or bypasses expanded-surface validation. | Split until token-tree carriers and one audited reparse boundary exist. |
| Binder-introducing macros | Current safety is a whitelist plus free-variable rejection; binder templates are rejected. | High if current substitution model permits binders, because it has no definition-site/call-site/generated-name metadata. | Split behind binder hygiene metadata model and capture-resistance regressions in both directions. |
| Typed macro signatures/checking | No typed fields exist on `MacroDef`, `MacroInvocation`, `LocalMacroEntry`, or exports. Typechecker assumes expanded input. | High if type checking is added after expansion or via ordinary callable summaries. | Split behind typed signature carriers and fail-closed macro/type diagnostics before expansion output is accepted. |
| Bounded macro type inference | No typed macro summary substrate exists. | Very high if inference is attempted without principal/unambiguous constraints. | Defer until typed checking is implemented; infer only from annotated arguments/template bodies and reject ambiguity. |
| LSP-facing macro UX | LSP treats macros as function-like symbols and cache summaries omit macro detail. | Medium for summaries/token trees; high for typed/hygiene UX. | Gate Phase 173 LSP work on macro-specific labels/kinds, cache invalidation, and tests before exposing typed/hygiene data. |

## Go/split decision gates

1. Keep PLAN-173 as one phase, but preserve the existing task sequence as hard split gates. Implementation tasks may proceed only after TASK-1762 amends specs with explicit contracts.
2. TASK-1763 may add macro summary carriers and export collection, but must not activate imported macros and must not put macros in callable summaries.
3. TASK-1764 may activate imported/exported macros only through explicit macro summary carriers and only with positive import/export tests plus negative callable-leakage and malformed-summary tests.
4. TASK-1765 must replace raw-string-only bracket/brace execution assumptions with delimiter-preserving token-tree carriers before TASK-1766 accepts structured bracket/brace macro input.
5. TASK-1767 must use one audited token-tree reparse boundary that re-enters expanded-surface validation; no raw source-snippet bypass is acceptable.
6. TASK-1768 must define binder hygiene metadata before TASK-1769 enables any binder-introducing template currently rejected by Phase 172.
7. TASK-1770 must define typed macro signature carriers before TASK-1771 checks typed macros; TASK-1772 must reject ambiguous inference rather than guessing.
8. Engine/module-loader gates must keep `ModuleExports` callable surfaces macro-free unless a separate macro-summary field is added, and both module checking and importable-module export collection must reject unresolved carriers before acceptance.
9. Typechecker gates must either document expanded-surface preconditions on direct entry points or add preflight rejection before auxiliary passes that currently skip macro carriers.
10. LSP gates must stop presenting macros as ordinary functions where that implies callability, add macro-aware cache invalidation, and add completion/hover/goto/symbol regressions when summary or typed carriers land.

## Closeout decision

TASK-1761 does not require splitting Phase 173 into a separate plan yet. The implementation packet can remain a single phase because PLAN-173 already serializes the risky tracks behind explicit tasks. The conservative decision is to treat each carrier introduction as a hard gate: no imported macro activation before summary carriers, no bracket/brace execution before token trees, no binder templates before hygiene metadata, and no typed inference before typed signature and checking carriers.
