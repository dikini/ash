# TASK-903 Type-hole and partial-constructor acceptance matrix

Date: 2026-05-15
Scope: SPEC-066 §8 acceptance matrix plus non-interference boundaries from TASK-898 rows A1-A8.

This artifact records focused, non-zero evidence for the Phase 119 MVP. The accepted slice is intentionally narrow: explicit source `_` holes are enabled only in audited do-target type-argument positions; partial constructor terms are carrier/elaboration substrate; `do:Result<_, E>` reaches missing SPEC-067 Monad evidence after target-shape elaboration; bare `Result`, multiple holes, nested holes, and non-inverting associated-family/type-function-style outputs remain rejected before dictionary lookup.

## Focused target counts

| Package | Test target | Count |
| --- | --- | ---: |
| ash-core | task_899_type_hole_partial_application_carriers | 3 tests |
| ash-parser | task_900_type_hole_surface | 4 tests |
| ash-typeck | task_901_partial_constructor_kinding | 9 tests |
| ash-typeck | task_902_do_target_partial_application | 6 tests |

Commands used for counts:

```text
cargo test -p ash-core --test task_899_type_hole_partial_application_carriers -- --list
cargo test -p ash-parser --test task_900_type_hole_surface -- --list
cargo test -p ash-typeck --test task_901_partial_constructor_kinding -- --list
cargo test -p ash-typeck --test task_902_do_target_partial_application -- --list
```

## SPEC-066 §8 acceptance matrix

| §8 row | Acceptance requirement | Exact evidence | Target count |
| --- | --- | --- | ---: |
| H-1 | Parse `Result<_, E>` in enabled do-target position with hole span preserved. | `crates/ash-parser/tests/task_900_type_hole_surface.rs::parses_do_target_type_argument_hole_with_distinct_span`; core carrier identity/spans in `crates/ash-core/tests/task_899_type_hole_partial_application_carriers.rs::type_hole_id_preserves_stable_numeric_identity_and_metadata` | 4, 3 |
| H-2 | Bare `Result` as do target is not implicitly curried and reports a wrong-shape diagnostic with a hole hint. | `crates/ash-typeck/tests/task_901_partial_constructor_kinding.rs::task_901_bare_higher_arity_constructor_suggests_explicit_hole`; `crates/ash-typeck/tests/task_902_do_target_partial_application.rs::task_902_bare_result_reports_wrong_shape_with_hole_hint`; in-module regression `crates/ash-typeck/src/do_target.rs::do_target_result_is_deferred_without_dictionary` | 9, 6 |
| H-3 | `Result<_, E>` without Monad evidence reaches target-shape elaboration first, then reports missing Monad evidence. | `crates/ash-typeck/tests/task_901_partial_constructor_kinding.rs::task_901_result_hole_error_elaborates_to_unary_partial_application`; `crates/ash-typeck/tests/task_902_do_target_partial_application.rs::task_902_result_hole_error_reaches_missing_monad_evidence`; in-module regression `do_target_with_partial_explicit_args_reaches_missing_monad_evidence` | 9, 6 |
| H-4 | Multiple holes in MVP do-target position are rejected before dictionary/evidence lookup. | `crates/ash-typeck/tests/task_901_partial_constructor_kinding.rs::task_901_multiple_holes_are_rejected`; `crates/ash-typeck/tests/task_902_do_target_partial_application.rs::task_902_result_with_two_holes_reports_multiple_holes` | 9, 6 |
| H-5 | `_` in type-function patterns remains a pattern wildcard, not a source type hole. | `crates/ash-parser/tests/task_900_type_hole_surface.rs::keeps_type_function_pattern_underscore_as_wildcard_not_type_hole`; parser fail-closed coverage `rejects_type_holes_in_ordinary_type_aliases` and `rejects_type_holes_in_ordinary_workflow_return_types` | 4 |
| H-6 | Holes under neutral type-function/associated-family-style outputs are rejected or deferred without inversion. | `crates/ash-typeck/tests/task_901_partial_constructor_kinding.rs::task_901_holes_under_associated_family_projection_do_not_invert_outputs`; `task_901_nested_holes_are_unsupported_positions_not_inversion`; `task_901_non_hole_associated_family_argument_is_not_an_inversion_boundary`; do-target integration `crates/ash-typeck/tests/task_902_do_target_partial_application.rs::task_902_associated_family_hole_reports_no_inversion_not_missing_evidence` and `task_902_nested_result_hole_reports_unsupported_shape_not_missing_evidence` | 9, 6 |

## Non-interference and explicit deferrals

| Boundary | Evidence / status |
| --- | --- |
| Parser scope | TASK-900 keeps holes limited to audited do-target type arguments and rejects ordinary aliases/workflow return types. |
| Core representation | TASK-899 preserves `TypeHoleId`, `PartialArg::Hole`, and `TypeConstructorExpr::PartialApplication`; no fake saturated nominal constructor is introduced. |
| TypeEnv shape checking | TASK-901 validates arity, single-hole MVP shape, saturated-result rejection, nested-hole rejection, and no-inversion boundary before do-target dictionary selection. |
| Do dictionaries | TASK-902 preserves existing Act/Proc/Workflow hidden dictionaries and adds only target-shape elaboration for explicit partial targets; no SPEC-067 Monad dictionary/evidence implementation is included. |
| Engine/runtime | TASK-898 row A8 remains authoritative: Phase 119 adds no engine/runtime semantics before summary-visible carriers require transport changes. |
| Later specs | HKT binders, constructor variables, generalized Monad evidence, do-target inference, arbitrary type lambdas, and output-driven inversion remain deferred to SPEC-067 or later. |

## Verification notes

Focused evidence after TASK-902 review remediation:

```text
cargo test -p ash-typeck --test task_902_do_target_partial_application  # 6 passed
cargo test -p ash-typeck --lib do_target -- --test-threads=1           # 9 passed
```

Broad closeout evidence is recorded in `docs/plan/tasks/TASK-903-type-hole-closeout.md` after the final code/doc change.
