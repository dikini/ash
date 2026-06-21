use ash_core::core_ash::{CoreAtom, CoreExpr, CoreRow, CoreRowItem, CoreType, CoreValue};
use ash_core::core_ash_typecheck::{
    CorePublicFunctionSummary, CorePublicRowItemSummary, CorePublicSummaryError, CoreTypeCheckEnv,
    summarize_core_public_function_type, summarize_core_public_row, type_check_core_program,
};
use ash_core::core_ash_validate::{RawCoreProgram, validate_core_program};

const POSITIVE_PREDICATE: &str = "result > 0";

fn int_ty() -> CoreType {
    CoreType::Base("Int".into())
}

fn positive_int_ty() -> CoreType {
    CoreType::Refinement {
        base: Box::new(int_ty()),
        predicate: POSITIVE_PREDICATE.into(),
    }
}

fn cap(path: &[&str], operation: &str) -> CoreRowItem {
    CoreRowItem::Capability {
        path: path.iter().map(|part| (*part).to_owned()).collect(),
        operation: operation.to_owned(),
    }
}

fn evidence(path: &[&str]) -> CoreRowItem {
    CoreRowItem::Evidence {
        path: path.iter().map(|part| (*part).to_owned()).collect(),
    }
}

fn contract(contract: &str) -> CoreRowItem {
    CoreRowItem::Contract {
        contract: contract.to_owned(),
    }
}

fn public_function(row: CoreRow, result: CoreType) -> CoreType {
    CoreType::Function {
        params: vec![int_ty()],
        result: Box::new(result),
        row,
    }
}

fn app_ty(name: &str, args: Vec<CoreType>) -> CoreType {
    CoreType::App {
        name: name.to_owned(),
        args,
    }
}

fn env_with_positive_predicate() -> CoreTypeCheckEnv {
    let mut env = CoreTypeCheckEnv::default();
    env.discharges_mut()
        .insert_refinement_predicate(POSITIVE_PREDICATE);
    env
}

#[test]
fn public_row_summary_preserves_row_item_namespaces() {
    let row = CoreRow::closed(vec![
        cap(&["audit"], "record"),
        evidence(&["audit", "record"]),
        contract("audit.record"),
    ]);

    let summary = summarize_core_public_row(&row).expect("public row summarizes");

    assert_eq!(
        summary.items(),
        &[
            CorePublicRowItemSummary::Capability {
                path: vec!["audit".into()],
                operation: "record".into(),
            },
            CorePublicRowItemSummary::Evidence {
                path: vec!["audit".into(), "record".into()],
            },
            CorePublicRowItemSummary::Contract {
                contract: "audit.record".into(),
            },
        ],
        "public summaries must keep cap/evidence/contract namespaces distinct"
    );
}

#[test]
fn private_group_reference_in_public_row_is_rejected_before_export() {
    let row = CoreRow::closed(vec![CoreRowItem::EffectGroupRef {
        path: vec!["private".into(), "io".into()],
    }]);

    let err = summarize_core_public_row(&row)
        .expect_err("private or ambiguous group references must not leak into a public summary");

    assert_eq!(
        err,
        CorePublicSummaryError::PrivateRowReference {
            path: vec!["private".into(), "io".into()],
            public_item: None,
            detail: "private effect group private.io must be expanded or exported before summary"
                .into(),
        }
    );
}

#[test]
fn public_function_summary_preserves_refinement_obligation_identity_metadata() {
    let env = env_with_positive_predicate();
    let expr = CoreExpr::LetVal {
        name: "x".into(),
        ty: positive_int_ty(),
        value: CoreValue::Atom(CoreAtom::LitInt(7)),
        body: Box::new(CoreExpr::Atom(CoreAtom::Var("x".into()))),
    };
    let valid =
        validate_core_program(RawCoreProgram::new(expr)).expect("Core expression validates");
    let typed = type_check_core_program(valid, &env)
        .expect("plain Int checked against refinement emits an obligation");
    assert_eq!(typed.obligations().len(), 1);

    let summary: CorePublicFunctionSummary = summarize_core_public_function_type(
        "checked_positive",
        &public_function(CoreRow::default(), positive_int_ty()),
        typed.obligations(),
        typed.discharges(),
    )
    .expect("public function type summarizes");

    assert_eq!(summary.exported_name(), "checked_positive");
    assert_eq!(
        summary.refinement_obligations().len(),
        1,
        "TASK-1648 obligation identity must survive into public summary metadata"
    );
    let obligation = &summary.refinement_obligations()[0];
    assert_eq!(obligation.predicate(), POSITIVE_PREDICATE);
    assert_eq!(obligation.value_name(), Some("x"));
    assert_eq!(obligation.base_type(), &int_ty());
    assert_eq!(obligation.refinement_type(), &positive_int_ty());
}

#[test]
fn public_function_summary_preserves_type_constructor_identity_and_arity() {
    let ty = CoreType::Function {
        params: vec![app_ty("Box", vec![int_ty()])],
        result: Box::new(app_ty(
            "Result",
            vec![int_ty(), CoreType::Base("String".into())],
        )),
        row: CoreRow::default(),
    };

    let summary = summarize_core_public_function_type("boxed_result", &ty, &[], &[])
        .expect("public function type summarizes");

    let constructors = summary.type_constructors();
    assert_eq!(constructors.len(), 2);
    assert_eq!(constructors[0].name(), "Box");
    assert_eq!(constructors[0].arity(), 1);
    assert_eq!(constructors[1].name(), "Result");
    assert_eq!(constructors[1].arity(), 2);
}

#[test]
fn public_function_summary_preserves_row_payload_type_constructors() {
    let ty = CoreType::Function {
        params: vec![int_ty()],
        result: Box::new(CoreType::Base("Unit".into())),
        row: CoreRow::closed(vec![
            CoreRowItem::Channel {
                path: vec!["events".into()],
                mode: "recv".into(),
                payload_type: Box::new(app_ty("Event", vec![int_ty()])),
            },
            CoreRowItem::Failure {
                ty: Some(Box::new(app_ty(
                    "Result",
                    vec![int_ty(), CoreType::Base("String".into())],
                ))),
            },
        ]),
    };

    let summary = summarize_core_public_function_type("row_payloads", &ty, &[], &[])
        .expect("public function type summarizes row payload constructors");

    let constructors = summary.type_constructors();
    assert_eq!(constructors.len(), 2);
    assert_eq!(constructors[0].name(), "Event");
    assert_eq!(constructors[0].arity(), 1);
    assert_eq!(constructors[1].name(), "Result");
    assert_eq!(constructors[1].arity(), 2);
}
