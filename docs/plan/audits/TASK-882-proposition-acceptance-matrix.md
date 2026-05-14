# TASK-882 SPEC-H acceptance/non-interference matrix

Status: implemented evidence artifact for SPEC-064 §12 H1-H12.
Scope: Phase 116 SPEC-064 acceptance rows plus TASK-872 named non-interference suites. This artifact adds evidence only; it does not broaden parser/typeck/engine proposition semantics.

## Focused TASK-882 aggregator commands

| Command | Test count | Expected result | Actual result |
| --- | ---: | --- | --- |
| `cargo test -p ash-core --test task_882_spec_h_summary_non_interference` | 2 | pass, non-zero | pass: 2 passed, 0 failed |
| `cargo test -p ash-parser --test task_882_spec_h_surface_non_interference` | 3 | pass, non-zero | pass: 3 passed, 0 failed |
| `cargo test -p ash-typeck --test task_882_spec_h_acceptance_matrix` | 5 | pass, non-zero | pass: 5 passed, 0 failed |
| `cargo test -p ash-engine --test task_882_spec_h_transport_non_interference` | 2 | pass, non-zero | pass: 2 passed, 0 failed |

Zero-test guard evidence: `-- --list` was run for all four TASK-882 suites and listed 12 `task_882_` tests total before execution.

## SPEC-064 §12 H1-H12 acceptance matrix

| ID | Requirement | Expected outcome | TASK-882 focused evidence | Earlier owning task evidence | Command / count / actual |
| --- | --- | --- | --- | --- | --- |
| H1 | `Cons<A, T> != Nil` over a sealed `TypeList` domain | Satisfied by sealed-domain constructor-head disjointness, even with open constructor arguments | `crates/ash-typeck/tests/task_882_spec_h_acceptance_matrix.rs::task_882_h1_constructor_disequality_and_h2_open_append_equality_are_conservative` asserts `PropositionEvidenceRule::SealedDomainConstructorDisjointness` and local evidence. | TASK-876 (`task_876_disequality_satisfied_for_sealed_domain_constructor_head_disjointness_with_open_args`) | `cargo test -p ash-typeck --test task_882_spec_h_acceptance_matrix`: 5 tests, pass |
| H2 | `Append<Xs, Ys> == Cons<A, Nil>` with open `Xs`/`Ys` | Deferred/blocked with no substitution for `Xs` or `Ys` | Same TASK-882 typeck test asserts deferred `BlockedByNeutrality` with `no_inversion_boundary`. | TASK-876 (`task_876_equality_deferred_at_neutral_no_inversion_boundary_without_solving_inputs`), TASK-825 no-inversion regression | `cargo test -p ash-typeck --test task_882_spec_h_acceptance_matrix`: 5 tests, pass |
| H3 | Unsupported named predicate in a proposition list | Explicit deferred-feature diagnostic | Typeck: `task_882_h3_named_predicate_defers_and_h11_private_predicate_leak_rejects` asserts registered ordinary named predicates defer with `UnsupportedNamedPredicate`. Parser: `task_882_parser_h3_h7_surface_clauses_are_raw_and_runtime_contracts_stay_separate` proves named predicate surface remains raw. | TASK-878 named predicate registration/deferred solving; TASK-881 diagnostic family | `cargo test -p ash-typeck --test task_882_spec_h_acceptance_matrix`: 5 tests, pass; `cargo test -p ash-parser --test task_882_spec_h_surface_non_interference`: 3 tests, pass |
| H4 | Equality after direct type-function normalization | Satisfied without legacy unification fallback | `task_882_h4_direct_type_fn_normalization_satisfies_without_unification_fallback` normalizes source-backed `Append<Nil, Ys>` through `Normalizer::normalize_known_computation_app` and checks definitional equality is `Equal`. | TASK-838 source type-function normalizer; TASK-840 SPEC-061 acceptance; TASK-876 equality proposition wrapper | `cargo test -p ash-typeck --test task_882_spec_h_acceptance_matrix`: 5 tests, pass |
| H5 | Associated-family projection equality from SPEC-063 | Satisfied when unique family reduction applies | `task_882_h5_associated_family_equality_satisfies_and_h6_rigid_projection_defers` solves `<Iterator<List<String>>>::Item == String` via definitional equality evidence. | TASK-866 associated-family normalizer; TASK-870 public lowering; TASK-876 proposition equality | `cargo test -p ash-typeck --test task_882_spec_h_acceptance_matrix`: 5 tests, pass |
| H6 | Rigid `T::Item` equality under only `T: Iterator` | Deferred on rigid projection, not solved | Same TASK-882 typeck test asserts `RigidAssociatedProjection` with `no_inversion_boundary`. | TASK-864 rigid where-bound projection; TASK-876 rigid proposition equality | `cargo test -p ash-typeck --test task_882_spec_h_acceptance_matrix`: 5 tests, pass |
| H7 | Interface bound proposition for known impl | Satisfied by existing impl/bound evidence | `task_882_h7_known_interface_bound_satisfies_and_h8_missing_bound_defers_without_search` asserts concrete impl evidence satisfies `Int: Displayable`; parser aggregator also proves raw `T: Debug` proposition tails parse separately from runtime contracts. | TASK-877 interface-bound proposition solving | `cargo test -p ash-typeck --test task_882_spec_h_acceptance_matrix`: 5 tests, pass; `cargo test -p ash-parser --test task_882_spec_h_surface_non_interference`: 3 tests, pass |
| H8 | Interface bound proposition with no evidence | Refuted or checking-error diagnostic, not search | Same TASK-882 typeck test asserts missing `String: Displayable` evidence defers with `MissingInterfaceEvidence` and no inversion/search boundary. | TASK-877 missing/non-exact interface evidence; TASK-880 required-checking integration | `cargo test -p ash-typeck --test task_882_spec_h_acceptance_matrix`: 5 tests, pass |
| H9 | V5 summary with proposition requirements | Imported/revalidated or explicitly deferred | Core: `task_882_h9_v5_summary_preserves_public_proposition_requirements_without_touching_legacy_payloads` validates V5 schema. Engine: `task_882_engine_h9_transports_v5_proposition_requirements_without_engine_solving` transports a satisfied public equality requirement as V5 and checks engine does not solve it. | TASK-873 V5 schema; TASK-879 summary transport/import; TASK-880 public checking integration | `cargo test -p ash-core --test task_882_spec_h_summary_non_interference`: 2 tests, pass; `cargo test -p ash-engine --test task_882_spec_h_transport_non_interference`: 2 tests, pass |
| H10 | V4 summary carrying proposition facts | Rejected as malformed before partial registration | `task_882_h10_v4_and_older_summaries_reject_proposition_facts_before_legacy_registration` checks V1/V2/V3/V4 fail with `PropositionFactsRequireV5`. | TASK-873 schema contract; TASK-879 typeck V4 rejection (`task_879_v4_summary_with_proposition_fact_rejects_before_registering_predicate`) | `cargo test -p ash-core --test task_882_spec_h_summary_non_interference`: 2 tests, pass |
| H11 | Private predicate/helper leakage in public proposition summary | Rejected with private-dependency diagnostic | `task_882_h3_named_predicate_defers_and_h11_private_predicate_leak_rejects` imports a V5 public proposition summary with a private predicate dependency and asserts private diagnostic plus no partial obligations. | TASK-879 private predicate/domain/type/projection/interface leak checks; TASK-881 diagnostics | `cargo test -p ash-typeck --test task_882_spec_h_acceptance_matrix`: 5 tests, pass |
| H12 | Existing SPEC-035/SPEC-063 associated-type behavior | Non-interference: unchanged focused regressions | Parser: `task_882_parser_h12_legacy_impl_where_bounds_are_not_generalized_to_propositions` and `task_882_parser_h12_capability_and_workflow_where_syntax_do_not_enter_type_propositions`. Engine: `task_882_engine_h12_pub_use_and_glob_transport_do_not_duplicate_or_interpret_proposition_facts`. TASK-872 regression suites were also run. | SPEC-035/TASK-862 compatibility, SPEC-057..063 suites named in TASK-872 | TASK-882 parser: 3 tests, pass; TASK-882 engine: 2 tests, pass; non-interference suites below: 109 tests, pass |

## TASK-872 non-interference suites run

### ash-typeck SPEC-035/SPEC-057 through SPEC-063

Command:

```sh
cargo test -p ash-typeck \
  --test task_864_rigid_where_bound_projection \
  --test task_866_associated_family_normalizer \
  --test task_867_associated_family_import \
  --test task_870_associated_family_public_lowering \
  --test task_824_definitional_equality \
  --test task_825_non_inverting_unification_boundary \
  --test task_827_normalizer_diagnostics \
  --test task_787_semantic_summary_typeenv \
  --test task_798_canonical_lowering_typeenv_registry_red \
  --test task_812_domain_registration_validation \
  --test task_840_type_function_acceptance \
  --test task_854_type_computation_summary_acceptance
```

Expected result: pass, non-zero.
Actual result: pass; 101 tests passed, 0 failed.

Breakdown:

| Suite | SPEC row coverage | Count | Actual |
| --- | --- | ---: | --- |
| `task_787_semantic_summary_typeenv` | SPEC-057 ordinary summaries | 24 | pass |
| `task_798_canonical_lowering_typeenv_registry_red` | SPEC-058 canonical projection/kind/arity boundaries | 8 | pass |
| `task_812_domain_registration_validation` | SPEC-059 sealed-domain registration | 9 | pass |
| `task_824_definitional_equality` | SPEC-060 definitional equality | 7 | pass |
| `task_825_non_inverting_unification_boundary` | SPEC-060 no-inversion boundary | 4 | pass |
| `task_827_normalizer_diagnostics` | SPEC-060 diagnostics and non-interference | 12 | pass |
| `task_840_type_function_acceptance` | SPEC-061 direct structural type functions | 7 | pass |
| `task_854_type_computation_summary_acceptance` | SPEC-062 public type-function summaries | 2 | pass |
| `task_864_rigid_where_bound_projection` | SPEC-063 rigid where-bound behavior / SPEC-035 compatibility boundary | 8 | pass |
| `task_866_associated_family_normalizer` | SPEC-063 family reduction | 8 | pass |
| `task_867_associated_family_import` | SPEC-063 V4 import/export | 9 | pass |
| `task_870_associated_family_public_lowering` | SPEC-063 public lowering / SPEC-035 spelling compatibility | 3 | pass |

### ash-engine SPEC-062/SPEC-063 transport

Command:

```sh
cargo test -p ash-engine \
  --test task_867_associated_family_summary_transport \
  --test task_870_associated_family_public_lowering \
  --test task_854_type_computation_summary_acceptance
```

Expected result: pass, non-zero.
Actual result: pass; 8 tests passed, 0 failed.

Breakdown:

| Suite | SPEC row coverage | Count | Actual |
| --- | --- | ---: | --- |
| `task_854_type_computation_summary_acceptance` | SPEC-062 selected/glob/pub-use type-function summary transport | 3 | pass |
| `task_867_associated_family_summary_transport` | SPEC-063 associated-family summary transport | 4 | pass |
| `task_870_associated_family_public_lowering` | SPEC-063 public family projection loading | 1 | pass |

## Notes and residual boundaries

- Unsupported ordinary named predicates remain deferred, not proved. Public checking points reject unsupported deferred requirements; TASK-882 engine transport therefore uses a satisfied public equality requirement for non-interference instead of an unsupported public named predicate.
- TASK-882 did not implement new solver features and did not broaden enabled parser surfaces. It only adds matrix evidence and focused aggregator smoke/regression tests that cite or reuse TASK-873 through TASK-881 behavior.
