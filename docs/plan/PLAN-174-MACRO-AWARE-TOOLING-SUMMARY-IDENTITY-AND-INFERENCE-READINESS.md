# PLAN-174: Macro-Aware Tooling, Summary Identity, and Inference Readiness

**Status:** 📝 Planned
**Spec:** [SPEC-095c: Surface AST, Macro Expansion, and Notation](../spec/SPEC-095c-SURFACE-AST-MACROS-AND-NOTATION.md); [SPEC-098c: Surface-to-Core Lowering](../spec/SPEC-098c-SURFACE-TO-CORE-LOWERING.md); [SPEC-097b: Target Type System](../spec/SPEC-097b-TARGET-TYPE-SYSTEM.md)
**Depends on:** Phase 173 macro summaries, token trees, hygienic binders, and typed macros.
**Task range:** TASK-1774 through TASK-1783.

## Goal

Make macro-aware analysis and tooling honest after Phase 173. Macros now have syntax-phase summaries, token-tree carriers, hygiene metadata, and typed signature carriers; Phase 174 ensures LSP-facing tools, cache summaries, symbol identity, and bounded inference decisions use those carriers without presenting macros as runtime callables.

## Rationale

Phase 173 made macro summaries/token trees/hygiene/typed carriers real. The remaining immediate debt is that analysis/tooling still treats macros too much like functions, and bounded macro type inference intentionally refuses ordinary call expressions without a proven callable-identity substrate. This phase is safer than jumping straight to generalized mixfix or full macro-by-example semantics because it tightens the already-built boundaries before adding more surface power.

## Scope

Phase 174 owns:

- macro-specific LSP symbol/completion/hover/goto/reference behavior;
- macro-aware parse-summary and cache invalidation keys;
- a callable-identity summary audit and minimal substrate for future inference;
- bounded macro inference through ordinary calls only where callable identity is unique and syntax-phase safe;
- parser/engine/LSP cross-boundary regressions proving macros remain syntax-phase metadata.

## Non-goals

- No generalized mixfix or binder-introducing notation.
- No full macro-by-example/procedural macro system.
- No runtime macro authority, rows, contracts, failures, proof evidence, or provider effects.
- No broad `SPEC-098c` lowering completion beyond macro/tooling boundaries.
- No imported notation activation; notation summary carriers remain a separate future phase.
- No speculative inference through ambiguous call names, overloaded operators, private callables, or unresolved module-qualified paths.

## Current context

Phase 173 closed the macro-system carrier slice, including public macro summaries, token-tree reparse, binder hygiene, typed macro signatures, and bounded macro inference. Its closeout tests prove macro metadata does not create runtime callable bindings.

The remaining debt is visible in tooling and identity surfaces:

- `crates/ash-lsp-core/src/completion.rs` still presents `Definition::Macro(_)` as `CompletionItemKind::FUNCTION`.
- `crates/ash-lsp-core/src/symbols.rs` and `crates/ash-lsp-core/src/db.rs` classify macro symbols as function-like.
- `crates/ash-lsp-core/src/db.rs::ParseSummary` tracks only broad parse shape, so same-count macro signature/body edits can be invisible to cache invalidation.
- `TASK-1772` deliberately kept ordinary call expressions uninferred unless a later typed/callable-summary substrate proves unique callable identity.

## Decision gates

| Gate | Question | Owner task | Default decision |
|---|---|---|---|
| D1 | Which LSP surfaces imply runtime callability today? | TASK-1775 | Audit before changing behavior. |
| D2 | What macro-specific symbol/cache representation is needed without storing full AST in Salsa? | TASK-1776 | Add lightweight macro summary keys, not full AST hashing. |
| D3 | How should completion/hover present macros and typed signatures? | TASK-1777 | Present macros as syntax-phase macros, not functions. |
| D4 | Which goto/reference paths can be macro-aware without cross-file overclaiming? | TASK-1778 | Same-file/local and explicit imported-summary paths only. |
| D5 | What counts as a proven callable identity for macro inference? | TASK-1779 | Require a unique resolved callable summary with type information. |
| D6 | Which ordinary-call templates may infer safely now? | TASK-1780 | Implement only positive cases proven by D5; reject ambiguity. |
| D7 | Do parser, engine/module-loader, and LSP agree on macro metadata boundaries? | TASK-1781 | Add paired positive/negative cross-boundary tests. |

## Tasks

| Task | Title | Status |
|---|---|---|
| [TASK-1774](tasks/TASK-1774-phase-174-plan-packet.md) | Create the Phase 174 plan packet | ✅ Complete |
| [TASK-1775](tasks/TASK-1775-macro-aware-tooling-audit.md) | Audit macro-aware tooling, LSP, and summary-identity seams | 📝 Planned |
| [TASK-1776](tasks/TASK-1776-macro-symbol-cache-model.md) | Add macro-specific symbol kinds and cache-summary invalidation keys | 📝 Planned |
| [TASK-1777](tasks/TASK-1777-macro-completion-hover-signature-ux.md) | Implement macro-aware completion and hover/signature presentation | 📝 Planned |
| [TASK-1778](tasks/TASK-1778-macro-goto-reference-boundaries.md) | Harden macro goto-definition, symbols, and references without callable overclaiming | 📝 Planned |
| [TASK-1779](tasks/TASK-1779-callable-identity-summary-audit.md) | Audit and specify callable identity summaries for macro inference | 📝 Planned |
| [TASK-1780](tasks/TASK-1780-bounded-callable-identity-inference.md) | Implement bounded macro inference through proven callable identities | 📝 Planned |
| [TASK-1781](tasks/TASK-1781-cross-boundary-tooling-validation.md) | Add parser/engine/LSP cross-boundary tooling and inference validation | 📝 Planned |
| [TASK-1782](tasks/TASK-1782-phase-174-docs-spec-reconciliation.md) | Reconcile specs, docs, and indexes for Phase 174 boundaries | 📝 Planned |
| [TASK-1783](tasks/TASK-1783-phase-174-closeout.md) | Close out Phase 174 with broad gates and review | 📝 Planned |

## Implementation order

1. TASK-1775 audits live surfaces and patches downstream tasks if the current state differs from this plan.
2. TASK-1776 introduces macro-specific LSP/cache primitives before user-facing tooling changes.
3. TASK-1777 and TASK-1778 can proceed after TASK-1776; they should not run before the symbol/cache model is stable.
4. TASK-1779 audits callable identity and patches TASK-1780 with exact positive/negative cases.
5. TASK-1780 implements only the bounded inference cases justified by TASK-1779.
6. TASK-1781 validates the whole boundary across parser, engine/module-loader, and LSP-facing consumers.
7. TASK-1782 reconciles specs/docs/status language.
8. TASK-1783 reruns broad gates, performs independent review, and closes the phase.

## Acceptance criteria

- [ ] LSP-facing macro symbols are not presented as ordinary runtime functions where that implies callability.
- [ ] Macro declaration/signature/body changes invalidate relevant LSP cache summaries even when broad definition counts stay the same.
- [ ] Completion and hover show syntax-phase macro metadata, including typed macro signatures when available.
- [ ] Goto/symbol/reference behavior is macro-aware and honest about same-file versus imported-summary support.
- [ ] Callable identity summaries are explicitly defined before ordinary-call macro inference expands beyond Phase 173.
- [ ] Bounded ordinary-call macro inference succeeds only when a unique callable identity and type summary are proven.
- [ ] Ambiguous, private, unresolved, overloaded, or module-qualified call templates remain annotation-required or fail closed.
- [ ] Parser, engine/module-loader, typechecker-facing, and LSP tests agree that macro metadata remains syntax-phase metadata.
- [ ] PLAN-INDEX, task files, specs/docs, and CHANGELOG agree on Phase 174 status.

## Verification baseline

```bash
cargo fmt --check
cargo test -p ash-parser
cargo test -p ash-typeck
cargo test -p ash-engine
cargo test -p ash-lsp-core
cargo check --workspace
cargo clippy --workspace --all-targets --all-features -- -D warnings
git diff --check
python3 tools/docs/validate_orientation_indexes.py --self-test
bash scripts/check-docs-gate.sh
```

## Expected follow-on after Phase 174

If Phase 174 closes cleanly, the next plausible language-feature packets are imported/exported notation summary carriers or generalized mixfix/binder notation. If the callable identity substrate remains too weak, follow-on work should harden public callable/type summaries before expanding macro inference again.
