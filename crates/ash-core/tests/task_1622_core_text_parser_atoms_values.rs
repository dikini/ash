use ash_core::core_ash::{
    CoreAtom, CoreContractDischarge, CoreDischargeMode, CoreExpr, CoreMultiplicity, CoreParam,
    CorePrimOp, CoreRow, CoreRowItem, CoreType, CoreValue,
};
use ash_core::core_ash_text::{parse_atom, parse_row, parse_row_item, parse_type, parse_value};

#[test]
fn parses_literal_and_variable_atoms_without_conflating_strings() {
    assert_eq!(parse_atom("(lit-int 42)").unwrap(), CoreAtom::LitInt(42));
    assert_eq!(
        parse_atom("(lit-string \"name\")").unwrap(),
        CoreAtom::LitString("name".to_string())
    );
    assert_eq!(
        parse_atom("(lit-bool true)").unwrap(),
        CoreAtom::LitBool(true)
    );
    assert_eq!(parse_atom("(lit-unit)").unwrap(), CoreAtom::LitUnit);
    assert_eq!(
        parse_atom("name").unwrap(),
        CoreAtom::Var("name".to_string())
    );
    assert_eq!(
        parse_atom("add").unwrap(),
        CoreAtom::PrimName(CorePrimOp::Add)
    );
}

#[test]
fn parses_function_and_continuation_types_with_rows() {
    assert_eq!(
        parse_type("(fn (Int String) -> Unit {cap console.write})").unwrap(),
        CoreType::Function {
            params: vec![
                CoreType::Base("Int".to_string()),
                CoreType::Base("String".to_string())
            ],
            result: Box::new(CoreType::Base("Unit".to_string())),
            row: CoreRow::closed(vec![CoreRowItem::Capability {
                path: vec!["console".to_string()],
                operation: "write".to_string(),
            }]),
        }
    );

    assert_eq!(
        parse_type("(cont Unit Unit {fail Error} affine)").unwrap(),
        CoreType::Cont {
            input: Box::new(CoreType::Base("Unit".to_string())),
            answer: Box::new(CoreType::Base("Unit".to_string())),
            row: CoreRow::closed(vec![CoreRowItem::Failure {
                ty: Some(Box::new(CoreType::Named("Error".to_string()))),
            }]),
            multiplicity: CoreMultiplicity::Affine,
        }
    );
}

#[test]
fn parses_capability_and_failure_rows_without_deduplicating() {
    assert_eq!(
        parse_row_item("cap console.write").unwrap(),
        CoreRowItem::Capability {
            path: vec!["console".to_string()],
            operation: "write".to_string(),
        }
    );
    assert_eq!(
        parse_row_item("fail Error").unwrap(),
        CoreRowItem::Failure {
            ty: Some(Box::new(CoreType::Named("Error".to_string()))),
        }
    );
    assert_eq!(
        parse_row("{cap console.write, fail Error}").unwrap(),
        CoreRow::closed(vec![
            CoreRowItem::Capability {
                path: vec!["console".to_string()],
                operation: "write".to_string(),
            },
            CoreRowItem::Failure {
                ty: Some(Box::new(CoreType::Named("Error".to_string()))),
            },
        ])
    );
}

#[test]
fn parses_lambda_tuple_record_and_discharge_marker_values() {
    assert_eq!(
        parse_value("(lam ((x : Int)) : {} x)").unwrap(),
        CoreValue::Lam {
            params: vec![CoreParam {
                name: "x".to_string(),
                ty: CoreType::Base("Int".to_string()),
            }],
            body: Box::new(CoreExpr::Atom(CoreAtom::Var("x".to_string()))),
            row: CoreRow::default(),
        }
    );
    assert_eq!(
        parse_value("(tuple (lit-int 1) x)").unwrap(),
        CoreValue::Tuple {
            elems: vec![CoreAtom::LitInt(1), CoreAtom::Var("x".to_string())],
        }
    );
    assert_eq!(
        parse_value("(record (answer (lit-int 42)) (name (lit-string \"ash\")))").unwrap(),
        CoreValue::Record {
            fields: vec![
                ("answer".to_string(), CoreAtom::LitInt(42)),
                ("name".to_string(), CoreAtom::LitString("ash".to_string())),
            ],
        }
    );
    assert_eq!(
        parse_value("(discharge-marker (contract requires-positive dynamic))").unwrap(),
        CoreValue::DischargeMarker {
            discharge: CoreContractDischarge {
                contract: "requires-positive".to_string(),
                mode: CoreDischargeMode::Dynamic,
                evidence: None,
                source_span: None,
            },
        }
    );
}

#[test]
fn rejects_unsupported_effect_item_spelling_at_parse_time() {
    let error = parse_row_item("ContractViolation requires-positive").unwrap_err();
    assert!(
        error.to_string().contains("unsupported row item"),
        "unexpected error: {error}"
    );
}
