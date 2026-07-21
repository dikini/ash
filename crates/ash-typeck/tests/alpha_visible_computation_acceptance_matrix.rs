use std::fs;
use std::path::PathBuf;

fn workspace_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(|path| path.parent())
        .expect("ash-typeck lives under crates/")
        .to_path_buf()
}

fn read_workspace_file(relative: &str) -> String {
    let path = workspace_root().join(relative);
    fs::read_to_string(&path)
        .unwrap_or_else(|error| panic!("missing TASK-931 evidence at {}: {error}", path.display()))
}

fn assert_contains_all(artifact: &str, text: &str, snippets: &[&str]) {
    for snippet in snippets {
        assert!(
            text.contains(snippet),
            "{artifact} is missing concrete evidence string {snippet:?}"
        );
    }
}

fn assert_rows(artifact: &str, text: &str, rows: &[(&str, &[&str])]) {
    for (id, snippets) in rows {
        assert!(text.contains(id), "{artifact} is missing row {id}");
        assert_contains_all(artifact, text, snippets);
    }
}

#[test]
fn spec069_acceptance_cases_are_mapped_to_focused_tests() {
    let audit = read_workspace_file("docs/plan/audits/TASK-931-alpha-acceptance-matrix.md");
    let spec =
        read_workspace_file("docs/spec/SPEC-069-ALPHA-VISIBLE-TOWER-ALGEBRA-AND-DO-LOWERING.md");
    let task = read_workspace_file(
        "docs/plan/tasks/TASK-931-alpha-semantics-correspondence-and-acceptance-matrix.md",
    );

    assert_contains_all(
        "SPEC-069",
        &spec,
        &[
            "| A69-1 |",
            "| A69-2 |",
            "| A69-3 |",
            "| A69-4 |",
            "| A69-5 |",
            "| A69-6 |",
            "| A69-7 |",
            "| A69-8 |",
            "| A69-9 |",
            "| A69-10 |",
            "| A69-11 |",
            "| A69-12 |",
        ],
    );

    assert_rows(
        "TASK-931 audit",
        &audit,
        &[
            (
                "A69-1",
                &[
                    "crates/ash-typeck/tests/task_749_typed_do.rs",
                    "typed_elaboration_lowers_act_return_through_hidden_act_dictionary",
                    "crates/ash-typeck/tests/alpha_visible_computation_manifest.rs",
                    "visible_intrinsic_mapping_has_no_hidden_unrelated_do_magic",
                ],
            ),
            (
                "A69-2",
                &[
                    "crates/ash-typeck/tests/alpha_visible_computation_manifest.rs",
                    "visible_intrinsic_mapping_has_no_hidden_unrelated_do_magic",
                ],
            ),
            (
                "A69-4",
                &[
                    "crates/ash-typeck/tests/alpha_generalized_do_full_bind_lowering.rs",
                    "do_result_bind_lowers_through_monad_bind_evidence",
                ],
            ),
            (
                "A69-5",
                &[
                    "crates/ash-typeck/tests/alpha_generalized_do_full_bind_lowering.rs",
                    "user_option_do_bind_uses_selected_monad_evidence",
                ],
            ),
            (
                "A69-6",
                &[
                    "crates/ash-typeck/tests/alpha_generalized_do_full_bind_lowering.rs",
                    "generic_monad_do_specializes_before_execution",
                ],
            ),
            (
                "A69-7",
                &[
                    "crates/ash-typeck/tests/alpha_monad_evidence_method_body_lowering.rs",
                    "ambiguous_monad_evidence_rejected_before_lowering",
                ],
            ),
            (
                "A69-8",
                &[
                    "crates/ash-typeck/tests/task_708_operational_bottom.rs",
                    "fail_typechecks_as_bottom_compatible_value",
                    "Result domain failures remain selected Monad evidence, not Act failure",
                ],
            ),
            (
                "A69-10",
                &[
                    "crates/ash-core/tests/alpha_amir_bytecode_schema.rs",
                    "bytecode_schema_validates_without_source_reparse",
                ],
            ),
            (
                "A69-11",
                &[
                    "crates/ash-typeck/tests/task_750_target_act_do_sugar.rs",
                    "removed_act_statement_forms_do_not_parse_or_typecheck",
                ],
            ),
            (
                "A69-12",
                &[
                    "crates/ash-core/tests/alpha_runtime_kernel_carriers.rs",
                    "runtime_kernel_host_modes_share_definition_and_artifact_identity",
                    "crates/ash-cli/tests/alpha_ashd_local_daemon_control_plane.rs",
                ],
            ),
        ],
    );

    assert_contains_all(
        "TASK-931 task",
        &task,
        &[
            "docs/plan/audits/TASK-931-alpha-acceptance-matrix.md",
            "crates/ash-typeck/tests/alpha_visible_computation_acceptance_matrix.rs",
            "spec069_acceptance_cases_are_mapped_to_focused_tests",
        ],
    );
}

#[test]
fn spec070_runtime_acceptance_cases_are_mapped_to_focused_tests() {
    let audit = read_workspace_file("docs/plan/audits/TASK-931-alpha-acceptance-matrix.md");
    let spec = read_workspace_file("docs/spec/SPEC-070-ALPHA-RUNTIME-KERNEL-AND-OS-SURFACE.md");
    let task = read_workspace_file(
        "docs/plan/tasks/TASK-931-alpha-semantics-correspondence-and-acceptance-matrix.md",
    );

    assert_contains_all(
        "SPEC-070",
        &spec,
        &[
            "| A70-1 |",
            "| A70-2 |",
            "| A70-3 |",
            "| A70-4 |",
            "| A70-5 |",
            "| A70-6 |",
            "| A70-7 |",
            "| A70-8 |",
            "Provider/resource existence is not authority",
            "Successful reload indexes new definitions/artifacts for future starts",
        ],
    );

    assert_rows(
        "TASK-931 audit",
        &audit,
        &[
            (
                "A70-1",
                &[
                    "crates/ash-cli/tests/alpha_ash_run_runtime_kernel_mode.rs",
                    "ash_run_executes_entry_through_one_shot_runtime_kernel",
                    "ash_run_reports_kernel_instance_and_artifact_identity",
                ],
            ),
            (
                "A70-2",
                &[
                    "crates/ash-cli/tests/alpha_ash_run_runtime_kernel_mode.rs",
                    "ash_run_emits_runtime_kernel_report_on_parse_failure_after_local_source_read",
                    "crates/ash-interp/tests/invoke_runtime_dispatch.rs",
                    "registered_provider_without_admitted_binding_cannot_execute_through_invoke_fallback",
                ],
            ),
            (
                "A70-3",
                &[
                    "crates/ash-cli/tests/alpha_ashd_local_daemon_control_plane.rs",
                    "ashd_serve_indexes_definitions_without_running_workflows",
                ],
            ),
            (
                "A70-4",
                &[
                    "crates/ash-cli/tests/alpha_ashd_local_daemon_control_plane.rs",
                    "ashd_serve_indexes_definitions_without_running_workflows",
                    "args/config/admission-profile fields remain deferred",
                ],
            ),
            (
                "A70-5",
                &[
                    "crates/ash-cli/tests/alpha_ashd_local_daemon_control_plane.rs",
                    "ashd_reload_updates_definition_table_and_preserves_kernel_mode",
                ],
            ),
            (
                "A70-6",
                &[
                    "crates/ash-core/tests/alpha_runtime_kernel_carriers.rs",
                    "runtime_kernel_ids_cover_root_definition_artifact_instance_and_host_mode",
                    "provider_in_act_env_without_runtime_state_binding_cannot_execute_through_invoke_fallback",
                ],
            ),
            (
                "A70-7",
                &[
                    "crates/ash-core/tests/alpha_runtime_kernel_carriers.rs",
                    "process_tree",
                    "child process failure observation remains a deferred execution trace boundary",
                ],
            ),
            (
                "A70-8",
                &[
                    "crates/ash-core/tests/alpha_runtime_kernel_carriers.rs",
                    "runtime_kernel_host_modes_share_definition_and_artifact_identity",
                    "host lifetime/control plane differs",
                ],
            ),
        ],
    );

    assert_contains_all(
        "TASK-931 task",
        &task,
        &[
            "spec070_runtime_acceptance_cases_are_mapped_to_focused_tests",
            "Do not mark TASK-932 complete",
        ],
    );
}

#[test]
fn alpha_non_interference_matrix_covers_removed_surfaces() {
    let audit = read_workspace_file("docs/plan/audits/TASK-931-alpha-acceptance-matrix.md");
    let spec069 =
        read_workspace_file("docs/spec/SPEC-069-ALPHA-VISIBLE-TOWER-ALGEBRA-AND-DO-LOWERING.md");
    let spec070 = read_workspace_file("docs/spec/SPEC-070-ALPHA-RUNTIME-KERNEL-AND-OS-SURFACE.md");

    assert_contains_all(
        "SPEC-069 non-interference",
        &spec069,
        &[
            "Do not regress SPEC-066/SPEC-067 target-resolution behavior",
            "must not broaden associated-family inversion or proof search",
            "must not expose hidden runtime environment representations",
            "must not bypass capability/admission semantics",
        ],
    );
    assert_contains_all(
        "SPEC-070 authority/reload",
        &spec070,
        &[
            "Provider/resource existence is not authority",
            "`Act` capability invocation checks admitted grant state",
            "Existing running instances keep the artifact/version they were admitted with",
        ],
    );

    assert_contains_all(
        "TASK-931 audit non-interference",
        &audit,
        &[
            "NI-1 SPEC-066/SPEC-067 target resolution",
            "crates/ash-typeck/tests/task_902_do_target_partial_application.rs",
            "task_902_associated_family_hole_reports_no_inversion_not_missing_evidence",
            "crates/ash-typeck/tests/task_910_hkt_acceptance_matrix.rs",
            "hkt8_do_list_without_monad_evidence_reports_missing_evidence",
            "NI-2 associated-family/proposition inversion boundary",
            "crates/ash-typeck/tests/task_876_proposition_solver.rs",
            "task_876_equality_deferred_at_neutral_no_inversion_boundary_without_solving_inputs",
            "crates/ash-typeck/tests/task_882_spec_h_acceptance_matrix.rs",
            "task_882_h7_known_interface_bound_satisfies_and_h8_missing_bound_defers_without_search",
            "NI-3 hidden ActEnv/process/runtime identity non-denotability",
            "visible_intrinsic_mapping_has_no_hidden_unrelated_do_magic",
            "NI-4 visible authority/admission boundary",
            "registered_provider_without_admitted_binding_cannot_execute_through_invoke_fallback",
            "provider_in_act_env_without_runtime_state_binding_cannot_execute_through_invoke_fallback",
        ],
    );
}
