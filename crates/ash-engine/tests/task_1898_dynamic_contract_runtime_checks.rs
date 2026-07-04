//! TASK-1898: Dynamic contract runtime checks integration tests.
//!
//! These integration tests exercise the engine/runtime boundary for dynamic
//! contract checks: a callable workflow registered with a dynamic contract
//! discharge record traps on false `requires`/`ensures` predicates and on
//! predicate evaluator faults, while true predicates allow execution to succeed.

use ash_core::{
    Expr, Span, Value, Workflow,
    core_ash::{CoreName, CoreSourceSpan, CoreType},
    core_ash_contract::{
        BoundaryKind, ContractDischargeRecord, ContractRecoverability, CoreBlameLabel,
        CoreBlameParty, CoreBlamePolarity, CoreBoundaryId, DiagnosticShape, DynamicPredicatePlan,
        PredicateBinder, PredicateBinderKind, PredicateEnvironment, PredicateNode, PredicateRef,
        RuntimeCheckPlan,
    },
};
use ash_engine::Engine;
use ash_interp::ExecError;

const fn zero_span() -> CoreSourceSpan {
    CoreSourceSpan {
        file: None,
        start: 0,
        end: 0,
    }
}

const fn expr_span() -> Span {
    Span { start: 0, end: 0 }
}

fn int_type() -> CoreType {
    CoreType::Base("Int".into())
}

fn requires_boundary(name: &str) -> CoreBoundaryId {
    CoreBoundaryId::new(format!("fn:{name}:requires"))
}

fn ensures_boundary(name: &str) -> CoreBoundaryId {
    CoreBoundaryId::new(format!("fn:{name}:ensures"))
}

fn parameter_binder(boundary: CoreBoundaryId, local: &str, name: &str) -> PredicateBinder {
    PredicateBinder::new(
        boundary,
        local,
        CoreName::from(name),
        PredicateBinderKind::Parameter,
        int_type(),
        zero_span(),
    )
}

fn result_binder(boundary: CoreBoundaryId) -> PredicateBinder {
    PredicateBinder::new(
        boundary,
        "result",
        CoreName::from("result"),
        PredicateBinderKind::Result,
        int_type(),
        zero_span(),
    )
}

fn parameter_node(boundary: CoreBoundaryId, local: &str) -> PredicateNode {
    PredicateNode::Binder(parameter_binder(boundary, local, local).ref_())
}

fn result_node(boundary: CoreBoundaryId) -> PredicateNode {
    PredicateNode::Result(result_binder(boundary).ref_())
}

fn make_predicate_ref(boundary: CoreBoundaryId) -> PredicateRef {
    PredicateRef {
        id: ash_core::core_ash_contract::PredicateId::new(format!("pred:{}", boundary.as_str())),
        stable_hash: ash_core::core_ash_contract::PredicateHash::new("0000"),
        boundary,
        source_span: None,
    }
}

fn runtime_check_plan(
    boundary: CoreBoundaryId,
    binders: Vec<PredicateBinder>,
    predicate: PredicateNode,
    blame: CoreBlameLabel,
    boundary_kind: BoundaryKind,
) -> RuntimeCheckPlan {
    let env = PredicateEnvironment::new(boundary.clone(), binders, vec![], vec![]);
    let plan = DynamicPredicatePlan::Interpreter {
        boundary_kind,
        environment_binders: env.binders().to_vec(),
        predicate_node: predicate,
    };
    RuntimeCheckPlan::new(
        make_predicate_ref(boundary),
        env.ref_(),
        plan,
        blame,
        vec![],
        DiagnosticShape::predicate_false("contract.false"),
        ContractRecoverability::TrapDefault,
    )
}

fn register_dynamic_requires(engine: &mut Engine, callable_name: &str, predicate: PredicateNode) {
    let boundary = requires_boundary(callable_name);
    let binders = vec![parameter_binder(boundary.clone(), "x", "x")];
    let blame = CoreBlameLabel::new(
        CoreBlameParty::Caller,
        CoreBlamePolarity::Negative,
        boundary.clone(),
    );
    let plan = runtime_check_plan(
        boundary.clone(),
        binders,
        predicate,
        blame,
        BoundaryKind::Requires,
    );
    let record = ContractDischargeRecord::dynamic(callable_name, boundary, plan, zero_span(), None);

    let stub = engine
        .parse("fn stub() -> Int { 0 }\nworkflow main { ret 0 }")
        .expect("stub workflow parses");
    engine.set_contract_discharge_for_callable(callable_name, record, &stub);
}

fn register_dynamic_ensures(engine: &mut Engine, callable_name: &str, predicate: PredicateNode) {
    let boundary = ensures_boundary(callable_name);
    let binders = vec![
        parameter_binder(boundary.clone(), "x", "x"),
        result_binder(boundary.clone()),
    ];
    let blame = CoreBlameLabel::new(
        CoreBlameParty::Callee,
        CoreBlamePolarity::Positive,
        boundary.clone(),
    );
    let plan = runtime_check_plan(
        boundary.clone(),
        binders,
        predicate,
        blame,
        BoundaryKind::Ensures,
    );
    let record = ContractDischargeRecord::dynamic(callable_name, boundary, plan, zero_span(), None);

    let stub = engine
        .parse("fn stub() -> Int { 0 }\nworkflow main { ret 0 }")
        .expect("stub workflow parses");
    engine.set_contract_discharge_for_callable(callable_name, record, &stub);
}

fn identity_callable_workflow() -> Workflow {
    Workflow::Ret {
        expr: Expr::Variable {
            name: "x".to_string(),
            span: expr_span(),
        },
    }
}

fn caller_workflow(arg: Value) -> Workflow {
    Workflow::Call {
        target: "identity".to_string(),
        arguments: vec![Expr::Literal(arg)],
        continuation: Box::new(Workflow::Ret {
            expr: Expr::Literal(Value::Int(99)),
        }),
    }
}

#[tokio::test]
async fn requires_false_traps_with_caller_blame() {
    let mut engine = Engine::new().build().expect("engine builds");
    register_dynamic_requires(&mut engine, "identity", PredicateNode::BoolLit(false));
    engine
        .register_callable_workflow_with_params(
            "identity",
            identity_callable_workflow(),
            1,
            vec!["x".to_string()],
        )
        .await;

    let mut caller = engine
        .parse("workflow main { ret 0 }")
        .expect("caller parses");
    caller.core = caller_workflow(Value::Int(1));

    let err = engine
        .execute(&caller)
        .await
        .expect_err("false requires should trap");

    match err {
        ExecError::ContractViolation(diagnostic) => {
            assert_eq!(diagnostic.blame().party, CoreBlameParty::Caller);
            assert_eq!(diagnostic.blame().polarity, CoreBlamePolarity::Negative);
        }
        other => panic!("expected ContractViolation, got {other:?}"),
    }
}

#[tokio::test]
async fn ensures_false_traps_with_callee_blame() {
    let mut engine = Engine::new().build().expect("engine builds");
    let boundary = ensures_boundary("identity");
    let predicate = PredicateNode::Ne(
        Box::new(result_node(boundary.clone())),
        Box::new(parameter_node(boundary.clone(), "x")),
    );
    register_dynamic_ensures(&mut engine, "identity", predicate);

    engine
        .register_callable_workflow_with_params(
            "identity",
            identity_callable_workflow(),
            1,
            vec!["x".to_string()],
        )
        .await;

    let mut caller = engine
        .parse("workflow main { ret 0 }")
        .expect("caller parses");
    caller.core = caller_workflow(Value::Int(1));

    let err = engine
        .execute(&caller)
        .await
        .expect_err("false ensures should trap");

    match err {
        ExecError::ContractViolation(diagnostic) => {
            assert_eq!(diagnostic.blame().party, CoreBlameParty::Callee);
            assert_eq!(diagnostic.blame().polarity, CoreBlamePolarity::Positive);
        }
        other => panic!("expected ContractViolation, got {other:?}"),
    }
}

#[tokio::test]
async fn missing_binder_yields_predicate_fault() {
    let mut engine = Engine::new().build().expect("engine builds");
    let boundary = requires_boundary("identity");
    // Reference a binder that is not present in the predicate environment.
    let predicate = PredicateNode::Binder(
        PredicateBinder::new(
            boundary.clone(),
            "missing",
            CoreName::from("missing"),
            PredicateBinderKind::Parameter,
            int_type(),
            zero_span(),
        )
        .ref_(),
    );
    register_dynamic_requires(&mut engine, "identity", predicate);

    engine
        .register_callable_workflow_with_params(
            "identity",
            identity_callable_workflow(),
            1,
            vec!["x".to_string()],
        )
        .await;

    let mut caller = engine
        .parse("workflow main { ret 0 }")
        .expect("caller parses");
    caller.core = caller_workflow(Value::Int(1));

    let err = engine
        .execute(&caller)
        .await
        .expect_err("missing binder should produce a predicate fault");

    assert!(
        matches!(err, ExecError::ContractPredicateFault(_)),
        "expected ContractPredicateFault, got {err:?}"
    );
}

#[tokio::test]
async fn requires_true_and_ensures_true_allow_execution() {
    let mut engine = Engine::new().build().expect("engine builds");
    register_dynamic_requires(&mut engine, "identity", PredicateNode::BoolLit(true));

    let ens_boundary = ensures_boundary("identity");
    let predicate = PredicateNode::Eq(
        Box::new(result_node(ens_boundary.clone())),
        Box::new(parameter_node(ens_boundary.clone(), "x")),
    );
    register_dynamic_ensures(&mut engine, "identity", predicate);

    engine
        .register_callable_workflow_with_params(
            "identity",
            identity_callable_workflow(),
            1,
            vec!["x".to_string()],
        )
        .await;

    let mut caller = engine
        .parse("workflow main { ret 0 }")
        .expect("caller parses");
    caller.core = caller_workflow(Value::Int(42));

    let result = engine
        .execute(&caller)
        .await
        .expect("true requires and ensures should allow execution");

    assert_eq!(result, Value::Int(99));
}
