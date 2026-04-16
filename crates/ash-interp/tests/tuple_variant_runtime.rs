use ash_core::{Expr, Pattern, Value};
use ash_interp::{Context, eval_expr, match_pattern};
use std::path::PathBuf;

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .unwrap()
}

#[test]
fn tuple_constructor_evaluation_preserves_positional_payloads() {
    let expr = Expr::Constructor {
        name: "RuntimeError".into(),
        fields: vec![
            ("_0".into(), Expr::Literal(Value::Int(2))),
            (
                "_1".into(),
                Expr::Literal(Value::String("missing config".into())),
            ),
        ],
    };

    let value = eval_expr(&expr, &Context::new()).expect("tuple constructor should evaluate");

    assert_eq!(
        value,
        Value::Variant {
            name: "RuntimeError".into(),
            fields: Box::new(vec![
                ("_0".into(), Value::Int(2)),
                ("_1".into(), Value::String("missing config".into())),
            ]),
        }
    );
}

#[test]
fn tuple_variant_patterns_match_positionally() {
    let pattern = Pattern::Variant {
        name: "RuntimeError".into(),
        fields: Some(vec![
            (
                "_0".into(),
                Pattern::Variable {
                    name: "code".into(),
                    span: ash_core::ast::Span::default(),
                },
            ),
            (
                "_1".into(),
                Pattern::Variable {
                    name: "message".into(),
                    span: ash_core::ast::Span::default(),
                },
            ),
        ]),
    };
    let value = Value::Variant {
        name: "RuntimeError".into(),
        fields: Box::new(vec![
            ("_0".into(), Value::Int(7)),
            ("_1".into(), Value::String("boom".into())),
        ]),
    };

    let bindings = match_pattern(&pattern, &value).expect("tuple variant pattern should match");

    assert_eq!(bindings.get("code"), Some(&Value::Int(7)));
    assert_eq!(bindings.get("message"), Some(&Value::String("boom".into())));
}

#[test]
fn nested_tuple_variant_patterns_extract_payloads() {
    let pattern = Pattern::Variant {
        name: "Err".into(),
        fields: Some(vec![(
            "error".into(),
            Pattern::Variant {
                name: "RuntimeError".into(),
                fields: Some(vec![
                    (
                        "_0".into(),
                        Pattern::Variable {
                            name: "code".into(),
                            span: ash_core::ast::Span::default(),
                        },
                    ),
                    (
                        "_1".into(),
                        Pattern::Tuple(vec![
                            Pattern::Variable {
                                name: "line".into(),
                                span: ash_core::ast::Span::default(),
                            },
                            Pattern::Variable {
                                name: "column".into(),
                                span: ash_core::ast::Span::default(),
                            },
                        ]),
                    ),
                ]),
            },
        )]),
    };
    let value = Value::Variant {
        name: "Err".into(),
        fields: Box::new(vec![(
            "error".into(),
            Value::Variant {
                name: "RuntimeError".into(),
                fields: Box::new(vec![
                    ("_0".into(), Value::Int(9)),
                    (
                        "_1".into(),
                        Value::List(Box::new(vec![Value::Int(12), Value::Int(34)])),
                    ),
                ]),
            },
        )]),
    };

    let bindings =
        match_pattern(&pattern, &value).expect("nested tuple variant pattern should match");

    assert_eq!(bindings.get("code"), Some(&Value::Int(9)));
    assert_eq!(bindings.get("line"), Some(&Value::Int(12)));
    assert_eq!(bindings.get("column"), Some(&Value::Int(34)));
}

#[test]
fn tuple_variant_patterns_require_exact_runtime_arity() {
    let pattern = Pattern::Variant {
        name: "RuntimeError".into(),
        fields: Some(vec![
            ("_0".into(), Pattern::Wildcard),
            ("_1".into(), Pattern::Wildcard),
        ]),
    };
    let value = Value::Variant {
        name: "RuntimeError".into(),
        fields: Box::new(vec![
            ("_0".into(), Value::Int(2)),
            ("_1".into(), Value::String("boom".into())),
            ("_2".into(), Value::Bool(true)),
        ]),
    };

    assert!(
        match_pattern(&pattern, &value).is_err(),
        "tuple variant patterns should reject runtime values with mismatched positional arity"
    );
}

#[test]
fn tuple_variant_display_uses_positional_payload_formatting() {
    let value = Value::Variant {
        name: "RuntimeError".into(),
        fields: Box::new(vec![
            ("_0".into(), Value::Int(2)),
            ("_1".into(), Value::String("boom".into())),
        ]),
    };

    assert_eq!(format!("{value}"), "RuntimeError(2, \"boom\")");
}

#[test]
fn runtime_error_contract_surfaces_are_tuple_shaped() {
    let runtime_error = std::fs::read_to_string(repo_root().join("std/src/runtime/error.ash"))
        .expect("should read runtime/error.ash");
    let supervisor = std::fs::read_to_string(repo_root().join("std/src/runtime/supervisor.ash"))
        .expect("should read runtime/supervisor.ash");

    assert!(
        runtime_error.contains("pub type RuntimeError = RuntimeError(Int, String);"),
        "runtime/error.ash should expose RuntimeError as a tuple variant"
    );
    assert!(
        !runtime_error.contains("RuntimeError {"),
        "runtime/error.ash should not expose record-shaped RuntimeError syntax"
    );
    assert!(
        supervisor.contains("Err { error: RuntimeError(code, _) }"),
        "runtime/supervisor.ash should destructure RuntimeError positionally"
    );
}
