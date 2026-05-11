# TASK-844: Type-computation summary audit

## Scope

Docs-only audit of the live pre-SPEC-062 substrate. No Rust implementation changes are made by TASK-844.

## Live carriers and callsites

### ash-core

- `crates/ash-core/src/semantic_summary.rs`
  - `ModuleSemanticSummary` currently owns public ordinary type metadata plus sealed-domain metadata: `module`, `version`, `exported_types`, `exported_constructors`, `re_exports`, `imported_summary_refs`, `interface_identities`, `associated_member_identities`, `reserved_identity_slots`, `diagnostic_anchors`, and `exported_sealed_domains`.
  - `SummaryVersion` has only `SPEC057_ORDINARY_TYPE_V1 = 1` and `SPEC059_SEALED_DOMAIN_V2 = 2`; no V3 value exists.
  - `ReservedSemanticIdentitySlots::future_type_functions` is an uninterpreted placeholder, not a public computation-summary contract.
  - `ModuleSummaryRef` keys only `module` and `version`.
- `crates/ash-core/src/type_ir.rs`
  - Checked computation carriers already exist: `TypeComputationHeadId`, `TypeFunctionDef`, `TypeFunctionParam`, `TypeFunctionEquation`, `TypeFunctionPattern`, `TypeFunctionResultExpr`, `TypeFunctionResultConstraint`, and `TypeFunctionSourceAnchors`.
  - Normalizer-facing carriers already exist: `CanonicalTypeExpr::ComputationHeadApp`, `NormalTypeExpr::NeutralComputationApp`, and `NormalFormBlockReason`.
  - There is no core-owned `TypeFunctionSummary`/export-mode carrier and no `ModuleSemanticSummary::exported_type_functions` field.

### ash-parser

- `crates/ash-parser/src/surface.rs`
  - `Definition::TypeFn(TypeFnDef)` carries parsed source declarations.
  - `TypeFnDef` preserves `visibility`, `name`, `params`, `return_type`, `decreases`, `equations`, `header_span`, and `span`.
  - `TypeFnEquation` preserves raw `head`, `patterns`, `result`, `result_span`, and spans; `TypePattern` preserves constructor/var/wildcard syntax only.
- `crates/ash-parser/src/parse_module.rs`
  - `parse_type_fn_definition` parses module-level `type fn` but cuts on `visibility.is_pub()` with no structured SPEC-062 handoff carrier.
  - `starts_with_type_fn_definition` claims `[visibility] type fn` before ordinary `type` parsing.
  - Inline module parsing rejects `type fn` by backtracking when `starts_with_type_fn_definition` is seen inside inline definitions.
- `crates/ash-parser/src/lower.rs`
  - `lower_module_type_metadata` preserves parsed `type_function_defs` in metadata for engine/typeck handoff; it does not lower public computation summaries.

### ash-typeck

- `crates/ash-typeck/src/type_env.rs`
  - `TypeEnv` stores module-local computation state in `local_type_function_heads: HashMap<String, TypeComputationHeadId>` and `local_type_functions: HashMap<TypeComputationHeadId, TypeFunctionDef>`.
  - `register_module_semantic_summary` stages one `ModuleSemanticSummary` at a time through `register_module_semantic_summary_inner`.
  - `register_module_semantic_summary_inner` calls `validate_summary_visibility_and_duplicates`, declares ordinary identities, registers interface/member identities, exposes ordinary representations, then declares and validates sealed domains in two passes within that one summary.
  - `validate_summary_visibility_and_duplicates` accepts only V1/V2, rejects V1 summaries carrying `exported_sealed_domains`, and has no V3/type-function malformed-content checks because no type-function field exists.
  - `register_local_type_functions` stages source declarations and publishes only successfully lowered definitions.
  - `lower_local_type_function` rejects public declarations with `type function '<name>' cannot be public before SPEC-F summaries`.
  - `lookup_local_type_function`, crate-local `lookup_local_type_function_by_head`, and `local_type_function_names` expose only local published definitions.
- `crates/ash-typeck/src/normalizer.rs`
  - `reduce_normalized_computation_app` tries fixture equations first, then `source_first_match_or_blocker`.
  - `source_first_match_or_blocker` consults only `env.lookup_local_type_function_by_head(head)`; imported public computation summaries cannot reduce.
  - Missing heads and blocked matches become `NormalTypeExpr::NeutralComputationApp` with `NormalFormBlockReason::Unsupported` or match blocker reasons.

### ash-engine

- `crates/ash-engine/src/module_loader.rs`
  - `ModuleExports` carries engine-private `type_defs`, `constructor_defs`, `callables`, `semantic_summary: Option<ModuleSemanticSummary>`, and `child_modules`; it has no type-function export table.
  - `load_ordinary_file` gathers `imported_semantic_summaries` and `imported_summary_keys`, pushing summaries for glob, named, constructor, and signature dependencies before typechecking.
  - `merge_or_push_imported_semantic_summary` merges selected summaries when `imported_summary_type_set_matches` agrees, then appends missing constructors and sealed domains.
  - `imported_summary_type_set_matches` compares module, version, and exported ordinary type `(id, exported_name)` pairs only; it ignores sealed-domain sets and has no type-function dimension.
  - `exportable_module_semantic_summary` filters public ordinary exports and public sealed domains and rejects public sealed-domain fields that reference non-exportable sealed domains.
  - `selected_import_type_semantic_summary`, `selected_type_semantic_summary_with_aliases`, `selected_constructor_semantic_summary_with_dependency_visibility`, and side-metadata copy paths select ordinary type/constructor-centered summary slices; no computation-head selection exists.
- `crates/ash-engine/src/lib.rs`
  - Imported summaries are registered by repeated `type_env.register_module_semantic_summary(&summary)` calls in import order, including in `check_file`, `check_module_file`, and related module-check paths.

## Current public type-function rejection points

1. Parser hard rejection: `crates/ash-parser/src/parse_module.rs::parse_type_fn_definition` rejects any `pub`/public visibility `type fn` by returning a cut error before constructing `TypeFnDef`.
2. Typechecker hard rejection: `crates/ash-typeck/src/type_env.rs::lower_local_type_function` rejects `def.visibility.is_pub()` with the SPEC-F diagnostic text before checked carriers are published.
3. Engine public leakage fences:
   - `append_callable_signature_type_function_leaks` reports public callable signatures mentioning local type functions before SPEC-F.
   - `public_callable_signature_resolution_errors` collects local type-function names from module metadata/source and applies that fence before missing ordinary type diagnostics.
   - `public_representation_type_function_leak_errors` reports public ordinary type representations mentioning local type functions before SPEC-F.

## Current summary leakage fences

- Core: `ModuleSemanticSummary` has no equation/head field, so type-function equations cannot serialize through the normative core summary today.
- Engine export: `exportable_module_semantic_summary` filters ordinary/private representation exposure and sealed-domain visibility, and rejects public sealed-domain fields whose `domain_constraint` names are outside the public domain export set.
- TypeEnv import: `validate_summary_visibility_and_duplicates` rejects unsupported summary versions and rejects V1 sealed-domain payloads; ordinary type summaries must be public unless builtin opaque compatibility applies.
- Normalizer: cross-module reductions are fenced because `source_first_match_or_blocker` only sees local `TypeEnv` definitions; unknown imported heads remain neutral/unsupported.

## Import-order risks

- `TypeEnv::register_module_semantic_summary` is one-summary-at-a-time. Sealed-domain declaration/validation is two-pass only inside the current summary, so a domain field referencing a public domain declared in a later imported summary can fail depending on textual import order.
- `ash-engine` registers imported summaries sequentially in `crates/ash-engine/src/lib.rs` instead of using a batch API that first declares all ordinary types, sealed domains, interface/member identities, and computation heads across the import set.
- Re-export/named/glob selected summaries may fragment a module's public facts into multiple selected `ModuleSemanticSummary` values; current merge logic is ordinary-type keyed and may not reconstruct a deterministic all-identities-before-validation batch.
- Normalizer first-match source equation order is deterministic for local definitions, but no imported equation-order preservation path exists yet.

## Existing dedup/cache key gaps

- `ModuleSummaryRef` includes only `module` and `version`; it has no digest/content dimension for computation facts or dependencies.
- `imported_summary_key`/`ImportedSummaryKey` in `ash-engine::module_loader` is used for in-memory dedup of imported summaries, but current selection/merge evidence shows ordinary-type-centered identity. It cannot distinguish future summaries that differ only in sealed-domain closure or type-function equations unless extended.
- `imported_summary_type_set_matches` ignores `exported_sealed_domains`; selected summaries from the same module/version with the same ordinary type set but different domain closure can merge. It also has no computation-head/equation dimension.
- `merge_or_push_imported_semantic_summary` appends missing sealed domains after ordinary-type-keyed matching, which can mask fragmented-domain selection issues but does not validate conflicts before registration.
- No V3 computation-aware version, exported computation field, public dependency refs/digests, compiler algorithm version, or equation/mode content participates in keys today.

## Handoff targets for TASK-845 through TASK-854

- Add a core-owned V3 summary field and explicit public computation summary carrier in `ash-core::semantic_summary`, reusing `ash-core::type_ir` carriers only where the export contract remains explicit.
- Move parser from hard `pub type fn` rejection to span-preserving handoff without assigning summary semantics to parser metadata.
- Add typeck export-closure validation before lowering public equations into summaries.
- Replace one-summary import registration with a batch/two-pass `TypeEnv` API before imported normalizer registration.
- Extend engine summary selection, merge, dedup, re-export, and cache/invalidation keys with sealed-domain and type-function dimensions while keeping semantics in `ash-core`.
