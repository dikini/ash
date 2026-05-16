# TASK-904 HKT Audit Gate

Status: Complete
Date: 2026-05-16
Branch: phase-120-hkt
Phase: Phase 120 / PLAN-116
Spec: SPEC-067

## Decision

Phase 120 must implement constructor-kinded binders and higher-kinded interface evidence as shared type-system substrate. It must not be implemented as parser-only lowering or as a do-only special case.

The first implementation slice must add explicit constructor-kinded carriers before parser/typechecker behavior depends on them:

- a binder needs `name`, `kind`, source span, and any bounds;
- applying a constructor variable such as `M<A>` must remain a typed constructor-variable application, not a nominal type named `M`;
- TypeEnv evidence lookup must include kinded argument spines;
- generalized `do:K` may consume `Monad<K>` evidence only after `K` has been elaborated as a unary constructor expression.

Frozen non-goals from SPEC-067 remain active for every downstream task: no higher-rank polymorphism, no arbitrary source type lambdas, no automatic do-target inference, no Monad/Functor/Applicative law proving, no associated-type inversion during evidence search, and no multi-parameter constructor classes beyond the audited MVP.

## Live callsites inspected

| ID | Seam | Live files / public entrypoints inspected | Current state | Downstream owner |
|---|---|---|---|---|
| A1 | Core kind substrate | `crates/ash-core/src/kind.rs:13` `Kind`, `Kind::arrow`, `Kind::n_ary`, `Kind::arity` | DONE for representing `* -> *`; PARTIAL for binder use because no source/core binder carrier stores explicit kinded parameters. | TASK-905 |
| A2 | Core canonical constructor carriers | `crates/ash-core/src/type_ir.rs:143` `PartialTypeArg`, `type_ir.rs:159` `PartialTypeConstructorApp`, `type_ir.rs:220` `TypeConstructorExpr`, `type_ir.rs:356` `CanonicalTypeExpr` | PARTIAL. Partial nominal constructor applications exist for explicit holes; `CanonicalTypeExpr` has `Var`, `NominalApp`, `Projection`, `ComputationHeadApp`, and promoted apps. MISSING: constructor-variable identity/application carrier for `M<A>` and a kinded binder carrier shared by parser/typeck/summary code. | TASK-905 |
| A3 | Parser raw type expression surface | `crates/ash-parser/src/surface.rs:2020` `Type`, `crates/ash-parser/src/parse_module.rs:1789` `parse_surface_type`, `parse_module.rs:1853` `parse_surface_type_atom`, `parse_expr.rs:112` `parse_do_type` | PARTIAL. Parser can preserve `Type::Constructor { name, args }` and do-target holes. MISSING: kind grammar such as `* -> *`; current `F<A>` is syntactically indistinguishable from nominal `F<A>`. | TASK-906 |
| A4 | Parser interface/impl binders | `crates/ash-parser/src/surface.rs:789` `InterfaceTypeParam`, `parse_module.rs:1304` `parse_interface_definition`, `parse_module.rs:1431` `parse_impl_definition`, `parse_module.rs:1555` `parse_optional_interface_type_params` | PARTIAL. Interface/impl params have `domain: Option<Type>` from associated-family work, so `F: TypeList` style domains exist. MISSING: explicit kind annotation carrier and parsing for `F : * -> *`. | TASK-906 |
| A5 | Parser function/workflow/type-function/proposition binder surfaces | `surface.rs:553` `FnDef`, `surface.rs:578` `BuiltinFnDef`, `surface.rs:879` `TypeParam`, `parse_workflow.rs:611` `parse_type_params`, `surface.rs:257` `TypeFnParam`, `parse_module.rs:570` `parse_type_fn_param`, `surface.rs:246` `PropositionPredicateParam`, `parse_module.rs:520` `parse_proposition_predicate_param`, `parse_module.rs:385` `parse_proposition_tail` | MISSING for HKT. Function/workflow type params still carry names and interface bounds only; type-function and proposition params parse `name: Type`, not `name: Kind`; proposition clauses parse type expressions but cannot distinguish constructor variables. | TASK-906 |
| A6 | TypeEnv source-to-Type lowering | `crates/ash-typeck/src/types.rs:16` `Type`, `types.rs:53` `Type::Constructor`, `type_env.rs:266` `type_expr_to_type`, `type_env.rs:12245` `TypeEnv::lower_surface_type_to_canonical` | PARTIAL. Ordinary nominal/projection/computation canonical lowering exists. MISSING: constructor-variable environment, kinded variable tracking, and `M<A>` lowering that avoids nominal encoding. | TASK-907 |
| A7 | TypeEnv partial constructor elaboration | `type_env.rs:12396` `TypeEnv::elaborate_do_target_constructor_expr`, `type_env.rs:12405` `TypeEnv::elaborate_partial_type_constructor`, `type_env.rs:12552` `elaborate_constructor_application` | DONE for SPEC-066 explicit hole targets; PARTIAL for HKT because it handles registered nominal constructor heads and holes, not constructor-variable heads. | TASK-907 / TASK-909 |
| A8 | TypeEnv unification and kinding | `crates/ash-typeck/src/kind.rs:1` re-export, `type_env.rs:13034` `TypeEnv::unify_types`, `crates/ash-typeck/src/types.rs:631` `bind_var`, `types.rs:649` `occurs_in` | PARTIAL. Proper-type unification and definitional equality forcing exist. MISSING: separate constructor metas, constructor-variable application unification, and fail-closed wrong-kind diagnostics for applying proper type variables. | TASK-907 |
| A9 | Interface/impl registration and coherence | `type_env.rs:13178` `TypeEnv::register_interface`, `type_env.rs:14532` `TypeEnv::register_impl`, `type_env.rs:15671` `lookup_interface`, `type_env.rs:15732` `impl_schemes`, `type_env.rs:15823` `resolve_interface_method_call`, `type_env.rs:15843` `select_impl_scheme`, `type_env.rs:15923` `find_matching_impl_scheme` | PARTIAL. Proper-type interface impls, where-bounds, overlap checks, and proposition evidence assumptions exist. MISSING: kinded interface params, higher-kinded impl heads, evidence keys for constructor expressions, and ambiguity/overlap reporting for `Monad<Option>`-style evidence. | TASK-908 |
| A10 | Proposition/interface-bound evidence | `type_env.rs:12179` `record_type_var_interface_bound_assumption`, `type_env.rs:12198` `record_concrete_impl_interface_assumption`, `type_env.rs:11536` interface-bound solving path | PARTIAL. Interface-bound propositions use canonical terms and selected concrete impl assumptions. MISSING: constructor-kind subjects/arguments and kinded evidence spines. Must remain non-inverting. | TASK-908 |
| A11 | Do-target parser and resolution bridge | `crates/ash-parser/src/surface.rs:1417` `DoTarget`, `parse_expr.rs:31` `parse_do_block_expr`, `crates/ash-typeck/src/do_target.rs:41` `resolve_do_target`, `crates/ash-typeck/src/lib.rs:94` `resolve_do_target_for_test` | PARTIAL. `DoTarget` preserves target args; `resolve_do_target` validates explicit partial targets then reports missing Monad evidence, but actual dictionary selection is compiler-known Act/Proc/Workflow only. | TASK-909 |
| A12 | Typed do elaboration | `crates/ash-typeck/src/check_expr.rs:1168` `elaborate_typed_do_parts`, `check_expr.rs:1360` `elaborate_do_stmts`, `check_expr.rs:1929` `dictionary_call`, `check_expr.rs:2015` `check_do_block` | PARTIAL. Typed do uses `DoDictionary` with hidden Act operations or ordinary `proc::*`/`workflow::*` functions. MISSING: `Monad<K>` evidence lookup and method selection. Workflow artifact preservation is special-cased and must be retained. | TASK-909 |
| A13 | Engine semantic-summary import/export | `crates/ash-core/src/semantic_summary.rs:997` `InterfaceIdentitySummary`, `crates/ash-typeck/src/type_env.rs:5440` `register_module_semantic_summary`, `type_env.rs:10472` `register_interface_identity_summary`, `crates/ash-engine/src/module_loader.rs:3241` public proposition summary path, `module_loader.rs:3475` `collect_public_interface_identity_summaries`, `module_loader.rs:3668`/`4070`/`4305` canonical type walkers | PARTIAL. Interface identities and canonical type walkers are summary-aware, but interface summaries do not carry kinded parameter/evidence metadata. Any new `CanonicalTypeExpr` variant must update engine/typeck walkers fail-closed. | TASK-908 / TASK-910 |

## Phase 120 seam status

| Task | Seam classification | Current state | Required first proof |
|---|---|---|---|
| TASK-905 | Core kinded binders and constructor-variable carriers | PARTIAL | Core tests prove kinded binder metadata and `ConstructorVarApp`/adjacent carrier preserve `M<A>` without nominal lowering. |
| TASK-906 | Parser kinded-binder surface | MISSING | Parser tests prove `F : * -> *` is preserved at interfaces, impl params, functions/workflows, type functions, and proposition predicates; unsupported sites fail closed. |
| TASK-907 | TypeEnv constructor-variable kinding/unification | MISSING | TypeEnv tests prove constructor variables are tracked by kind, `M<A>` lowers by kind, proper type variables cannot be applied, and unification remains non-inverting. |
| TASK-908 | Higher-kinded interface/impl coherence | PARTIAL | TypeEnv tests prove `Functor<F>`/`Monad<Option>` registration, duplicate/overlap rejection, and no output-directed selection. |
| TASK-909 | Monad dictionary do-target resolution | PARTIAL | TypeEnv/checker tests prove `do:Option` consumes `Monad<Option>` evidence while Act/Proc/Workflow bridge behavior is preserved. No target inference. |
| TASK-910 | Diagnostics and acceptance matrix | MISSING | Parser/typeck/engine acceptance tests map SPEC-067 HKT-1 through HKT-8 and non-interference rows to concrete evidence. |

## Implementation order and blockers

1. TASK-905 must land first. Parser/typeck must not encode constructor variables as nominal `Type::Constructor` or plain `CanonicalTypeExpr::Var`.
2. TASK-906 depends on TASK-905 carriers so parsed kinded binders have a destination. The parser can preserve raw syntax first but must fail closed for arbitrary type lambdas and higher-rank forms.
3. TASK-907 depends on TASK-906 surfaces and owns semantic kinding, constructor-variable application, and non-inverting constructor unification.
4. TASK-908 depends on TASK-907 evidence keys. Existing `ImplScheme { head: Type }` is a blocker for constructor-expression heads unless it is extended or wrapped.
5. TASK-909 depends on TASK-908 ordinary evidence lookup. It may keep compiler-prelude Act/Proc/Workflow evidence during migration, but TypeEnv boundary behavior must be shaped as `Monad<K>`.
6. TASK-910 closes diagnostics and acceptance. It must update any summary/type walkers touched by TASK-905/TASK-908 and prove no private/evidence summary leakage.

Known blockers for TASK-905:

- No core `KindedTypeParam`/constructor-binder carrier exists.
- `CanonicalTypeExpr::Var(String)` is too weak to distinguish proper type variables from constructor variables.
- `TypeConstructorExpr` currently models nominal heads and partial applications, not constructor-variable heads.
- Engine/typeck walkers pattern-match all current `CanonicalTypeExpr` variants and must be updated whenever a new variant is introduced.

## Downstream focused verification commands

TASK-905 core carriers:

```bash
cargo test -p ash-core --test task_905_hkt_core_carriers
cargo test -p ash-typeck --test task_905_hkt_typeenv_fail_closed
cargo fmt --check
git diff --check
cargo check --workspace
```

TASK-906 parser surface:

```bash
cargo test -p ash-parser --test task_906_hkt_kinded_binder_surface
cargo test -p ash-parser --test task_906_hkt_non_interference
cargo fmt --check
git diff --check
cargo check --workspace
```

TASK-907 TypeEnv constructor-variable kinding and unification:

```bash
cargo test -p ash-typeck --test task_907_constructor_variable_kinding
cargo test -p ash-typeck --test task_907_constructor_unification_non_inverting
cargo fmt --check
git diff --check
cargo check --workspace
```

TASK-908 higher-kinded interface and impl coherence:

```bash
cargo test -p ash-typeck --test task_908_hkt_interface_impl_coherence
cargo test -p ash-typeck --test task_908_hkt_evidence_lookup
cargo test -p ash-engine --test task_908_hkt_summary_non_interference
cargo fmt --check
git diff --check
cargo check --workspace
```

TASK-909 Monad dictionary do-target resolution:

```bash
cargo test -p ash-typeck --test task_909_monad_do_target_resolution
cargo test -p ash-typeck --test task_909_act_proc_workflow_bridge_non_interference
cargo fmt --check
git diff --check
cargo check --workspace
```

TASK-910 diagnostics and acceptance matrix:

```bash
cargo test -p ash-parser --test task_910_hkt_diagnostics_surface
cargo test -p ash-typeck --test task_910_hkt_acceptance_matrix
cargo test -p ash-engine --test task_910_hkt_summary_non_interference
cargo fmt --check
git diff --check
cargo check --workspace
```

## Fail-closed expectations

- Reject kinded binders on surfaces not explicitly enabled by TASK-906.
- Reject applying a proper type variable as a constructor.
- Reject wrong-arity or wrong-kind constructor-variable applications before evidence lookup.
- Reject overlapping higher-kinded impl evidence unless a future specialization/coherence spec changes the rule.
- Reject `do:K` without `Monad<K>` evidence after target shape elaboration.
- Keep Act/Proc/Workflow bridge effects and workflow artifact preservation explicit during migration.
- Keep `Result<_, E>` dependent on SPEC-066 partial-constructor support; do not infer partial targets.
- Do not prove or assume Monad/Functor/Applicative laws.
- Do not invert associated types/families or type functions to discover evidence.
- Do not add arbitrary type lambdas, higher-rank polymorphism, or broad multi-parameter constructor classes.

## TASK-904 verification

Required gate commands:

```bash
cargo fmt --check
test -f docs/plan/audits/TASK-904-hkt-audit-gate.md
git diff --check
! rg -n 'false # TASK-90[5-9]|false # TASK-910|placeholder|PLACEHOLDER|TODO replace|must replace this guard' docs/plan/tasks/TASK-90{5,6,7,8,9}-*.md docs/plan/tasks/TASK-910-*.md
```

This task does not implement Rust production behavior for TASK-905 through TASK-910.
