# TASK-858: Associated Family Computation Audit

**Date:** 2026-05-12
**Phase:** 115 / SPEC-063 Associated Type-Family Computation
**Status:** Pre-implementation gate for TASK-859 through TASK-868

## 1. Scope and Gate Result

This audit binds the live Phase 115 implementation surfaces before any TASK-859+ Rust changes. It validates the post-SPEC-062 substrate and assigns each associated-family forcing point to a downstream task.

Gate decision:

- TASK-859+ Rust implementation may start only after this artifact is present and TASK-858 verification passes.
- Associated-family semantics remain owned by `ash-core` carriers and `ash-typeck::TypeEnv`; `ash-parser` remains raw-surface-only and `ash-engine` only transports validated summaries.
- Existing SPEC-035 associated-type substitution remains compatibility behavior until the TASK-862 bridge is verified.

## 2. Live Carrier Inventory

| Area | Live carrier / function | Evidence | Gap for SPEC-063 | Owner |
|------|-------------------------|----------|------------------|-------|
| Parser inline module dispatch | `parse_definitions` dispatches inline module definitions | `crates/ash-parser/src/parse_module.rs:137-220` | Inline modules explicitly reject `type fn` (`159-162`) and let sealed domains fall to unsupported-inline handling (`169-171`); `sealed type family` inside interfaces still needs its own body branch. | TASK-859 |
| Parser top-level module dispatch | Top-level module-file loop dispatches `type fn`, ordinary `type`, sealed domains, interfaces, and impls | `crates/ash-parser/src/parse_module.rs:2870-2937` | Top-level dispatch can parse direct type-fn and sealed-domain declarations, but interface bodies still recognize only ordinary associated `type`. | TASK-859 |
| Parser interface params | `parse_optional_type_parameter_names` returns `Vec<Name>` | `crates/ash-parser/src/parse_module.rs:1071-1080`; `crates/ash-parser/src/surface.rs:632-646` | Cannot preserve `Xs: TypeList` domain-constrained params or spans. | TASK-859 |
| Parser associated member decl | `surface::AssociatedTypeDecl { name, span }`; `parse_associated_type_decl` accepts `type Name;` | `crates/ash-parser/src/surface.rs:601-608`; `crates/ash-parser/src/parse_module.rs:1109-1119` | Cannot encode ordinary vs sealed family, result domain, decreases clause, or member spans beyond name. | TASK-859 |
| Parser impl associated binding | `surface::AssociatedTypeBinding { name, ty, span }`; `parse_associated_type_binding` accepts `type Name = Type;` | `crates/ash-parser/src/surface.rs:610-620`; `crates/ash-parser/src/parse_module.rs:1228-1243` | Can carry simple SPEC-035 RHS only; no family-specific pattern/result scheme metadata. | TASK-859, TASK-861 |
| Parser projection type | `surface::Type::Associated { base, name }` | `crates/ash-parser/src/surface.rs:1838-1854` | Supports `Base::Assoc`; no explicit leading `<Interface<Args>>::Assoc` raw carrier. | TASK-859 |
| Parser lowering to workflow-contract type expr | `lower_type_to_type_expr` rewrites `Type::Associated` into `TypeExpr::Constructor` | `crates/ash-parser/src/lower.rs:1199-1202` | This workflow-contract lowering path erases projection shape and must either preserve or explicitly reject associated-family projections before they can be semantically meaningful there. | TASK-859/TASK-868 |
| Parser lowering to core AST type expr | `lower_surface_type` preserves `Type::Associated` as `ast::TypeExpr::Associated` | `crates/ash-parser/src/lower.rs:1206-1240` | Core AST path preserves compatibility projections but still lacks explicit family projection/declaration metadata. | TASK-859/TASK-860 |
| Core AST associated projection | `ast::TypeExpr::Associated { base, name }` | `crates/ash-core/src/ast.rs:795-808` | Compatibility-only carrier; explicit family projection needs distinguishable carrier or preservation rule. | TASK-859/TASK-860 |
| Core associated member | `ast::AssociatedType { name }` and `semantic_summary::AssociatedMemberIdentityKind::AssociatedType` | `crates/ash-core/src/ast.rs:810-820`; `crates/ash-core/src/semantic_summary.rs:196-222` | No sealed-family member kind, result-domain/decreases metadata, or summary schema. | TASK-860 |
| Canonical projection | `CanonicalTypeExpr::Projection { interface, member, args, kind, rigidity }` | `crates/ash-core/src/type_ir.rs:48-71` | Sufficient identity spine, but lacks named associated-family head helper APIs and family-specific scheme/result tables. | TASK-860 |
| Checked type-fn result carrier | `TypeFunctionResultExpr::{DomainConstructorApp, Projection, ComputationHeadApp, ...}` | `crates/ash-core/src/type_ir.rs:156-207` | Reusable shape exists, but associated-family scheme carrier must state ownership/head/decl metadata instead of overloading ordinary impl scheme. | TASK-860 |
| Normal form projection | `NormalTypeExpr::Projection { interface, member, args, kind, rigidity, reason }` | `crates/ash-core/src/type_ir.rs:224-272` | Blocker carrier exists but reasons are too coarse for associated-family private/export/decreases/selection diagnostics. | TASK-866/TASK-868 |
| Summary versions | `SummaryVersion::{SPEC057_ORDINARY_TYPE_V1, SPEC059_SEALED_DOMAIN_V2, SPEC062_TYPE_COMPUTATION_V3}` | `crates/ash-core/src/semantic_summary.rs:549-565` | V4 summary version and family facts missing; V1/V2/V3 with family facts must be malformed. | TASK-860/TASK-867 |
| TypeEnv interfaces | `InterfaceInfo { type_params: Vec<String>, associated_types: Vec<String>, ... }` | `crates/ash-typeck/src/type_env.rs:352-357` | Name-only params/members cannot validate domain-constrained decreases or sealed family member metadata. | TASK-861 |
| TypeEnv impls | `ImplScheme { head: Type, where_bounds, associated_type_bindings, methods }` | `crates/ash-typeck/src/type_env.rs:556-560` | Ordinary impl scheme lacks family scheme ownership, module context, pattern table, closedness, and result-domain validation. | TASK-861/TASK-863/TASK-865 |
| TypeEnv associated syntax conversion | `type_expr_to_type` / `surface_type_to_type` converts associated projections to `Type::Associated` after bound lookup | `crates/ash-typeck/src/type_env.rs:260-285`; `crates/ash-typeck/src/type_env.rs:753-758` | Only bound-driven rigid compatibility path; no explicit interface-spine family projection elaboration. | TASK-862/TASK-864 |
| TypeEnv canonical projection conversion | `lower_associated_projection_to_canonical`, `lower_core_type_expr_to_canonical`, and `lower_surface_type_to_canonical` lower associated projections into canonical projection IR | `crates/ash-typeck/src/type_env.rs:5245-5343`; `crates/ash-typeck/src/type_env.rs:5359-5438`; `crates/ash-typeck/src/type_env.rs:5442-5520` | Canonical conversion exists for associated projections, but Phase 115 still needs explicit family projection elaboration, rigid-bound preservation, and normalizer-reduction ownership to avoid hidden impl search. | TASK-862/TASK-864/TASK-866 |
| Normalizer | `Normalizer` normalizes computation heads and preserves projection normals | `crates/ash-typeck/src/normalizer.rs:444-580`; `crates/ash-typeck/src/normalizer.rs:723-790` | No local/imported associated-family lookup/reduction path yet. | TASK-866 |
| Engine summary transport | `register_imported_semantic_summaries`, `ModuleSemanticSummary` routing, `module_loader` selected summaries | `crates/ash-engine/src/lib.rs:1818-1822`; `crates/ash-engine/src/module_loader.rs:380-520`; `crates/ash-engine/src/module_loader.rs:526-535` | V4 family summaries absent; engine must transport only, not interpret. | TASK-867 |
| Engine type conversion | `SurfaceType::Associated` rejected in one engine conversion path | `crates/ash-engine/src/lib.rs:1936-1939` | TASK-867/TASK-868 must check whether summary/export paths route associated-family type expressions through this conversion. | TASK-867/TASK-868 |
| Engine SPEC-035 bridge | `monomorphize.rs` calls `TypeEnv::normalize_associated_types` | `crates/ash-engine/src/monomorphize.rs:187-216` | Existing simple associated substitution path must stay green and not become hidden family search. | TASK-862 |

## 3. Live Call Graph / Flow Map

### 3.1 Source parsing and lowering

1. Inline module definitions enter `parse_module.rs::parse_definitions` (`crates/ash-parser/src/parse_module.rs:137-220`). This path dispatches interfaces (`193-195`) and impls (`198-200`), rejects inline `type fn` (`159-162`), and intentionally leaves sealed domains to unsupported-inline handling (`169-171`).
2. Top-level module-file definitions are dispatched by the module-file loop (`crates/ash-parser/src/parse_module.rs:2870-2937`). This path dispatches direct `type fn` (`2880-2882`), ordinary `type` (`2885-2887`), sealed domains (`2890-2892`), interfaces (`2915-2917`), and impls (`2920-2922`).
3. `parse_interface_definition` parses `interface Name<Params> { ... }` (`1071-1106`). The interface body currently recognizes only `type` for associated declarations (`1084-1089`) and otherwise parses method signatures.
4. `parse_associated_type_decl` requires `type Name;` (`1109-1119`).
5. `parse_impl_definition` parses `impl <Params> Interface<TypeArgs> where ... { ... }` (`1154-1199`). Impl bodies currently recognize only `type Name = Type;` for associated type bindings (`1175-1180`, `1228-1243`).
6. `lower_surface_type` preserves core AST associated projections (`crates/ash-parser/src/lower.rs:1230-1232`), but `lower_type_to_type_expr` erases associated projections into constructor-shaped workflow-contract type expressions (`1199-1202`).
7. `lower.rs` maps surface interface/impl definitions into core `ast::InterfaceDef` / `ast::ImplDef` with the same name-only associated type model.

TASK-859 must add the raw parser/surface/core-lowering shape before semantic tasks depend on it, and must keep the workflow-contract lowering seam honest by preserving or rejecting family projections instead of silently erasing them.

### 3.2 TypeEnv compatibility path

1. `ash-typeck::type_env::type_expr_to_type` and `surface_type_to_type` encounter `TypeExpr::Associated` / `surface::Type::Associated` (`crates/ash-typeck/src/type_env.rs:260-285`, `753-758`).
2. They resolve the base type, then call `resolve_associated_interface_from_type_var_bounds` for type-variable bounds (`595-648`).
3. The result is `Type::Associated { interface, base, name }`.
4. `normalize_associated_types` and `crates/ash-engine/src/monomorphize.rs:187-216` use selected impl substitution for SPEC-035 concrete paths.
5. Ambiguity and unresolved cases are reported with existing `TypeEnvError` variants.

TASK-862 owns preserving this behavior while new family declarations are introduced. TASK-864 owns rigid bound behavior where this path produces evidence but no equation.

### 3.3 Normalizer path

1. `Normalizer::normalize` consumes `CanonicalTypeExpr` and returns `NormalTypeExpr` (`crates/ash-typeck/src/normalizer.rs:444-580`).
2. Computation heads reduce through existing type-function tables.
3. Projections are preserved as `NormalTypeExpr::Projection` with rigidity/reason metadata (`723-790`).
4. Definitional equality compares normal forms and classifies neutral/projection blockers (`768-790`, `858-971`).

TASK-866 owns the first local associated-family lookup path. TASK-867 may expose imported validated family tables to the same lookup after V4 import validation.

### 3.4 Summary transport path

1. `ash-core::semantic_summary::ModuleSemanticSummary` owns versions and public type/type-function facts (`crates/ash-core/src/semantic_summary.rs:548-556` for `SummaryVersion`; `crates/ash-core/src/semantic_summary.rs:722-749` for `ModuleSemanticSummary` public summary fields).
2. `ash-engine::module_loader` selects and routes imported public summaries (`crates/ash-engine/src/module_loader.rs:380-520`) and source-visible type-function head helpers (`crates/ash-engine/src/module_loader.rs:526-535`).
3. `ash-engine::Engine` stores imported summaries and registers them into `TypeEnv` (`crates/ash-engine/src/lib.rs:79-83`, `732-785`, `1818-1822`).
4. `TypeEnv::register_module_semantic_summaries` batch-registers identity/type-function facts (`crates/ash-typeck/src/type_env.rs:2316-2357`) and declares/validates imported type-function summaries (`crates/ash-typeck/src/type_env.rs:2398-2455`).

TASK-860 creates V4 carriers and validation hooks; TASK-867 wires export/import/engine transport. Engine remains non-semantic.

## 4. Forcing and Selection Matrix

| ID | Forcing point | Current behavior | Required Phase 115 behavior | Owner |
|----|---------------|------------------|-----------------------------|-------|
| AF-PARSE-01 | Interface type params | `Vec<Name>` only | Preserve typed/domain-constrained params and spans. | TASK-859 |
| AF-PARSE-02 | Interface member body item | `type Name;` only | Parse ordinary associated type vs `sealed type family Name: Domain decreases Param`. | TASK-859 |
| AF-PARSE-03 | Projection type syntax | `Base::Assoc` only | Parse explicit `<Interface<Args>>::Assoc` without path-qualified interface names in MVP. | TASK-859 |
| AF-PARSE-04 | Lowering boundary | Core AST path preserves `Type::Associated`; workflow-contract path erases it | Preserve raw family metadata or explicitly fail closed before semantic consumers see degraded shape. | TASK-859 |
| AF-CORE-01 | Family head identity | Existing pair `(InterfaceIdentityId, AssociatedMemberIdentityId)` implicit | Add named helper/newtype APIs for reducible family head identity. | TASK-860 |
| AF-CORE-02 | Family scheme/result carriers | Direct type-fn carriers only | Add associated-family declaration/scheme/result carriers reusing checked result expression shapes where sound. | TASK-860 |
| AF-CORE-03 | Summary versioning | V1/V2/V3 only | Add V4 and malformed older-summary-with-family-facts validation. | TASK-860 |
| AF-TYPECK-01 | Interface declaration registry | Name-only params and associated members | Register sealed family metadata, result domain, decreases, module owner. | TASK-861 |
| AF-TYPECK-02 | Downstream impls | Ordinary impl coherence | Reject unauthorized downstream sealed-family equations. | TASK-861 |
| AF-TYPECK-03 | SPEC-035 substitution | Selected concrete impl substitution | Preserve simple non-family associated substitution and rigid fallback. | TASK-862 |
| AF-CANON-01 | Canonical projection conversion | Existing lowerers produce canonical projection IR for associated projections | Preserve compatibility projections, mark rigid where-bound projections without impl search, and route only validated family projections into normalizer reduction. | TASK-862/TASK-864/TASK-866 |
| AF-SEL-01 | Unique scheme selection | No family table | One-way selected scheme matching; bind scheme-owned vars only. | TASK-863 |
| AF-RIGID-01 | Where-bound projection | Bound-driven `Type::Associated` | Keep rigid projection evidence; do not search all impls. | TASK-864 |
| AF-REC-01 | Recursive family validation | Direct `type fn` totality only | Adapt coverage/overlap/decreasingness to associated-family heads. | TASK-865 |
| AF-NORM-01 | Projection normalization | Preserve projection as stuck/rigid/neutral | Reduce only validated sealed-family projections through unique local/imported tables. | TASK-866 |
| AF-SUM-01 | Public family summaries | No V4 family facts | Export/import closed public family summaries only with public-visible dependency closure. | TASK-867 |
| AF-DIAG-01 | Diagnostic mapping | Existing `TypeEnvError` / normalizer blockers | Add precise family diagnostics, spans, and acceptance matrix evidence. | TASK-868 |

## 5. Downstream Binding Table

| Task | Source files | Test targets | Callsite/audit-row IDs | Task-file action |
|------|--------------|--------------|------------------------|------------------|
| TASK-859 | `crates/ash-parser/src/surface.rs`; `crates/ash-parser/src/parse_module.rs`; `crates/ash-parser/src/parse_type_def.rs`; `crates/ash-parser/src/lower.rs`; `crates/ash-parser/src/error.rs` | Exact zero-test-safe verification confirmed in `docs/plan/tasks/TASK-859-associated-family-surface-and-compat-parser.md:86-107`: list target, grep guard for associated family markers, then `cargo test -p ash-parser --test task_859_associated_family_surface -- --nocapture` | AF-PARSE-01, AF-PARSE-02, AF-PARSE-03, AF-PARSE-04 | confirmed unchanged |
| TASK-860 | `crates/ash-core/src/type_ir.rs`; `crates/ash-core/src/semantic_summary.rs`; conditional `crates/ash-typeck/src/type_env.rs`; conditional `crates/ash-engine/src/module_loader.rs` | Exact zero-test-safe verification confirmed in `docs/plan/tasks/TASK-860-core-associated-family-identity-carriers.md:86-107`: list target, grep guard for associated family markers, then `cargo test -p ash-core --test task_860_associated_family_carriers -- --nocapture` | AF-CORE-01, AF-CORE-02, AF-CORE-03 | confirmed unchanged |
| TASK-861 | `crates/ash-typeck/src/type_env.rs`; `crates/ash-typeck/src/error.rs`; conditional `crates/ash-engine/src/module_loader.rs` | Exact zero-test-safe verification confirmed in `docs/plan/tasks/TASK-861-typeck-family-declaration-registration-coherence.md:85-106`: list target, grep guard for associated family markers, then `cargo test -p ash-typeck --test task_861_associated_family_registration -- --nocapture` | AF-TYPECK-01, AF-TYPECK-02 | confirmed unchanged |
| TASK-862 | `crates/ash-typeck/src/type_env.rs`; conditional `crates/ash-typeck/src/normalizer.rs`; `crates/ash-engine/src/monomorphize.rs` as regression surface | Exact zero-test-safe verification confirmed in `docs/plan/tasks/TASK-862-spec035-substitution-compatibility-bridge.md:82-103`: list target, grep guard for SPEC-035 associated markers, then `cargo test -p ash-typeck --test task_862_spec035_associated_compat -- --nocapture` | AF-TYPECK-03, AF-CANON-01 | confirmed unchanged |
| TASK-863 | `crates/ash-typeck/src/type_env.rs`; conditional `crates/ash-typeck/src/normalizer.rs` | Exact zero-test-safe verification confirmed in `docs/plan/tasks/TASK-863-unique-generic-impl-family-selection.md:83-104`: list target, grep guard for associated family markers, then `cargo test -p ash-typeck --test task_863_associated_family_selection -- --nocapture` | AF-SEL-01 | confirmed unchanged |
| TASK-864 | `crates/ash-typeck/src/type_env.rs`; `crates/ash-typeck/src/normalizer.rs`; conditional `crates/ash-typeck/src/error.rs` | Exact zero-test-safe verification confirmed in `docs/plan/tasks/TASK-864-rigid-where-bound-projection-boundary.md:83-104`: list target, grep guard for rigid where-bound markers, then `cargo test -p ash-typeck --test task_864_rigid_where_bound_projection -- --nocapture` | AF-RIGID-01, AF-CANON-01 | confirmed unchanged |
| TASK-865 | `crates/ash-typeck/src/type_env.rs`; `crates/ash-typeck/src/error.rs` | Exact zero-test-safe verification confirmed in `docs/plan/tasks/TASK-865-recursive-associated-family-totality.md:85-106`: list target, grep guard for recursive associated family markers, then `cargo test -p ash-typeck --test task_865_recursive_associated_family -- --nocapture` | AF-REC-01 | confirmed unchanged |
| TASK-866 | `crates/ash-typeck/src/normalizer.rs`; `crates/ash-typeck/src/type_env.rs`; conditional `crates/ash-core/src/type_ir.rs` | Exact zero-test-safe verification confirmed in `docs/plan/tasks/TASK-866-normalizer-projection-family-integration.md:86-107`: list target, grep guard for normalizer associated family markers, then `cargo test -p ash-typeck --test task_866_associated_family_normalizer -- --nocapture` | AF-NORM-01, AF-CANON-01 | confirmed unchanged |
| TASK-867 | `crates/ash-core/src/semantic_summary.rs`; `crates/ash-engine/src/module_loader.rs`; `crates/ash-engine/src/lib.rs`; `crates/ash-typeck/src/type_env.rs`; conditional `crates/ash-typeck/src/normalizer.rs` | Exact zero-test-safe verification confirmed in `docs/plan/tasks/TASK-867-associated-family-summary-export-import.md:88-117`: core, engine, and typeck list targets with grep guards, then exact nocapture targets for `task_867_associated_family_summary`, `task_867_associated_family_summary_transport`, and `task_867_associated_family_import` | AF-SUM-01 | confirmed unchanged |
| TASK-868 | `docs/plan/audits/TASK-868-associated-family-acceptance-matrix.md`; `crates/ash-typeck/src/error.rs`; `crates/ash-typeck/src/type_env.rs`; `crates/ash-typeck/src/normalizer.rs`; conditional `crates/ash-engine/src/module_loader.rs` | Exact zero-test-safe verification confirmed in `docs/plan/tasks/TASK-868-associated-family-diagnostics-acceptance-matrix.md:85-114`: typeck list target with diagnostic grep guard and nocapture test; optional engine target guarded by file existence | AF-DIAG-01 plus all acceptance rows in SPEC-063 section 13 | confirmed unchanged |

## 6. Non-Interference Risks

| Risk | Guard |
|------|-------|
| SPEC-035 substitution accidentally replaced by family lookup | TASK-862 must include positive selected-concrete impl substitution and negative rigid/no-hidden-search tests. |
| Generic where-bound evidence triggers impl search | TASK-864 must assert bound-only projections remain rigid. |
| Family selection solves caller variables or uses expected output | TASK-863/TASK-866 must include non-inversion tests over abstract arguments. |
| Parser accepts `sealed type family` but lowering erases metadata | TASK-859 must preserve raw metadata or add explicit fail-closed lowering diagnostics. |
| V4 export leaks private family equations/dependencies | TASK-867 must include positive public closure and negative private dependency/export rejection tests. |
| Older summaries accidentally accept family facts | TASK-860/TASK-867 must reject V1/V2/V3 family facts before partial registration. |
| Engine starts interpreting family semantics | TASK-867 review must verify engine only transports summaries and delegates validation/reduction to TypeEnv/normalizer. |

## 7. Diagnostics and Span Seams

Existing diagnostics live primarily in `crates/ash-typeck/src/error.rs:173-326` and engine span extraction currently pattern-matches `TypeEnvError` in `crates/ash-engine/src/module_loader.rs:52-74`. Adding family-specific `TypeEnvError` variants may require one of:

1. extending engine match arms immediately when variants are introduced; or
2. adding a central `TypeEnvError::span()` helper and routing engine diagnostics through it.

TASK-868 owns final diagnostic acceptance. Earlier semantic tasks that introduce new variants must add minimal routing before their focused tests can be considered clean.

## 8. Pre-Implementation Checklist

- [x] Live parser/interface/impl/projection surfaces audited.
- [x] Live core canonical/normal-form/summary carriers audited.
- [x] Live TypeEnv associated-type and impl scheme paths audited.
- [x] Live normalizer projection behavior audited.
- [x] Live engine summary and SPEC-035 monomorphization seams audited.
- [x] Downstream TASK-859 through TASK-868 binding rows include exact files, test targets, zero-test-safe verification commands, audit IDs, and task-file action.
- [x] No Rust implementation performed by TASK-858.
