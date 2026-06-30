# PLAN-173: Macro Summaries, Token Trees, Hygienic Binders, and Typed Macros

## Status: 🟢 In Progress; bounded hygienic binder macros complete

## Overview

Phase 173 expands the conservative Phase 172 local expression-macro MVP into the next coherent macro-system slice. It owns four connected extensions:

1. imported/exported macro activation through explicit macro summary carriers;
2. token-tree, bracket, and brace macro parsing with honest delimiter-preserving carriers;
3. hygienic binder-introducing macros with explicit definition-site/call-site/generated-name metadata;
4. typed macro checking and bounded macro type inference before expanded surface lowering.

The phase is intentionally still parser-first and fail-closed. It must not become a full macro-by-example system, procedural macro system, or Core/runtime macro representation. Every expansion must happen before Core lowering; unsupported macro syntax, summaries, hygiene states, and typed macro obligations must reject before public export acceptance or Core/typecheck acceptance.

## Source specs and prior artifacts

- `docs/spec/SPEC-095c-SURFACE-AST-MACROS-AND-NOTATION.md`
- `docs/spec/SPEC-098c-SURFACE-TO-CORE-LOWERING.md`
- `docs/spec/SPEC-097b-TARGET-TYPE-SYSTEM.md`
- `docs/plan/PLAN-171-MACRO-NOTATION-HYGIENE-AND-EXPANSION-BOUNDARIES.md`
- `docs/plan/PLAN-172-PARSER-FIRST-MACRO-EXECUTION-MVP.md`
- `docs/audit/phase-171-hygiene-origin-scope-audit.md`
- `docs/audit/phase-172-macro-execution-mvp-audit.md`

## Goals

- [x] Create the Phase 173 plan and task packet.
- [x] Audit the Phase 172 implementation and decide the exact sequencing/possible split points for summary import, token trees, binder hygiene, and typed checking.
- [x] Patch specs so imported macro activation, token-tree carriers, binder hygiene, and typed macro checking have implementation-grade contracts before code changes.
- [x] Add explicit macro summary carriers for public macro declarations and export collection without treating macros as callables.
- [x] Activate imported/exported macros only through those summaries, with positive import tests and negative leakage/re-export tests.
- [x] Replace raw bracket/brace diagnostic substrings with delimiter-preserving token-tree carriers that remain syntax-first and source-preserving.
- [x] Parse and validate bracket/brace invocations without executing unsupported token-tree shapes accidentally.
- [x] Add a bounded token-tree expansion/reparse seam for macro templates that explicitly opt into token-tree output.
- [x] Add definition-site/call-site/generated identifier metadata sufficient for binder-introducing macro templates.
- [x] Execute a bounded hygienic binder-introducing macro subset with capture-resistant tests.
- [ ] Add typed macro signature carriers and fail-closed checking of macro arguments/templates before expansion is accepted.
- [ ] Add bounded macro type inference for unannotated local/imported macro templates where inference is unambiguous.
- [ ] Validate parser, engine/module-loader, typechecker, lowering, LSP-facing consumers, and docs/status surfaces before closeout.

## Non-goals

- No arbitrary Rust-style procedural macro callbacks.
- No full macro-by-example/rules matcher beyond the explicitly specified token-tree carrier and expansion subset.
- No macro-generated module declarations or arbitrary item-level generation unless a task explicitly narrows and proves that surface.
- No Core/runtime macro representation; Core still receives only ordinary expanded surface/Core forms plus diagnostic origins.
- No authority, row, contract, failure, proof, evidence, or provider effects from macro metadata itself.
- No imported notation propagation unless needed as a test fixture for imported macro expansion; notation remains separately owned.
- No silent fallback to Phase 172 local lookup when an import/export summary is malformed or ambiguous.

## Decision gates

| Gate | Question | Tier | Blocks | Default |
|---|---|---|---|---|
| D1 | Should Phase 173 land all four requested tracks in one phase, or split after summaries/token trees if binder/typed checks expose larger design debt? | T1 | TASK-1761 | Keep one plan, but TASK-1761 may split later work into Phase 173A/173B before implementation begins. |
| D2 | What is the public macro summary shape? | T1/T2 | TASK-1762, TASK-1763 | Store syntax metadata and typed summaries only; never export macros as callables. |
| D3 | Are `pub macro` declarations importable by ordinary `use`, explicit `use macro`, or both? | T1 | TASK-1762, TASK-1764 | Start with existing module import syntax but only activate entries whose summary kind is macro; add negative callable-shadowing tests. |
| D4 | What token-tree carrier is honest for current parser infrastructure? | T1 | TASK-1765, TASK-1766 | Preserve delimiter, token spans, nested groups, and raw text fallback; do not claim full Rust token-stream fidelity until tested. |
| D5 | Which binder-introducing templates are safe first? | T1 | TASK-1768, TASK-1769 | Only expression-local binders with explicit hygiene metadata; reject generated definitions/modules. |
| D6 | What typed macro syntax is accepted? | T1 | TASK-1762, TASK-1770 | Prefer a minimal extension of existing param/result type syntax; reject if the parser/typechecker cannot attach summaries soundly. |
| D7 | How much macro type inference is allowed? | T1 | TASK-1772 | Infer only from annotated arguments/template body where principal and unambiguous; require annotations otherwise. |

## Phase structure

### Phase 1: Planning, audit, and spec contracts

- TASK-1760: Create the Phase 173 plan and task packet. ✅
- TASK-1761: Audit macro-system expansion seams and split-risk decisions. ✅
- TASK-1762: Amend macro specs for summaries, token trees, binder hygiene, and typed checking. ✅

### Phase 2: Imported/exported macro activation

- TASK-1763: Add macro summary carrier design and export collection. ✅
- TASK-1764: Implement bounded imported/exported macro activation. ✅

### Phase 3: Token-tree and delimiter parsing

- TASK-1765: Add delimiter-preserving macro token-tree carriers. ✅
- TASK-1766: Parse bracket and brace macro invocations into structured carriers. ✅
- TASK-1767: Add bounded token-tree expansion and reparse boundaries. ✅

### Phase 4: Hygienic binder-introducing macros

- TASK-1768: Add binder hygiene metadata model and validation rules. ✅
- TASK-1769: Implement bounded hygienic binder-introducing macro expansion. ✅

### Phase 5: Typed macro checking and inference

- TASK-1770: Add typed macro signature carriers. 📝
- TASK-1771: Implement fail-closed typed macro checking. 📝
- TASK-1772: Implement bounded macro type inference. 📝

### Phase 6: Cross-boundary validation and closeout

- TASK-1773: Close out Phase 173 with cross-boundary validation, review, and status reconciliation. 📝

## Dependency graph

```text
TASK-1760
  -> TASK-1761
      -> TASK-1762
          -> TASK-1763
              -> TASK-1764
          -> TASK-1765
              -> TASK-1766
                  -> TASK-1767
          -> TASK-1768
              -> TASK-1769
          -> TASK-1770
              -> TASK-1771
                  -> TASK-1772
      -> TASK-1773
```

Implementation tasks may proceed in parallel only after TASK-1761/TASK-1762 settle shared carriers. In particular:

- TASK-1764 must not start until TASK-1763 defines summary serialization/export semantics.
- TASK-1767 must not start until TASK-1765/TASK-1766 prove delimiter/token-tree shape preservation.
- TASK-1769 must not start until TASK-1768 proves binder hygiene metadata and negative capture tests.
- TASK-1771/TASK-1772 must not start until TASK-1770 defines typed macro signature carriers.
- TASK-1773 depends on every implementation track and must include an independent review.

## Implementation constraints

- Start from live code in:
  - `crates/ash-parser/src/surface.rs`
  - `crates/ash-parser/src/parse_expr.rs`
  - `crates/ash-parser/src/parse_module.rs`
  - `crates/ash-parser/src/lower.rs`
  - `crates/ash-engine/src/module_loader.rs`
  - `crates/ash-typeck/src/check_expr/mod.rs`
  - `crates/ash-typeck/src/lib.rs`
  - `crates/ash-lsp-core/src/{completion,db,goto,hover,symbols}.rs`
- Reuse Phase 171/172 carriers where they are semantically right: `MacroDef`, `MacroInvocation`, `SurfaceOrigin::MacroExpansion`, `ExpandedSurfaceOrigin`, and expanded-surface validation helpers.
- Do not encode macro activation as callable import. Macro summaries are syntax-phase summaries, not runtime callables.
- Do not allow imported macro activation through ordinary callable summaries, raw source snippets, or reparsed body strings.
- Token-tree carriers must preserve enough span/delimiter structure for diagnostics and hygiene review; a raw substring alone is insufficient for executable bracket/brace macros.
- Binder-introducing macros must prove capture resistance in both directions: generated binders cannot capture caller variables, and caller/source binders cannot capture generated identifiers unless the spec explicitly opts into call-site lookup.
- Typed macro checking must happen before expansion output is accepted as expanded surface. Type errors in macros are macro/type diagnostics, not later arbitrary Core failures.
- Every unsupported shape must fail before Core lowering and public export acceptance with explicit diagnostics.

## Baseline closeout commands

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

## Acceptance criteria

- [ ] Specs describe macro summaries, token-tree carriers, binder hygiene, typed checking, and inference without overclaiming full procedural/macros-by-example semantics.
- [ ] Public macro summaries can be exported/imported only through explicit macro summary carriers.
- [ ] Imported/re-exported macro activation has positive execution tests and negative callable-leakage tests.
- [x] Bracket/brace invocations preserve delimiter/token-tree structure and reject unsupported executable shapes fail-closed.
- [x] Token-tree expansion reparses into ordinary surface syntax through a single audited boundary and cannot bypass expanded-surface validation.
- [ ] Binder-introducing macro templates preserve definition-site/call-site/generated identifier metadata and pass capture-resistance regressions.
- [ ] Typed macro signatures and inferred macro summaries are checked before expansion output is accepted.
- [ ] Parser, engine/module-loader, typechecker, lowering, and LSP-facing consumers agree on macro syntax, summaries, diagnostics, and boundaries.
- [ ] PLAN-INDEX, this plan, task files, specs, and CHANGELOG agree on Phase 173 status.

## Packet creation evidence

Created in TASK-1760. Structural verification must prove this plan references TASK-1760 through TASK-1773, every task file exists exactly once, `PLAN-INDEX.md` has the Phase 173 row and section, and `CHANGELOG.md` records the planning packet under `[Unreleased]`.
