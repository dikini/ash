# Phase 177 Row Syntax, Core, and CPS Seam Audit

**Tasks:** TASK-1807 and TASK-1808
**Date:** 2026-07-02
**Scope:** Live parser, typechecker, engine/module, Core, CPS, and Core-to-CPS row seams for the Phase 177 target-Ash row syntax packet.

## Current Row Flow

The current implementation has three distinct row substrates:

1. Source/surface rows are parsed as `ComputationRow` and `ComputationRowItem` values in `crates/ash-parser/src/surface.rs`.
2. Typechecker and engine paths validate or ignore surface rows while still lowering callable types into `ash_typeck::Type::Fn`, which has no row field.
3. Core and CPS rows are represented independently by `CoreRow`/`CoreRowItem` in `crates/ash-core/src/core_ash.rs` and `EffectRow`/`EffectItemKind` in `crates/ash-core/src/cps.rs`.

That means Phase 177 is not yet an end-to-end source-row-to-Core-row pipeline. TASK-1809 and TASK-1811 made surface parsing and fail-closed validation real, while TASK-1813 hardened the Core-to-CPS bridge. TASK-1810, TASK-1812, and TASK-1814 still own the missing identity and cross-boundary preservation work.

## Seam Ownership Map

| Seam | Owner files | Current row carriers | Current behavior | Phase 177 decision |
|---|---|---|---|---|
| Surface AST | `crates/ash-parser/src/surface.rs` | `ComputationRow`, `ComputationRowItem`, `PropositionWhereRow`, `Type::Fn(..., Option<ComputationRow>, ...)` | Represents operation/default, resource, role, policy, channel, process/proc, fail, evidence, group, and tail items with spans. | Preserve as source syntax. Rows are requirement metadata and do not grant authority. |
| Surface parser | `crates/ash-parser/src/parse_module.rs` | Inline callable rows and `where row { ... }` blocks | Parses `-> { ... } T`, callable type rows, row variables, and expanded proposition rows. Rejects duplicate `where row` blocks and tails followed by later row syntax. | Keep parser source-preserving. Duplicate inline plus expanded row is a typechecker validation error. |
| Surface lowerers | `crates/ash-parser/src/lower.rs`, `crates/ash-typeck/src/surface_type_lowering.rs`, `crates/ash-typeck/src/type_env/proofs.rs`, `crates/ash-typeck/src/type_env/support.rs` | Surface `Type::Fn` rows at input only | Existing lowerers bind `Type::Fn(params, ret)` and discard the surface row field. | This is a named lossy boundary. TASK-1814 must not count parser-only tests as Core/CPS preservation evidence. |
| Typechecker validation | `crates/ash-typeck/src/lib.rs`, `crates/ash-typeck/src/error.rs`, `crates/ash-typeck/src/diagnostic.rs` | Surface rows during signature registration | Validates duplicate row spelling, row-tail placement, duplicate tails, and predicate-like operation row items before function signature lowering. | Preserve fail-closed validation; do not infer authority or Core rows here unless TASK-1814 adds a bounded summary bridge. |
| Engine/module summaries | `crates/ash-engine/src/module_loader.rs`, `crates/ash-engine/src/legacy_workflow_adapter.rs`, `crates/ash-engine/src/lib.rs`, `crates/ash-engine/src/entry.rs` | Surface function types at parse/import boundaries | Engine paths convert `Type::Fn(params, _row, ret)` into rowless typechecker function types or summaries. | This is a second source-row loss boundary. TASK-1814 owns public-summary preservation or explicit closeout deferral. |
| Core rows | `crates/ash-core/src/core_ash.rs` | `CoreRow { items, tail }`, `CoreRowItem` | Core supports capability/operation, resource, role, policy, contract, channel, process, failure, evidence, effect group references, and open row tails. | Core is already broader than the source parser in some places. TASK-1812 should make compatibility names explicit instead of broad churn. |
| Core text rows | `crates/ash-core/src/core_ash_text.rs` | Core row parser/formatter | Parses and formats row items, tails, function rows, continuation rows, handler clause rows, and effect operations. | TASK-1812 should add targeted round-trip evidence for Phase 177 families and document any retained legacy spelling. |
| Core typecheck summaries | `crates/ash-core/src/core_ash_typecheck.rs` | `CorePublicRowItemSummary` | Normalizes row items and exposes summaries for capability, resource, role, policy, contract, channel, process, failure, and evidence. Effect group refs are rejected before public summary mapping. | Preserve family identity during normalization. Effect group refs remain a special closed-world case unless TASK-1812 changes it. |
| Core-to-CPS lowering | `crates/ash-core/src/core_ash_lower.rs` | `lower_row`, `lower_row_item` | Rejects open row tails with `UnsupportedCoreRow`, deduplicates items, and maps Core families into CPS `EffectItemKind` values. | Keep fail-closed on unrepresentable open rows. Supported closed families must not be silently dropped. |
| CPS rows | `crates/ash-core/src/cps.rs` | `EffectRow`, `EffectItem`, `EffectItemKind` | CPS has closed effect rows with explicit kinds for capability, role, policy, contract, resource, channel, process, evidence, failure, alias, and group. | CPS has no row-tail field, so open Core rows cannot be preserved without a future carrier change. |

## Named Lossiness Boundaries

- Surface callable rows are dropped when `ash_parser::surface::Type::Fn(params, _row, ret)` becomes `ash_typeck::Type::Fn(params, ret)`.
- Proposition `where row { ... }` syntax is retained in `PropositionTail`, but current proposition lowering and proof/type-environment helpers primarily consume clauses rather than row requirements.
- Engine/module paths that parse or import callable function types also discard `Type::Fn` row fields before binding rowless typechecker function types.
- Core-to-CPS lowering cannot preserve open row tails because `EffectRow` is a closed item list. It now fails closed instead of silently dropping the tail.
- Core effect group references are not exposed through `CorePublicRowItemSummary`; they are rejected before summary mapping.
- Operation row identity still uses legacy Core/CPS `Capability` terminology and source operation paths until TASK-1810 resolves impl-qualified identities.

## Phase 177 Implementation Decisions

- Source kind spelling is `Row`. Older spec prose that says `EffectRow` is background wording unless a task explicitly updates it.
- Inline callable rows and `where row { ... }` are alternate layouts for a single callable row. A declaration that uses both is invalid.
- Duplicate row spelling is a fail-closed validation error, not a merge.
- Row items express requirements. They do not grant provider authority, role admission, workflow admission, host access, or handler installation.
- Evidence row items name evidence requirements. Raw predicate or law bodies are not row item syntax in this phase.
- Operation declarations use `interface`, but operation row identity must become impl-type-qualified where proven, such as `F::read` or `PosixFs::read`. Interface-qualified or ambiguous row identities fail closed.
- `handler` remains a marker/surface concept for this packet. Handler execution, dispatch, and provider admission remain out of scope except where row and identity carriers need to avoid blocking future work.
- Phase 177 does not mark `SPEC-095b`, `SPEC-096b`, `SPEC-097b`, `SPEC-098b`, or `SPEC-098c` as implemented. It lands bounded carriers, validation, Core/CPS alignment, and tests.

## Downstream Task Adjustments

- TASK-1810 remains required because source operation rows are still unresolved source paths and Core/CPS retain legacy `Capability` naming for operation-like requirements.
- TASK-1812 should focus on compatibility-visible naming, Core text round trips, normalization, and public summary behavior. The audit did not find a need for wholesale Core row replacement.
- TASK-1814 must include the source-row-to-summary/Core preservation gap explicitly. Parser-only and Core-only tests are insufficient as end-to-end evidence.
- TASK-1815 must record any row-preservation boundary that remains validation-only after TASK-1814.

## Focused Verification Targets

- Parser carriers: `cargo test -p ash-parser --test task_1809_computation_row_parser`
- Typechecker validation: `cargo test -p ash-typeck --test task_1811_row_validation_and_diagnostics`
- Core row text and normalization: targeted TASK-1812 tests under `crates/ash-core/tests/`
- Core-to-CPS bridge: existing and new TASK-1813/TASK-1814 tests under `crates/ash-core/tests/`
- Cross-boundary row preservation and non-authority: TASK-1814 parser, engine/module, typechecker, Core, and CPS tests
