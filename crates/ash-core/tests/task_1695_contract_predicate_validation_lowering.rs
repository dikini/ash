use ash_core::core_ash::{CoreSourceSpan, CoreType};
use ash_core::core_ash_contract::{
    ContractPredicateExpr, ContractPredicateLoweringError, ContractRecoverability, CoreBlameLabel,
    CoreBlameParty, CoreBlamePolarity, PredicateBinder, PredicateBinderKind,
    PredicateClassification, PredicateEnvironment, PredicateFunctionRef, PredicateNode,
    lower_contract_predicate,
};

fn bool_ty() -> CoreType {
    CoreType::Base("Bool".into())
}

fn int_ty() -> CoreType {
    CoreType::Base("Int".into())
}

fn span(start: usize, end: usize) -> CoreSourceSpan {
    CoreSourceSpan {
        file: Some("task_1695.ash".into()),
        start,
        end,
    }
}

fn binder(boundary: &str, id: &str, name: &str, ty: CoreType) -> PredicateBinder {
    PredicateBinder::new(
        boundary,
        id,
        name,
        PredicateBinderKind::Parameter,
        ty,
        span(0, 1),
    )
}

fn dynamic_blame(boundary: &str) -> CoreBlameLabel {
    CoreBlameLabel::new(
        CoreBlameParty::Callee,
        CoreBlamePolarity::Positive,
        boundary,
    )
}

#[test]
fn smt_safe_predicate_lowers_to_static_obligation_without_runtime_plan() {
    let boundary = "fn:absolute:ensures";
    let result = binder(boundary, "result", "result", int_ty());
    let env = PredicateEnvironment::new(boundary, vec![result.clone()], Vec::new(), Vec::new());

    let lowered = lower_contract_predicate(
        boundary,
        env,
        ContractPredicateExpr::Ge(
            Box::new(ContractPredicateExpr::Binder(result.ref_())),
            Box::new(ContractPredicateExpr::IntLit(0)),
        ),
        bool_ty(),
        span(10, 21),
        "result >= 0",
        dynamic_blame(boundary),
        ContractRecoverability::TrapDefault,
    )
    .expect("SMT-safe comparison should lower");

    assert_eq!(
        lowered.predicate.classification(),
        PredicateClassification::Static
    );
    assert_eq!(lowered.proof_obligations.len(), 1);
    assert!(lowered.runtime_check.is_none());
    assert!(matches!(lowered.predicate.root(), PredicateNode::Ge(_, _)));
}

#[test]
fn pure_non_smt_predicate_lowers_to_runtime_check_plan() {
    let boundary = "fn:push:ensures";
    let xs = binder(boundary, "xs", "xs", CoreType::Named("List".into()));
    let sorted = PredicateFunctionRef::new(
        vec!["contracts".into(), "sorted".into()],
        vec![xs.ty().clone()],
        bool_ty(),
    );
    let env =
        PredicateEnvironment::new(boundary, vec![xs.clone()], Vec::new(), vec![sorted.clone()]);

    let lowered = lower_contract_predicate(
        boundary,
        env,
        ContractPredicateExpr::PredicateCall {
            callee: sorted,
            args: vec![ContractPredicateExpr::Binder(xs.ref_())],
            smt_safe: false,
        },
        bool_ty(),
        span(0, 10),
        "sorted(xs)",
        dynamic_blame(boundary),
        ContractRecoverability::TrapDefault,
    )
    .expect("admitted pure non-SMT predicate should lower dynamically");

    assert_eq!(
        lowered.predicate.classification(),
        PredicateClassification::Dynamic
    );
    let runtime = lowered
        .runtime_check
        .expect("dynamic predicates need runtime checks");
    assert_eq!(runtime.predicate(), lowered.predicate.predicate_ref());
    assert!(lowered.proof_obligations.is_empty());
}

#[test]
fn old_path_lowers_to_boundary_local_snapshot_ref() {
    let boundary = "fn:push:ensures";
    let xs = binder(boundary, "xs", "xs", CoreType::Named("List".into()));
    let env = PredicateEnvironment::new(boundary, vec![xs.clone()], Vec::new(), Vec::new());

    let lowered = lower_contract_predicate(
        boundary,
        env,
        ContractPredicateExpr::Eq(
            Box::new(ContractPredicateExpr::OldPath {
                root: xs.ref_(),
                path: vec!["len".into()],
                ty: int_ty(),
                source_span: span(20, 31),
            }),
            Box::new(ContractPredicateExpr::IntLit(3)),
        ),
        bool_ty(),
        span(0, 40),
        "old(xs.len) == 3",
        dynamic_blame(boundary),
        ContractRecoverability::TrapDefault,
    )
    .expect("field path old(...) should lower to SnapshotRef");

    assert_eq!(lowered.predicate.snapshot_refs().len(), 1);
    assert_eq!(
        lowered.predicate.snapshot_refs()[0].boundary().as_str(),
        boundary
    );
}

#[test]
fn rejected_predicates_do_not_produce_runtime_or_proof_artifacts() {
    let boundary = "fn:push:ensures";
    let env = PredicateEnvironment::new(boundary, Vec::new(), Vec::new(), Vec::new());

    let error = lower_contract_predicate(
        boundary,
        env,
        ContractPredicateExpr::CapabilityCall {
            path: vec!["PosixFs".into()],
            operation: "read".into(),
            source_span: span(4, 18),
        },
        bool_ty(),
        span(0, 20),
        "PosixFs.read()",
        dynamic_blame(boundary),
        ContractRecoverability::TrapDefault,
    )
    .expect_err("capability calls must be rejected, not lowered dynamically");

    assert_eq!(
        error,
        ContractPredicateLoweringError::ForbiddenCapabilityCall {
            source_span: span(4, 18)
        }
    );
}

#[test]
fn arbitrary_computation_inside_old_is_rejected_before_snapshot_artifacts_exist() {
    let boundary = "fn:push:ensures";
    let env = PredicateEnvironment::new(boundary, Vec::new(), Vec::new(), Vec::new());

    let error = lower_contract_predicate(
        boundary,
        env,
        ContractPredicateExpr::OldComputation {
            source_span: span(0, 12),
        },
        bool_ty(),
        span(0, 12),
        "old(f(xs))",
        dynamic_blame(boundary),
        ContractRecoverability::TrapDefault,
    )
    .expect_err("old(...) accepts paths, not arbitrary computation");

    assert_eq!(
        error,
        ContractPredicateLoweringError::InvalidSnapshotPath {
            source_span: span(0, 12)
        }
    );
}

#[test]
fn binder_from_outside_predicate_environment_is_rejected() {
    let boundary = "fn:push:ensures";
    let foreign = binder("fn:other:ensures", "x", "x", int_ty());
    let env = PredicateEnvironment::new(boundary, Vec::new(), Vec::new(), Vec::new());

    let error = lower_contract_predicate(
        boundary,
        env,
        ContractPredicateExpr::Binder(foreign.ref_()),
        bool_ty(),
        span(0, 1),
        "x",
        dynamic_blame(boundary),
        ContractRecoverability::TrapDefault,
    )
    .expect_err("predicate binders must be admitted by the predicate environment");

    assert_eq!(
        error,
        ContractPredicateLoweringError::UnknownPredicateBinder {
            binder: foreign.ref_()
        }
    );
}

#[test]
fn unadmitted_predicate_function_is_rejected() {
    let boundary = "fn:push:ensures";
    let xs = binder(boundary, "xs", "xs", CoreType::Named("List".into()));
    let sorted = PredicateFunctionRef::new(
        vec!["contracts".into(), "sorted".into()],
        vec![xs.ty().clone()],
        bool_ty(),
    );
    let env = PredicateEnvironment::new(boundary, vec![xs.clone()], Vec::new(), Vec::new());

    let error = lower_contract_predicate(
        boundary,
        env,
        ContractPredicateExpr::PredicateCall {
            callee: sorted.clone(),
            args: vec![ContractPredicateExpr::Binder(xs.ref_())],
            smt_safe: false,
        },
        bool_ty(),
        span(0, 10),
        "sorted(xs)",
        dynamic_blame(boundary),
        ContractRecoverability::TrapDefault,
    )
    .expect_err("predicate functions must be admitted by the environment");

    assert_eq!(
        error,
        ContractPredicateLoweringError::UnadmittedPredicateFunction {
            function: Box::new(sorted)
        }
    );
}
