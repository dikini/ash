//! TASK-1972: ordinary function values remain ordinary at the application boundary.

use ash_core::Expr;
use ash_core::runtime::{ApplicationFailureKind, ApplicationReportStatus};
use ash_engine::{ApplicationAdmissionOutcome, ApplicationAdmissionRequest};

fn return_function_expr(value: i64) -> Expr {
    Expr::FnDef {
        params: Vec::new(),
        return_type: None,
        body: Box::new(Expr::Literal(ash_core::Value::Int(value))),
    }
}

#[tokio::test]
async fn ordinary_function_value_rejects_at_the_checked_cps_application_boundary() {
    let engine = ash_engine::Engine::new().build().expect("engine builds");
    let body = return_function_expr(9);

    let admitted = engine
        .admit_application(ApplicationAdmissionRequest {
            entry_name: "main".to_string(),
            body,
            application_id: None,
            run_id: None,
            active_role: None,
            admitted_role: None,
            required_capabilities: vec![],
            requires: vec![],
            ensures: vec![],
        })
        .await;

    match admitted {
        ApplicationAdmissionOutcome::Rejected { failure, report } => {
            assert_eq!(
                failure.kind,
                ApplicationFailureKind::AdmissionFailure,
                "the ordinary function and application prerequisites must reach the shared closed admission boundary"
            );
            assert_eq!(report.status, ApplicationReportStatus::Failed);
        }
        other @ ApplicationAdmissionOutcome::Admitted { .. } => {
            panic!("expected checked Core/CPS admission rejection, got {other:?}")
        }
    }
}
