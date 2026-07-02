# Phase 178 Source-to-Core Row-Loss Audit

**Task:** [TASK-1818](../plan/tasks/TASK-1818-source-to-core-row-loss-audit.md)
**Status:** Complete
**Date:** 2026-07-02

## Purpose

Map where Phase 177 callable computation rows are parsed, validated, transported, converted, preserved, or lost before reaching typechecker-facing summaries and Core callable rows. This audit intentionally does not implement row lowering.

Phase 178 must preserve this invariant while bridging the gap: source rows are requirements metadata only. They must not install providers, admission facts, handlers, host hooks, resource ownership, roles, workflow authority, or runtime implementations.

## Current Row Flow

```text
source fn / builtin fn / callable type
  -> ash-parser surface AST
     - Type::Fn(Vec<Type>, Option<ComputationRow>, Box<Type>)
     - FnDef::proposition_tail.row for expanded where row { ... }
     - BuiltinFnDef::proposition_tail.row for expanded where row { ... }
  -> ash-typeck validation
     - validate_surface_type_rows validates nested inline rows
     - validate_callable_rows validates expanded rows and rejects inline+expanded duplicates
     - operation row identity validation resolves concrete and interface-bound operations
  -> rowless typechecker conversion
     - workflow_surface_type_to_type converts SurfaceType::Fn(_, _row, _) to Type::Fn(params, ret)
     - function_signature_type and builtin_fn_signature_type return Type::Fn(params, ret)
  -> engine import/export
     - InlineCallable.signature preserves original FnDef/BuiltinFnDef AST, including rows
     - bind_imported_callable_types still binds rowless ash_typeck::Type::Fn
  -> Core
     - Core supports CoreType::Function { params, row, ret }
     - source-to-Core callable lowering currently has no row-aware bridge from parsed source rows
```

## Parser Carriers

Owner files:

- `crates/ash-parser/src/surface.rs`
- `crates/ash-parser/src/parse_module.rs`
- `crates/ash-parser/src/lower.rs`

Findings:

- `surface::Type::Fn(Vec<Type>, Option<ComputationRow>, Box<Type>)` preserves inline callable rows.
- `surface::FnDef::proposition_tail` and `surface::BuiltinFnDef::proposition_tail` preserve expanded `where row { ... }` rows.
- `surface::PropositionWhereRow` carries the parsed `ComputationRow`, `row_keyword_span`, and complete block span.
- Parser tests from Phase 177 already prove source spans and row item families survive surface parsing, including `crates/ash-parser/tests/task_1809_computation_row_parser.rs` and `crates/ash-parser/tests/task_1814_row_cross_boundary_parser.rs`.
- Parser-local `CallableTypeSummary` in `surface.rs` is used for bounded macro type inference. It carries `param_types` and `return_type`; because those are surface `Type`, inline rows nested inside those types remain present. It does not carry an explicit callable-level expanded row from `FnDef::proposition_tail`.

Row-loss points:

- `crates/ash-parser/src/lower.rs::lower_surface_type` matches `Type::Fn(params, _row, ret)` and lowers to core AST constructor `Fn(params..., ret)`, dropping row metadata.
- `crates/ash-parser/src/lower.rs::lower_type_to_type_expr` matches `Type::Fn(_, _, _)` and lowers to workflow-contract `TypeExpr::Constructor { name: "Fn", args: [] }`, dropping both function shape detail and row metadata.
- `crates/ash-parser/src/lower.rs::lower_module_type_metadata` stores `TypeFnDef` values but does not add row-bearing callable summaries for ordinary `fn`/`builtin fn`.

Disposition:

- TASK-1819 should add a minimal source callable row summary shape rather than replacing parser `Type::Fn`.
- TASK-1820 should populate it from inline return rows and expanded `where row` rows.
- TASK-1821 should avoid relying on legacy `lower_surface_type` for row-aware Core callable lowering.

## Typechecker Validation and Conversion

Owner files:

- `crates/ash-typeck/src/lib.rs`
- `crates/ash-typeck/src/surface_type_lowering.rs`
- `crates/ash-typeck/src/type_env/support.rs`
- `crates/ash-typeck/src/type_env/lookup_and_unfold.rs`

Findings:

- `validate_surface_type_rows` recursively validates rows nested in surface `Type::Fn`.
- `validate_callable_rows` validates parameters, return types, expanded `where row` blocks, and rejects duplicate inline plus expanded callable rows.
- `validate_operation_row_identity` and `TypeEnv::resolve_operation_row_identity` support concrete impl-qualified operation identities and interface-bound abstract operation identities from Phase 177.
- `row_item_text` and related helpers preserve source spellings for diagnostics.

Row-loss points:

- `crates/ash-typeck/src/surface_type_lowering.rs::workflow_surface_type_to_type` matches `ash_parser::surface::Type::Fn(params, _row, ret)` and returns rowless `Type::Fn(param_types, ret_type)`.
- `crates/ash-typeck/src/lib.rs::function_signature_type` converts parsed function signatures into rowless `Type::Fn(params, ret)`.
- `crates/ash-typeck/src/lib.rs::builtin_fn_signature_type` converts parsed builtin signatures into rowless `Type::Fn(params, ret)`.
- `crates/ash-typeck/src/type_env/support.rs::surface_type_to_type` and related support helpers match `SurfaceType::Fn(params, _row, ret)` and drop rows for imported/type-env support paths.
- `ash_typeck::Type::Fn` itself has no row field, so any path that must remain compatible with existing inference has to carry source rows out of band until a later type representation phase.

Disposition:

- TASK-1819 should introduce row metadata beside the existing typechecker function summaries or imported signature carriers.
- TASK-1820 should thread explicit parsed rows through the existing validation/conversion boundaries without changing rowless unification semantics.
- TASK-1822 must keep validation as requirement checking only; no TypeEnv provider/admission/handler installation may be added.

## Engine Import/Export Transport

Owner files:

- `crates/ash-engine/src/module_loader.rs`
- `crates/ash-engine/src/lib.rs`
- `crates/ash-engine/src/entry.rs`
- `crates/ash-engine/src/legacy_workflow_adapter.rs`

Findings:

- `module_loader::InlineCallable` carries `signature: Option<CallableSignature>`.
- `CallableSignature::Function(FnDef)` and `CallableSignature::Builtin(BuiltinFnDef)` preserve the original parser AST, including inline `Type::Fn` rows and expanded proposition-tail rows.
- Public function export paths use `imported_callable_from_fn_def(function.clone())`, so the original `FnDef` can be the Phase 178 source of truth.
- `rewrite_callable_signature_aliases` and `rewrite_surface_type_aliases` rewrite names inside surface types but match `Type::Fn(params, _row, ret)` and do not rewrite names inside row items.
- `callable_signature_type_names` collects ordinary type names from params/returns but matches `Type::Fn(params, _row, ret)` and ignores row item paths.
- `bind_imported_callable_types` uses preserved `CallableSignature` values to bind imported names, but the resulting `ash_typeck::Type::Fn` is rowless.

Row-loss points:

- `crates/ash-engine/src/lib.rs::surface_type_to_typeck` matches `SurfaceType::Fn(params, _row, ret)` and returns rowless `ash_typeck::Type::Fn`.
- `bind_imported_callable_types` binds imported functions and builtins via rowless `function_signature_type`/`builtin_fn_signature_type`; arity-only fallbacks also create rowless fresh `Type::Fn`.
- `entry.rs::format_type` and `legacy_workflow_adapter.rs::type_summary` ignore row metadata in display output. These are presentation losses, not the main Core bridge.

Disposition:

- TASK-1819 should expose a callable row summary derived from `InlineCallable.signature`.
- TASK-1820 should ensure imported and re-exported callable signatures keep that row summary across module boundaries.
- TASK-1822 negative tests should target imports because preserved signatures are the place where metadata could accidentally be treated as authority.

## Core Row Support

Owner files:

- `crates/ash-core/src/core_ash.rs`
- `crates/ash-core/src/core_ash_typecheck.rs`
- `crates/ash-core/src/core_ash_lower.rs`
- `crates/ash-core/src/semantic_summary.rs`

Findings:

- `CoreRow`, `CoreRowItem`, and `CoreType::Function { params, row, ret }` already support row-bearing callable types.
- Core lowering to CPS already uses rows for functions, continuations, mode bindings, and effect operations, and fails closed for unsupported open rows in CPS lowering.
- Core semantic summaries currently model ordinary type/type-function/proposition/interface metadata, but there is no public ordinary callable row summary in `ModuleSemanticSummary`.

Row-loss points:

- There is no source-to-Core function-type construction path that consumes parser `ComputationRow` and creates `CoreRow`.
- Existing semantic summaries do not export ordinary callable rows as a Core-owned summary surface.

Disposition:

- TASK-1821 should implement source `ComputationRow` to `CoreRow` conversion for supported Phase 177 row families.
- TASK-1821 should fail closed for row forms that cannot be represented safely, especially open row tails if the target Core consumer cannot preserve them through the chosen path.
- TASK-1823 should inspect actual `CoreType::Function { row, .. }` values or a Core-owned callable-row summary, not just parser AST retention.

## Downstream Task Adjustments

| Task | Audit disposition |
|---|---|
| TASK-1819 | Add an out-of-band row-bearing callable summary/carrier beside existing rowless `ash_typeck::Type::Fn`; derive it from parser AST signatures already preserved by `InlineCallable.signature`. |
| TASK-1820 | Thread rows through module export/import and typechecker-facing callable summary paths; do not require changing `ash_typeck::Type::Fn` in this phase. |
| TASK-1821 | Add explicit source-row to `CoreRow` lowering instead of routing through legacy `lower_surface_type` or workflow-contract lowering. |
| TASK-1822 | Add negative import/module tests proving row summaries do not populate providers, admission facts, handlers, host hooks, resources, roles, workflow summaries, or runtime callables. |
| TASK-1823 | Add end-to-end parser -> engine/typecheck -> Core preservation tests that inspect row-bearing summaries/Core rows after the rowless typechecker conversion boundary. |

No PLAN-178 scope change is required. The live code matches the packet's assumption that rows are retained in parser AST and validation, then lost at rowless function/type conversion boundaries.

## Recommended Focused Tests

- Parser/engine test: public `fn` with `where row { PosixFs::read }` exports a callable row summary while its `ash_typeck::Type::Fn` remains compatible.
- Engine import test: imported public `fn` row summary survives named imports and re-exports.
- Typechecker test: validated inline and expanded rows can be extracted from function/builtin signatures after type validation without granting authority.
- Core test: source row families lower to `CoreRowItem::{Capability, Resource, Role, Policy, Channel, Process, Failure, Evidence, EffectGroupRef}` as appropriate.
- Negative test: row requirements referencing providers/admission/handlers do not create runtime provider tables, public workflow summaries, imported runtime callables, or admission/proposition facts.
