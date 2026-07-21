//! TASK-719: RED stdlib integration coverage for `proc::from_act`.

use ash_core::Expr;
use ash_core::runtime::{ApplicationBoundaryOutcome, ApplicationReportStatus};
use ash_engine::{ApplicationAdmissionOutcome, ApplicationAdmissionRequest};

fn return_act_expr(value: i64) -> Expr {
    Expr::FnDef {
        params: vec![("__act_env".to_string(), None)],
        return_type: None,
        body: Box::new(Expr::Constructor {
            name: "Cons".to_string(),
            fields: vec![
                (
                    "head".to_string(),
                    Expr::Literal(ash_core::Value::ActEnvToken),
                ),
                (
                    "tail".to_string(),
                    Expr::Literal(ash_core::Value::list_from_vec(vec![ash_core::Value::Int(
                        value,
                    )])),
                ),
            ],
        }),
    }
}

#[tokio::test]
async fn proc_from_act_preserves_application_boundary_as_proc_closure_value() {
    let engine = ash_engine::Engine::new().build().expect("engine builds");
    let body = Expr::Call {
        func: "from_act".to_string(),
        module: Some("proc".to_string()),
        arguments: vec![return_act_expr(9)],
    };

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
                    panic!("expected Proc closure at application boundary, got {value:?}");
                };
                assert_eq!(params, &vec![("__proc_env".to_string(), None)]);
                let Some(ash_core::Value::Closure {
                    params: report_params,
                    ..
                }) = report.result.as_ref()
                else {
                    panic!(
                        "expected application report result to preserve the Proc closure value, got {:?}",
                        report.result
                    );
                };
                assert_eq!(report_params, params);
                assert_eq!(report.status, ApplicationReportStatus::Succeeded);
                assert!(
                    !matches!(value, ash_core::Value::ProcessHandle(_)),
                    "proc::from_act should not eagerly expose a process handle at the application boundary"
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
