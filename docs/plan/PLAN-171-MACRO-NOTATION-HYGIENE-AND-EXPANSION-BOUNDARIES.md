# PLAN-171: Macro/Notation Hygiene and Expansion Boundaries

## Status: ✅ Complete

## Overview

Phase 171 is a conservative implementation-grade follow-on to Phase 170. Phase 170 made the expanded-surface boundary hard to bypass, kept notation module-local, and added narrow expansion-origin sidecars. Phase 171 should not jump to a full macro system. Instead, it should make the hygiene contract explicit and enforceable: every generated or notation-expanded surface node has a stable expansion identity, every origin chain remains available for diagnostics, and every macro/notation scope boundary is fail-closed before Core lowering.

The packet focuses on invariants that future macro expansion and generalized mixfix can safely depend on. It deliberately avoids typed macros, arbitrary token-tree rewriting, binder-introducing mixfix notation, imported notation propagation, and broad `SPEC-098c` lowering completion.

## Source specs and prior artifacts

- `docs/spec/SPEC-095c-SURFACE-AST-MACROS-AND-NOTATION.md`
- `docs/spec/SPEC-098c-SURFACE-TO-CORE-LOWERING.md`
- `docs/spec/SPEC-097b-TARGET-TYPE-SYSTEM.md`
- `docs/design/phase-170-notation-summary-export-semantics.md`
- `docs/plan/PLAN-168-SURFACE-AST-NOTATION-SUBSTRATE.md`
- `docs/plan/PLAN-169-SURFACE-EXPANSION-AND-NOTATION-ELABORATION.md`
- `docs/plan/PLAN-170-EXPANDED-SURFACE-INTEGRATION-AND-NOTATION-SCOPING.md`
- `docs/audit/phase-168-surface-to-core-lowering-inventory.md`

## Goals

- [x] Audit the live surface expansion, origin, name-resolution, notation-table, and module-loader seams that hygiene must constrain.
- [x] Patch `SPEC-095c`/`SPEC-098c` only where needed to state hygiene invariants in implementation-grade terms.
- [x] Add a stable expansion identity and origin-chain model for generated surface nodes without changing Core provenance APIs.
- [x] Fence source-spellable identifiers from generated operator-section helper names, and ensure generated names cannot accidentally capture or be captured by source names.
- [x] Make macro and notation scope boundaries explicit: local notation remains local, macro placeholders remain fail-closed, and no import/export path silently activates syntax.
- [x] Add negative leakage tests proving unresolved macro forms cannot bypass expanded-surface validation through parser, engine, module-loader, or typechecker paths.
- [x] Preserve ordinary callable authority: hygiene metadata never grants rows, capabilities, failures, contracts, or evidence.
- [x] Close out with focused parser/engine/typeck gates, docs gates, and independent review.

## Non-goals

- No full macro expander.
- No typed macro system.
- No arbitrary token-tree rewrite execution.
- No binder-introducing/generalized mixfix notation.
- No imported/exported notation propagation unless a task explicitly proves a bounded carrier and adds positive plus negative tests.
- No Core/runtime provenance schema change beyond preserving surface-origin data at the boundary.
- No broad `SPEC-098c` lowering completion for unrelated surface forms.

## Decision gates

| Gate | Question | Tier | Blocks | Default |
|---|---|---|---|---|
| D1 | Is the live `SurfaceOrigin` carrier sufficient for expansion identity chains, or is a new `ExpansionId`/origin-stack carrier required? | T1 if parser-only, T2 if public API wider | TASK-1745 | Add a narrow parser/surface carrier only; do not alter Core provenance. |
| D2 | Which identifier classes need first-class carriers now versus documentation-only constraints? | T1 | TASK-1746 | Implement only a source-spellable/generated-name fence plus diagnostics; defer full def-site/call-site hygiene and first-class identifier-origin carriers. |
| D3 | Should macro invocations parse as durable fail-closed surface nodes in this phase? | T1/T2 depending parser API | TASK-1748 | Add or audit fail-closed carriers only; no macro execution. |
| D4 | Can notation or macro scope tables cross module boundaries honestly? | T1/T2 if summary schema changes | TASK-1747 | Preserve Phase 170 local-only notation; add negative import/export leakage tests. |

## Phase structure

### Phase 1: Register and audit hygiene constraints

Tasks:

- TASK-1743: Create the Phase 171 plan and task packet. ✅
- TASK-1744: Audit hygiene, origin, and scope boundary seams. ✅

### Phase 2: Make origin and identifier hygiene enforceable

Tasks:

- TASK-1745: Add expansion identity and origin-chain carriers for generated surface nodes. ✅
- TASK-1746: Implement source/generated identifier hygiene fences. ✅

### Phase 3: Constrain notation and macro scope boundaries

Tasks:

- TASK-1747: Harden notation and macro scope-table boundaries. ✅
- TASK-1748: Add fail-closed macro invocation boundary without macro execution. ✅

### Phase 4: Boundary validation and closeout

Tasks:

- TASK-1749: Add cross-boundary hygiene and negative-leakage validation tests. ✅
- TASK-1750: Close out Phase 171 with verification, changelog, index reconciliation, and review. ✅

## Dependency graph

```text
TASK-1743
  -> TASK-1744
      -> TASK-1745
          -> TASK-1746
      -> TASK-1747
          -> TASK-1748
      -> TASK-1749
          -> TASK-1750
```

TASK-1745 and TASK-1747 may proceed in parallel after TASK-1744 if D1 and D4 are resolved independently. TASK-1749 depends on all implementation tasks because it validates parser, engine/module-loader, and typechecker boundaries together.

## Implementation constraints

- Start from live code in `crates/ash-parser/src/surface.rs`, `crates/ash-parser/src/parse_expr.rs`, `crates/ash-parser/src/parse_module.rs`, `crates/ash-parser/src/lower.rs`, and `crates/ash-engine/src/module_loader.rs`.
- Reuse Phase 168–170 carriers (`ParsedSurfaceModule`, `ExpandedSurfaceModule`, `SurfaceOrigin`, `RawOperatorToken`, `OperatorSection`, notation declarations) rather than inventing parallel structures.
- Treat hygiene metadata as syntax metadata only. It cannot add capability authority, latent rows, failure behavior, contracts, or proof/evidence obligations.
- Every unresolved macro, notation, operator-section, or generated identifier boundary must fail before Core lowering with an explicit diagnostic or an existing unsupported-feature error.
- Positive visibility and negative leakage tests are required for any scope boundary claim.
- Keep compatibility helpers available only when explicitly parser-only; high-level module/file validation must use expanded-surface validation.
- Do not claim imported notation or macro activation unless module summary carriers are updated and tested in both directions.

## Verification policy

Each task must run focused parser/engine/typeck tests for the surface it changes, formatting, and relevant crate checks. Any task touching public surface carriers, `Definition`/`Expr` variants, module summaries, lowering APIs, or downstream consumers must run `cargo check --workspace`.

Baseline closeout commands:

```bash
cargo fmt --check
cargo test -p ash-parser
cargo test -p ash-typeck
cargo test -p ash-engine
cargo check --workspace
cargo clippy -p ash-parser -p ash-typeck -p ash-engine --all-targets --all-features -- -D warnings
git diff --check
python3 tools/docs/validate_orientation_indexes.py --self-test
bash scripts/check-docs-gate.sh
```

## Acceptance criteria

- [x] The hygiene/origin/scope audit maps every high-level path that can consume parsed or expanded surface forms.
- [x] `SPEC-095c` and/or `SPEC-098c` state implementation-grade hygiene invariants without overclaiming macro execution.
- [x] Generated surface nodes carry stable expansion identity and origin-chain metadata suitable for diagnostics.
- [x] Source and generated identifiers cannot silently capture each other across expansion boundaries.
- [x] Macro invocation carriers, if parsed, are durable but fail-closed before Core until a future macro expander exists.
- [x] Local notation remains local unless a task implements real summary carriers; import/export non-propagation remains tested.
- [x] High-level engine/module-loader/typechecker paths reject unresolved macro/notation forms.
- [x] `PLAN-INDEX.md`, this plan, task files, and `CHANGELOG.md` agree on Phase 171 status.

## Packet creation evidence

Created as the planning packet in TASK-1743. Structural verification must prove that this plan references TASK-1743 through TASK-1750, every task file exists, `PLAN-INDEX.md` has the Phase 171 row and section, and `CHANGELOG.md` records the packet under `[Unreleased]`.

## TASK-1744 evidence

`docs/audit/phase-171-hygiene-origin-scope-audit.md` records the current carrier and boundary map. It keeps macro execution, typed macros, generalized/binder mixfix, imported notation activation, Core/runtime provenance schema changes, and broad `SPEC-098c` lowering out of scope. The audit assigns expansion identity/origin chains to TASK-1745, generated identifier fences to TASK-1746, notation/macro scope boundaries to TASK-1747, macro fail-closed representation to TASK-1748, cross-boundary validation to TASK-1749, and closeout/status reconciliation to TASK-1750.

## TASK-1745 evidence

`crates/ash-parser/src/surface.rs` now assigns stable `ExpansionId` values to generated surface origins and records a parent origin for nested expansion products. `crates/ash-parser/tests/task_1745_expansion_origin_chain.rs` covers distinct generated IDs, local notation target preservation, nested notation/operator-section origin chains, and review-remediated parent preservation through non-call recursive expression shapes.

## TASK-1746 evidence

Generated section parameters now use non-source-spellable `$ash_generated_section_<id>_<role>` names tied to expansion identity. `crates/ash-parser/tests/task_1746_generated_identifier_hygiene.rs` verifies source text cannot spell those generated helpers and legacy helper-like source bindings do not capture generated parameters.

## TASK-1747 evidence

`crates/ash-engine/tests/task_1747_notation_macro_scope_boundaries.rs` verifies that re-exported callables remain directly callable while provider notation remains inactive transitively, and that macro-like placeholder syntax cannot activate or bypass module-boundary validation.

## TASK-1748 evidence

`MacroInvocation` surface carriers preserve an unqualified macro name, delimiter, conservative raw delimited body text, and span while remaining fail-closed. Parser, lowerer, engine/module-loader, and typechecker paths reject macro invocation before Core or public export acceptance. Qualified macro-like paths are explicitly outside this carrier. Focused tests live in `crates/ash-parser/tests/task_1748_macro_invocation_boundary.rs` and `crates/ash-engine/tests/task_1748_macro_invocation_boundary.rs`.

## TASK-1749 evidence

`crates/ash-engine/tests/task_1749_cross_boundary_hygiene_validation.rs` validates the Phase 171 boundary as an integrated system. It proves local notation/operator-section expansion still succeeds with origin metadata and non-source-spellable generated binders, callable imports remain usable without activating imported notation, imported notation remains inactive at high-level engine execution, and macro invocations are rejected by engine/module validation and typechecker-facing expression checking before Core acceptance.

## TASK-1750 evidence so far

Closeout verification passed for parser, typechecker, engine, workspace check, focused clippy, formatting, diff whitespace, orientation-index, and docs gates. `SPEC-095c`, `SPEC-098c`, and `SPEC-INDEX.md` now record the conservative hygiene/fail-closed invariants. Independent review findings were addressed: parent-origin preservation now threads through all recursive expression shapes, status surfaces agree, and macro carrier wording/tests honestly limit the carrier to unqualified names plus conservative raw delimited body text.
