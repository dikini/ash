use super::*;
use crate::RuntimeState;
use ash_core::{
    ControlLink, Instance, InstanceAddr, MatchArm, ProcessHandle, ProcessId, UnaryOp, WorkflowId,
};

#[test]
fn test_eval_literal() {
    let ctx = Context::new();
    let expr = Expr::Literal(Value::Int(42));
    assert_eq!(eval_expr(&expr, &ctx).unwrap(), Value::Int(42));
}

#[test]
fn test_eval_variable_found() {
    let mut ctx = Context::new();
    ctx.set("x".to_string(), Value::Int(42));
    let expr = Expr::Variable {
        name: "x".to_string(),
        span: ash_core::ast::Span::default(),
    };
    assert_eq!(eval_expr(&expr, &ctx).unwrap(), Value::Int(42));
}

#[test]
fn test_eval_variable_not_found() {
    let ctx = Context::new();
    let expr = Expr::Variable {
        name: "x".to_string(),
        span: ash_core::ast::Span::default(),
    };
    assert!(eval_expr(&expr, &ctx).is_err());
}

#[test]
fn test_eval_field_access() {
    let mut ctx = Context::new();
    let mut record = HashMap::new();
    record.insert("name".to_string(), Value::String("Alice".to_string()));
    ctx.set("person".to_string(), Value::Record(Box::new(record)));

    let expr = Expr::FieldAccess {
        expr: Box::new(Expr::Variable {
            name: "person".to_string(),
            span: ash_core::ast::Span::default(),
        }),
        field: "name".to_string(),
    };
    assert_eq!(
        eval_expr(&expr, &ctx).unwrap(),
        Value::String("Alice".to_string())
    );
}

#[test]
fn test_eval_field_access_named_variant_payload() {
    let ctx = Context::new();
    let expr = Expr::FieldAccess {
        expr: Box::new(Expr::Literal(Value::Variant {
            name: "UserPayload".to_string(),
            fields: Box::new(vec![
                ("name".to_string(), Value::String("Ada".to_string())),
                ("age".to_string(), Value::Int(41)),
            ]),
        })),
        field: "age".to_string(),
    };
    assert_eq!(eval_expr(&expr, &ctx).unwrap(), Value::Int(41));
}

#[test]
fn test_eval_field_access_numeric_tuple_aliases() {
    let ctx = Context::new();
    let mut record = HashMap::new();
    record.insert("_0".to_string(), Value::Int(20));
    let record_expr = Expr::FieldAccess {
        expr: Box::new(Expr::Literal(Value::Record(Box::new(record)))),
        field: "0".to_string(),
    };
    assert_eq!(eval_expr(&record_expr, &ctx).unwrap(), Value::Int(20));

    let variant_expr = Expr::FieldAccess {
        expr: Box::new(Expr::Literal(Value::Variant {
            name: "Box".to_string(),
            fields: Box::new(vec![("_0".to_string(), Value::Int(21))]),
        })),
        field: "0".to_string(),
    };
    assert_eq!(eval_expr(&variant_expr, &ctx).unwrap(), Value::Int(21));

    let list_expr = Expr::FieldAccess {
        expr: Box::new(Expr::Literal(Value::list_from_vec(vec![
            Value::Int(22),
            Value::Int(23),
        ]))),
        field: "0".to_string(),
    };
    assert_eq!(eval_expr(&list_expr, &ctx).unwrap(), Value::Int(22));
}

#[test]
fn test_eval_field_access_not_found() {
    let ctx = Context::new();
    let mut record = HashMap::new();
    record.insert("x".to_string(), Value::Int(1));
    let expr = Expr::FieldAccess {
        expr: Box::new(Expr::Literal(Value::Record(Box::new(record)))),
        field: "missing".to_string(),
    };
    assert!(eval_expr(&expr, &ctx).is_err());
}

#[test]
fn test_eval_field_access_not_record() {
    let ctx = Context::new();
    let expr = Expr::FieldAccess {
        expr: Box::new(Expr::Literal(Value::Int(42))),
        field: "x".to_string(),
    };
    assert!(eval_expr(&expr, &ctx).is_err());
}

#[test]
fn test_eval_index_list() {
    let ctx = Context::new();
    let expr = Expr::IndexAccess {
        expr: Box::new(Expr::Literal(Value::list_from_vec(vec![
            Value::Int(10),
            Value::Int(20),
            Value::Int(30),
        ]))),
        index: Box::new(Expr::Literal(Value::Int(1))),
    };
    assert_eq!(eval_expr(&expr, &ctx).unwrap(), Value::Int(20));
}

#[test]
fn test_eval_index_out_of_bounds() {
    let ctx = Context::new();
    let expr = Expr::IndexAccess {
        expr: Box::new(Expr::Literal(Value::list_from_vec(vec![Value::Int(10)]))),
        index: Box::new(Expr::Literal(Value::Int(5))),
    };
    assert!(eval_expr(&expr, &ctx).is_err());
}

#[test]
fn test_eval_unary_not() {
    let ctx = Context::new();
    let expr = Expr::Unary {
        op: UnaryOp::Not,
        expr: Box::new(Expr::Literal(Value::Bool(true))),
    };
    assert_eq!(eval_expr(&expr, &ctx).unwrap(), Value::Bool(false));
}

#[test]
fn test_eval_unary_neg() {
    let ctx = Context::new();
    let expr = Expr::Unary {
        op: UnaryOp::Neg,
        expr: Box::new(Expr::Literal(Value::Int(42))),
    };
    assert_eq!(eval_expr(&expr, &ctx).unwrap(), Value::Int(-42));
}

#[test]
fn test_eval_binary_arithmetic() {
    let ctx = Context::new();

    // Addition
    let expr = Expr::Binary {
        op: BinaryOp::Add,
        left: Box::new(Expr::Literal(Value::Int(10))),
        right: Box::new(Expr::Literal(Value::Int(5))),
    };
    assert_eq!(eval_expr(&expr, &ctx).unwrap(), Value::Int(15));

    // Subtraction
    let expr = Expr::Binary {
        op: BinaryOp::Sub,
        left: Box::new(Expr::Literal(Value::Int(10))),
        right: Box::new(Expr::Literal(Value::Int(5))),
    };
    assert_eq!(eval_expr(&expr, &ctx).unwrap(), Value::Int(5));

    // Multiplication
    let expr = Expr::Binary {
        op: BinaryOp::Mul,
        left: Box::new(Expr::Literal(Value::Int(10))),
        right: Box::new(Expr::Literal(Value::Int(5))),
    };
    assert_eq!(eval_expr(&expr, &ctx).unwrap(), Value::Int(50));

    // Division
    let expr = Expr::Binary {
        op: BinaryOp::Div,
        left: Box::new(Expr::Literal(Value::Int(10))),
        right: Box::new(Expr::Literal(Value::Int(5))),
    };
    assert_eq!(eval_expr(&expr, &ctx).unwrap(), Value::Int(2));
}

#[test]
fn test_eval_binary_div_by_zero() {
    let ctx = Context::new();
    let expr = Expr::Binary {
        op: BinaryOp::Div,
        left: Box::new(Expr::Literal(Value::Int(10))),
        right: Box::new(Expr::Literal(Value::Int(0))),
    };
    assert!(matches!(
        eval_expr(&expr, &ctx),
        Err(EvalError::DivisionByZero)
    ));
}

#[test]
fn test_eval_binary_logical() {
    let ctx = Context::new();

    // AND
    let expr = Expr::Binary {
        op: BinaryOp::And,
        left: Box::new(Expr::Literal(Value::Bool(true))),
        right: Box::new(Expr::Literal(Value::Bool(false))),
    };
    assert_eq!(eval_expr(&expr, &ctx).unwrap(), Value::Bool(false));

    // OR
    let expr = Expr::Binary {
        op: BinaryOp::Or,
        left: Box::new(Expr::Literal(Value::Bool(true))),
        right: Box::new(Expr::Literal(Value::Bool(false))),
    };
    assert_eq!(eval_expr(&expr, &ctx).unwrap(), Value::Bool(true));
}

#[test]
fn test_eval_binary_comparison() {
    let ctx = Context::new();

    // Less than
    let expr = Expr::Binary {
        op: BinaryOp::Lt,
        left: Box::new(Expr::Literal(Value::Int(1))),
        right: Box::new(Expr::Literal(Value::Int(2))),
    };
    assert_eq!(eval_expr(&expr, &ctx).unwrap(), Value::Bool(true));

    // Greater than
    let expr = Expr::Binary {
        op: BinaryOp::Gt,
        left: Box::new(Expr::Literal(Value::Int(2))),
        right: Box::new(Expr::Literal(Value::Int(1))),
    };
    assert_eq!(eval_expr(&expr, &ctx).unwrap(), Value::Bool(true));

    // Equal
    let expr = Expr::Binary {
        op: BinaryOp::Eq,
        left: Box::new(Expr::Literal(Value::Int(42))),
        right: Box::new(Expr::Literal(Value::Int(42))),
    };
    assert_eq!(eval_expr(&expr, &ctx).unwrap(), Value::Bool(true));
}

#[test]
fn test_eval_binary_in_list() {
    let ctx = Context::new();
    let expr = Expr::Binary {
        op: BinaryOp::In,
        left: Box::new(Expr::Literal(Value::Int(2))),
        right: Box::new(Expr::Literal(Value::list_from_vec(vec![
            Value::Int(1),
            Value::Int(2),
            Value::Int(3),
        ]))),
    };
    assert_eq!(eval_expr(&expr, &ctx).unwrap(), Value::Bool(true));
}

#[test]
fn test_eval_call_len() {
    let ctx = Context::new();
    let expr = Expr::Call {
        func: "len".to_string(),
        module: None,
        arguments: vec![Expr::Literal(Value::list_from_vec(vec![
            Value::Int(1),
            Value::Int(2),
        ]))],
    };
    assert_eq!(eval_expr(&expr, &ctx).unwrap(), Value::Int(2));
}

#[test]
fn test_eval_call_append() {
    let ctx = Context::new();
    let expr = Expr::Call {
        func: "append".to_string(),
        module: None,
        arguments: vec![
            Expr::Literal(Value::list_from_vec(vec![Value::Int(1)])),
            Expr::Literal(Value::Int(2)),
        ],
    };
    assert_eq!(
        eval_expr(&expr, &ctx).unwrap(),
        Value::list_from_vec(vec![Value::Int(1), Value::Int(2)])
    );
}

#[test]
fn test_eval_call_concat() {
    let ctx = Context::new();
    let expr = Expr::Call {
        func: "concat".to_string(),
        module: None,
        arguments: vec![
            Expr::Literal(Value::list_from_vec(vec![Value::Int(1)])),
            Expr::Literal(Value::list_from_vec(vec![Value::Int(2)])),
        ],
    };
    assert_eq!(
        eval_expr(&expr, &ctx).unwrap(),
        Value::list_from_vec(vec![Value::Int(1), Value::Int(2)])
    );
}

#[test]
fn test_eval_call_unknown() {
    let ctx = Context::new();
    let expr = Expr::Call {
        func: "unknown".to_string(),
        module: None,
        arguments: vec![],
    };
    assert!(eval_expr(&expr, &ctx).is_err());
}

#[test]
fn test_eval_call_wrong_arity() {
    let ctx = Context::new();
    let expr = Expr::Call {
        func: "len".to_string(),
        module: None,
        arguments: vec![],
    };
    let err = eval_expr(&expr, &ctx).unwrap_err();
    assert!(
        matches!(
            err,
            EvalError::WrongArity {
                expected: 1,
                actual: 0,
                callee: Some(ref callee),
            } if callee == "len"
        ),
        "expected exact-arity WrongArity, got {err:?}"
    );
}

#[test]
fn test_eval_nested_expr() {
    let mut ctx = Context::new();
    ctx.set("x".to_string(), Value::Int(5));

    // (x + 3) * 2
    let expr = Expr::Binary {
        op: BinaryOp::Mul,
        left: Box::new(Expr::Binary {
            op: BinaryOp::Add,
            left: Box::new(Expr::Variable {
                name: "x".to_string(),
                span: ash_core::ast::Span::default(),
            }),
            right: Box::new(Expr::Literal(Value::Int(3))),
        }),
        right: Box::new(Expr::Literal(Value::Int(2))),
    };
    assert_eq!(eval_expr(&expr, &ctx).unwrap(), Value::Int(16));
}

#[test]
fn test_eval_string_concat() {
    let ctx = Context::new();
    let expr = Expr::Binary {
        op: BinaryOp::Add,
        left: Box::new(Expr::Literal(Value::String("hello ".to_string()))),
        right: Box::new(Expr::Literal(Value::String("world".to_string()))),
    };
    assert_eq!(
        eval_expr(&expr, &ctx).unwrap(),
        Value::String("hello world".to_string())
    );
}

#[test]
fn test_eval_type_checks() {
    let ctx = Context::new();

    let expr = Expr::Call {
        func: "is_int".to_string(),
        module: None,
        arguments: vec![Expr::Literal(Value::Int(42))],
    };
    assert_eq!(eval_expr(&expr, &ctx).unwrap(), Value::Bool(true));

    let expr = Expr::Call {
        func: "is_string".to_string(),
        module: None,
        arguments: vec![Expr::Literal(Value::Int(42))],
    };
    assert_eq!(eval_expr(&expr, &ctx).unwrap(), Value::Bool(false));
}

// ============================================================
// TASK-131: Constructor Evaluation Tests
// ============================================================

#[test]
fn test_eval_constructor_some_with_value() {
    let ctx = Context::new();
    let expr = Expr::Constructor {
        name: "Some".to_string(),
        fields: vec![("value".to_string(), Expr::Literal(Value::Int(42)))],
    };
    let result = eval_expr(&expr, &ctx).unwrap();
    assert_eq!(
        result,
        Value::Variant {
            name: "Some".to_string(),
            fields: Box::new(vec![("value".to_string(), Value::Int(42))]),
        }
    );
}

#[test]
fn test_eval_constructor_none_empty() {
    let ctx = Context::new();
    let expr = Expr::Constructor {
        name: "None".to_string(),
        fields: vec![],
    };
    let result = eval_expr(&expr, &ctx).unwrap();
    assert_eq!(
        result,
        Value::Variant {
            name: "None".to_string(),
            fields: Box::new(vec![]),
        }
    );
}

#[test]
fn test_eval_match_wildcard_fallback() {
    let ctx = Context::new();

    // match 2 { 1 => "one", _ => "other" } → "other"
    let arms = vec![
        MatchArm {
            pattern: Pattern::Literal(Value::Int(1)),
            body: Expr::Literal(Value::String("one".to_string())),
        },
        MatchArm {
            pattern: Pattern::Wildcard,
            body: Expr::Literal(Value::String("other".to_string())),
        },
    ];

    let expr = Expr::Match {
        scrutinee: Box::new(Expr::Literal(Value::Int(2))),
        arms,
    };

    assert_eq!(
        eval_expr(&expr, &ctx).unwrap(),
        Value::String("other".to_string())
    );
}

#[test]
fn test_eval_constructor_ok_with_string() {
    let ctx = Context::new();
    let expr = Expr::Constructor {
        name: "Ok".to_string(),
        fields: vec![(
            "value".to_string(),
            Expr::Literal(Value::String("hello".to_string())),
        )],
    };
    let result = eval_expr(&expr, &ctx).unwrap();
    assert_eq!(
        result,
        Value::Variant {
            name: "Ok".to_string(),
            fields: Box::new(vec![(
                "value".to_string(),
                Value::String("hello".to_string())
            )]),
        }
    );
}

#[test]
fn test_eval_constructor_err_with_value() {
    let ctx = Context::new();
    let expr = Expr::Constructor {
        name: "Err".to_string(),
        fields: vec![(
            "error".to_string(),
            Expr::Literal(Value::String("not found".to_string())),
        )],
    };
    let result = eval_expr(&expr, &ctx).unwrap();
    assert_eq!(
        result,
        Value::Variant {
            name: "Err".to_string(),
            fields: Box::new(vec![(
                "error".to_string(),
                Value::String("not found".to_string())
            )]),
        }
    );
}

#[test]
fn test_eval_constructor_nested() {
    let ctx = Context::new();
    // Some { value: Ok { value: 42 } }
    let inner = Expr::Constructor {
        name: "Ok".to_string(),
        fields: vec![("value".to_string(), Expr::Literal(Value::Int(42)))],
    };
    let outer = Expr::Constructor {
        name: "Some".to_string(),
        fields: vec![("value".to_string(), inner)],
    };
    let result = eval_expr(&outer, &ctx).unwrap();
    assert_eq!(
        result,
        Value::Variant {
            name: "Some".to_string(),
            fields: Box::new(vec![(
                "value".to_string(),
                Value::Variant {
                    name: "Ok".to_string(),
                    fields: Box::new(vec![("value".to_string(), Value::Int(42))]),
                }
            )]),
        }
    );
}

#[test]
fn test_eval_constructor_with_variable() {
    let mut ctx = Context::new();
    ctx.set("x".to_string(), Value::Int(100));

    let expr = Expr::Constructor {
        name: "Some".to_string(),
        fields: vec![(
            "value".to_string(),
            Expr::Variable {
                name: "x".to_string(),
                span: ash_core::ast::Span::default(),
            },
        )],
    };
    let result = eval_expr(&expr, &ctx).unwrap();
    assert_eq!(
        result,
        Value::Variant {
            name: "Some".to_string(),
            fields: Box::new(vec![("value".to_string(), Value::Int(100))]),
        }
    );
}

#[test]
fn test_eval_constructor_with_expression_in_field() {
    let ctx = Context::new();
    // Point { x: 1 + 2, y: 3 * 4 }
    let expr = Expr::Constructor {
        name: "Point".to_string(),
        fields: vec![
            (
                "x".to_string(),
                Expr::Binary {
                    op: BinaryOp::Add,
                    left: Box::new(Expr::Literal(Value::Int(1))),
                    right: Box::new(Expr::Literal(Value::Int(2))),
                },
            ),
            (
                "y".to_string(),
                Expr::Binary {
                    op: BinaryOp::Mul,
                    left: Box::new(Expr::Literal(Value::Int(3))),
                    right: Box::new(Expr::Literal(Value::Int(4))),
                },
            ),
        ],
    };
    let result = eval_expr(&expr, &ctx).unwrap();
    assert_eq!(
        result,
        Value::Variant {
            name: "Point".to_string(),
            fields: Box::new(vec![
                ("x".to_string(), Value::Int(3)),
                ("y".to_string(), Value::Int(12)),
            ]),
        }
    );
}

#[test]
fn test_eval_constructor_multiple_fields() {
    let ctx = Context::new();
    // Person { name: "Alice", age: 30, active: true }
    let expr = Expr::Constructor {
        name: "Person".to_string(),
        fields: vec![
            (
                "name".to_string(),
                Expr::Literal(Value::String("Alice".to_string())),
            ),
            ("age".to_string(), Expr::Literal(Value::Int(30))),
            ("active".to_string(), Expr::Literal(Value::Bool(true))),
        ],
    };
    let result = eval_expr(&expr, &ctx).unwrap();
    assert_eq!(
        result,
        Value::Variant {
            name: "Person".to_string(),
            fields: Box::new(vec![
                ("name".to_string(), Value::String("Alice".to_string())),
                ("age".to_string(), Value::Int(30)),
                ("active".to_string(), Value::Bool(true)),
            ]),
        }
    );
}

#[test]
fn test_value_variant_helpers() {
    // Test Value::variant helper
    let v = Value::variant("Some", vec![("value", Value::Int(42))]);
    assert_eq!(
        v,
        Value::Variant {
            name: "Some".to_string(),
            fields: Box::new(vec![("value".to_string(), Value::Int(42))]),
        }
    );

    // Test Value::unit_variant helper
    let v = Value::unit_variant("None");
    assert_eq!(
        v,
        Value::Variant {
            name: "None".to_string(),
            fields: Box::new(vec![]),
        }
    );
}

#[test]
fn test_eval_match_list_destructure() {
    let ctx = Context::new();

    // match [1, 2, 3] { [a, b, ..] => a + b, _ => 0 } → 3
    let arms = vec![
        MatchArm {
            pattern: Pattern::List(
                vec![
                    Pattern::Variable {
                        name: "a".to_string(),
                        span: ash_core::ast::Span::default(),
                    },
                    Pattern::Variable {
                        name: "b".to_string(),
                        span: ash_core::ast::Span::default(),
                    },
                ],
                Some("_".to_string()),
            ),
            body: Expr::Binary {
                op: BinaryOp::Add,
                left: Box::new(Expr::Variable {
                    name: "a".to_string(),
                    span: ash_core::ast::Span::default(),
                }),
                right: Box::new(Expr::Variable {
                    name: "b".to_string(),
                    span: ash_core::ast::Span::default(),
                }),
            },
        },
        MatchArm {
            pattern: Pattern::Wildcard,
            body: Expr::Literal(Value::Int(0)),
        },
    ];

    let expr = Expr::Match {
        scrutinee: Box::new(Expr::Literal(Value::list_from_vec(vec![
            Value::Int(1),
            Value::Int(2),
            Value::Int(3),
        ]))),
        arms,
    };

    assert_eq!(eval_expr(&expr, &ctx).unwrap(), Value::Int(3));
}

#[test]
fn test_eval_match_tuple_destructure() {
    let ctx = Context::new();

    // match (1, 2) { (x, y) => x + y } → 3
    let arms = vec![MatchArm {
        pattern: Pattern::Tuple(vec![
            Pattern::Variable {
                name: "x".to_string(),
                span: ash_core::ast::Span::default(),
            },
            Pattern::Variable {
                name: "y".to_string(),
                span: ash_core::ast::Span::default(),
            },
        ]),
        body: Expr::Binary {
            op: BinaryOp::Add,
            left: Box::new(Expr::Variable {
                name: "x".to_string(),
                span: ash_core::ast::Span::default(),
            }),
            right: Box::new(Expr::Variable {
                name: "y".to_string(),
                span: ash_core::ast::Span::default(),
            }),
        },
    }];

    let expr = Expr::Match {
        scrutinee: Box::new(Expr::Literal(Value::list_from_vec(vec![
            Value::Int(1),
            Value::Int(2),
        ]))),
        arms,
    };

    assert_eq!(eval_expr(&expr, &ctx).unwrap(), Value::Int(3));
}

#[test]
fn test_eval_match_option_some_branch_binds_value() {
    let mut ctx = Context::new();
    ctx.set(
        "opt".to_string(),
        Value::Variant {
            name: "Some".to_string(),
            fields: Box::new(vec![("value".to_string(), Value::Int(42))]),
        },
    );

    let expr = Expr::Match {
        scrutinee: Box::new(Expr::Variable {
            name: "opt".to_string(),
            span: ash_core::ast::Span::default(),
        }),
        arms: vec![
            MatchArm {
                pattern: Pattern::Variant {
                    name: "Some".to_string(),
                    fields: Some(vec![(
                        "value".to_string(),
                        Pattern::Variable {
                            name: "x".to_string(),
                            span: ash_core::ast::Span::default(),
                        },
                    )]),
                },
                body: Expr::Binary {
                    op: BinaryOp::Mul,
                    left: Box::new(Expr::Variable {
                        name: "x".to_string(),
                        span: ash_core::ast::Span::default(),
                    }),
                    right: Box::new(Expr::Literal(Value::Int(2))),
                },
            },
            MatchArm {
                pattern: Pattern::Variant {
                    name: "None".to_string(),
                    fields: None,
                },
                body: Expr::Literal(Value::Int(0)),
            },
        ],
    };

    assert_eq!(eval_expr(&expr, &ctx).unwrap(), Value::Int(84));
}

#[test]
fn test_eval_match_option_none_branch_selected() {
    let mut ctx = Context::new();
    ctx.set(
        "opt".to_string(),
        Value::Variant {
            name: "None".to_string(),
            fields: Box::new(vec![]),
        },
    );

    let expr = Expr::Match {
        scrutinee: Box::new(Expr::Variable {
            name: "opt".to_string(),
            span: ash_core::ast::Span::default(),
        }),
        arms: vec![
            MatchArm {
                pattern: Pattern::Variant {
                    name: "Some".to_string(),
                    fields: Some(vec![(
                        "value".to_string(),
                        Pattern::Variable {
                            name: "x".to_string(),
                            span: ash_core::ast::Span::default(),
                        },
                    )]),
                },
                body: Expr::Variable {
                    name: "x".to_string(),
                    span: ash_core::ast::Span::default(),
                },
            },
            MatchArm {
                pattern: Pattern::Variant {
                    name: "None".to_string(),
                    fields: None,
                },
                body: Expr::Literal(Value::Int(0)),
            },
        ],
    };

    assert_eq!(eval_expr(&expr, &ctx).unwrap(), Value::Int(0));
}

#[test]
fn test_eval_if_let_option_some_then_branch_binds_value() {
    let mut ctx = Context::new();
    ctx.set(
        "opt".to_string(),
        Value::Variant {
            name: "Some".to_string(),
            fields: Box::new(vec![("value".to_string(), Value::Int(99))]),
        },
    );

    let expr = Expr::IfLet {
        pattern: Pattern::Variant {
            name: "Some".to_string(),
            fields: Some(vec![(
                "value".to_string(),
                Pattern::Variable {
                    name: "x".to_string(),
                    span: ash_core::ast::Span::default(),
                },
            )]),
        },
        expr: Box::new(Expr::Variable {
            name: "opt".to_string(),
            span: ash_core::ast::Span::default(),
        }),
        then_branch: Box::new(Expr::Variable {
            name: "x".to_string(),
            span: ash_core::ast::Span::default(),
        }),
        else_branch: Box::new(Expr::Literal(Value::Int(0))),
    };

    assert_eq!(eval_expr(&expr, &ctx).unwrap(), Value::Int(99));
}

// ============================================================
// TASK-134: Spawn and Split Tests
// ============================================================

#[test]
fn test_eval_spawn_returns_instance() {
    let ctx = Context::new();

    // spawn Worker with { init: 42 }
    let expr = Expr::Spawn {
        entry_type: "Worker".to_string(),
        init: Box::new(Expr::Literal(Value::Int(42))),
    };

    let result = eval_expr(&expr, &ctx).unwrap();

    // Should return an Instance value
    match result {
        Value::Instance(instance) => {
            assert_eq!(instance.addr.entry_type, "Worker");
            assert!(instance.control.is_some());
            assert_eq!(
                instance.control.unwrap().instance_id,
                instance.addr.instance_id
            );
        }
        _ => panic!("Expected Instance, got {:?}", result),
    }
}

#[test]
fn test_eval_spawn_creates_unique_ids() {
    let ctx = Context::new();

    // spawn two instances
    let expr1 = Expr::Spawn {
        entry_type: "Worker".to_string(),
        init: Box::new(Expr::Literal(Value::Int(1))),
    };
    let expr2 = Expr::Spawn {
        entry_type: "Worker".to_string(),
        init: Box::new(Expr::Literal(Value::Int(2))),
    };

    let result1 = eval_expr(&expr1, &ctx).unwrap();
    let result2 = eval_expr(&expr2, &ctx).unwrap();

    let id1 = match &result1 {
        Value::Instance(inst) => inst.addr.instance_id,
        _ => panic!("Expected Instance"),
    };
    let id2 = match &result2 {
        Value::Instance(inst) => inst.addr.instance_id,
        _ => panic!("Expected Instance"),
    };

    // IDs should be unique
    assert_ne!(id1, id2);
}

#[test]
fn test_eval_split_returns_tuple() {
    let ctx = Context::new();

    // First spawn an instance
    let spawn_expr = Expr::Spawn {
        entry_type: "Worker".to_string(),
        init: Box::new(Expr::Literal(Value::Int(42))),
    };

    // Then split it
    let split_expr = Expr::Split(Box::new(spawn_expr));

    let result = eval_expr(&split_expr, &ctx).unwrap();

    // Should return a tuple (InstanceAddr, ControlLink)
    let tuple = result
        .list_to_vec()
        .unwrap_or_else(|| panic!("Expected tuple (List), got {:?}", result));
    assert_eq!(tuple.len(), 2);
    // First element should be InstanceAddr
    assert!(matches!(tuple[0], Value::InstanceAddr(_)));
    // Second element should be ControlLink
    assert!(matches!(tuple[1], Value::ControlLink(_)));
}

#[test]
fn test_eval_split_type_mismatch() {
    let ctx = Context::new();

    // Try to split a non-instance value
    let split_expr = Expr::Split(Box::new(Expr::Literal(Value::Int(42))));

    let result = eval_expr(&split_expr, &ctx);
    assert!(result.is_err());
}

#[test]
fn test_instance_addr_display() {
    let id = WorkflowId::new();
    let addr = InstanceAddr {
        entry_type: "Worker".to_string(),
        instance_id: id,
    };
    let display = format!("{}", addr);
    assert!(display.starts_with("InstanceAddr<Worker:"));
    assert!(display.ends_with(">"));
}

#[test]
fn test_control_link_display() {
    let link = ControlLink {
        instance_id: WorkflowId::new(),
    };
    let display = format!("{}", link);
    assert!(display.starts_with("ControlLink<"));
    assert!(display.ends_with(">"));
}

#[test]
fn test_instance_display() {
    let id = WorkflowId::new();
    let instance = Instance {
        addr: InstanceAddr {
            entry_type: "Worker".to_string(),
            instance_id: id,
        },
        control: Some(ControlLink { instance_id: id }),
    };
    let display = format!("{}", instance);
    assert!(display.contains("Instance {"));
    assert!(display.contains("addr:"));
    assert!(display.contains("control: Some(ControlLink"));
}

#[test]
fn test_instance_display_no_control() {
    let instance = Instance {
        addr: InstanceAddr {
            entry_type: "Worker".to_string(),
            instance_id: WorkflowId::new(),
        },
        control: None,
    };
    let display = format!("{}", instance);
    assert!(display.contains("control: None"));
}

// ============================================================
// TASK-559: SPEC-031 End-to-End Conformance Tests
// ============================================================

/// SPEC-031 §5.1 – FnDef evaluates to Value::Closure capturing the current env.
#[test]
fn task559_fndef_produces_value_closure() {
    let mut ctx = Context::new();
    ctx.set("offset".to_string(), Value::Int(10));

    let expr = Expr::FnDef {
        params: vec![("x".to_string(), None)],
        return_type: None,
        body: Box::new(Expr::Binary {
            op: BinaryOp::Add,
            left: Box::new(Expr::Variable {
                name: "x".to_string(),
                span: ash_core::ast::Span::default(),
            }),
            right: Box::new(Expr::Variable {
                name: "offset".to_string(),
                span: ash_core::ast::Span::default(),
            }),
        }),
    };

    let result = eval_expr(&expr, &ctx).unwrap();
    match &result {
        Value::Closure { params, .. } => assert_eq!(params.len(), 1),
        other => panic!("expected Value::Closure, got {other:?}"),
    }
}

/// SPEC-031 §5.4 – FnApply calls a closure and binds params correctly.
#[test]
fn task559_fnapply_calls_closure() {
    let ctx = Context::new();

    // (fn(x) { x + 1 })(5)  =>  6
    let expr = Expr::FnApply {
        func: Box::new(Expr::FnDef {
            params: vec![("x".to_string(), None)],
            return_type: None,
            body: Box::new(Expr::Binary {
                op: BinaryOp::Add,
                left: Box::new(Expr::Variable {
                    name: "x".to_string(),
                    span: ash_core::ast::Span::default(),
                }),
                right: Box::new(Expr::Literal(Value::Int(1))),
            }),
        }),
        args: vec![Expr::Literal(Value::Int(5))],
    };

    assert_eq!(eval_expr(&expr, &ctx).unwrap(), Value::Int(6));
}

#[test]
fn task689c_projected_callable_invocation_evaluates_through_fnapply() {
    let mut ctx = Context::new();
    let check_closure = Value::Closure {
        params: vec![("_p".to_string(), None)],
        body: Box::new(Expr::Literal(Value::Bool(true))),
        env: std::sync::Arc::new(ash_core::env_frame::EnvFrame::new()),
    };

    let mut policies = HashMap::new();
    policies.insert("check".to_string(), check_closure);

    let mut env_record = HashMap::new();
    env_record.insert("policies".to_string(), Value::Record(Box::new(policies)));
    ctx.set("env".to_string(), Value::Record(Box::new(env_record)));

    let expr = Expr::FnApply {
        func: Box::new(Expr::FieldAccess {
            expr: Box::new(Expr::FieldAccess {
                expr: Box::new(Expr::Variable {
                    name: "env".to_string(),
                    span: ash_core::ast::Span::default(),
                }),
                field: "policies".to_string(),
            }),
            field: "check".to_string(),
        }),
        args: vec![Expr::Literal(Value::String("demo".to_string()))],
    };

    assert_eq!(eval_expr(&expr, &ctx).unwrap(), Value::Bool(true));
}

#[test]
fn task689c_policy_check_fails_closed_without_hidden_policy_context() {
    let ctx = Context::new();
    let expr = Expr::Call {
        func: "policy_check".to_string(),
        module: Some("act".to_string()),
        arguments: vec![Expr::Literal(Value::String("missing".to_string()))],
    };

    assert_eq!(eval_expr(&expr, &ctx).unwrap(), Value::Bool(false));
}

#[test]
fn task689c_policy_check_uses_hidden_policy_evaluator() {
    let mut evaluator = crate::policy::PolicyEvaluator::new();
    let policy = crate::policy::Policy::new("allow-large")
        .with_rule(crate::policy::PolicyRule::new(
            "allow x > 10",
            Expr::Binary {
                op: BinaryOp::Gt,
                left: Box::new(Expr::Variable {
                    name: "x".to_string(),
                    span: ash_core::ast::Span::default(),
                }),
                right: Box::new(Expr::Literal(Value::Int(10))),
            },
            ash_core::Decision::Permit,
        ))
        .with_default(ash_core::Decision::Deny);
    evaluator.register(policy);

    let mut ctx = Context::new().with_policy_evaluator(evaluator);
    ctx.set("x".to_string(), Value::Int(15));

    let expr = Expr::Call {
        func: "policy_check".to_string(),
        module: Some("act".to_string()),
        arguments: vec![Expr::Literal(Value::String("allow-large".to_string()))],
    };

    assert_eq!(eval_expr(&expr, &ctx).unwrap(), Value::Bool(true));

    let mut denied_ctx = ctx.clone();
    denied_ctx.set("x".to_string(), Value::Int(1));
    assert_eq!(eval_expr(&expr, &denied_ctx).unwrap(), Value::Bool(false));
}

/// SPEC-031 §5.3 – Closure captures the enclosing scope (make_adder pattern).
#[test]
fn task559_closure_captures_enclosing_scope() {
    // Build make_adder closure: fn(n) { fn(x) { x + n } }
    let mut ctx = Context::new();
    let make_adder = Expr::FnDef {
        params: vec![("n".to_string(), None)],
        return_type: None,
        body: Box::new(Expr::FnDef {
            params: vec![("x".to_string(), None)],
            return_type: None,
            body: Box::new(Expr::Binary {
                op: BinaryOp::Add,
                left: Box::new(Expr::Variable {
                    name: "x".to_string(),
                    span: ash_core::ast::Span::default(),
                }),
                right: Box::new(Expr::Variable {
                    name: "n".to_string(),
                    span: ash_core::ast::Span::default(),
                }),
            }),
        }),
    };

    // adder5 = make_adder(5)
    let adder_closure = eval_expr(
        &Expr::FnApply {
            func: Box::new(make_adder),
            args: vec![Expr::Literal(Value::Int(5))],
        },
        &ctx,
    )
    .unwrap();

    ctx.set("add5".to_string(), adder_closure);

    // add5(3) => 8
    let result = eval_expr(
        &Expr::FnApply {
            func: Box::new(Expr::Variable {
                name: "add5".to_string(),
                span: ash_core::ast::Span::default(),
            }),
            args: vec![Expr::Literal(Value::Int(3))],
        },
        &ctx,
    )
    .unwrap();

    assert_eq!(result, Value::Int(8));
}

/// SPEC-031 §5.2 – Higher-order function: apply(f, x) = f(x).
#[test]
fn task559_higher_order_function_apply() {
    let ctx = Context::new();

    // apply = fn(f, x) { f(x) }  -- wait, Core FnApply needs Expr::FnApply
    // Use: (fn(f) { f(10) })(fn(x) { x * 2 }) => 20
    let double_fn = Expr::FnDef {
        params: vec![("x".to_string(), None)],
        return_type: None,
        body: Box::new(Expr::Binary {
            op: BinaryOp::Mul,
            left: Box::new(Expr::Variable {
                name: "x".to_string(),
                span: ash_core::ast::Span::default(),
            }),
            right: Box::new(Expr::Literal(Value::Int(2))),
        }),
    };

    let apply_fn = Expr::FnDef {
        params: vec![("f".to_string(), None)],
        return_type: None,
        body: Box::new(Expr::FnApply {
            func: Box::new(Expr::Variable {
                name: "f".to_string(),
                span: ash_core::ast::Span::default(),
            }),
            args: vec![Expr::Literal(Value::Int(10))],
        }),
    };

    let result = eval_expr(
        &Expr::FnApply {
            func: Box::new(apply_fn),
            args: vec![double_fn],
        },
        &ctx,
    )
    .unwrap();

    assert_eq!(result, Value::Int(20));
}

/// SPEC-031 §5.5 – Recursion via BindingSlot::Late: factorial(5) = 120.
#[test]
fn task559_recursive_closure_via_late_binding() {
    use ash_core::env_frame::EnvFrame;
    use std::sync::Arc;

    // 1. Create env frame with a late slot for `fact`
    let mut frame = EnvFrame::new();
    let late_slot = frame.insert_late("fact".to_string());
    let env = Arc::new(frame);

    // 2. Build the factorial body:
    //    match n { 0 => 1, _ => n * fact(n-1) }
    let body = Expr::Match {
        scrutinee: Box::new(Expr::Variable {
            name: "n".to_string(),
            span: ash_core::ast::Span::default(),
        }),
        arms: vec![
            MatchArm {
                pattern: Pattern::Literal(Value::Int(0)),
                body: Expr::Literal(Value::Int(1)),
            },
            MatchArm {
                pattern: Pattern::Wildcard,
                body: Expr::Binary {
                    op: BinaryOp::Mul,
                    left: Box::new(Expr::Variable {
                        name: "n".to_string(),
                        span: ash_core::ast::Span::default(),
                    }),
                    right: Box::new(Expr::FnApply {
                        func: Box::new(Expr::Variable {
                            name: "fact".to_string(),
                            span: ash_core::ast::Span::default(),
                        }),
                        args: vec![Expr::Binary {
                            op: BinaryOp::Sub,
                            left: Box::new(Expr::Variable {
                                name: "n".to_string(),
                                span: ash_core::ast::Span::default(),
                            }),
                            right: Box::new(Expr::Literal(Value::Int(1))),
                        }],
                    }),
                },
            },
        ],
    };

    // 3. Construct the closure manually with the env that has the late slot
    let fact_closure = Value::Closure {
        params: vec![("n".to_string(), None)],
        body: Box::new(body),
        env: env.clone(),
    };

    // 4. Fill the late slot so recursive calls resolve
    late_slot.set_late(fact_closure.clone());

    // 5. Call fact(5) from a context that has fact bound
    let mut ctx = Context::new();
    ctx.set("fact".to_string(), fact_closure);

    let result = eval_expr(
        &Expr::FnApply {
            func: Box::new(Expr::Variable {
                name: "fact".to_string(),
                span: ash_core::ast::Span::default(),
            }),
            args: vec![Expr::Literal(Value::Int(5))],
        },
        &ctx,
    )
    .unwrap();

    assert_eq!(result, Value::Int(120), "fact(5) must equal 120");
}

/// SPEC-031 §10.2 – Value::Closure is Send + Sync (compile-time assertion).
///
/// This test doesn't run code — the fact it compiles proves the constraint.
#[test]
fn task559_closure_is_send_sync() {
    fn assert_send_sync<T: Send + Sync>() {}
    assert_send_sync::<Value>();
}

/// SPEC-031 §10.3 – Serializing a Value::Closure returns an error.
#[test]
fn task559_closure_serialization_returns_error() {
    use ash_core::env_frame::EnvFrame;
    use std::sync::Arc;

    let closure = Value::Closure {
        params: vec![("x".to_string(), None)],
        body: Box::new(Expr::Variable {
            name: "x".to_string(),
            span: ash_core::ast::Span::default(),
        }),
        env: Arc::new(EnvFrame::new()),
    };

    let result = serde_json::to_string(&closure);
    assert!(
        result.is_err(),
        "serializing Value::Closure must return an error, got Ok"
    );
    let err_msg = result.unwrap_err().to_string();
    assert!(
        err_msg.contains("cannot be serialized"),
        "error message should explain why: {err_msg}"
    );
}

/// SPEC-031 §5.6 – Calling a non-closure value via FnApply returns NotCallable.
#[test]
fn task559_fnapply_non_callable_returns_error() {
    let mut ctx = Context::new();
    ctx.set("not_a_fn".to_string(), Value::Int(42));

    let expr = Expr::FnApply {
        func: Box::new(Expr::Variable {
            name: "not_a_fn".to_string(),
            span: ash_core::ast::Span::default(),
        }),
        args: vec![Expr::Literal(Value::Int(1))],
    };

    let err = eval_expr(&expr, &ctx).unwrap_err();
    assert!(
        matches!(err, EvalError::NotCallable { .. }),
        "expected NotCallable, got {err:?}"
    );
}

/// SPEC-072 C72-3 – FnApply with wrong arity returns a WrongArity error.
#[test]
fn task559_fnapply_wrong_arity_returns_error() {
    let ctx = Context::new();

    let expr = Expr::FnApply {
        func: Box::new(Expr::FnDef {
            params: vec![("x".to_string(), None), ("y".to_string(), None)],
            return_type: None,
            body: Box::new(Expr::Variable {
                name: "x".to_string(),
                span: ash_core::ast::Span::default(),
            }),
        }),
        args: vec![Expr::Literal(Value::Int(1))], // only 1 arg, need 2
    };

    let err = eval_expr(&expr, &ctx).unwrap_err();
    assert!(
        matches!(
            err,
            EvalError::WrongArity {
                expected: 2,
                actual: 1,
                callee: None,
            }
        ),
        "expected exact-arity WrongArity, got {err:?}"
    );
}

#[test]
fn task962_builtin_call_too_few_args_returns_wrong_arity() {
    let ctx = Context::new();

    let expr = Expr::Call {
        func: "ends_with".to_string(),
        module: None,
        arguments: vec![Expr::Literal(Value::String(".md".to_string()))],
    };

    let err = eval_expr(&expr, &ctx).unwrap_err();
    assert!(
        matches!(
            err,
            EvalError::WrongArity {
                expected: 2,
                actual: 1,
                callee: Some(ref callee),
            } if callee == "ends_with"
        ),
        "expected builtin exact-arity WrongArity, got {err:?}"
    );
}

/// SPEC-031 §4.8 / §10 – BoundaryViolation can be constructed with a
/// Value::Closure and a descriptive context string.
///
/// The `is_pure()` guard exists in `eval_expr` and is exercised by
/// `task559_boundary_violation_in_pure_context` using the test-only
/// `Context::enter_pure()` method.  In production, the type checker is
/// the primary enforcement mechanism; the runtime safety net will
/// activate once the interpreter propagates purity context through
/// closure application.
#[test]
fn task559_boundary_violation_on_context_boundary_crossing() {
    use ash_core::env_frame::EnvFrame;
    use std::sync::Arc;

    // Construct a Value::Closure (the kind of value that would trigger a
    // boundary violation if it crossed into a pure context).
    let closure_value = Value::Closure {
        params: vec![("x".to_string(), None)],
        body: Box::new(Expr::Variable {
            name: "x".to_string(),
            span: ash_core::ast::Span::default(),
        }),
        env: Arc::new(EnvFrame::new()),
    };

    // Build the error — this is the code path that SPEC-031 §4.8 escape
    // case 5 describes.
    let err = EvalError::BoundaryViolation {
        value: Box::new(closure_value),
        context: "closure escaped pure vertex boundary".to_string(),
    };

    // Verify the display message contains the required fragments.
    let msg = err.to_string();
    assert!(
        msg.contains("three-vertex boundary"),
        "BoundaryViolation message should mention three-vertex boundary, got: {msg}"
    );
    assert!(
        msg.contains("closure escaped pure vertex boundary"),
        "BoundaryViolation message should contain context string, got: {msg}"
    );
}

/// SPEC-088 – Runtime enforcement: Expr::FnDef inside a pure context
/// with no effectful captures is allowed. Only effectful captures are rejected.
#[test]
fn task559_pure_closure_with_no_captures_allowed() {
    use crate::context::Context;

    // Create a pure context
    let base = Context::new();
    let pure_ctx = base.enter_pure();

    // FnDef with no captures inside a pure context should be ALLOWED
    let expr = Expr::FnDef {
        params: vec![("x".into(), None)],
        return_type: None,
        body: Box::new(Expr::Variable {
            name: "x".into(),
            span: ash_core::ast::Span::default(),
        }),
    };

    let result = eval_expr(&expr, &pure_ctx);
    assert!(
        matches!(result, Ok(Value::Closure { .. })),
        "expected Closure in pure context with no captures, got {result:?}"
    );
}

/// SPEC-088 – Runtime enforcement: Expr::FnDef inside a pure context
/// with effectful captures raises CaptureEffectViolation.
#[test]
fn task559_capture_effect_violation_in_pure_context() {
    use crate::context::Context;

    // Create a pure context with an effectful binding (capability)
    let mut base = Context::new();
    base.set("fs".into(), Value::Cap("std::io::fs".into()));
    let pure_ctx = base.enter_pure();

    // FnDef that captures an effectful value in a pure context should be rejected
    let expr = Expr::FnDef {
        params: vec![("x".into(), None)],
        return_type: None,
        body: Box::new(Expr::Variable {
            name: "fs".into(),
            span: ash_core::ast::Span::default(),
        }),
    };

    let result = eval_expr(&expr, &pure_ctx);
    assert!(
        matches!(result, Err(EvalError::CaptureEffectViolation { .. })),
        "expected CaptureEffectViolation in pure context with effectful capture, got {result:?}"
    );
}

/// SPEC-031 §13.1 – Module-level functions are never Value::Closure.
///
/// Module-level functions are invoked directly by name (Expr::Call in a
/// module context) and return their evaluated result, not an intermediate
/// Value::Closure.  By contrast, Expr::FnDef at the expression level DOES
/// produce Value::Closure (see `task559_fndef_produces_value_closure`).
///
/// This test simulates the return value of a module-level function call
/// and asserts it is never a Closure.
#[test]
fn task559_module_level_fndef_never_produces_closure() {
    // Simulate the result of calling a module-level function.
    // Module-level functions evaluate to their *body result*, not a Closure.
    let result = Value::Int(42);

    // A module-level function return must never be a Closure.
    assert!(
        !matches!(result, Value::Closure { .. }),
        "module-level function call must not return Value::Closure, got {result:?}"
    );

    // Positive contrast: Expr::FnDef at expression level DOES produce Closure.
    // (Already covered by `task559_fndef_produces_value_closure` — this block
    // confirms the same fact inline for documentation purposes.)
    let ctx = Context::new();
    let fndef = Expr::FnDef {
        params: vec![("x".to_string(), None)],
        return_type: None,
        body: Box::new(Expr::Variable {
            name: "x".to_string(),
            span: ash_core::ast::Span::default(),
        }),
    };
    let closure_result = eval_expr(&fndef, &ctx).unwrap();
    assert!(
        matches!(closure_result, Value::Closure { .. }),
        "expression-level FnDef should produce Value::Closure, got {closure_result:?}"
    );
}

// ── TASK-653: Short-circuit and/or evaluation (SPEC-004) ──────────

/// SPEC-004 EXPR-AND-FALSE: `false && <error>` returns `false` without
/// evaluating the right operand.
#[test]
fn task653_and_short_circuits_on_false() {
    let ctx = Context::new();

    // false and (1 / 0) — division by zero on the right must not fire
    let expr = Expr::Binary {
        op: BinaryOp::And,
        left: Box::new(Expr::Literal(Value::Bool(false))),
        right: Box::new(Expr::Binary {
            op: BinaryOp::Div,
            left: Box::new(Expr::Literal(Value::Int(1))),
            right: Box::new(Expr::Literal(Value::Int(0))),
        }),
    };

    let result = eval_expr(&expr, &ctx);
    assert_eq!(result.unwrap(), Value::Bool(false));
}

/// SPEC-004 EXPR-OR-TRUE: `true || <error>` returns `true` without
/// evaluating the right operand.
#[test]
fn task653_or_short_circuits_on_true() {
    let ctx = Context::new();

    // true or (1 / 0) — division by zero on the right must not fire
    let expr = Expr::Binary {
        op: BinaryOp::Or,
        left: Box::new(Expr::Literal(Value::Bool(true))),
        right: Box::new(Expr::Binary {
            op: BinaryOp::Div,
            left: Box::new(Expr::Literal(Value::Int(1))),
            right: Box::new(Expr::Literal(Value::Int(0))),
        }),
    };

    let result = eval_expr(&expr, &ctx);
    assert_eq!(result.unwrap(), Value::Bool(true));
}

/// `true && false` returns `false` (both operands evaluated).
#[test]
fn task653_and_both_evaluated_when_left_true() {
    let ctx = Context::new();
    let expr = Expr::Binary {
        op: BinaryOp::And,
        left: Box::new(Expr::Literal(Value::Bool(true))),
        right: Box::new(Expr::Literal(Value::Bool(false))),
    };
    assert_eq!(eval_expr(&expr, &ctx).unwrap(), Value::Bool(false));
}

/// `false || true` returns `true` (both operands evaluated).
#[test]
fn task653_or_both_evaluated_when_left_false() {
    let ctx = Context::new();
    let expr = Expr::Binary {
        op: BinaryOp::Or,
        left: Box::new(Expr::Literal(Value::Bool(false))),
        right: Box::new(Expr::Literal(Value::Bool(true))),
    };
    assert_eq!(eval_expr(&expr, &ctx).unwrap(), Value::Bool(true));
}

/// Non-boolean left operand in `and` produces a type error.
#[test]
fn task653_and_non_bool_left_is_error() {
    let ctx = Context::new();
    let expr = Expr::Binary {
        op: BinaryOp::And,
        left: Box::new(Expr::Literal(Value::Int(1))),
        right: Box::new(Expr::Literal(Value::Bool(true))),
    };
    assert!(eval_expr(&expr, &ctx).is_err());
}

/// Non-boolean left operand in `or` produces a type error.
#[test]
fn task653_or_non_bool_left_is_error() {
    let ctx = Context::new();
    let expr = Expr::Binary {
        op: BinaryOp::Or,
        left: Box::new(Expr::Literal(Value::Int(0))),
        right: Box::new(Expr::Literal(Value::Bool(false))),
    };
    assert!(eval_expr(&expr, &ctx).is_err());
}

// ── TASK-650: Expr::Let evaluation tests ────────────────────────

/// Simple let binding: `let x = 42; x` evaluates to 42
#[test]
fn task650_let_simple_binding() {
    let ctx = Context::new();
    let expr = Expr::Let {
        pattern: Pattern::Variable {
            name: "x".to_string(),
            span: ash_core::ast::Span::default(),
        },
        expr: Box::new(Expr::Literal(Value::Int(42))),
        body: Box::new(Expr::Variable {
            name: "x".to_string(),
            span: ash_core::ast::Span::default(),
        }),
        span: ash_core::ast::Span::default(),
    };
    assert_eq!(eval_expr(&expr, &ctx).unwrap(), Value::Int(42));
}

/// Nested let: `let x = 1; let y = 2; y` evaluates to 2
#[test]
fn task650_let_nested_binding() {
    let ctx = Context::new();
    let inner = Expr::Let {
        pattern: Pattern::Variable {
            name: "y".to_string(),
            span: ash_core::ast::Span::default(),
        },
        expr: Box::new(Expr::Literal(Value::Int(2))),
        body: Box::new(Expr::Variable {
            name: "y".to_string(),
            span: ash_core::ast::Span::default(),
        }),
        span: ash_core::ast::Span::default(),
    };
    let outer = Expr::Let {
        pattern: Pattern::Variable {
            name: "x".to_string(),
            span: ash_core::ast::Span::default(),
        },
        expr: Box::new(Expr::Literal(Value::Int(1))),
        body: Box::new(inner),
        span: ash_core::ast::Span::default(),
    };
    assert_eq!(eval_expr(&outer, &ctx).unwrap(), Value::Int(2));
}

/// Scope isolation: `let x = 1; let x = 2; x` evaluates to 2 (inner shadows outer)
#[test]
fn task650_let_shadowing() {
    let ctx = Context::new();
    let inner = Expr::Let {
        pattern: Pattern::Variable {
            name: "x".to_string(),
            span: ash_core::ast::Span::default(),
        },
        expr: Box::new(Expr::Literal(Value::Int(2))),
        body: Box::new(Expr::Variable {
            name: "x".to_string(),
            span: ash_core::ast::Span::default(),
        }),
        span: ash_core::ast::Span::default(),
    };
    let outer = Expr::Let {
        pattern: Pattern::Variable {
            name: "x".to_string(),
            span: ash_core::ast::Span::default(),
        },
        expr: Box::new(Expr::Literal(Value::Int(1))),
        body: Box::new(inner),
        span: ash_core::ast::Span::default(),
    };
    assert_eq!(eval_expr(&outer, &ctx).unwrap(), Value::Int(2));
}

/// Let binding doesn't leak into parent scope.
#[test]
fn task650_let_no_scope_leak() {
    let ctx = Context::new();
    // let x = 42; x  -- evaluates to 42, but x is not in parent
    let let_expr = Expr::Let {
        pattern: Pattern::Variable {
            name: "x".to_string(),
            span: ash_core::ast::Span::default(),
        },
        expr: Box::new(Expr::Literal(Value::Int(42))),
        body: Box::new(Expr::Variable {
            name: "x".to_string(),
            span: ash_core::ast::Span::default(),
        }),
        span: ash_core::ast::Span::default(),
    };
    // After evaluating the let, x should NOT be in ctx
    let result = eval_expr(&let_expr, &ctx);
    assert_eq!(result.unwrap(), Value::Int(42));
    // Verify x is NOT accessible in the original context
    assert!(ctx.get("x").is_none());
}

/// Tuple destructuring: `let (a, b) = (1, 2); a` — uses List since no Value::Tuple.
/// Test list pattern destructuring: `let [a, b] = [1, 2]; a`
#[test]
fn task650_let_list_destructure() {
    let ctx = Context::new();
    let expr = Expr::Let {
        pattern: Pattern::List(
            vec![
                Pattern::Variable {
                    name: "a".to_string(),
                    span: ash_core::ast::Span::default(),
                },
                Pattern::Variable {
                    name: "b".to_string(),
                    span: ash_core::ast::Span::default(),
                },
            ],
            None,
        ),
        expr: Box::new(Expr::Literal(Value::list_from_vec(vec![
            Value::Int(1),
            Value::Int(2),
        ]))),
        body: Box::new(Expr::Variable {
            name: "a".to_string(),
            span: ash_core::ast::Span::default(),
        }),
        span: ash_core::ast::Span::default(),
    };
    assert_eq!(eval_expr(&expr, &ctx).unwrap(), Value::Int(1));
}

// ── markdown::parse builtin tests ──

#[test]
fn test_markdown_parse_heading() {
    let ctx = Context::new();
    let result = eval_function_call(
        "parse",
        Some("markdown"),
        &[Value::String("# Hello\n\nWorld".to_string())],
        &ctx,
    );
    let json_str = match result.unwrap() {
        Value::String(s) => s,
        other => panic!("expected String, got {other:?}"),
    };
    let val: serde_json::Value = serde_json::from_str(&json_str).expect("should be valid JSON");
    let blocks = val["blocks"].as_array().expect("blocks should be array");
    assert_eq!(blocks.len(), 2);
    assert_eq!(blocks[0]["type"], "heading");
    assert_eq!(blocks[0]["level"], 1);
    assert_eq!(blocks[0]["text"], "Hello");
    assert_eq!(blocks[1]["type"], "paragraph");
    assert_eq!(blocks[1]["text"], "World");
}

#[test]
fn test_markdown_parse_paragraph() {
    let ctx = Context::new();
    let result = eval_function_call(
        "parse",
        Some("markdown"),
        &[Value::String("Hello world".to_string())],
        &ctx,
    );
    let json_str = match result.unwrap() {
        Value::String(s) => s,
        other => panic!("expected String, got {other:?}"),
    };
    let val: serde_json::Value = serde_json::from_str(&json_str).expect("should be valid JSON");
    let blocks = val["blocks"].as_array().expect("blocks should be array");
    assert_eq!(blocks.len(), 1);
    assert_eq!(blocks[0]["type"], "paragraph");
    assert_eq!(blocks[0]["text"], "Hello world");
}

#[test]
fn test_markdown_parse_code_block() {
    let ctx = Context::new();
    let input = "```rust\nfn main() {}\n```".to_string();
    let result = eval_function_call("parse", Some("markdown"), &[Value::String(input)], &ctx);
    let json_str = match result.unwrap() {
        Value::String(s) => s,
        other => panic!("expected String, got {other:?}"),
    };
    let val: serde_json::Value = serde_json::from_str(&json_str).expect("should be valid JSON");
    let blocks = val["blocks"].as_array().expect("blocks should be array");
    assert_eq!(blocks.len(), 1);
    assert_eq!(blocks[0]["type"], "code_block");
    assert_eq!(blocks[0]["language"], "rust");
    assert_eq!(blocks[0]["text"], "fn main() {}");
}

#[test]
fn test_markdown_parse_empty_input() {
    let ctx = Context::new();
    let result = eval_function_call(
        "parse",
        Some("markdown"),
        &[Value::String(String::new())],
        &ctx,
    );
    let json_str = match result.unwrap() {
        Value::String(s) => s,
        other => panic!("expected String, got {other:?}"),
    };
    let val: serde_json::Value = serde_json::from_str(&json_str).expect("should be valid JSON");
    let blocks = val["blocks"].as_array().expect("blocks should be array");
    assert!(blocks.is_empty());
}

#[test]
fn test_markdown_parse_arity_error() {
    let ctx = Context::new();
    let result = eval_function_call("parse", Some("markdown"), &[], &ctx);
    assert!(result.is_err());
}

// ============================================================
// TASK-661: string::to_upper, string::to_lower, string::trim
// ============================================================

#[test]
fn test_string_to_upper_basic() {
    let ctx = Context::new();
    let result = eval_function_call(
        "to_upper",
        Some("string"),
        &[Value::String("hello".to_string())],
        &ctx,
    );
    assert_eq!(result.unwrap(), Value::String("HELLO".to_string()));
}

#[test]
fn test_string_to_upper_already_upper() {
    let ctx = Context::new();
    let result = eval_function_call(
        "to_upper",
        Some("string"),
        &[Value::String("HELLO".to_string())],
        &ctx,
    );
    assert_eq!(result.unwrap(), Value::String("HELLO".to_string()));
}

#[test]
fn test_string_to_upper_mixed_case() {
    let ctx = Context::new();
    let result = eval_function_call(
        "to_upper",
        Some("string"),
        &[Value::String("hElLo".to_string())],
        &ctx,
    );
    assert_eq!(result.unwrap(), Value::String("HELLO".to_string()));
}

#[test]
fn test_string_to_upper_empty() {
    let ctx = Context::new();
    let result = eval_function_call(
        "to_upper",
        Some("string"),
        &[Value::String(String::new())],
        &ctx,
    );
    assert_eq!(result.unwrap(), Value::String(String::new()));
}

#[test]
fn test_string_to_upper_type_error() {
    let ctx = Context::new();
    let result = eval_function_call("to_upper", Some("string"), &[Value::Int(42)], &ctx);
    assert!(result.is_err());
}

#[test]
fn test_string_to_upper_arity_error() {
    let ctx = Context::new();
    let result = eval_function_call("to_upper", Some("string"), &[], &ctx);
    assert!(result.is_err());
}

#[test]
fn test_string_to_lower_basic() {
    let ctx = Context::new();
    let result = eval_function_call(
        "to_lower",
        Some("string"),
        &[Value::String("HELLO".to_string())],
        &ctx,
    );
    assert_eq!(result.unwrap(), Value::String("hello".to_string()));
}

#[test]
fn test_string_to_lower_already_lower() {
    let ctx = Context::new();
    let result = eval_function_call(
        "to_lower",
        Some("string"),
        &[Value::String("hello".to_string())],
        &ctx,
    );
    assert_eq!(result.unwrap(), Value::String("hello".to_string()));
}

#[test]
fn test_string_to_lower_mixed_case() {
    let ctx = Context::new();
    let result = eval_function_call(
        "to_lower",
        Some("string"),
        &[Value::String("HeLLo".to_string())],
        &ctx,
    );
    assert_eq!(result.unwrap(), Value::String("hello".to_string()));
}

#[test]
fn test_string_to_lower_empty() {
    let ctx = Context::new();
    let result = eval_function_call(
        "to_lower",
        Some("string"),
        &[Value::String(String::new())],
        &ctx,
    );
    assert_eq!(result.unwrap(), Value::String(String::new()));
}

#[test]
fn test_string_to_lower_type_error() {
    let ctx = Context::new();
    let result = eval_function_call("to_lower", Some("string"), &[Value::Bool(true)], &ctx);
    assert!(result.is_err());
}

#[test]
fn test_string_to_lower_arity_error() {
    let ctx = Context::new();
    let result = eval_function_call("to_lower", Some("string"), &[], &ctx);
    assert!(result.is_err());
}

#[test]
fn test_string_trim_basic() {
    let ctx = Context::new();
    let result = eval_function_call(
        "trim",
        Some("string"),
        &[Value::String("  hi  ".to_string())],
        &ctx,
    );
    assert_eq!(result.unwrap(), Value::String("hi".to_string()));
}

#[test]
fn test_string_trim_leading() {
    let ctx = Context::new();
    let result = eval_function_call(
        "trim",
        Some("string"),
        &[Value::String("   hello".to_string())],
        &ctx,
    );
    assert_eq!(result.unwrap(), Value::String("hello".to_string()));
}

#[test]
fn test_string_trim_trailing() {
    let ctx = Context::new();
    let result = eval_function_call(
        "trim",
        Some("string"),
        &[Value::String("hello   ".to_string())],
        &ctx,
    );
    assert_eq!(result.unwrap(), Value::String("hello".to_string()));
}

#[test]
fn test_string_trim_no_whitespace() {
    let ctx = Context::new();
    let result = eval_function_call(
        "trim",
        Some("string"),
        &[Value::String("hello".to_string())],
        &ctx,
    );
    assert_eq!(result.unwrap(), Value::String("hello".to_string()));
}

#[test]
fn test_string_trim_empty() {
    let ctx = Context::new();
    let result = eval_function_call(
        "trim",
        Some("string"),
        &[Value::String(String::new())],
        &ctx,
    );
    assert_eq!(result.unwrap(), Value::String(String::new()));
}

#[test]
fn test_string_trim_only_whitespace() {
    let ctx = Context::new();
    let result = eval_function_call(
        "trim",
        Some("string"),
        &[Value::String("   ".to_string())],
        &ctx,
    );
    assert_eq!(result.unwrap(), Value::String(String::new()));
}

#[test]
fn test_string_trim_type_error() {
    let ctx = Context::new();
    let result = eval_function_call("trim", Some("string"), &[Value::Int(42)], &ctx);
    assert!(result.is_err());
}

#[test]
fn test_string_trim_arity_error() {
    let ctx = Context::new();
    let result = eval_function_call("trim", Some("string"), &[], &ctx);
    assert!(result.is_err());
}

// ============================================================
// TASK-664: list::filter and list::map closure callback tests
// ============================================================

/// Helper: build a simple 1-param closure (`|x| -> body`).
fn simple_closure(body: Expr) -> Value {
    use ash_core::env_frame::EnvFrame;
    use std::sync::Arc;
    Value::Closure {
        params: vec![("x".to_string(), None)],
        body: Box::new(body),
        env: Arc::new(EnvFrame::new()),
    }
}

// ── filter tests ──────────────────────────────────────────────

#[test]
fn test_filter_keep_greater_than_3() {
    let ctx = Context::new();
    // filter [1, 4, 2, 5, 6, 3] with (x > 3)
    let closure = simple_closure(Expr::Binary {
        op: BinaryOp::Gt,
        left: Box::new(Expr::Variable {
            name: "x".to_string(),
            span: ash_core::ast::Span::default(),
        }),
        right: Box::new(Expr::Literal(Value::Int(3))),
    });
    let list = Value::list_from_vec(vec![
        Value::Int(1),
        Value::Int(4),
        Value::Int(2),
        Value::Int(5),
        Value::Int(6),
        Value::Int(3),
    ]);
    let result = eval_function_call("filter", None, &[list, closure], &ctx).unwrap();
    assert_eq!(
        result,
        Value::list_from_vec(vec![Value::Int(4), Value::Int(5), Value::Int(6)])
    );
}

#[test]
fn test_filter_keeps_nothing() {
    let ctx = Context::new();
    // filter [1, 2, 3] with (x > 100) → []
    let closure = simple_closure(Expr::Binary {
        op: BinaryOp::Gt,
        left: Box::new(Expr::Variable {
            name: "x".to_string(),
            span: ash_core::ast::Span::default(),
        }),
        right: Box::new(Expr::Literal(Value::Int(100))),
    });
    let list = Value::list_from_vec(vec![Value::Int(1), Value::Int(2), Value::Int(3)]);
    let result = eval_function_call("filter", None, &[list, closure], &ctx).unwrap();
    assert_eq!(result, Value::list_nil());
}

#[test]
fn test_filter_keeps_everything() {
    let ctx = Context::new();
    // filter [1, 2, 3] with (x > 0) → [1, 2, 3]
    let closure = simple_closure(Expr::Binary {
        op: BinaryOp::Gt,
        left: Box::new(Expr::Variable {
            name: "x".to_string(),
            span: ash_core::ast::Span::default(),
        }),
        right: Box::new(Expr::Literal(Value::Int(0))),
    });
    let list = Value::list_from_vec(vec![Value::Int(1), Value::Int(2), Value::Int(3)]);
    let result = eval_function_call("filter", None, &[list, closure], &ctx).unwrap();
    assert_eq!(
        result,
        Value::list_from_vec(vec![Value::Int(1), Value::Int(2), Value::Int(3),])
    );
}

// ── map tests ─────────────────────────────────────────────────

#[test]
fn test_map_double_elements() {
    let ctx = Context::new();
    // map [1, 2, 3] with (x * 2) → [2, 4, 6]
    let closure = simple_closure(Expr::Binary {
        op: BinaryOp::Mul,
        left: Box::new(Expr::Variable {
            name: "x".to_string(),
            span: ash_core::ast::Span::default(),
        }),
        right: Box::new(Expr::Literal(Value::Int(2))),
    });
    let list = Value::list_from_vec(vec![Value::Int(1), Value::Int(2), Value::Int(3)]);
    let result = eval_function_call("map", None, &[list, closure], &ctx).unwrap();
    assert_eq!(
        result,
        Value::list_from_vec(vec![Value::Int(2), Value::Int(4), Value::Int(6),])
    );
}

#[test]
fn test_map_string_transform() {
    let ctx = Context::new();
    // map ["a", "b"] with (x + "!") → ["a!", "b!"]
    let closure = simple_closure(Expr::Binary {
        op: BinaryOp::Add,
        left: Box::new(Expr::Variable {
            name: "x".to_string(),
            span: ash_core::ast::Span::default(),
        }),
        right: Box::new(Expr::Literal(Value::String("!".to_string()))),
    });
    let list = Value::list_from_vec(vec![
        Value::String("a".to_string()),
        Value::String("b".to_string()),
    ]);
    let result = eval_function_call("map", None, &[list, closure], &ctx).unwrap();
    assert_eq!(
        result,
        Value::list_from_vec(vec![
            Value::String("a!".to_string()),
            Value::String("b!".to_string()),
        ])
    );
}

// ── filter/map error cases ────────────────────────────────────

#[test]
fn test_filter_wrong_first_arg_type() {
    let ctx = Context::new();
    let closure = simple_closure(Expr::Literal(Value::Bool(true)));
    // filter(42, closure) → TypeMismatch
    let result = eval_function_call("filter", None, &[Value::Int(42), closure], &ctx);
    assert!(result.is_err());
}

#[test]
fn test_filter_wrong_second_arg_type() {
    let ctx = Context::new();
    let list = Value::list_from_vec(vec![Value::Int(1)]);
    // filter(list, 99) → TypeMismatch
    let result = eval_function_call("filter", None, &[list, Value::Int(99)], &ctx);
    assert!(result.is_err());
}

#[test]
fn test_map_wrong_first_arg_type() {
    let ctx = Context::new();
    let closure = simple_closure(Expr::Literal(Value::Int(0)));
    // map("hello", closure) → TypeMismatch
    let result = eval_function_call(
        "map",
        None,
        &[Value::String("hello".to_string()), closure],
        &ctx,
    );
    assert!(result.is_err());
}

#[test]
fn test_map_wrong_second_arg_type() {
    let ctx = Context::new();
    let list = Value::list_from_vec(vec![Value::Int(1)]);
    // map(list, true) → TypeMismatch
    let result = eval_function_call("map", None, &[list, Value::Bool(true)], &ctx);
    assert!(result.is_err());
}

#[test]
fn test_filter_wrong_arity_too_few() {
    let ctx = Context::new();
    let result = eval_function_call("filter", None, &[], &ctx);
    assert!(result.is_err());
}

#[test]
fn test_filter_wrong_arity_too_many() {
    let ctx = Context::new();
    let closure = simple_closure(Expr::Literal(Value::Bool(true)));
    let result = eval_function_call(
        "filter",
        None,
        &[Value::list_nil(), closure, Value::Int(1)],
        &ctx,
    );
    assert!(result.is_err());
}

#[test]
fn test_map_wrong_arity_too_few() {
    let ctx = Context::new();
    let result = eval_function_call("map", None, &[], &ctx);
    assert!(result.is_err());
}

#[test]
fn test_map_wrong_arity_too_many() {
    let ctx = Context::new();
    let closure = simple_closure(Expr::Literal(Value::Int(0)));
    let result = eval_function_call(
        "map",
        None,
        &[Value::list_nil(), closure, Value::Int(1)],
        &ctx,
    );
    assert!(result.is_err());
}

#[test]
fn test_filter_closure_wrong_param_count() {
    let ctx = Context::new();
    use ash_core::env_frame::EnvFrame;
    use std::sync::Arc;
    // Closure with 0 params → WrongArity
    let closure = Value::Closure {
        params: vec![],
        body: Box::new(Expr::Literal(Value::Bool(true))),
        env: Arc::new(EnvFrame::new()),
    };
    let list = Value::list_from_vec(vec![Value::Int(1)]);
    let result = eval_function_call("filter", None, &[list, closure], &ctx);
    assert!(result.is_err());
}

#[test]
fn test_map_closure_wrong_param_count() {
    let ctx = Context::new();
    use ash_core::env_frame::EnvFrame;
    use std::sync::Arc;
    // Closure with 2 params → WrongArity
    let closure = Value::Closure {
        params: vec![("x".to_string(), None), ("y".to_string(), None)],
        body: Box::new(Expr::Literal(Value::Int(0))),
        env: Arc::new(EnvFrame::new()),
    };
    let list = Value::list_from_vec(vec![Value::Int(1)]);
    let result = eval_function_call("map", None, &[list, closure], &ctx);
    assert!(result.is_err());
}

fn proc_unit_expr(expr: Expr) -> Expr {
    Expr::Call {
        func: "unit".to_string(),
        module: Some("proc".to_string()),
        arguments: vec![expr],
    }
}

fn proc_par_expr(left: Expr, right: Expr) -> Expr {
    Expr::Call {
        func: "par".to_string(),
        module: Some("proc".to_string()),
        arguments: vec![left, right],
    }
}

fn proc_scatter_expr(items: Vec<Value>, mapper: Expr) -> Expr {
    Expr::Call {
        func: "scatter".to_string(),
        module: Some("proc".to_string()),
        arguments: vec![Expr::Literal(Value::list_from_vec(items)), mapper],
    }
}

fn proc_await_expr(handle: ProcessHandle) -> Expr {
    Expr::Call {
        func: "await".to_string(),
        module: Some("proc".to_string()),
        arguments: vec![Expr::Literal(Value::ProcessHandle(handle))],
    }
}

async fn force_proc_in_context(ctx: &Context, proc_value: Value) -> EvalResult<Value> {
    let mut proc_ctx = ctx.clone();
    proc_ctx.set("p".to_string(), proc_value);
    eval_expr_async(
        &Expr::Call {
            func: "p".to_string(),
            module: None,
            arguments: vec![Expr::Literal(Value::Null)],
        },
        &proc_ctx,
    )
    .await
}

fn expect_handle_list(value: Value, expected_len: usize) -> Vec<ProcessHandle> {
    let items = value
        .list_to_vec()
        .unwrap_or_else(|| panic!("expected ordered handle list, got {value:?}"));
    assert_eq!(items.len(), expected_len, "expected {expected_len} handles");
    items
        .iter()
        .map(|value| match value {
            Value::ProcessHandle(handle) => handle.clone(),
            other => panic!("expected process handle, got {other:?}"),
        })
        .collect()
}

#[tokio::test]
async fn proc_par_returns_ordered_child_handles_and_defers_child_failure_to_later_await() {
    let runtime_state = RuntimeState::new();
    let parent_process_id = ProcessId::new();
    runtime_state
        .register_root_process(parent_process_id)
        .await
        .expect("parent process registers");

    let failed_dependency = ProcessId::new();
    runtime_state
        .register_root_process(failed_dependency)
        .await
        .expect("dependency process registers");
    runtime_state
        .record_process_terminal(
            failed_dependency,
            ash_core::runtime::ProcessTerminalState::Failed {
                process_id: failed_dependency,
                failure: Box::new(ash_core::runtime::OperationalFailure::new(
                    ash_core::runtime::FailureBoundary::Process,
                    ash_core::runtime::FailureEntity::Process(failed_dependency),
                    Value::String("boom".to_string()),
                    "String",
                )),
            },
        )
        .await
        .expect("dependency terminal state records");

    let proc_ctx = Context::new()
        .with_runtime_state(runtime_state.clone())
        .project_process_child(
            crate::process_env::ProcessEnvIdentity::new(parent_process_id, None, 0),
            None,
        );

    let proc_value = eval_expr(
        &proc_par_expr(
            proc_await_expr(ProcessHandle::new(
                failed_dependency,
                Some("Int".to_string()),
            )),
            proc_unit_expr(Expr::Literal(Value::Int(7))),
        ),
        &proc_ctx,
    )
    .expect("proc::par should build a Proc closure");

    let handles = expect_handle_list(
        force_proc_in_context(&proc_ctx, proc_value)
            .await
            .expect("proc::par should return child handles before child failure is observed"),
        2,
    );

    let children = runtime_state.process_children(parent_process_id).await;
    assert_eq!(
        children.len(),
        2,
        "proc::par should register two child processes"
    );
    assert_eq!(handles[0].process_id, children[0]);
    assert_eq!(handles[1].process_id, children[1]);

    for _ in 0..1024 {
        if runtime_state
            .process_terminal_state(handles[0].process_id)
            .await
            .is_some()
        {
            break;
        }
        tokio::task::yield_now().await;
    }

    let await_proc = eval_expr(&proc_await_expr(handles[0].clone()), &Context::new())
        .expect("await closure builds");
    let err = force_proc_in_context(&proc_ctx, await_proc)
        .await
        .expect_err("child failure should be observed only through later await");
    assert!(matches!(err, EvalError::OperationalFailure(_)));
}

#[tokio::test]
async fn proc_await_waits_for_running_child_completion() {
    let runtime_state = RuntimeState::new();
    let process_id = ProcessId::new();
    runtime_state
        .register_root_process(process_id)
        .await
        .expect("process registers");
    runtime_state
        .mark_process_running(process_id)
        .await
        .expect("process starts running");

    let proc_ctx = Context::new().with_runtime_state(runtime_state.clone());
    let await_proc = eval_expr(
        &proc_await_expr(ProcessHandle::new(process_id, Some("Int".to_string()))),
        &Context::new(),
    )
    .expect("await closure builds");

    let completing_runtime_state = runtime_state.clone();
    tokio::spawn(async move {
        tokio::time::sleep(std::time::Duration::from_millis(20)).await;
        completing_runtime_state
            .record_process_terminal(
                process_id,
                ash_core::runtime::ProcessTerminalState::Succeeded {
                    value: Value::Int(42),
                },
            )
            .await
            .expect("terminal state records");
    });

    assert_eq!(
        force_proc_in_context(&proc_ctx, await_proc)
            .await
            .expect("proc::await should wait for running child completion"),
        Value::Int(42)
    );
}

#[tokio::test]
async fn proc_scatter_returns_handles_in_input_order() {
    let runtime_state = RuntimeState::new();
    let parent_process_id = ProcessId::new();
    runtime_state
        .register_root_process(parent_process_id)
        .await
        .expect("parent process registers");

    let proc_ctx = Context::new()
        .with_runtime_state(runtime_state.clone())
        .project_process_child(
            crate::process_env::ProcessEnvIdentity::new(parent_process_id, None, 0),
            None,
        );

    let mapper = Expr::FnDef {
        params: vec![("x".to_string(), None)],
        return_type: None,
        body: Box::new(proc_unit_expr(Expr::Variable {
            name: "x".to_string(),
            span: ash_core::ast::Span::default(),
        })),
    };
    let proc_value = eval_expr(
        &proc_scatter_expr(vec![Value::Int(1), Value::Int(2), Value::Int(3)], mapper),
        &proc_ctx,
    )
    .expect("proc::scatter should build a Proc closure");

    let handles = expect_handle_list(
        force_proc_in_context(&proc_ctx, proc_value)
            .await
            .expect("proc::scatter should return one handle per input element"),
        3,
    );

    let children = runtime_state.process_children(parent_process_id).await;
    assert_eq!(
        children.len(),
        3,
        "proc::scatter should admit every child before returning"
    );
    assert_eq!(
        handles
            .iter()
            .map(|handle| handle.process_id)
            .collect::<Vec<_>>(),
        children,
        "proc::scatter should preserve stable input order in returned handles"
    );
}
