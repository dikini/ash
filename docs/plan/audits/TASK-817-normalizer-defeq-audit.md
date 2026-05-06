# TASK-817: Normalizer / Definitional Equality Audit

**Status:** Complete
**Date:** 2026-05-06
**Task:** [TASK-817](../tasks/TASK-817-normalizer-defeq-audit-gate.md)
**Spec/Plan:** [SPEC-060](../../spec/SPEC-060-NORMALIZER-DEFINITIONAL-EQUALITY-CORE.md), [PLAN-108](../PLAN-108-NORMALIZER-DEFINITIONAL-EQUALITY-CORE.md)
**Scope:** Docs-only audit gate. No Rust files changed.

## 1. Executive summary

The live Phase 112 baseline has the Phase 110/111 substrates needed to start a normalizer, but no normal-form view, fixture equation table, or definitional-equality API exists yet.

Current equality is still centered on `ash-typeck::TypeEnv::canonicalize_type_for_equality` followed by the legacy `types::unify` solver. That path:

- preserves ordinary nominal constructor decomposition;
- recursively peels transparent aliases;
- canonicalizes imported/local nominal and associated-projection textual heads through identity maps;
- can solve `Type::Var(TypeVar)` metas anywhere legacy unification reaches;
- does not understand `CanonicalTypeExpr::ComputationHeadApp` as reducible or neutral computation;
- does not lower every `Type` shape to `CanonicalTypeExpr`.

TASK-826 must therefore adopt definitional equality only at the exact owned forcing points below and must keep legacy fallback status wherever lowering/canonical IR coverage is not yet complete.

## 2. Core substrate seams

### 2.1 `crates/ash-core/src/type_ir.rs`

Live carriers:

- `TypeComputationHeadId { module: ModuleIdentity, name: String }` (`type_ir.rs:18-33`).
- `ProjectionRigidity::{Rigid, Neutral}` (`type_ir.rs:35-41`).
- `CanonicalTypeExpr` (`type_ir.rs:43-66`):
  - `Primitive(String)`;
  - `Var(String)`;
  - `NominalApp { origin: TypeDeclId, visible_name, args, kind }`;
  - `Projection { interface: InterfaceIdentityId, member: AssociatedMemberIdentityId, args, kind, rigidity }`;
  - `ComputationHeadApp { head: TypeComputationHeadId, args, kind }`.

Audit finding: this is an IR substrate only. The file explicitly says it does not yet define lowering, normalization, equality, or diagnostics (`type_ir.rs:10`). There is no `NormalTypeExpr`/normal-form enum and no domain-constructor normal-form variant yet. TASK-818 owns those core carriers.

### 2.2 `crates/ash-core/src/semantic_summary.rs`

Relevant live identities and summaries:

- `ModuleIdentity` equality/hash identity uses only `crate_id` and `module_id` (`semantic_summary.rs:22-43`).
- Ordinary type identities: `TypeDeclId`, `ConstructorId` (`semantic_summary.rs:115-168`).
- Interface/projection identities: `InterfaceIdentityId`, `AssociatedMemberIdentityId` (`semantic_summary.rs:170-220`).
- Sealed-domain identities: `SealedDomainId`, `DomainConstructorId` (`semantic_summary.rs:222-262`).
- Domain field/constructor/domain summaries: `DomainFieldSummary`, `DomainConstructorSummary`, `SealedDomainSummary` (`semantic_summary.rs:264-385`).
- `ModuleSemanticSummary` exports ordinary types, constructors, interface/member identity summaries, reserved identity slots, and `exported_sealed_domains` (`semantic_summary.rs:628-651`).
- `ReservedSemanticIdentitySlots::future_type_functions` remains a placeholder string list and is not an equation export format (`semantic_summary.rs:553-566`).

Audit finding: sealed-domain metadata is available for TASK-818/TASK-821 normal forms, but public type-function source syntax and public equation export/import are not present. Fixture equations for SPEC-060 must remain internal/test setup only and must not be added to `ModuleSemanticSummary` in Phase 112.

## 3. TypeEnv canonicalization/equality seams

### 3.1 Canonical lowering

Exact functions:

- `TypeEnv::lower_core_type_expr_to_canonical(&TypeExpr) -> Result<CanonicalTypeExpr, TypeError>` (`type_env.rs:2433-2513`).
- `TypeEnv::lower_surface_type_to_canonical(&SurfaceType) -> Result<CanonicalTypeExpr, TypeError>` (`type_env.rs:2516-2627`).
- Helper `TypeEnv::lower_associated_projection_to_canonical` returns `CanonicalTypeExpr::Projection` with `ProjectionRigidity` after identity/member lookup and arity checks (`type_env.rs:2400-2417`).

Coverage:

- primitives lower to `Primitive`;
- unresolved names lower to canonical abstract `Var(String)`;
- nominal constructors lower to `NominalApp` after `resolve_type` and arity checks;
- associated projections lower to canonical `Projection` if base shape is supported.

Fallback/unsupported:

- core tuple/record type expressions are rejected (`type_env.rs:2472-2483`);
- surface tuple/record/list/capability/function types are rejected for canonical lowering (`type_env.rs:2596-2625`);
- nested projection bases and non-nominal projection bases are rejected (`type_env.rs:2484-2511`, `2555-2595`).

TASK-826 implication: only route through definitional equality where both sides can be represented as canonical expressions or where TASK-825 has defined a safe top-level-meta bridge. Otherwise use existing `unify_types` fallback.

### 3.2 Transparent aliases and current equality

Exact functions:

- `TypeEnv::canonicalize_transparent_aliases(&Type) -> Type` (`type_env.rs:2698-2754`).
- `TypeEnv::render_type_for_diagnostics(&Type) -> String` (`type_env.rs:2756-2759`).
- `TypeEnv::canonicalize_type_for_equality(&Type) -> Type` (`type_env.rs:2761-2819`).
- `TypeEnv::unify_types(&Type, &Type) -> Result<Substitution, UnifyError>` (`type_env.rs:2821-2827`).
- `TypeEnv::types_equivalent_for_equality(&Type, &Type) -> bool` (`type_env.rs:2829-2832`).

Current behavior:

- `canonicalize_transparent_aliases` recursively expands transparent aliases but intentionally does not own equality rollout.
- `render_type_for_diagnostics` is currently a thin `ty.to_string()` wrapper.
- `canonicalize_type_for_equality` recursively expands transparent aliases and rewrites nominal constructor names through `canonical_constructor_name_for_equality` (`type_env.rs:2664-2674`).
- `canonicalize_type_for_equality` rewrites associated projection interface/member spelling through `canonical_associated_projection_for_equality` (`type_env.rs:2676-2696`, `2802-2816`).
- `unify_types` invokes legacy `types::unify` on canonicalized `Type` values.
- `types_equivalent_for_equality` is boolean `unify_types(...).is_ok()`.

Compatibility constraint: `types::unify` decomposes same-headed `Type::Constructor` arguments and solves `Type::Var(TypeVar)` metas with occurs checks (`types.rs:435-646`). This must remain available for ordinary nominal constructor unification.

### 3.3 Associated output compatibility path

Exact functions:

- `TypeEnv::normalize_associated_types(&Type, &ImplScheme, &Substitution) -> Result<Type, TypeEnvError>` (`type_env.rs:4423-4496`).
- `TypeEnv::resolve_interface_method_call` selects an impl and calls `normalize_associated_types` on the method return (`type_env.rs:4498-4517`).
- `TypeEnv::register_impl` computes method expected return type with `normalize_associated_types` (`type_env.rs:3644-3648`).

Audit finding: this is SPEC-035 simple selected-impl substitution only. It looks like a normalizer by name, but it only substitutes associated output bindings from an already selected scheme and recurses through ordinary `Type` structure. It is not recursive associated-family computation and must not become the Phase 112 type-family normalizer.

## 4. Canonical abstract variables vs inference metas

| Carrier | Location | Meaning | Current producer | Current solver behavior | Phase 112 bridge guidance |
|---|---|---|---|---|---|
| `CanonicalTypeExpr::Var(String)` | `ash-core::type_ir::CanonicalTypeExpr` (`type_ir.rs:47`) | Abstract source/canonical type variable or unresolved canonical name in a canonical type expression. It is part of canonical IR and should normalize to a neutral variable/name. | `lower_core_type_expr_to_canonical` on `TypeExpr::Named` unbound by `resolve_type` (`type_env.rs:2453-2455`); `lower_surface_type_to_canonical` on `SurfaceType::Name` unbound by `resolve_type` (`type_env.rs:2536-2538`). | None today. It participates only in structural equality of canonical values because no normalizer/equality API exists. | Treat as rigid/abstract in normalizer. Do not solve it. It may block fixture equation selection and produce neutral/stuck normal forms. |
| `Type::Var(TypeVar)` | `ash-typeck::types::Type` (`types.rs:41-42`, `74-85`) | Inference meta variable used by the legacy typechecker. | Fresh variables from expression inference, type params, annotations fallback, generic constructors, impl params. Examples: `check_expr.rs:490`, `543`, `638`; `TypeVar::fresh` (`types.rs:78-85`). | Legacy `types::unify` solves it through `bind_var` with occurs check (`types.rs:450-452`, `631-646`) anywhere unification reaches. `Substitution::apply` recursively resolves it (`types.rs:135-182`). | TASK-825 owns any bridge. For TASK-826, allow fallback to existing `unify_types` for meta-solving. If definitional equality sees `Type::Var` through a bridge, solving must be top-level only and must not solve underneath neutral computation heads. |

Key constraint: `CanonicalTypeExpr::Var(String)` is not a unification meta and must not be rewritten as `Type::Var(TypeVar)` just to reuse legacy solving. Conversely, `Type::Var(TypeVar)` is not a stable canonical abstract variable name for exported/summary IR.

## 5. Forcing-point matrix for TASK-826

TASK-826 must consume this matrix rather than widening by search-and-replace.

| ID | SPEC-060 forcing point | Exact live function/callsite | Current behavior | TASK owner | TASK-826 status |
|---|---|---|---|---|---|
| FP-1 | Central `TypeEnv` equality API | `TypeEnv::unify_types` (`type_env.rs:2822-2827`) | Canonicalizes aliases/identity spelling, then calls legacy `types::unify`; solves metas and decomposes constructors. | TASK-825 defines non-inverting boundary; TASK-826 rolls out. | **Owned, guarded.** Route through definitional equality only when both sides can be canonicalized/lowered without losing ordinary behavior. Preserve fallback to legacy `unify_types` for unsupported `Type` shapes and meta-solving. |
| FP-2 | Boolean equality convenience | `TypeEnv::types_equivalent_for_equality` (`type_env.rs:2830-2831`) | Boolean wrapper over `unify_types`. | TASK-826 | **Owned, guarded.** May call a boolean definitional-equality wrapper only after FP-1 constraints exist; otherwise keep fallback. |
| FP-3 | Capability implementation signature param comparison | `register_capability_implementation`, parameter loop using `types_equivalent_for_equality` (`type_env.rs:3176-3189`) | Boolean equality; diagnostic uses direct `{expected_param}` / `{actual_param}` Display. | TASK-826/TASK-827 | **Fallback/deferred unless canonicalizable.** Candidate adoption through FP-2. Rendering can be improved by TASK-827 after normalized evidence exists. |
| FP-4 | Capability implementation signature return comparison | `register_capability_implementation` return check using `types_equivalent_for_equality` (`type_env.rs:3192-3201`) | Boolean equality; direct Display in mismatch. | TASK-826/TASK-827 | **Fallback/deferred unless canonicalizable.** Candidate adoption through FP-2. |
| FP-5 | Capability implementation operation body return check | `validate_capability_implementation_operation_body`, `self.unify_types(&operation_info.return_type, &actual_return_ty)` (`type_env.rs:3382-3396`) | Legacy unification; direct Display in error. | TASK-826 | **Owned as declared-return/actual-return seam, guarded.** Use structured normalized mismatch only where canonicalizable; preserve fallback. |
| FP-6 | Impl overlap/coherence check | `register_impl` overlap loop, `self.unify_types(&scheme.head, &impl_head).is_ok()` (`type_env.rs:3498-3512`) | Legacy unification on interface-head constructors; can solve impl-scheme metas. | TASK-826 with TASK-825 boundary | **Owned, guarded.** Normalize compatible canonical heads before comparison. Do not solve under neutral computation heads. Preserve current duplicate/overlap behavior for nominal heads. |
| FP-7 | Impl method return checking | `register_impl`, `self.unify_types(&expected_return_ty, &actual_return_ty)` (`type_env.rs:3673-3683`) | Legacy unification after simple associated-output substitution; direct Display in error. | TASK-826/TASK-827 | **Owned, guarded.** Declared expected-vs-actual return seam. Structured normalized mismatch if canonicalizable; fallback otherwise. |
| FP-8 | Interface method call argument selection | `select_impl_scheme`, loop `self.unify_types(&subst.apply(expected), actual)` (`type_env.rs:4550-4555`) | Solves method type params from actual args. | TASK-825/TASK-826 | **Fallback-heavy.** Keep legacy meta-solving for current call inference. Defeq may be used only for already-ground canonical slices after TASK-825 defines bridge. |
| FP-9 | Impl selection / where-bound selection | `find_matching_impl_scheme`, `self.unify_types(&scheme.head, target_head)` (`type_env.rs:4599-4649`) | Selects impl by unifying scheme head against target head, recursing on where-bounds. | TASK-825/TASK-826 | **Fallback-heavy.** Preserve current impl selection. If normalized heads are introduced, they must not invert neutral computation heads and must keep recursion/fuel behavior separate from semantic normalizer fuel. |
| FP-10 | Expression branch expected/actual comparison | `check_expr` `Expr::If` branches: direct `unify(&then_ty, &else_ty)` and `unify(&then_ty, &Type::Null)` (`check_expr.rs:496-535`) | Bypasses TypeEnv; direct legacy unification; direct Display in diagnostic. | TASK-826 if selected | **Deferred/fallback for TASK-826 unless explicitly lifted.** This is a current expression comparison seam, but it lacks TypeEnv equality routing. If adopted, replace only these exact calls and keep fallback. |
| FP-11 | Function return annotation vs body | `check_expr` `Expr::FnDef`, `unify(&ann_ty, &body_ty)` (`check_expr.rs:648-664`) | Bypasses TypeEnv; direct legacy unification; direct body type Display in diagnostic. | TASK-826 if selected | **Deferred/fallback.** Candidate declared-return seam, but no TypeEnv method is used today. TASK-826 may touch only this exact callsite if it decides to route annotation checks through env-aware defeq. |
| FP-12 | Constructor tuple field expected/actual | `check_tuple_constructor_fields`, `unify(&expected_ty_subst, &field_result.ty)` (`check_expr.rs:2898-2913`) | Legacy unification; direct `to_string()` rendering. | Later / TASK-827 diagnostics if selected | **Deferred for TASK-826.** SPEC-060 says constructor-field/pattern/exhaustiveness callsites remain out unless named. Do not adopt by search. |
| FP-13 | Constructor named field expected/actual | `check_named_constructor_fields`, `unify(&expected_ty_subst, &field_result.ty)` (`check_expr.rs:2958-2977`) | Legacy unification; direct `to_string()` rendering. | Later / TASK-827 diagnostics if selected | **Deferred for TASK-826.** Same boundary as FP-12. |
| FP-14 | Capability-binding operation argument expected/actual | `check_capability_binding_operation_call`, `unify(&expected_ty, &actual_ty)` (`check_expr.rs:2371-2379`) | Legacy unification for admitted binding operation args; direct Display in mismatch diagnostic. | TASK-826 if selected / TASK-827 diagnostics | **Deferred/fallback.** Expression argument seam. TASK-826 must not adopt it by search; if selected later, route only this exact callsite and preserve ordinary argument inference fallback. |
| FP-15 | Branch/result merge expected/actual | `merge_branch_results`, `unify(&left.ty, &right.ty).unwrap_or_else(...)` (`check_expr.rs:2565-2567`) | Legacy unification; on mismatch silently falls back to a fresh meta result type. | Later semantic cleanup / TASK-826 only if explicitly selected | **Deferred for TASK-826.** This branch-merge seam has special fallback semantics and must not be replaced by definitional equality in Phase 112 without a dedicated task decision. |
| FP-16 | `with_error` handler expected/actual | `check_with_error`, `unify(&expected_ty, &arm_ty)` (`check_expr.rs:2611-2618`) | Legacy unification between body type and handler arm type; direct Display in mismatch diagnostic. | TASK-826 if selected / TASK-827 diagnostics | **Deferred/fallback.** Handler/body comparison seam. TASK-826 may touch only if the forcing-point rollout explicitly selects this exact callsite after preserving current error-handling semantics. |
| FP-17 | Associated projection resolution | `lower_associated_projection_to_canonical` result at `type_env.rs:2411-2417`; `normalize_associated_types` simple substitution at `type_env.rs:4423-4496` | Canonical projection lowering preserves `ProjectionRigidity`; selected-impl helper substitutes simple associated outputs only. | TASK-823/TASK-826 | **Partially owned.** TASK-823 normalizes projection argument spines and aliases without associated-family computation. TASK-826 may only force equality at callsites above; no recursive associated-family normalization. |
| FP-18 | Final inferred-type rendering | `TypeEnv::render_type_for_diagnostics` (`type_env.rs:2757-2758`) | Thin `ty.to_string()`. No callsites found in `crates/ash-typeck/src` besides definition. | TASK-826/TASK-827 | **Owned wrapper, currently unused.** TASK-826/TASK-827 may use this wrapper at selected diagnostics to render smallest normalized slice. Do not globally rewrite direct Display/to_string diagnostics. |

## 6. Rendering callsites TASK-826/TASK-827 may touch

### 6.1 Central wrapper

- `TypeEnv::render_type_for_diagnostics(&Type) -> String` (`type_env.rs:2757-2758`) currently returns `ty.to_string()` and has no live callers found under `crates/ash-typeck/src`.

### 6.2 Direct type rendering callsites relevant to the forcing matrix

These are exact direct Display/`to_string()` sites that report type mismatch/equality output and may be touched only if their forcing point is selected:

- `check_expr.rs:505-508`: if-branch mismatch renders `{then_ty}` and `{else_ty}`.
- `check_expr.rs:530-533`: if-without-else mismatch renders `{then_ty}`.
- `check_expr.rs:657-659`: `FnDef` return annotation conflict renders `{body_ty}` and the annotation spelling.
- `check_expr.rs:715-717`: `FnApply` type mismatch includes `{func_ty}` and `UnifyError` Display.
- `check_expr.rs:720-729`: `FnApply` non-function/too-many-args renders `{func_ty}`.
- `check_expr.rs:2375-2379`: capability-binding operation argument mismatch renders `{expected_ty}` and `{actual_ty}`.
- `check_expr.rs:2617`: `with_error` handler mismatch renders `{expected_ty}` and `{arm_ty}`.
- `check_expr.rs:2909-2910`: tuple constructor field mismatch uses `expected_ty.to_string()` and `field_result.ty.to_string()`.
- `check_expr.rs:2972-2973`: named constructor field mismatch uses `expected_ty.to_string()` and `field_result.ty.to_string()`.
- `type_env.rs:3185`: capability implementation parameter mismatch renders `{expected_param}` / `{actual_param}`.
- `type_env.rs:3197-3198`: capability implementation return mismatch renders `expected.return_type` / `operation_info.return_type`.
- `type_env.rs:3388-3392`: capability operation body return mismatch renders `operation_info.return_type` / `actual_return_ty`.
- `type_env.rs:3504`: duplicate impl diagnostic renders `impl_head.to_string()`.
- `type_env.rs:3678-3679`: impl method return mismatch renders `expected_return_ty` / `actual_return_ty`.
- `type_env.rs:4647`: missing impl diagnostic renders `target_head.to_string()`.

Many other `.to_string()` calls in `check_expr.rs` and other typeck files are name/string conversions, test assertions, capability diagnostics, or non-type rendering. They are out of the TASK-826 rendering boundary unless a later task names them.

## 7. Out-of-scope and deferred work

Explicitly out of scope for TASK-817 and Phase 112 unless later tasks say otherwise:

- public `type fn` source syntax;
- source equation parsing/lowering;
- public type-function/equation export or import through `ModuleSemanticSummary`;
- recursive associated type-family computation;
- proposition solving, disequality solving, injectivity, proof search, or type-function inversion;
- promoted data constructors or DataKinds-style runtime constructor promotion;
- generalized associated projection surface syntax beyond current compatibility boundaries;
- global replacement of every `unify(...)`, `unify_types(...)`, or direct `to_string()` diagnostic by search-and-replace.

## 8. Downstream ownership summary

- TASK-818: add core normal-form/domain-constructor carriers, including sealed-domain constructor normal forms backed by `DomainConstructorId`/`SealedDomainId`.
- TASK-819: add `ash-typeck` normalizer API skeleton and identity behavior.
- TASK-820: add internal fixture equation registry; no source/module-summary export.
- TASK-821/TASK-822: implement closed/open/partial fixture reduction.
- TASK-823: aliases plus neutral/rigid projection argument normalization; no associated-family computation.
- TASK-824: structured definitional equality API.
- TASK-825: non-inverting unification boundary and any top-level inference-meta bridge.
- TASK-826: adopt only FP-1 through FP-18 as marked above.
- TASK-827: diagnostics/rendering/non-interference around the same selected seams.

## 9. Audit checklist

- [x] Core type IR and semantic-summary carriers inspected.
- [x] TypeEnv lowering, canonicalization, equality, associated-output substitution, impl selection, and sealed-domain lookup seams inspected.
- [x] Exact forcing-point matrix recorded for TASK-826, including deferred direct expression-checking seams that must not be adopted accidentally.
- [x] Canonical abstract variables vs inference metas mapped.
- [x] Rendering wrapper and direct type-rendering diagnostics mapped.
- [x] Public type-function syntax/source equations/equation export marked out of scope.
- [x] No Rust code changed.
