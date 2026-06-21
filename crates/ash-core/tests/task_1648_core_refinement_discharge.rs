use ash_core::core_ash::{
    CoreAtom, CoreContractDischarge, CoreDischargeMode, CoreEvidenceSource, CoreEvidenceStatus,
    CoreExpr, CoreRefinementEvidence, CoreRow, CoreRowItem, CoreTrapReason, CoreType, CoreValue,
};
use ash_core::core_ash_typecheck::{
    CoreTypeCheckEnv, CoreTypeCheckError, check_core_type_well_formed, type_check_core_program,
};
use ash_core::core_ash_validate::{RawCoreProgram, validate_core_program};

const POSITIVE_PREDICATE: &str = "result > 0";
const NONZERO_PREDICATE: &str = "result != 0";
const POSITIVE_CONTRACT: &str = "requires-positive";

fn int_ty() -> CoreType {
    CoreType::Base("Int".into())
}

fn unit_ty() -> CoreType {
    CoreType::Base("Unit".into())
}

fn positive_int_ty() -> CoreType {
    refinement_ty(POSITIVE_PREDICATE)
}

fn refinement_ty(predicate: &str) -> CoreType {
    CoreType::Refinement {
        base: Box::new(int_ty()),
        predicate: predicate.to_owned(),
    }
}

fn env_with_predicates(predicates: &[&str]) -> CoreTypeCheckEnv {
    let mut env = CoreTypeCheckEnv::default();
    for predicate in predicates {
        env.discharges_mut()
            .insert_refinement_predicate((*predicate).to_owned());
    }
    env
}

fn type_check(
    expr: CoreExpr,
    env: &CoreTypeCheckEnv,
) -> Result<ash_core::core_ash_typecheck::TypedCoreProgram, CoreTypeCheckError> {
    let valid =
        validate_core_program(RawCoreProgram::new(expr)).expect("Core expression validates");
    type_check_core_program(valid, env)
}

fn let_val(name: &str, ty: CoreType, value: CoreValue, body: CoreExpr) -> CoreExpr {
    CoreExpr::LetVal {
        name: name.to_owned(),
        ty,
        value,
        body: Box::new(body),
    }
}

fn static_discharge() -> CoreContractDischarge {
    CoreContractDischarge {
        contract: POSITIVE_CONTRACT.into(),
        mode: CoreDischargeMode::Static,
        evidence: Some(CoreRefinementEvidence {
            source: CoreEvidenceSource::HoareClause,
            status: CoreEvidenceStatus::Proven,
            predicate: POSITIVE_PREDICATE.into(),
            diagnostic: None,
        }),
        source_span: None,
    }
}

fn evidence_discharge(status: CoreEvidenceStatus) -> CoreContractDischarge {
    CoreContractDischarge {
        contract: POSITIVE_CONTRACT.into(),
        mode: CoreDischargeMode::Evidence,
        evidence: Some(CoreRefinementEvidence {
            source: CoreEvidenceSource::ExternalProof(vec!["proofs".into(), "positive".into()]),
            status,
            predicate: POSITIVE_PREDICATE.into(),
            diagnostic: None,
        }),
        source_span: None,
    }
}

fn dynamic_discharge() -> CoreContractDischarge {
    CoreContractDischarge {
        contract: POSITIVE_CONTRACT.into(),
        mode: CoreDischargeMode::Dynamic,
        evidence: None,
        source_span: None,
    }
}

fn contract_row(contract: &str) -> CoreRow {
    CoreRow::closed(vec![CoreRowItem::Contract {
        contract: contract.to_owned(),
    }])
}

fn record_discharge(discharge: CoreContractDischarge) -> CoreExpr {
    CoreExpr::RecordDischarge {
        discharge,
        body: Box::new(CoreExpr::Atom(CoreAtom::LitUnit)),
    }
}

fn call_contract_function(name: &str) -> CoreExpr {
    CoreExpr::Call {
        func: CoreAtom::Var(name.to_owned()),
        args: Vec::new(),
    }
}

#[test]
fn checking_plain_base_value_as_refinement_emits_obligation_metadata() {
    let env = env_with_predicates(&[POSITIVE_PREDICATE]);
    let expr = let_val(
        "x",
        positive_int_ty(),
        CoreValue::Atom(CoreAtom::LitInt(7)),
        CoreExpr::Atom(CoreAtom::Var("x".into())),
    );

    let typed = type_check(expr, &env)
        .expect("plain Int checked against Int refinement should type-check with an obligation");

    assert_eq!(typed.ty(), &positive_int_ty());
    assert_eq!(typed.row(), &CoreRow::default());
    let obligations = typed.obligations();
    assert_eq!(obligations.len(), 1);
    assert_eq!(obligations[0].predicate(), POSITIVE_PREDICATE);
    assert_eq!(obligations[0].value_name(), Some("x"));
    assert_eq!(obligations[0].base_type(), &int_ty());
    assert_eq!(obligations[0].refinement_type(), &positive_int_ty());
}

#[test]
fn using_existing_refinement_at_base_type_emits_no_new_obligation() {
    let mut env = env_with_predicates(&[POSITIVE_PREDICATE]);
    env.values_mut()
        .insert("already_refined", positive_int_ty());
    let expr = let_val(
        "plain",
        int_ty(),
        CoreValue::Atom(CoreAtom::Var("already_refined".into())),
        CoreExpr::Atom(CoreAtom::Var("plain".into())),
    );

    let typed = type_check(expr, &env)
        .expect("refinement subtyping should allow an already-refined value at its base type");

    assert_eq!(typed.ty(), &int_ty());
    assert_eq!(typed.row(), &CoreRow::default());
    assert!(
        typed.obligations().is_empty(),
        "forgetting an existing refinement to its base type must not create another proof duty"
    );
}

#[test]
fn lambda_body_refinement_obligations_keep_local_owner_names() {
    let env = env_with_predicates(&[POSITIVE_PREDICATE]);
    let fn_ty = CoreType::Function {
        params: Vec::new(),
        result: Box::new(positive_int_ty()),
        row: CoreRow::default(),
    };
    let expr = let_val(
        "f",
        fn_ty.clone(),
        CoreValue::Lam {
            params: Vec::new(),
            row: CoreRow::default(),
            body: Box::new(let_val(
                "x",
                positive_int_ty(),
                CoreValue::Atom(CoreAtom::LitInt(7)),
                CoreExpr::Atom(CoreAtom::Var("x".into())),
            )),
        },
        CoreExpr::Atom(CoreAtom::Var("f".into())),
    );

    let typed = type_check(expr, &env)
        .expect("lambda body refinement obligations should be preserved for public metadata");

    assert_eq!(typed.ty(), &fn_ty);
    let obligations = typed.obligations();
    assert_eq!(obligations.len(), 1);
    assert_eq!(obligations[0].value_name(), Some("x"));
    assert_eq!(obligations[0].predicate(), POSITIVE_PREDICATE);
}

#[test]
fn unknown_predicate_metadata_stays_rejected_by_well_formedness() {
    let err = check_core_type_well_formed(
        &refinement_ty(NONZERO_PREDICATE),
        &CoreTypeCheckEnv::default(),
    )
    .expect_err("textual refinement predicates require scoped predicate metadata");

    assert_eq!(
        err,
        CoreTypeCheckError::UnknownRefinementPredicate {
            predicate: NONZERO_PREDICATE.into()
        }
    );
}

#[test]
fn record_discharge_accepts_static_evidence_and_dynamic_shapes_and_records_metadata() {
    let env = env_with_predicates(&[POSITIVE_PREDICATE]);

    for discharge in [
        static_discharge(),
        evidence_discharge(CoreEvidenceStatus::Proven),
        dynamic_discharge(),
    ] {
        let typed = type_check(record_discharge(discharge.clone()), &env)
            .expect("coherent discharge metadata should type-check");

        assert_eq!(typed.ty(), &unit_ty());
        assert_eq!(typed.row(), &CoreRow::default());
        assert_eq!(typed.discharges(), &[discharge]);
        assert!(
            typed.obligations().is_empty(),
            "recorded discharge metadata must not become a residual obligation"
        );
    }
}

#[test]
fn record_discharge_removes_matching_contract_from_residual_row() {
    let mut env = env_with_predicates(&[POSITIVE_PREDICATE]);
    env.values_mut().insert(
        "checked_unit",
        CoreType::Function {
            params: Vec::new(),
            result: Box::new(unit_ty()),
            row: contract_row(POSITIVE_CONTRACT),
        },
    );
    let discharge = dynamic_discharge();
    let expr = CoreExpr::RecordDischarge {
        discharge: discharge.clone(),
        body: Box::new(call_contract_function("checked_unit")),
    };

    let typed = type_check(expr, &env)
        .expect("record discharge should remove its matching contract requirement");

    assert_eq!(typed.ty(), &unit_ty());
    assert_eq!(
        typed.row(),
        &CoreRow::default(),
        "discharged contract requirements must not remain residual"
    );
    assert_eq!(typed.discharges(), &[discharge]);
}

#[test]
fn record_discharge_rejects_malformed_static_evidence_and_dynamic_shapes() {
    let env = env_with_predicates(&[POSITIVE_PREDICATE]);
    let malformed = [
        CoreContractDischarge {
            evidence: None,
            ..static_discharge()
        },
        CoreContractDischarge {
            evidence: Some(CoreRefinementEvidence {
                source: CoreEvidenceSource::Assumption(vec!["runtime".into()]),
                status: CoreEvidenceStatus::Unknown,
                predicate: POSITIVE_PREDICATE.into(),
                diagnostic: Some("not-static-proof".into()),
            }),
            ..dynamic_discharge()
        },
        CoreContractDischarge {
            evidence: None,
            ..evidence_discharge(CoreEvidenceStatus::Proven)
        },
    ];

    for discharge in malformed {
        let err = type_check(record_discharge(discharge), &env)
            .expect_err("malformed discharge mode/evidence combinations must be rejected");

        assert!(
            matches!(err, CoreTypeCheckError::InvalidDischarge { .. }),
            "expected invalid discharge diagnostic, got {err:?}"
        );
    }
}

#[test]
fn disproved_or_statistical_evidence_does_not_satisfy_hard_refinement() {
    let env = env_with_predicates(&[POSITIVE_PREDICATE]);

    for status in [
        CoreEvidenceStatus::Disproved,
        CoreEvidenceStatus::Statistical,
    ] {
        let err = type_check(record_discharge(evidence_discharge(status)), &env)
            .expect_err("non-proven evidence must not discharge a hard refinement");

        assert!(
            matches!(err, CoreTypeCheckError::InvalidDischarge { .. }),
            "expected invalid discharge diagnostic for {status:?}, got {err:?}"
        );
    }
}

#[test]
fn contract_violation_trap_stays_row_free_and_not_obligation_or_discharge_value() {
    let env = env_with_predicates(&[POSITIVE_PREDICATE]);
    let expr = CoreExpr::If {
        cond: CoreAtom::LitBool(false),
        then_branch: Box::new(CoreExpr::Atom(CoreAtom::LitInt(1))),
        else_branch: Box::new(CoreExpr::Trap {
            reason: CoreTrapReason::ContractViolation(POSITIVE_CONTRACT.into()),
        }),
    };

    let typed = type_check(expr, &env)
        .expect("contract violation traps should type-check as trap metadata");

    assert_eq!(typed.ty(), &int_ty());
    assert_eq!(typed.row(), &CoreRow::default());
    assert!(typed.obligations().is_empty());
    assert!(typed.discharges().is_empty());
}
