//! TASK-1972: ordinary function values remain ordinary at the application boundary.

use ash_core::Expr;
use ash_core::runtime::{ApplicationBoundaryOutcome, ApplicationReportStatus};
use ash_engine::{ApplicationAdmissionOutcome, ApplicationAdmissionRequest};

fn return_function_expr(value: i64) -> Expr {
    Expr::FnDef {
        params: Vec::new(),
        return_type: None,
        body: Box::new(Expr::Literal(ash_core::Value::Int(value))),
    }
}

#[tokio::test]
async fn ordinary_function_value_preserves_application_boundary_result() {
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
        ApplicationAdmissionOutcome::Admitted { boundary } => match boundary.outcome() {
            ApplicationBoundaryOutcome::ApplicationSucceeded { value, report } => {
                let ash_core::Value::Closure { params, .. } = value else {
                    panic!(
                        "expected ordinary function closure at application boundary, got {value:?}"
                    );
                };
                assert!(params.is_empty());
                let Some(ash_core::Value::Closure {
                    params: report_params,
                    ..
                }) = report.result.as_ref()
                else {
                    panic!(
                        "expected application report result to preserve the function closure value, got {:?}",
                        report.result
                    );
                };
                assert_eq!(report_params, params);
                assert_eq!(report.status, ApplicationReportStatus::Succeeded);
                assert!(
                    !matches!(value, ash_core::Value::ProcessHandle(_)),
                    "ordinary function values must not expose process handles at the application boundary"
                );
            }
            other @ ApplicationBoundaryOutcome::ApplicationFailed { .. } => {
                panic!("expected succeeded application boundary outcome, got {other:?}")
            }
        },
        other @ ApplicationAdmissionOutcome::Rejected { .. } => {
            panic!("expected admitted application boundary outcome, got {other:?}")
        }
    }
}
