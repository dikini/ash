//! TASK-719: RED stdlib integration coverage for `proc::from_act`.

use ash_core::runtime::{WorkflowBoundaryOutcome, WorkflowReportStatus};
use ash_core::{Expr, Workflow};
use ash_engine::{WorkflowAdmissionOutcome, WorkflowAdmissionRequest};
use ash_parser::lower::lower_expr;
use ash_parser::surface::{ActStmt, Expr as SurfaceExpr, Literal};

fn proc_main_source(imports: &str, body: &str) -> String {
    format!("use proc::{{{imports}}}\nworkflow main {{ {body} }}\n")
}

fn span() -> ash_parser::token::Span {
    ash_parser::token::Span::default()
}

fn return_act_expr(value: i64) -> Expr {
    lower_expr(&SurfaceExpr::ActBlock {
        stmts: vec![ActStmt::Return {
            value: Box::new(SurfaceExpr::Literal(Literal::Int(value))),
            span: span(),
        }],
        span: span(),
    })
    .expect("single-return act block should lower")
}

#[tokio::test]
async fn proc_stdlib_from_act_import_typechecks_and_returns_proc_closure_value() {
    let tmp_dir = tempfile::tempdir().expect("temp dir");
    let dir = tmp_dir.path();

    std::fs::write(
        dir.join("main.ash"),
        proc_main_source("from_act", "ret from_act(act { ret 7; })"),
    )
    .expect("write main.ash");

    let engine = ash_engine::Engine::new().build().expect("engine builds");
    let mut workflow = engine.parse_file(dir.join("main.ash")).expect("parse");
    engine
        .check(&mut workflow)
        .expect("typecheck proc::from_act stdlib import");
    let result = engine.execute(&workflow).await.expect("execute");
    let ash_core::Value::Closure { params, .. } = result else {
        panic!("expected Proc runtime closure from proc::from_act, got {result:?}");
    };
    assert_eq!(params, vec![("__proc_env".to_string(), None)]);
}

#[tokio::test]
async fn proc_stdlib_from_act_preserves_workflow_boundary_as_proc_closure_value() {
    let engine = ash_engine::Engine::new().build().expect("engine builds");
    let workflow = Workflow::Ret {
        expr: Expr::Call {
            func: "from_act".to_string(),
            module: Some("proc".to_string()),
            arguments: vec![return_act_expr(9)],
        },
    };

    let admitted = engine
        .admit_workflow(WorkflowAdmissionRequest {
            workflow_name: "main".to_string(),
            workflow,
            workflow_id: None,
            run_id: None,
            active_role: None,
            admitted_role: None,
            required_capabilities: vec![],
            requires: vec![],
            ensures: vec![],
        })
        .await;

    match admitted {
        WorkflowAdmissionOutcome::Admitted { boundary } => match boundary.outcome() {
            WorkflowBoundaryOutcome::WorkflowSucceeded { value, report } => {
                let ash_core::Value::Closure { params, .. } = value else {
                    panic!("expected Proc closure at workflow boundary, got {value:?}");
                };
                assert_eq!(params, &vec![("__proc_env".to_string(), None)]);
                let Some(ash_core::Value::Closure {
                    params: report_params,
                    ..
                }) = report.result.as_ref()
                else {
                    panic!(
                        "expected workflow report result to preserve the Proc closure value, got {:?}",
                        report.result
                    );
                };
                assert_eq!(report_params, params);
                assert_eq!(report.status, WorkflowReportStatus::Succeeded);
                assert!(
                    !matches!(value, ash_core::Value::ProcessHandle(_)),
                    "proc::from_act should not eagerly expose a process handle at the workflow boundary"
                );
            }
            other @ WorkflowBoundaryOutcome::WorkflowFailed { .. } => {
                panic!("expected succeeded workflow boundary outcome, got {other:?}")
            }
        },
        other @ WorkflowAdmissionOutcome::Rejected { .. } => {
            panic!("expected admitted workflow boundary outcome, got {other:?}")
        }
    }
}
