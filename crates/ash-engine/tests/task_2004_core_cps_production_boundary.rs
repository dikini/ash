//! TASK-2004: production/Core-CPS execution-boundary regression tests.
//!
//! These tests deliberately require a machine-readable boundary declaration.
//! They do not infer execution routing from source text: a future Core-to-CPS
//! integration must change the declaration and this test together.

use ash_core::{Value, runtime::ApplicationFailureKind};
use ash_engine::{
    ApplicationAdmissionOutcome, ApplicationAdmissionRequest, Engine, ProductionExecutionBoundary,
};
use ash_runtime::ExecError;

#[test]
fn engine_declares_checked_core_cps_closed_admission_as_its_production_boundary() {
    let engine = Engine::new().build().expect("engine builds");

    assert_eq!(
        engine.production_execution_boundary(),
        ProductionExecutionBoundary::CheckedCoreCpsClosedAdmission,
        "Path B rejects every source execution route until a validated production Core/CPS artifact exists"
    );
}

#[tokio::test]
async fn source_run_admits_a_supported_literal_through_checked_core_cps() {
    let engine = Engine::new().build().expect("engine builds");

    let value = engine.run("fn main() -> Int { 42 }").await.expect(
        "the supported literal source must execute through sealed checked Core/CPS admission",
    );

    assert_eq!(value, Value::Int(42));
    assert_eq!(
        engine.production_execution_boundary(),
        ProductionExecutionBoundary::CheckedCoreCpsClosedAdmission,
        "the source route must select the checked Core/CPS owner rather than the direct Expr evaluator"
    );
}

#[tokio::test]
async fn source_checked_body_rejects_at_application_admission_without_a_production_artifact() {
    let engine = Engine::new().build().expect("engine builds");
    let mut entry = engine
        .parse("fn main() -> Int { 7 }")
        .expect("source parses");
    engine.check(&mut entry).expect("source typechecks");

    let outcome = engine
        .admit_application(ApplicationAdmissionRequest {
            entry_name: "main".to_string(),
            body: entry.core.clone(),
            application_id: None,
            run_id: None,
            active_role: None,
            admitted_role: None,
            required_capabilities: Vec::new(),
            requires: Vec::new(),
            ensures: Vec::new(),
        })
        .await;

    assert!(matches!(
        outcome,
        ApplicationAdmissionOutcome::Rejected {
            failure,
            ..
        } if failure.kind == ApplicationFailureKind::AdmissionFailure
    ));
    assert_eq!(
        engine.production_execution_boundary(),
        ProductionExecutionBoundary::CheckedCoreCpsClosedAdmission,
        "application admission must reject before the direct Expr evaluator"
    );
}

#[tokio::test]
async fn source_unary_negation_without_validated_production_typed_lowering_rejects_before_direct_evaluation()
 {
    let engine = Engine::new().build().expect("engine builds");

    let error = engine
        .run(
            r"
            fn main() -> Int {
                do {
                    let value = 1;
                    return - value;
                }
            }
            ",
        )
        .await
        .expect_err(
            "unary negation remains outside validated production checked Core/CPS lowering and must reject at admission instead of returning a direct-evaluator result",
        );

    assert!(
        matches!(
            error,
            ExecError::ExecutionFailed(ref message)
                if message.contains("checked Core/CPS admission")
        ),
        "the public run route must classify the typechecked unary-negation lowering gap as closed admission, not execute the legacy expression evaluator: {error}",
    );
}
