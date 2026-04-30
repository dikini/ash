//! TASK-777 first-class workflow-valued function export summary regression tests.

use ash_core::workflow_carrier::{ProjectionEventKind, SourceOrigin, WorkflowObligation};
use ash_core::workflow_contract::{ArithConstraint, PostPredicate, Requirement, RolePolicy};
use ash_engine::module_loader::load_ordinary_file;

fn write_caller(dir: &tempfile::TempDir) -> std::path::PathBuf {
    let caller = dir.path().join("caller.ash");
    std::fs::write(
        &caller,
        r"use flows::{guarded}
workflow main { ret 0 }
",
    )
    .expect("write caller");
    caller
}

#[test]
fn pub_fn_returning_workflow_exports_public_contract_summary_from_do_workflow() {
    let dir = tempfile::tempdir().expect("tempdir");
    let module = dir.path().join("flows.ash");
    let caller = write_caller(&dir);

    std::fs::write(
        &module,
        r"pub fn guarded() -> Workflow<Int> {
    do:Workflow {
        requires: any_role([Reviewer, Approver]);
        ensures: result >= 1;
        return 1
    }
}
",
    )
    .expect("write module");

    let loaded = load_ordinary_file(&caller).expect("caller imports public workflow fn");
    let summary = loaded
        .imported_callables
        .get("guarded")
        .and_then(|callable| callable.workflow_summary.as_ref())
        .expect("pub fn returning Workflow<Int> exports a public workflow summary");

    assert!(
        summary.projection_events.iter().any(|event| matches!(
            &event.kind,
            ProjectionEventKind::Requires {
                requirement: Requirement::AnyRole(RolePolicy { roles })
            } if roles == &["Reviewer".to_string(), "Approver".to_string()]
        )),
        "first-class workflow fn summary should preserve public requires events"
    );
    assert!(
        summary.projection_events.iter().any(|event| matches!(
            &event.kind,
            ProjectionEventKind::Ensures { postcondition }
                if postcondition.predicate == PostPredicate::ResultSatisfies(ArithConstraint::Gte(1))
        )),
        "first-class workflow fn summary should preserve public ensures events"
    );
    assert!(
        summary
            .coverage
            .obligations
            .iter()
            .any(|obligation| matches!(
                obligation,
                WorkflowObligation::RequirementMustHold {
                    requirement: Requirement::AnyRole(RolePolicy { roles }),
                    ..
                } if roles == &["Reviewer".to_string(), "Approver".to_string()]
            )),
        "first-class workflow fn summary coverage should retain requires admission obligations"
    );
    assert!(
        summary.coverage.obligations.iter().any(|obligation| matches!(
            obligation,
            WorkflowObligation::OpenPostconditionTarget { postcondition, target_type, .. }
                if postcondition.predicate == PostPredicate::ResultSatisfies(ArithConstraint::Gte(1))
                    && target_type == "WorkflowResult"
        )),
        "first-class workflow fn summary coverage should retain ensures result-target obligations"
    );
    assert!(
        summary.projection_events.iter().all(|event| matches!(
            &event.origin,
            SourceOrigin::ImportedSummary { module, public_anchor }
                if module == "flows" && public_anchor == "guarded"
        )),
        "first-class workflow fn summary should expose only public imported-summary origins"
    );
}

#[test]
fn unsupported_workflow_returning_pub_fn_exports_no_fabricated_summary() {
    let dir = tempfile::tempdir().expect("tempdir");
    let module = dir.path().join("flows.ash");
    let caller = write_caller(&dir);

    std::fs::write(
        &module,
        r"pub fn guarded() -> Workflow<Int> {
    do:Workflow {
        let x = 1;
        return x
    }
}
",
    )
    .expect("write module");

    let loaded = load_ordinary_file(&caller).expect("caller imports unsupported workflow fn");
    let callable = loaded
        .imported_callables
        .get("guarded")
        .expect("guarded callable imported");

    assert!(
        callable.workflow_summary.is_none(),
        "unsupported first-class Workflow-returning pub fn bodies must remain opaque rather than fabricating a public summary"
    );
}

#[test]
fn legacy_and_first_class_workflow_exports_have_equivalent_public_contract_summary() {
    let dir = tempfile::tempdir().expect("tempdir");
    let legacy_module = dir.path().join("flows.ash");
    let first_class_module = dir.path().join("first_class.ash");
    let legacy_caller = dir.path().join("legacy_caller.ash");
    let first_class_caller = dir.path().join("first_class_caller.ash");

    std::fs::write(
        &legacy_module,
        r"pub workflow guarded() -> Workflow<Int>
    requires: any_role([Reviewer, Approver])
    ensures: result >= 1
{
    done
}
",
    )
    .expect("write legacy module");
    std::fs::write(
        &first_class_module,
        r"pub fn guarded() -> Workflow<Int> {
    do:Workflow {
        requires: any_role([Reviewer, Approver]);
        ensures: result >= 1;
        return 1
    }
}
",
    )
    .expect("write first-class module");
    std::fs::write(
        &legacy_caller,
        r"use flows::{guarded}
workflow main { ret 0 }
",
    )
    .expect("write legacy caller");
    std::fs::write(
        &first_class_caller,
        r"use first_class::{guarded}
workflow main { ret 0 }
",
    )
    .expect("write first-class caller");

    let legacy_loaded = load_ordinary_file(&legacy_caller).expect("legacy import");
    let first_class_loaded = load_ordinary_file(&first_class_caller).expect("first-class import");
    let legacy = legacy_loaded
        .imported_callables
        .get("guarded")
        .and_then(|callable| callable.workflow_summary.as_ref())
        .expect("legacy summary");
    let first_class = first_class_loaded
        .imported_callables
        .get("guarded")
        .and_then(|callable| callable.workflow_summary.as_ref())
        .expect("first-class summary");

    let legacy_kinds = legacy
        .projection_events
        .iter()
        .map(|event| &event.kind)
        .filter(|kind| {
            matches!(
                kind,
                ProjectionEventKind::Requires { .. } | ProjectionEventKind::Ensures { .. }
            )
        })
        .collect::<Vec<_>>();
    let first_class_kinds = first_class
        .projection_events
        .iter()
        .map(|event| &event.kind)
        .filter(|kind| {
            matches!(
                kind,
                ProjectionEventKind::Requires { .. } | ProjectionEventKind::Ensures { .. }
            )
        })
        .collect::<Vec<_>>();

    assert!(legacy_kinds.iter().any(|kind| matches!(
        kind,
        ProjectionEventKind::Requires {
            requirement: Requirement::AnyRole(RolePolicy { roles })
        } if roles == &["Reviewer".to_string(), "Approver".to_string()]
    )));
    assert_eq!(
        legacy_kinds, first_class_kinds,
        "legacy workflow headers and equivalent first-class workflow expressions must export the same public requires/ensures contract event sequence modulo public origin stamping and private body-summary metadata"
    );
}
