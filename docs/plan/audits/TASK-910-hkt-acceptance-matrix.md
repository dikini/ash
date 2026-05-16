# TASK-910 HKT Acceptance and Non-Interference Matrix

Status: Complete
Date: 2026-05-16
Branch: phase-120-hkt
Phase: Phase 120 / PLAN-116
Spec: SPEC-067

## Scope

TASK-910 closes the SPEC-067 diagnostics and acceptance evidence surface for the Functor/Applicative/Monad HKT slice. It does not add higher-rank polymorphism, arbitrary source type lambdas, automatic do-target inference, law proving, associated-type/type-function inversion, or broad multi-parameter constructor classes.

## SPEC-067 acceptance rows

| ID | SPEC-067 case | Evidence | Status |
|---|---|---|---|
| HKT-1 | Parse `interface Functor<F : * -> *>` with binder kind preserved | `crates/ash-parser/tests/task_910_hkt_diagnostics_surface.rs::hkt1_parses_functor_applicative_and_monad_constructor_binders` | Covered |
| HKT-2 | Typecheck `F<A>` in an interface method signature | `crates/ash-typeck/tests/task_910_hkt_acceptance_matrix.rs::hkt2_interface_method_signature_accepts_constructor_application` | Covered |
| HKT-3 | `impl Monad<Option>` registers evidence when method shape is accepted | `crates/ash-typeck/tests/task_910_hkt_acceptance_matrix.rs::hkt3_impl_monad_option_registers_empty_method_mvp_evidence` | Covered for empty-method MVP evidence |
| HKT-4 | `impl Monad<Result<_, E>>` requires SPEC-066 partial target support | `crates/ash-parser/tests/task_910_hkt_diagnostics_surface.rs::hkt4_impl_head_preserves_partial_constructor_hole_surface`; `crates/ash-typeck/tests/task_910_hkt_acceptance_matrix.rs::hkt4_result_partial_impl_head_is_registered_only_as_shape_evidence` | Covered as shape evidence only; no generalized runtime method lowering claimed |
| HKT-5 | `M` used where `M<A>` is required reports wrong kind | `crates/ash-typeck/tests/task_910_hkt_acceptance_matrix.rs::hkt5_bare_constructor_variable_in_proper_type_position_is_wrong_kind` | Covered |
| HKT-6 | Overlapping `Monad<Option>` impls are rejected | `crates/ash-typeck/tests/task_910_hkt_acceptance_matrix.rs::hkt6_duplicate_monad_option_impls_are_rejected_as_overlap` | Covered |
| HKT-7 | `do:Option` after evidence uses Monad evidence at the typed boundary | `crates/ash-typeck/tests/task_910_hkt_acceptance_matrix.rs::hkt7_do_option_uses_registered_monad_evidence_at_type_boundary` | Covered for target resolution and return-only type boundary; law/runtime method semantics deferred |
| HKT-8 | `do:List` without evidence reports missing `Monad<List>` evidence | `crates/ash-typeck/tests/task_910_hkt_acceptance_matrix.rs::hkt8_do_list_without_monad_evidence_reports_missing_evidence` | Covered |

## Diagnostic rows

| Diagnostic / boundary | Evidence | Status |
|---|---|---|
| Kinded binder syntax unsupported at non-enabled sites | `crates/ash-parser/tests/task_910_hkt_diagnostics_surface.rs::kinded_binder_syntax_stays_rejected_at_non_enabled_type_alias_site` | Covered |
| Impl-head `_` hole parsing does not broaden ordinary surface type positions | `crates/ash-parser/tests/task_910_hkt_diagnostics_surface.rs::hkt_holes_stay_rejected_in_ordinary_function_type_positions`; `::hkt_holes_stay_rejected_in_ordinary_interface_method_type_positions`; `::hkt_holes_stay_rejected_in_ordinary_proposition_type_positions`; `::hkt_holes_stay_rejected_in_ordinary_alias_resource_and_capability_type_positions`; `::hkt_holes_stay_rejected_in_associated_type_bindings_inside_impls`; `::underscore_prefixed_type_names_are_not_treated_as_holes` | Covered |
| Malformed kinded binder syntax stays a parser diagnostic | `crates/ash-parser/tests/task_910_hkt_diagnostics_surface.rs::malformed_kinded_binders_remain_parser_diagnostics` | Covered |
| Applying a proper type variable as a constructor | `crates/ash-typeck/tests/task_910_hkt_acceptance_matrix.rs::applying_proper_type_variable_as_constructor_is_rejected` | Covered |
| Constructor variable applied to wrong number/kind of arguments | `crates/ash-typeck/tests/task_910_hkt_acceptance_matrix.rs::constructor_variable_wrong_argument_count_is_rejected_before_evidence_lookup`; HKT-5 row above | Covered |
| Impl head wrong kind for an interface parameter | `crates/ash-typeck/tests/task_910_hkt_acceptance_matrix.rs::impl_head_wrong_kind_for_interface_parameter_is_rejected` | Covered |
| Missing `Monad<K>` evidence for do target | HKT-8 row above | Covered |
| Ambiguous/overlapping higher-kinded impl evidence | HKT-6 row above; prior non-inverting evidence lookup remains covered by `crates/ash-typeck/tests/task_908_hkt_evidence_lookup.rs` | Covered |
| Attempted law proof or automatic law assumption beyond type-shape checking | HKT-3, HKT-4, and HKT-7 assertions explicitly cover shape/evidence/type-boundary behavior only; no source syntax or TypeEnv API records law proofs | Mapped as non-goal; deferred to future spec if laws are introduced |

## Non-interference and non-goals

| Non-goal / boundary | Evidence | Status |
|---|---|---|
| Summary transport preserves public HKT interface information | `crates/ash-engine/tests/task_910_hkt_summary_non_interference.rs::summary_transport_preserves_public_hkt_interfaces_without_private_or_evidence_leakage` | Covered |
| Private interfaces/evidence do not leak through summaries | same TASK-910 engine test | Covered |
| Impl evidence does not create duplicate public interface summaries | same TASK-910 engine test | Covered |
| No higher-rank polymorphism | No parser/typechecker surface added in TASK-910; SPEC-067 non-goal remains active | Deferred/non-goal |
| No arbitrary source type lambdas | No parser/typechecker surface added in TASK-910; HKT-4 uses explicit SPEC-066 holes only | Deferred/non-goal |
| No automatic do-target inference | HKT-7 and HKT-8 use explicit `do:Option`/`do:List`; no inference path is introduced | Covered by scope |
| No Monad/Functor/Applicative law proving or automatic law assumption | HKT-3/HKT-4/HKT-7 record evidence and target boundaries only; no law carrier or solver exists | Deferred/non-goal |
| No associated-type/type-function inversion during evidence search | Prior `task_908_hkt_evidence_lookup` tests remain the evidence for non-inverting method/evidence lookup | Covered by prior focused regression |
| No broad multi-parameter constructor classes | TASK-910 uses unary constructor-kinded parameters only (`* -> *`) | Covered by scope |

## Limitations deferred beyond TASK-910

- Generalized runtime lowering through user-defined Monad `return`/`bind` method bodies remains out of this row; TASK-909 and TASK-910 cover target resolution and return-only type boundaries.
- Generic partial evidence such as `Monad<Result<_, E>>` is accepted as SPEC-066-shaped impl-head evidence, but TASK-910 does not claim output-directed selection or associated/type-function inversion for evidence lookup.
- Any law syntax, law checker, or automatic law assumption requires a future spec.
