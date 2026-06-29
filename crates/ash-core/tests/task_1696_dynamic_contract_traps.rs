use ash_core::core_ash::{CoreExpr, CoreRow, CoreRowItem, CoreTrapReason, CoreType};
use ash_core::core_ash_contract::{
    ContractDiagnostic, CoreBlameLabel, CoreBlameParty, CoreBlamePolarity, DiagnosticShape,
    LoweredPredicateBuilder, PredicateEnvironment, PredicateFault, PredicateFaultDiagnostic,
    PredicateNode,
};
use ash_core::core_ash_lower::lower_core_program;
use ash_core::core_ash_text::{core_expr_to_string, parse_core_expr};
use ash_core::core_ash_typecheck::{CoreTypeCheckEnv, type_check_core_program};
use ash_core::core_ash_validate::{RawCoreProgram, validate_core_program};
use ash_core::cps::{Term, TrapReason};

fn bool_ty() -> CoreType {
    CoreType::Base("Bool".into())
}

fn blame(boundary: &str) -> CoreBlameLabel {
    CoreBlameLabel::new(
        CoreBlameParty::Callee,
        CoreBlamePolarity::Positive,
        boundary,
    )
}

fn predicate(boundary: &str) -> ash_core::core_ash_contract::LoweredPredicate {
    LoweredPredicateBuilder::new(
        boundary,
        PredicateEnvironment::new(boundary, Vec::new(), Vec::new(), Vec::new()),
        PredicateNode::BoolLit(false),
        bool_ty(),
    )
    .diagnostic_shape(DiagnosticShape::predicate_false("false-predicate"))
    .build()
}

#[test]
fn false_predicate_and_evaluator_fault_have_distinct_trap_reasons() {
    let boundary = "fn:push:ensures";
    let pred = predicate(boundary);
    let violation = ContractDiagnostic::new(
        pred.predicate_ref().clone(),
        "result > 0",
        blame(boundary),
        pred.classification(),
        Vec::new(),
    );
    let fault = PredicateFaultDiagnostic::new(
        pred.predicate_ref().clone(),
        "result > 0",
        blame(boundary),
        PredicateFault::EvaluatorTrap("division by zero".into()),
        Vec::new(),
    );

    let false_trap = CoreTrapReason::ContractViolationDiagnostic(violation.clone());
    let fault_trap = CoreTrapReason::ContractPredicateFault(fault.clone());

    assert_ne!(false_trap, fault_trap);
    assert!(
        matches!(false_trap, CoreTrapReason::ContractViolationDiagnostic(diag) if diag.predicate() == violation.predicate())
    );
    assert!(
        matches!(fault_trap, CoreTrapReason::ContractPredicateFault(diag) if diag.fault() == fault.fault())
    );
}

#[test]
fn contract_diagnostic_trap_typechecks_with_empty_local_row() {
    let boundary = "fn:push:ensures";
    let pred = predicate(boundary);
    let diagnostic = ContractDiagnostic::new(
        pred.predicate_ref().clone(),
        "result > 0",
        blame(boundary),
        pred.classification(),
        pred.snapshot_refs().to_vec(),
    );
    let expr = CoreExpr::Trap {
        reason: CoreTrapReason::ContractViolationDiagnostic(diagnostic),
    };

    let valid = validate_core_program(RawCoreProgram::new(expr)).expect("trap validates");
    let typed =
        type_check_core_program(valid, &CoreTypeCheckEnv::default()).expect("trap typechecks");

    assert_eq!(typed.row(), &CoreRow::default());
}

#[test]
fn contract_violation_diagnostic_is_not_a_row_item() {
    let row = CoreRow::closed(vec![CoreRowItem::Failure { ty: None }]);

    assert!(row.items.iter().all(|item| !matches!(item, CoreRowItem::Contract { contract } if contract == "ContractViolation")));
}

#[test]
fn structured_contract_trap_payload_survives_core_to_cps_lowering() {
    let boundary = "fn:push:ensures";
    let pred = predicate(boundary);
    let diagnostic = ContractDiagnostic::new(
        pred.predicate_ref().clone(),
        "result > 0",
        blame(boundary),
        pred.classification(),
        pred.snapshot_refs().to_vec(),
    );
    let expr = CoreExpr::Trap {
        reason: CoreTrapReason::ContractViolationDiagnostic(diagnostic.clone()),
    };
    let valid = validate_core_program(RawCoreProgram::new(expr)).expect("trap validates");
    let lowered = lower_core_program(valid).expect("trap lowers");

    assert!(
        matches!(lowered, Term::Trap { reason: TrapReason::ContractViolationDiagnostic(payload) } if payload.as_ref() == &diagnostic)
    );
}

#[test]
fn structured_contract_trap_payload_round_trips_through_core_text() {
    let boundary = "fn:push:ensures";
    let pred = predicate(boundary);
    let diagnostic = ContractDiagnostic::new(
        pred.predicate_ref().clone(),
        "result > 0",
        blame(boundary),
        pred.classification(),
        pred.snapshot_refs().to_vec(),
    );
    let expr = CoreExpr::Trap {
        reason: CoreTrapReason::ContractViolationDiagnostic(diagnostic),
    };

    let text = core_expr_to_string(&expr);
    let parsed = parse_core_expr(&text).expect("formatted structured trap parses");

    assert_eq!(parsed, expr);
}
