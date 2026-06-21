use ash_core::core_ash::{
    CoreAtom, CoreContractDischarge, CoreDischargeMode, CoreEffectOp, CoreExpr, CoreParam,
    CorePrimOp, CoreRow, CoreRowItem, CoreType, CoreValue,
};
use ash_core::core_ash_typecheck::{
    CoreTypeCheckEnv, CoreTypeCheckError, core_types_equivalent, synthesize_core_atom,
    synthesize_core_value, type_check_core_program,
};
use ash_core::core_ash_validate::{RawCoreProgram, validate_core_program};

fn cap(path: &[&str], operation: &str) -> CoreRowItem {
    CoreRowItem::Capability {
        path: path.iter().map(|part| (*part).to_owned()).collect(),
        operation: operation.to_owned(),
    }
}

fn chan(path: &[&str], mode: &str, payload: CoreType) -> CoreRowItem {
    CoreRowItem::Channel {
        path: path.iter().map(|part| (*part).to_owned()).collect(),
        mode: mode.to_owned(),
        payload_type: Box::new(payload),
    }
}

fn channel_op(path: &[&str], mode: &str, payload: CoreType) -> CoreEffectOp {
    CoreEffectOp::Channel {
        path: path.iter().map(|part| (*part).to_owned()).collect(),
        mode: mode.to_owned(),
        payload_type: payload,
        result_type: CoreType::Base("Unit".into()),
    }
}

#[test]
fn literals_synthesize_builtin_base_types() {
    let env = CoreTypeCheckEnv::default();

    assert_eq!(
        synthesize_core_atom(&CoreAtom::LitInt(1), &env).expect("int literal type"),
        CoreType::Base("Int".into())
    );
    assert_eq!(
        synthesize_core_atom(&CoreAtom::LitString("x".into()), &env).expect("string literal type"),
        CoreType::Base("String".into())
    );
    assert_eq!(
        synthesize_core_atom(&CoreAtom::LitBool(true), &env).expect("bool literal type"),
        CoreType::Base("Bool".into())
    );
    assert_eq!(
        synthesize_core_atom(&CoreAtom::LitUnit, &env).expect("unit literal type"),
        CoreType::Base("Unit".into())
    );
}

#[test]
fn unknown_variable_atom_fails() {
    let err = synthesize_core_atom(
        &CoreAtom::Var("missing".into()),
        &CoreTypeCheckEnv::default(),
    )
    .expect_err("unknown variable is rejected");

    assert_eq!(
        err,
        CoreTypeCheckError::UnknownValue {
            name: "missing".into()
        }
    );
}

#[test]
fn primitive_and_constructor_names_synthesize_function_types() {
    let mut env = CoreTypeCheckEnv::default();
    env.types_mut().insert_name("OptionInt");
    env.types_mut().insert_value_constructor(
        "SomeInt",
        CoreType::Function {
            params: vec![CoreType::Base("Int".into())],
            result: Box::new(CoreType::Named("OptionInt".into())),
            row: CoreRow::default(),
        },
    );

    assert_eq!(
        synthesize_core_atom(&CoreAtom::PrimName(CorePrimOp::Add), &env)
            .expect("primitive name type"),
        CoreType::Function {
            params: vec![CoreType::Base("Int".into()), CoreType::Base("Int".into())],
            result: Box::new(CoreType::Base("Int".into())),
            row: CoreRow::default(),
        }
    );
    assert_eq!(
        synthesize_core_atom(&CoreAtom::ConstructorName("SomeInt".into()), &env)
            .expect("constructor name type"),
        CoreType::Function {
            params: vec![CoreType::Base("Int".into())],
            result: Box::new(CoreType::Named("OptionInt".into())),
            row: CoreRow::default(),
        }
    );
}

#[test]
fn record_and_tuple_values_synthesize_structural_types_with_empty_construction_row() {
    let mut env = CoreTypeCheckEnv::default();
    env.values_mut()
        .insert("name", CoreType::Base("String".into()));
    let record = CoreValue::Record {
        fields: vec![
            ("name".into(), CoreAtom::Var("name".into())),
            ("age".into(), CoreAtom::LitInt(42)),
        ],
    };
    let tuple = CoreValue::Tuple {
        elems: vec![CoreAtom::LitString("Ada".into()), CoreAtom::LitInt(42)],
    };

    let typed_record = synthesize_core_value(&record, &env).expect("record value type");
    let typed_tuple = synthesize_core_value(&tuple, &env).expect("tuple value type");

    assert!(
        core_types_equivalent(
            typed_record.ty(),
            &CoreType::Record(vec![
                ("age".into(), CoreType::Base("Int".into())),
                ("name".into(), CoreType::Base("String".into())),
            ]),
            &env,
        )
        .expect("record type comparison")
    );
    assert_eq!(typed_record.row(), &CoreRow::default());
    assert_eq!(
        typed_tuple.ty(),
        &CoreType::Tuple(vec![
            CoreType::Base("String".into()),
            CoreType::Base("Int".into())
        ])
    );
    assert_eq!(typed_tuple.row(), &CoreRow::default());
}

fn cap_op(path: &[&str], operation: &str) -> CoreEffectOp {
    CoreEffectOp::Capability {
        path: path.iter().map(|part| (*part).to_owned()).collect(),
        operation: operation.to_owned(),
        arg_types: Vec::new(),
        result_type: CoreType::Base("Unit".into()),
    }
}

fn type_check(
    expr: CoreExpr,
    env: &CoreTypeCheckEnv,
) -> Result<(CoreType, CoreRow), CoreTypeCheckError> {
    let valid =
        validate_core_program(RawCoreProgram::new(expr)).expect("Core expression validates");
    let typed = type_check_core_program(valid, env)?;
    Ok((typed.ty().clone(), typed.row().clone()))
}

#[test]
fn lambda_latent_row_accepts_body_row_included_in_annotation() {
    let lambda = CoreValue::Lam {
        params: vec![CoreParam {
            name: "x".into(),
            ty: CoreType::Base("Int".into()),
        }],
        body: Box::new(CoreExpr::Atom(CoreAtom::Var("x".into()))),
        row: CoreRow::closed(vec![cap(&["fs"], "read")]),
    };

    let typed = synthesize_core_value(&lambda, &CoreTypeCheckEnv::default())
        .expect("pure lambda body is compatible with a broader annotation row");

    assert_eq!(typed.row(), &CoreRow::default());
    assert_eq!(
        typed.ty(),
        &CoreType::Function {
            params: vec![CoreType::Base("Int".into())],
            result: Box::new(CoreType::Base("Int".into())),
            row: CoreRow::closed(vec![cap(&["fs"], "read")]),
        }
    );
}

#[test]
fn lambda_latent_row_accepts_equivalent_channel_payload_record_order() {
    let param_payload = CoreType::Record(vec![
        ("a".into(), CoreType::Base("Int".into())),
        ("b".into(), CoreType::Base("String".into())),
    ]);
    let annotated_payload = CoreType::Record(vec![
        ("b".into(), CoreType::Base("String".into())),
        ("a".into(), CoreType::Base("Int".into())),
    ]);
    let mut env = CoreTypeCheckEnv::default();
    env.operations_mut()
        .insert(channel_op(&["jobs"], "send", param_payload.clone()));

    let lambda = CoreValue::Lam {
        params: vec![CoreParam {
            name: "payload".into(),
            ty: param_payload.clone(),
        }],
        body: Box::new(CoreExpr::Raise {
            op: channel_op(&["jobs"], "send", param_payload),
            args: vec![CoreAtom::Var("payload".into())],
        }),
        row: CoreRow::closed(vec![chan(&["jobs"], "send", annotated_payload)]),
    };

    let typed =
        synthesize_core_value(&lambda, &env).expect("channel latent row compares structurally");

    assert_eq!(typed.row(), &CoreRow::default());
    assert_eq!(
        typed.ty(),
        &CoreType::Function {
            params: vec![CoreType::Record(vec![
                ("a".into(), CoreType::Base("Int".into())),
                ("b".into(), CoreType::Base("String".into())),
            ])],
            result: Box::new(CoreType::Base("Unit".into())),
            row: CoreRow::closed(vec![chan(
                &["jobs"],
                "send",
                CoreType::Record(vec![
                    ("b".into(), CoreType::Base("String".into())),
                    ("a".into(), CoreType::Base("Int".into())),
                ])
            )]),
        }
    );
}

#[test]
fn lambda_latent_row_rejects_body_row_not_included_in_annotation() {
    let mut env = CoreTypeCheckEnv::default();
    env.operations_mut().insert(cap_op(&["fs"], "read"));
    let lambda = CoreValue::Lam {
        params: Vec::new(),
        body: Box::new(CoreExpr::Raise {
            op: cap_op(&["fs"], "read"),
            args: Vec::new(),
        }),
        row: CoreRow::default(),
    };

    let err = synthesize_core_value(&lambda, &env)
        .expect_err("effectful lambda body must be included in the annotation row");

    assert!(matches!(err, CoreTypeCheckError::RowMismatch { .. }));
}

#[test]
fn lambda_annotation_row_subtyping_allows_purer_function_as_argument_to_superset_requirement() {
    let pure_fn_ty = CoreType::Function {
        params: Vec::new(),
        result: Box::new(CoreType::Base("Unit".into())),
        row: CoreRow::default(),
    };
    let takes_fn_param_ty = CoreType::Function {
        params: Vec::new(),
        result: Box::new(CoreType::Base("Unit".into())),
        row: CoreRow::closed(vec![cap(&["fs"], "read")]),
    };
    let takes_fn_ty = CoreType::Function {
        params: vec![takes_fn_param_ty],
        result: Box::new(CoreType::Base("Unit".into())),
        row: CoreRow::default(),
    };
    let mut env = CoreTypeCheckEnv::default();
    env.values_mut().insert("pure_fn", pure_fn_ty);
    env.values_mut().insert("takes_fn", takes_fn_ty);

    let (ty, row) = type_check(
        CoreExpr::Call {
            func: CoreAtom::Var("takes_fn".into()),
            args: vec![CoreAtom::Var("pure_fn".into())],
        },
        &env,
    )
    .expect("pure function should satisfy superset function-row annotation");

    assert_eq!(ty, CoreType::Base("Unit".into()));
    assert_eq!(row, CoreRow::default());
}

#[test]
fn lambda_annotation_row_subtyping_rejects_purer_annotation_for_effectful_function_value() {
    let takes_fn_param_ty = CoreType::Function {
        params: Vec::new(),
        result: Box::new(CoreType::Base("Unit".into())),
        row: CoreRow::default(),
    };
    let cap_fn_ty = CoreType::Function {
        params: Vec::new(),
        result: Box::new(CoreType::Base("Unit".into())),
        row: CoreRow::closed(vec![cap(&["fs"], "read")]),
    };
    let mut env = CoreTypeCheckEnv::default();
    env.values_mut().insert("cap_fn", cap_fn_ty);
    env.values_mut().insert(
        "takes_cap_fn",
        CoreType::Function {
            params: vec![takes_fn_param_ty],
            result: Box::new(CoreType::Base("Unit".into())),
            row: CoreRow::default(),
        },
    );

    let err = type_check(
        CoreExpr::Call {
            func: CoreAtom::Var("takes_cap_fn".into()),
            args: vec![CoreAtom::Var("cap_fn".into())],
        },
        &env,
    )
    .expect_err("effectful annotation should reject a purer function argument");

    assert!(matches!(err, CoreTypeCheckError::RowMismatch { .. }));
}

#[test]
fn lambda_synthesis_preserves_body_refinement_obligation_facts() {
    let mut env = CoreTypeCheckEnv::default();
    env.discharges_mut()
        .insert_refinement_predicate("positive-result");
    let refined_int = CoreType::Refinement {
        base: Box::new(CoreType::Base("Int".into())),
        predicate: "positive-result".into(),
    };
    let lambda = CoreValue::Lam {
        params: Vec::new(),
        body: Box::new(CoreExpr::LetVal {
            name: "x".into(),
            ty: refined_int.clone(),
            value: CoreValue::Atom(CoreAtom::LitInt(7)),
            body: Box::new(CoreExpr::Atom(CoreAtom::Var("x".into()))),
        }),
        row: CoreRow::default(),
    };

    let typed = synthesize_core_value(&lambda, &env)
        .expect("lambda body refinement obligations should be checked");

    assert_eq!(typed.facts().refinement_obligations().len(), 1);
    assert_eq!(
        typed.facts().refinement_obligations()[0].refinement_type(),
        &refined_int
    );
}

#[test]
fn discharge_marker_is_administrative_unit_metadata() {
    let marker = CoreValue::DischargeMarker {
        discharge: CoreContractDischarge {
            contract: "positive-result".into(),
            mode: CoreDischargeMode::Static,
            evidence: None,
            source_span: None,
        },
    };

    let typed = synthesize_core_value(&marker, &CoreTypeCheckEnv::default())
        .expect("administrative marker is well shaped");

    assert_eq!(typed.ty(), &CoreType::Base("Unit".into()));
    assert_eq!(typed.row(), &CoreRow::default());
}

#[test]
fn lambda_rejects_unknown_row_tail_in_latent_annotation() {
    let lambda = CoreValue::Lam {
        params: Vec::new(),
        body: Box::new(CoreExpr::Atom(CoreAtom::LitUnit)),
        row: CoreRow::open(Vec::new(), "missing_row_var"),
    };

    let err = synthesize_core_value(&lambda, &CoreTypeCheckEnv::default())
        .expect_err("unknown row variable should be rejected before returning a function type");

    assert_eq!(
        err,
        CoreTypeCheckError::UnknownRowVariable {
            name: "missing_row_var".into()
        }
    );
}

#[test]
fn lambda_rejects_unknown_channel_payload_type_in_latent_row_annotation() {
    let lambda = CoreValue::Lam {
        params: Vec::new(),
        body: Box::new(CoreExpr::Atom(CoreAtom::LitUnit)),
        row: CoreRow::closed(vec![CoreRowItem::Channel {
            path: vec!["jobs".into()],
            mode: "send".into(),
            payload_type: Box::new(CoreType::Named("Payload".into())),
        }]),
    };

    let err = synthesize_core_value(&lambda, &CoreTypeCheckEnv::default())
        .expect_err("unknown payload type should be rejected in lambda row annotations");

    assert_eq!(
        err,
        CoreTypeCheckError::UnknownType {
            name: "Payload".into()
        }
    );
}
