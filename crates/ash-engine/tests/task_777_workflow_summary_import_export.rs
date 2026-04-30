//! TASK-777 regression tests for public workflow summary import/export.

use ash_core::workflow_carrier::{ProjectionEventKind, SourceOrigin, WorkflowObligation};
use ash_core::workflow_contract::{ArithConstraint, PostPredicate, Requirement, RolePolicy};
use ash_engine::module_loader::load_ordinary_file;

#[test]
fn load_ordinary_file_exports_public_workflow_summary_for_imported_workflow() {
    let dir = tempfile::tempdir().expect("tempdir");
    let module = dir.path().join("flows.ash");
    let caller = dir.path().join("caller.ash");

    std::fs::write(
        &module,
        r"pub workflow guarded() -> Workflow<Int> {
    done
}
",
    )
    .expect("write module");
    std::fs::write(
        &caller,
        r"use flows::{guarded}
workflow main { ret 0 }
",
    )
    .expect("write caller");

    let loaded = load_ordinary_file(&caller).expect("caller imports public workflow");
    let callable = loaded
        .imported_callables
        .get("guarded")
        .expect("guarded callable imported");
    let summary = callable
        .workflow_summary
        .as_ref()
        .expect("pub workflow export carries public workflow summary");

    assert!(
        summary.node_count > 0,
        "summary should expose workflow node shape"
    );
    assert!(
        summary.projection_events.iter().any(|event| {
            matches!(
                event.origin,
                SourceOrigin::ImportedSummary {
                    ref module,
                    ref public_anchor,
                } if module == "flows" && public_anchor == "guarded"
            )
        }),
        "summary events should use public imported-summary origins"
    );
}

#[test]
fn load_ordinary_file_exports_public_workflow_summary_contract_events() {
    let dir = tempfile::tempdir().expect("tempdir");
    let module = dir.path().join("flows.ash");
    let caller = dir.path().join("caller.ash");

    std::fs::write(
        &module,
        r"pub workflow guarded() -> Workflow<Int>
    requires: any_role([Reviewer, Approver])
    ensures: result >= 1
{
    done
}
",
    )
    .expect("write module");
    std::fs::write(
        &caller,
        r"use flows::{guarded}
workflow main { ret 0 }
",
    )
    .expect("write caller");

    let loaded = load_ordinary_file(&caller).expect("caller imports public workflow");
    let summary = loaded
        .imported_callables
        .get("guarded")
        .and_then(|callable| callable.workflow_summary.as_ref())
        .expect("guarded public workflow summary imported");

    assert!(
        summary.projection_events.iter().any(|event| matches!(
            &event.kind,
            ProjectionEventKind::Requires {
                requirement: Requirement::AnyRole(RolePolicy { roles })
            } if roles == &["Reviewer".to_string(), "Approver".to_string()]
        )),
        "exported public summary should preserve public requires contract events"
    );
    assert!(
        summary.projection_events.iter().any(|event| matches!(
            &event.kind,
            ProjectionEventKind::Ensures { postcondition }
                if postcondition.predicate == PostPredicate::ResultSatisfies(ArithConstraint::Gte(1))
        )),
        "exported public summary should preserve public ensures contract events"
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
        "exported public summary coverage should retain requires admission obligations"
    );
    assert!(
        summary.coverage.obligations.iter().any(|obligation| matches!(
            obligation,
            WorkflowObligation::OpenPostconditionTarget { postcondition, target_type, .. }
                if postcondition.predicate == PostPredicate::ResultSatisfies(ArithConstraint::Gte(1))
                    && target_type == "WorkflowResult"
        )),
        "exported public summary coverage should retain ensures result-target obligations"
    );
    assert!(
        summary.projection_events.iter().all(|event| matches!(
            &event.origin,
            SourceOrigin::ImportedSummary { module, public_anchor }
                if module == "flows" && public_anchor == "guarded"
        )),
        "exported public summary should expose only public imported-summary origins"
    );
}
