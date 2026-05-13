# TASK-868 Associated-family acceptance and non-interference matrix

Date: 2026-05-13
Scope: SPEC-063 §13 acceptance matrix plus §12 diagnostic routing evidence.

This artifact records exact focused tests/evidence for every SPEC-063 §13 row. Counts are from `cargo test -- --list` commands run in `/home/dikini/Projects/ash/.worktrees/phase-115-associated-family-computation`; every cited target has a non-zero test count.

## Focused target counts

| Package | Test target | Count |
| --- | --- | ---: |
| ash-parser | task_859_associated_family_surface | 10 tests |
| ash-core | task_860_associated_family_carriers | 6 tests |
| ash-typeck | task_861_associated_family_registration | 8 tests |
| ash-typeck | task_862_spec035_associated_compat | 8 tests |
| ash-typeck | task_863_associated_family_selection | 10 tests |
| ash-typeck | task_864_rigid_where_bound_projection | 8 tests |
| ash-typeck | task_865_recursive_associated_family | 6 tests |
| ash-typeck | task_866_associated_family_normalizer | 8 tests |
| ash-typeck | task_867_associated_family_import | 9 tests |
| ash-core | task_867_associated_family_summary | 3 tests |
| ash-engine | task_867_associated_family_summary_transport | 4 tests |
| ash-typeck | task_868_associated_family_diagnostics | 7 tests |
| ash-typeck | task_870_associated_family_public_lowering | 3 tests |
| ash-engine | task_870_associated_family_public_lowering | 1 test |

Commands used for counts:

```text
cargo test -p ash-parser --test task_859_associated_family_surface -- --list
cargo test -p ash-core --test task_860_associated_family_carriers -- --list
cargo test -p ash-typeck --test task_861_associated_family_registration -- --list
cargo test -p ash-typeck --test task_862_spec035_associated_compat -- --list
cargo test -p ash-typeck --test task_863_associated_family_selection -- --list
cargo test -p ash-typeck --test task_864_rigid_where_bound_projection -- --list
cargo test -p ash-typeck --test task_865_recursive_associated_family -- --list
cargo test -p ash-typeck --test task_866_associated_family_normalizer -- --list
cargo test -p ash-typeck --test task_867_associated_family_import -- --list
cargo test -p ash-core --test task_867_associated_family_summary -- --list
cargo test -p ash-engine --test task_867_associated_family_summary_transport -- --list
cargo test -p ash-typeck --test task_868_associated_family_diagnostics -- --list
cargo test -p ash-typeck --test task_870_associated_family_public_lowering -- --list
cargo test -p ash-engine --test task_870_associated_family_public_lowering -- --list
```

## SPEC-063 §13 acceptance matrix

| §13 row | Acceptance requirement | Exact evidence | Target count |
| ---: | --- | --- | ---: |
| 1 | `<Iterator<List<A>>>::Item` reduces to `A` through a unique generic impl. | `crates/ash-typeck/tests/task_863_associated_family_selection.rs::task_863_iterator_list_item_reduces_concrete_list_spine_to_element`; `crates/ash-typeck/tests/task_866_associated_family_normalizer.rs::task_866_reduces_local_iterator_list_item_projection` | 10, 8 |
| 2 | `<Iterator<List<X>>>::Item` reduces to `X` even when `X` is abstract. | `task_863_iterator_list_item_reduces_abstract_element_without_solving_query_var` | 10 |
| 3 | `T::Item` under only `T: Iterator` remains rigid in generic code. | `crates/ash-typeck/tests/task_864_rigid_where_bound_projection.rs::task_864_in_bounds_type_projection_lowers_to_rigid_canonical_projection`; `task_864_where_bound_projection_normalizes_as_rigid_projection`; TASK-868 blocker route `task_868_blocker_reasons_carry_non_fatal_associated_family_projection_notes` | 8, 7 |
| 4 | Existing SPEC-035 selected concrete impl substitution continues for non-family associated types. | `crates/ash-typeck/tests/task_862_spec035_associated_compat.rs::task_862_spec035_selected_impl_substitution_survives_with_family_table_present`; TASK-868 non-leakage `task_868_negative_leakage_boundaries_keep_prior_specs_non_regressed` | 8, 7 |
| 5 | Existing SPEC-035 compatibility projection spelling elaborates to the same canonical family projection as explicit syntax when unambiguous, including abstract arguments. | `crates/ash-typeck/tests/task_870_associated_family_public_lowering.rs::task_870_compat_and_explicit_iterator_list_x_item_are_canonical_equivalent_and_reduce`; `task_862_explicit_family_projection_lowers_to_canonical_projection_identity`; parser compatibility `crates/ash-parser/tests/task_859_associated_family_surface.rs::task_859_associated_family_keeps_spec035_compat_projection_syntax` | 3, 8, 10 |
| 6 | Ambiguous family impls are rejected before normalizer registration, or at a forcing point with a precise ambiguity diagnostic if malformed import reaches boundary. | `crates/ash-typeck/tests/task_861_associated_family_registration.rs::task_861_validates_family_overlap_result_kind_and_result_domain_before_publication`; `crates/ash-typeck/tests/task_867_associated_family_import.rs::task_867_import_rejects_result_domain_mismatch_overlap_and_unknown_dependency_transactionally`; TASK-868 diagnostic carriers `task_868_structured_type_env_diagnostics_preserve_codes_spans_and_family_identity` and blocker route `task_868_blocker_reasons_carry_non_fatal_associated_family_projection_notes` | 8, 9, 7 |
| 7 | Recursive `Append`-style computation passes only when sealed, exhaustive, coherent, and structurally decreasing. | `crates/ash-typeck/tests/task_865_recursive_associated_family.rs::task_865_accepts_append_like_recursive_associated_family`; `crates/ash-typeck/tests/task_866_associated_family_normalizer.rs::task_866_reduces_recursive_append_family_with_fuel` | 6, 8 |
| 8 | Non-decreasing recursive family equations are rejected and never registered. | `task_865_rejects_same_rebuilt_and_computed_recursive_arguments`; `task_865_rejects_missing_decreases_on_recursive_family`; `task_865_rejects_nonsealed_and_nonstructural_decreases_parameters`; TASK-868 fallback diagnostic routes for missing/invalid/not-decreasing decreases in `task_868_generic_registration_diagnostics_cover_remaining_spec063_families_without_new_variants` | 6, 7 |
| 9 | Public family summaries reduce downstream through V4 semantic summaries, independent of import order. | `crates/ash-typeck/tests/task_867_associated_family_import.rs::task_867_validated_v4_import_declares_family_and_reduces_downstream`; `task_867_batch_import_is_order_stable_for_associated_family_dependencies`; engine transport `crates/ash-engine/tests/task_867_associated_family_summary_transport.rs::task_867_glob_import_transports_public_associated_family_summary` | 9, 4 |
| 10 | Private/unavailable associated-family reduction boundaries do not silently reduce downstream; the public MVP evidence covers the unavailable-reduction blocker route and related export-closure diagnostics, not a full private-equation producer path. | Core blocker route `NormalFormBlockReason::AssociatedFamilyLocalUnavailable` asserted in `crates/ash-typeck/tests/task_868_associated_family_diagnostics.rs::task_868_blocker_reasons_carry_non_fatal_associated_family_projection_notes`; append comparison observes the same local-unavailable blocker in `task_868_append_output_comparison_is_associated_family_specific_non_inverting_evidence`; summary closure diagnostics cover private dependency/export-token routes in `task_868_summary_export_import_diagnostics_preserve_version_visibility_and_closure_tokens` | 7 |
| 11 | Where-bound evidence and family selection remain separate; adding a bound must not make a projection reduce unless a sealed scheme selects uniquely. | `crates/ash-typeck/tests/task_864_rigid_where_bound_projection.rs::task_864_where_bound_evidence_does_not_select_family_scheme`; `task_864_rigid_projection_equality_does_not_collapse_to_concrete_type`; TASK-868 rigid blocker route | 8, 7 |
| 12 | `<Append<Xs, Ys>>::Out == Cons<A, Nil>` does not solve `Xs` or `Ys`; remains non-inverting evidence. | `crates/ash-typeck/tests/task_868_associated_family_diagnostics.rs::task_868_append_output_comparison_is_associated_family_specific_non_inverting_evidence` asserts preserved `Xs`/`Ys`, non-inversion note, and blocked projection; related earlier evidence `task_863_expected_output_shape_is_not_used_to_select_open_input` | 7, 10 |
| 13 | SPEC-057 summaries, SPEC-058 projection identity, SPEC-060 non-inversion, SPEC-061 direct `type fn`, and SPEC-062 public type-function summaries remain non-regressed. | TASK-868 negative leakage test `task_868_negative_leakage_boundaries_keep_prior_specs_non_regressed`; SPEC-058 core carrier identity in `crates/ash-core/tests/task_860_associated_family_carriers.rs`; SPEC-062/version summary evidence in `crates/ash-core/tests/task_867_associated_family_summary.rs::task_867_v1_v2_v3_reject_non_empty_associated_family_facts` | 7, 6, 3 |
| 14 | Family selection binds only scheme-owned variables; queried projection variables/metas remain opaque and are never solved by expected output shape. | `crates/ash-typeck/tests/task_863_associated_family_selection.rs::task_863_unique_selection_returns_scheme_evidence_and_bindings`; `task_863_open_query_variable_does_not_select_by_inversion`; `task_863_neutral_query_head_is_not_captured_by_scheme_variable`; `task_863_rigid_projection_query_arg_is_not_captured_by_scheme_variable`; TASK-868 append non-inversion evidence | 10, 7 |
| 15 | Public summary export rejects private/incomplete family equation/dependency closures instead of exporting a partial reducible table. | `crates/ash-typeck/tests/task_867_associated_family_import.rs::task_867_import_rejects_omitted_associated_family_dependency_closure`; TASK-868 summary diagnostics `task_868_summary_export_import_diagnostics_preserve_version_visibility_and_closure_tokens` (`PrivateDependencyExportFailure`, closure conflict tokens) | 9, 7 |
| 16 | V4 import rejects malformed decreases metadata, result-domain mismatches, selected-scheme ambiguity, and dependency-closure conflicts before registration. | `task_867_import_rejects_malformed_metadata_before_registration`; `task_867_import_rejects_result_domain_mismatch_overlap_and_unknown_dependency_transactionally`; `task_867_import_rejects_omitted_associated_family_dependency_closure`; TASK-868 summary malformed route asserts decreases/domain/ambiguity/closure tokens | 9, 7 |
| 17 | `sealed type family Name` without `: ResultDomain` is rejected or diagnosed by mandatory-result-domain MVP rule. | `crates/ash-parser/tests/task_859_associated_family_surface.rs::task_859_associated_family_missing_result_domain_is_rejected`; TASK-868 syntax unsupported route keeps MVP diagnostic tokens in `task_868_generic_registration_diagnostics_cover_remaining_spec063_families_without_new_variants` | 10, 7 |

## SPEC-063 §12 diagnostic-family route audit

TASK-868 did not add broad production variants. Existing public `TypeEnvError` variants and `NormalFormBlockReason` values already route the families below. The focused test `crates/ash-typeck/tests/task_868_associated_family_diagnostics.rs` asserts stable error identity, LSP code, span, severity, message tokens, structured fields where variants expose them, and non-fatal blocker identity.

| §12 diagnostic family | Public route asserted by TASK-868 |
| --- | --- |
| AssociatedFamilySyntaxUnsupported | `TypeEnvError::InvalidDefinition` with associated-family MVP tokens (`E122`) |
| AssociatedFamilyNotSealed | `NormalFormBlockReason::AssociatedFamilyNotSealed` |
| AssociatedFamilyAmbiguousMember | `TypeEnvError::AmbiguousAssociatedType` (`E132`) |
| AssociatedFamilyImplNotInSealedSet | `TypeEnvError::UnauthorizedAssociatedFamilyExtension` (`E161`) |
| AssociatedFamilyMissingBinding | `TypeEnvError::MissingAssociatedFamilyBinding` (`E137`) |
| AssociatedFamilyExtraBinding | `TypeEnvError::ExtraAssociatedFamilyBinding` (`E138`) |
| AssociatedFamilyOverlap | `TypeEnvError::OverlappingAssociatedFamilyScheme` (`E163`) |
| AssociatedFamilyUnreachableRow | `TypeEnvError::InvalidDefinition` associated-family row diagnostic (`E122`) |
| AssociatedFamilyNonExhaustive | `TypeEnvError::InvalidDefinition` associated-family coverage diagnostic (`E122`) |
| AssociatedFamilyMissingDecreases | `TypeEnvError::InvalidDefinition` associated-family decreases diagnostic (`E122`) |
| AssociatedFamilyInvalidDecreases | `TypeEnvError::InvalidDefinition` associated-family decreases diagnostic (`E122`) |
| AssociatedFamilyNotDecreasing | `TypeEnvError::InvalidDefinition` associated-family non-decreasing diagnostic (`E122`) |
| AssociatedFamilyResultKindMismatch | `TypeEnvError::WrongAssociatedFamilyResultKind` (`E164`) |
| AssociatedFamilyResultDomainMismatch | `TypeEnvError::WrongAssociatedFamilyResultDomain` (`E165`) |
| AssociatedFamilyMutualRecursionUnsupported | `TypeEnvError::InvalidDefinition` associated-family mutual-recursion diagnostic (`E122`) |
| AssociatedFamilySelectionAmbiguous | `NormalFormBlockReason::AmbiguousAssociatedFamilySelection` |
| AssociatedFamilyRigidProjection | `NormalFormBlockReason::RigidProjection` |
| AssociatedFamilyPrivateReductionUnavailable | `NormalFormBlockReason::AssociatedFamilyLocalUnavailable` |
| AssociatedFamilyExportPrivateDependency | `TypeEnvError::PrivateDependencyExportFailure` (`E135`) |
| AssociatedFamilyExportNotClosed | `TypeEnvError::MalformedImportedComputationSummary` / `PrivateDependencyExportFailure` closure tokens (`E134`/`E135`) |
| AssociatedFamilyImportOrderConflict | `TypeEnvError::ImportOrderConflict` (`E136`) |
| AssociatedFamilyDependencyClosureConflict | `TypeEnvError::MalformedImportedComputationSummary` closure tokens (`E134`) |
| AssociatedFamilySummaryMalformed | `TypeEnvError::MalformedImportedComputationSummary` (`E134`) |
| AssociatedFamilySummaryUnsupportedVersion | `TypeEnvError::UnsupportedSummaryVersion` (`E133`) |

## Residual limitations

- Some SPEC-063 §12 diagnostic families intentionally route through existing generic public carriers (`InvalidDefinition`, malformed summary, or blocker reasons) rather than new one-variant-per-family production errors. TASK-868 tests lock message tokens and spans for those public routes but do not introduce new diagnostic code values for each conceptual family.
- Several fallback diagnostic-family assertions are public-carrier route tests: they manually construct the existing diagnostic carrier with the message tokens emitted by the producer paths rather than re-driving every parser/type-env/import producer. Behavior-producing paths for these conceptual families remain covered by the cited TASK-859 through TASK-867 suites.
- Negative leakage/non-interference is behavioral for the high-risk predecessor boundaries: the TASK-868 suite re-runs SPEC-035 ordinary associated-type substitution through TypeEnv impl selection, SPEC-058 projection identity through TypeEnv canonical lowering, SPEC-060 non-inversion through `Normalizer::definitional_equality`, SPEC-061 direct `type fn` registration/reduction through parser + TypeEnv + normalizer, and SPEC-062 summary-version validation through `ModuleSemanticSummary::validate_summary_version_contract`.
- This artifact cites existing TASK-859 through TASK-867 tests as acceptance evidence; TASK-868 itself adds focused diagnostics, non-interference, and the associated-family-specific non-inversion check rather than re-implementing all earlier acceptance scenarios.
- No SPEC-H proof search, type-function inversion, HKT, holes, or general proposition solving is implemented or tested as accepted behavior.
