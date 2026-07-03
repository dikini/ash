//! TASK-1850/TASK-1851/TASK-1852 operation authority discharge model tests.

use ash_core::core_ash::{CoreRow, CoreRowItem, CoreType};
use ash_engine::row_admission::{
    RowAdmissionCheck, RowAdmissionDischarge, RowAdmissionRequirement,
};
use ash_engine::{Engine, WorkflowAdmissionRequest};

fn workflow_stub() -> ash_engine::Workflow {
    Engine::new()
        .build()
        .expect("engine builds")
        .parse("workflow main { ret 0 }")
        .expect("workflow parses")
}

fn base_request(workflow: &ash_engine::Workflow) -> WorkflowAdmissionRequest {
    WorkflowAdmissionRequest {
        workflow_name: "operation_authority_model".into(),
        workflow: workflow.core.clone(),
        workflow_id: None,
        run_id: None,
        active_role: None,
        admitted_role: None,
        required_capabilities: vec![],
        requires: vec![],
        ensures: vec![],
    }
}

#[test]
fn task_1850_operation_rows_have_operation_authority_discharge_family() {
    let row = CoreRow::closed(vec![CoreRowItem::operation(
        vec!["PosixFs".to_string()],
        "read",
    )]);
    let requirements = RowAdmissionRequirement::from_core_row(&row);
    let [requirement] = &requirements[..] else {
        panic!("expected one operation requirement");
    };

    assert_eq!(
        requirement.discharge(),
        RowAdmissionDischarge::OperationAuthority {
            identity: "PosixFs::read".to_string(),
            authority: "PosixFs".to_string(),
            operation: "read".to_string(),
        }
    );
    assert_eq!(requirement.label(), "operation PosixFs::read");
}

#[test]
fn task_1851_operation_authority_diagnostic_preserves_impl_qualified_identity() {
    let req = RowAdmissionRequirement::Operation {
        authority: "PosixFs".to_string(),
        operation: "read".to_string(),
    };
    let engine = Engine::new().build().expect("engine builds");
    let workflow = workflow_stub();
    let request = base_request(&workflow);

    let RowAdmissionCheck::Missing { notes, .. } =
        RowAdmissionCheck::check(&engine, &request, &req)
    else {
        panic!("missing operation authority should reject");
    };
    let text = notes.join("\n");
    assert!(text.contains("operation authority"), "{text}");
    assert!(text.contains("PosixFs::read"), "{text}");
    assert!(text.contains("Rows do not grant authority"), "{text}");
}

#[test]
fn task_1852_row_families_have_distinct_discharge_paths() {
    let row = CoreRow::closed(vec![
        CoreRowItem::Resource {
            path: vec!["vault".to_string()],
            mode: "write".to_string(),
        },
        CoreRowItem::Role {
            path: vec!["tenant".to_string(), "admin".to_string()],
        },
        CoreRowItem::Policy {
            path: vec!["pii".to_string(), "redact".to_string()],
        },
        CoreRowItem::Evidence {
            path: vec!["signed".to_string()],
        },
        CoreRowItem::Failure {
            ty: Some(Box::new(CoreType::Named("HostFailure".to_string()))),
        },
    ]);

    let discharges: Vec<_> = RowAdmissionRequirement::from_core_row(&row)
        .iter()
        .map(RowAdmissionRequirement::discharge)
        .collect();

    assert!(
        discharges.contains(&RowAdmissionDischarge::ResourceAuthority {
            resource: "vault".to_string(),
            mode: "write".to_string(),
        })
    );
    assert!(discharges.contains(&RowAdmissionDischarge::RoleAuthority {
        role: "tenant.admin".to_string(),
    }));
    assert!(discharges.contains(&RowAdmissionDischarge::PolicyEvidence {
        policy: "pii.redact".to_string(),
    }));
    assert!(discharges.contains(&RowAdmissionDischarge::Evidence {
        evidence: "signed".to_string(),
    }));
    assert!(discharges.contains(&RowAdmissionDischarge::FailureHandler {
        ty: Some("HostFailure".to_string()),
    }));
}
