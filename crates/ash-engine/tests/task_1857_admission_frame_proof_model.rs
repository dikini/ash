//! TASK-1857 admission frame proof model tests.

use ash_engine::row_admission::{
    OperationAdmissionFrame, OperationAdmissionFrameKind, RowAdmissionCheck,
    RowAdmissionEnvironment, RowAdmissionProof, RowAdmissionRequirement,
};
use ash_engine::{ApplicationAdmissionRequest, Engine};

fn workflow_stub() -> ash_engine::Workflow {
    Engine::new()
        .build()
        .expect("engine builds")
        .parse("fn main() { 0 }")
        .expect("workflow parses")
}

fn base_request(workflow: &ash_engine::Workflow) -> ApplicationAdmissionRequest {
    ApplicationAdmissionRequest {
        entry_name: "handler_provider_admission".into(),
        workflow: workflow.core.clone(),
        application_id: None,
        run_id: None,
        active_role: None,
        admitted_role: None,
        required_capabilities: vec![],
        requires: vec![],
        ensures: vec![],
    }
}

fn read_requirement() -> RowAdmissionRequirement {
    RowAdmissionRequirement::Operation {
        authority: "PosixFs".to_string(),
        operation: "read".to_string(),
    }
}

#[test]
fn task_1857_provider_frame_proves_operation_requirement() {
    let env = RowAdmissionEnvironment::new().with_operation_frame(OperationAdmissionFrame {
        identity: "PosixFs::read".to_string(),
        kind: OperationAdmissionFrameKind::Provider {
            provider: "host.posix".to_string(),
        },
    });

    assert_eq!(
        env.prove_operation(&read_requirement()),
        Some(RowAdmissionProof::OperationProviderFrame {
            identity: "PosixFs::read".to_string(),
            provider: "host.posix".to_string(),
        })
    );
}

#[test]
fn task_1857_handler_frame_proves_operation_requirement_and_shadows_provider() {
    let env = RowAdmissionEnvironment::new()
        .with_operation_frame(OperationAdmissionFrame {
            identity: "PosixFs::read".to_string(),
            kind: OperationAdmissionFrameKind::Provider {
                provider: "outer.host".to_string(),
            },
        })
        .with_operation_frame(OperationAdmissionFrame {
            identity: "PosixFs::read".to_string(),
            kind: OperationAdmissionFrameKind::Handler {
                handler: "inner.handler".to_string(),
            },
        });

    assert_eq!(
        env.prove_operation(&read_requirement()),
        Some(RowAdmissionProof::OperationHandlerFrame {
            identity: "PosixFs::read".to_string(),
            handler: "inner.handler".to_string(),
        })
    );
}

#[test]
fn task_1860_missing_frame_or_provider_fails_closed_with_handler_provider_diagnostic() {
    let engine = Engine::new().build().expect("engine builds");
    let workflow = workflow_stub();
    let request = base_request(&workflow);
    let req = read_requirement();
    let env = RowAdmissionEnvironment::new();

    let RowAdmissionCheck::Missing { notes, .. } =
        RowAdmissionCheck::check_with_environment(&engine, &request, &req, &env)
    else {
        panic!("missing handler/provider discharge should reject");
    };

    let text = notes.join("\n");
    assert!(text.contains("PosixFs::read"), "{text}");
    assert!(text.contains("handler/provider frame"), "{text}");
    assert!(text.contains("Rows do not grant authority"), "{text}");
}

#[test]
fn task_1857_handler_frame_satisfies_admission_without_registered_provider() {
    let engine = Engine::new().build().expect("engine builds");
    let workflow = workflow_stub();
    let request = base_request(&workflow);
    let req = read_requirement();
    let env = RowAdmissionEnvironment::new().with_operation_frame(OperationAdmissionFrame {
        identity: "PosixFs::read".to_string(),
        kind: OperationAdmissionFrameKind::Handler {
            handler: "posix_fs_handler".to_string(),
        },
    });

    assert!(matches!(
        RowAdmissionCheck::check_with_environment(&engine, &request, &req, &env),
        RowAdmissionCheck::Satisfied
    ));
}
