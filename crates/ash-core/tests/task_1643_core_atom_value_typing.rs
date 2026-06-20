use ash_core::core_ash::{
    CoreAtom, CoreContractDischarge, CoreDischargeMode, CoreExpr, CoreParam, CorePrimOp, CoreRow,
    CoreRowItem, CoreType, CoreValue,
};
use ash_core::core_ash_typecheck::{
    CoreTypeCheckEnv, CoreTypeCheckError, core_types_equivalent, synthesize_core_atom,
    synthesize_core_value,
};

fn cap(path: &[&str], operation: &str) -> CoreRowItem {
    CoreRowItem::Capability {
        path: path.iter().map(|part| (*part).to_owned()).collect(),
        operation: operation.to_owned(),
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

#[test]
fn lambda_latent_row_mismatch_fails_without_charging_construction_row() {
    let lambda = CoreValue::Lam {
        params: vec![CoreParam {
            name: "x".into(),
            ty: CoreType::Base("Int".into()),
        }],
        body: Box::new(CoreExpr::Atom(CoreAtom::Var("x".into()))),
        row: CoreRow::closed(vec![cap(&["fs"], "read")]),
    };

    let err = synthesize_core_value(&lambda, &CoreTypeCheckEnv::default())
        .expect_err("pure lambda body cannot claim an unused latent requirement");

    assert!(matches!(err, CoreTypeCheckError::RowMismatch { .. }));
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
