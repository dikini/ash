//! Surface `tests` module.

use super::*;

// =========================================================================
// Construction Tests
// =========================================================================

#[test]
fn test_program_construction() {
    let program = Program {
        definitions: vec![],
        entry: ProgramEntry {
            function: "main".into(),
            span: Span::new(0, 10, 1, 1),
        },
    };

    assert!(program.definitions.is_empty());
    assert_eq!(program.entry.function, "main".into());
}

#[test]
fn test_definition_variants() {
    let cap_def = CapabilityDef {
        visibility: Visibility::Inherited,
        name: "read_file".into(),
        effect: EffectType::Read,
        params: vec![],
        return_type: None,
        constraints: vec![],
        target_provider: None,
        target_action: None,
        span: Span::new(0, 20, 1, 1),
    };
    let _def = Definition::Capability(cap_def);

    let policy_def = PolicyDef {
        name: "RateLimit".into(),
        type_params: vec![],
        fields: vec![
            PolicyField {
                name: "requests".into(),
                ty: Type::Name("Int".into()),
                default: None,
                span: Span::new(0, 10, 1, 1),
            },
            PolicyField {
                name: "window_secs".into(),
                ty: Type::Name("Int".into()),
                default: None,
                span: Span::new(0, 10, 1, 1),
            },
        ],
        where_clause: None,
        span: Span::new(0, 15, 1, 1),
    };
    let _def = Definition::Policy(policy_def);

    let role_def = RoleDef {
        name: "admin".into(),
        capabilities: vec![
            CapabilityDecl {
                capability: "read".into(),
                constraints: None,
                span: Span::new(0, 10, 1, 1),
            },
            CapabilityDecl {
                capability: "write".into(),
                constraints: None,
                span: Span::new(0, 10, 1, 1),
            },
        ],
        obligations: vec![],
        span: Span::new(0, 10, 1, 1),
    };
    let _def = Definition::Role(role_def);
}

#[test]
fn test_capability_def_construction() {
    let cap = CapabilityDef {
        visibility: Visibility::Inherited,
        name: "write_file".into(),
        effect: EffectType::Write,
        params: vec![
            Param {
                name: "path".into(),
                ty: Type::Name("String".into()),
            },
            Param {
                name: "content".into(),
                ty: Type::Name("String".into()),
            },
        ],
        return_type: Some(Type::Name("Bool".into())),
        constraints: vec![],
        target_provider: None,
        target_action: None,
        span: Span::new(0, 50, 1, 1),
    };

    assert_eq!(cap.name, "write_file".into());
    assert_eq!(cap.effect, EffectType::Write);
    assert_eq!(cap.params.len(), 2);
    assert!(cap.return_type.is_some());
}

#[test]
fn test_builtin_fn_def_construction() {
    let builtin = BuiltinFnDef {
        visibility: Visibility::Public,
        name: "find".into(),
        type_params: vec!["T".into()],
        params: vec![
            Param {
                name: "collection".into(),
                ty: Type::Constructor {
                    name: "List".into(),
                    args: vec![Type::Name("T".into())],
                },
            },
            Param {
                name: "predicate".into(),
                ty: Type::Fn(
                    vec![Type::Name("T".into())],
                    None,
                    Box::new(Type::Name("Bool".into())),
                ),
            },
        ],
        return_type: Type::Constructor {
            name: "Option".into(),
            args: vec![Type::Name("T".into())],
        },
        proposition_tail: None,
        span: Span::new(0, 60, 1, 1),
    };

    assert_eq!(builtin.name, "find".into());
    assert_eq!(builtin.visibility, Visibility::Public);
    assert_eq!(builtin.type_params, vec!["T".into()]);
    assert_eq!(builtin.params.len(), 2);
    assert_eq!(builtin.params[0].name, "collection".into());
    assert_eq!(builtin.params[1].name, "predicate".into());
    // return_type is required, not Optional
    match &builtin.return_type {
        Type::Constructor { name, args } => {
            assert_eq!(name.as_ref(), "Option");
            assert_eq!(args.len(), 1);
        }
        other => panic!("expected Constructor type, got {other:?}"),
    }
}

#[test]
fn function_type_display_uses_target_callable_syntax() {
    let rendered = format_type(&Type::Fn(
        vec![Type::Name("Int".into()), Type::Name("String".into())],
        None,
        Box::new(Type::Name("Bool".into())),
    ));

    assert_eq!(rendered, "(Int, String) -> Bool");
    assert!(
        !rendered.contains("Fn("),
        "function type display must not emit removed callable syntax: {rendered}"
    );
}

#[test]
fn test_definition_builtin_fn_variant() {
    let builtin = BuiltinFnDef {
        visibility: Visibility::Crate,
        name: "hash".into(),
        type_params: vec![],
        params: vec![Param {
            name: "input".into(),
            ty: Type::Name("String".into()),
        }],
        return_type: Type::Name("String".into()),
        proposition_tail: None,
        span: Span::new(0, 30, 1, 1),
    };

    let def = Definition::BuiltinFn(builtin);

    // Verify pattern matching works and fields are accessible
    match &def {
        Definition::BuiltinFn(b) => {
            assert_eq!(b.name, "hash".into());
            assert_eq!(b.visibility, Visibility::Crate);
            assert!(b.type_params.is_empty());
            assert_eq!(b.params.len(), 1);
            assert_eq!(b.return_type, Type::Name("String".into()));
        }
        other => panic!("expected BuiltinFn variant, got {other:?}"),
    }

    // Verify Debug trait works via the Definition enum
    assert!(format!("{def:?}").contains("BuiltinFn"));
}

#[test]
fn test_policy_def_construction() {
    let policy = PolicyDef {
        name: "BoundedResource".into(),
        type_params: vec![],
        fields: vec![
            PolicyField {
                name: "min".into(),
                ty: Type::Name("Int".into()),
                default: None,
                span: Span::new(0, 10, 1, 1),
            },
            PolicyField {
                name: "max".into(),
                ty: Type::Name("Int".into()),
                default: None,
                span: Span::new(0, 10, 1, 1),
            },
        ],
        where_clause: Some(Expr::Binary {
            op: BinaryOp::Leq,
            raw_operator: None,
            left: Box::new(Expr::Variable {
                name: "min".into(),
                span: crate::token::Span::default(),
            }),
            right: Box::new(Expr::Variable {
                name: "max".into(),
                span: crate::token::Span::default(),
            }),
            span: Span::new(0, 10, 1, 1),
        }),
        span: Span::new(0, 30, 1, 1),
    };

    assert_eq!(policy.name, "BoundedResource".into());
    assert_eq!(policy.fields.len(), 2);
    assert!(policy.where_clause.is_some());
}

#[test]
fn test_role_def_construction() {
    let role = RoleDef {
        name: "manager".into(),
        capabilities: vec![
            CapabilityDecl {
                capability: "approve".into(),
                constraints: None,
                span: Span::new(0, 50, 1, 1),
            },
            CapabilityDecl {
                capability: "review".into(),
                constraints: None,
                span: Span::new(0, 100, 1, 1),
            },
        ],
        obligations: vec!["audit_log".into()],
        span: Span::new(0, 150, 1, 1),
    };

    assert_eq!(role.name, "manager".into());
    assert_eq!(role.capabilities.len(), 2);
    assert_eq!(role.obligations.len(), 1);
    assert_eq!(role.obligations[0].as_ref(), "audit_log");
}

#[test]
fn test_role_def_with_capability_decl() {
    let role = RoleDef {
        name: "ai_agent".into(),
        capabilities: vec![CapabilityDecl {
            capability: "file".into(),
            constraints: Some(ConstraintBlock {
                fields: vec![ConstraintField {
                    name: "paths".into(),
                    value: ConstraintValue::Array(vec![ConstraintValue::String(
                        "/tmp/*".to_string(),
                    )]),
                    span: Span::new(0, 50, 1, 1),
                }],
                span: Span::new(0, 100, 1, 1),
            }),
            span: Span::new(0, 30, 1, 1),
        }],
        obligations: vec![],
        span: Span::new(0, 200, 1, 1),
    };

    assert_eq!(role.capabilities.len(), 1);
    assert_eq!(role.capabilities[0].capability.as_ref(), "file");
    assert!(role.capabilities[0].constraints.is_some());
}

// =========================================================================
// Expression Tests
// =========================================================================

#[test]
fn test_expr_literal() {
    let expr = Expr::Literal(Literal::Int(42));
    assert!(matches!(expr, Expr::Literal(Literal::Int(42))));
}

#[test]
fn test_expr_variable() {
    let expr = Expr::Variable {
        name: "my_var".into(),
        span: crate::token::Span::default(),
    };
    assert!(matches!(expr, Expr::Variable { .. }));
    if let Expr::Variable { name, .. } = expr {
        assert_eq!(name, "my_var".into());
    }
}

#[test]
fn test_expr_field_access() {
    let expr = Expr::FieldAccess {
        base: Box::new(Expr::Variable {
            name: "obj".into(),
            span: crate::token::Span::default(),
        }),
        field: "field".into(),
        span: Span::new(0, 10, 1, 1),
    };

    match expr {
        Expr::FieldAccess { base, field, .. } => {
            assert!(matches!(base.as_ref(), Expr::Variable { .. }));
            assert_eq!(field, "field".into());
        }
        _ => panic!("Expected FieldAccess"),
    }
}

#[test]
fn test_expr_index_access() {
    let expr = Expr::IndexAccess {
        base: Box::new(Expr::Variable {
            name: "arr".into(),
            span: crate::token::Span::default(),
        }),
        index: Box::new(Expr::Literal(Literal::Int(0))),
        span: Span::new(0, 8, 1, 1),
    };

    match expr {
        Expr::IndexAccess { base, index, .. } => {
            assert!(matches!(base.as_ref(), Expr::Variable { .. }));
            assert!(matches!(index.as_ref(), Expr::Literal(Literal::Int(0))));
        }
        _ => panic!("Expected IndexAccess"),
    }
}

#[test]
fn test_expr_unary() {
    let expr = Expr::Unary {
        op: UnaryOp::Not,
        operand: Box::new(Expr::Literal(Literal::Bool(false))),
        span: Span::new(0, 5, 1, 1),
    };

    match expr {
        Expr::Unary { op, .. } => {
            assert_eq!(op, UnaryOp::Not);
        }
        _ => panic!("Expected Unary"),
    }
}

#[test]
fn test_expr_binary() {
    let expr = Expr::Binary {
        op: BinaryOp::Add,
        raw_operator: None,
        left: Box::new(Expr::Literal(Literal::Int(1))),
        right: Box::new(Expr::Literal(Literal::Int(2))),
        span: Span::new(0, 5, 1, 1),
    };

    match expr {
        Expr::Binary {
            op, left, right, ..
        } => {
            assert_eq!(op, BinaryOp::Add);
            assert!(matches!(left.as_ref(), Expr::Literal(Literal::Int(1))));
            assert!(matches!(right.as_ref(), Expr::Literal(Literal::Int(2))));
        }
        _ => panic!("Expected Binary"),
    }
}

#[test]
fn test_expr_call() {
    let expr = Expr::Call {
        func: "foo".into(),
        module: None,
        args: vec![
            Expr::Literal(Literal::Int(1)),
            Expr::Literal(Literal::Int(2)),
        ],
        span: Span::new(0, 10, 1, 1),
    };

    match expr {
        Expr::Call { func, args, .. } => {
            assert_eq!(func, "foo".into());
            assert_eq!(args.len(), 2);
        }
        _ => panic!("Expected Call"),
    }
}

// =========================================================================
// PolicyExpr Tests
// =========================================================================

#[test]
fn test_policy_expr_var() {
    let expr = PolicyExpr::Var {
        name: "my_policy".into(),
        span: crate::token::Span::default(),
    };
    assert!(matches!(expr, PolicyExpr::Var { name, .. } if name.as_ref() == "my_policy"));
}

#[test]
fn test_policy_expr_and() {
    let expr = PolicyExpr::And(vec![
        PolicyExpr::Var {
            name: "p1".into(),
            span: crate::token::Span::default(),
        },
        PolicyExpr::Var {
            name: "p2".into(),
            span: crate::token::Span::default(),
        },
    ]);
    match expr {
        PolicyExpr::And(exprs) => assert_eq!(exprs.len(), 2),
        _ => panic!("Expected And"),
    }
}

#[test]
fn test_policy_expr_or() {
    let expr = PolicyExpr::Or(vec![
        PolicyExpr::Var {
            name: "p1".into(),
            span: crate::token::Span::default(),
        },
        PolicyExpr::Var {
            name: "p2".into(),
            span: crate::token::Span::default(),
        },
    ]);
    match expr {
        PolicyExpr::Or(exprs) => assert_eq!(exprs.len(), 2),
        _ => panic!("Expected Or"),
    }
}

#[test]
fn test_policy_expr_not() {
    let expr = PolicyExpr::Not(Box::new(PolicyExpr::Var {
        name: "p".into(),
        span: crate::token::Span::default(),
    }));
    match expr {
        PolicyExpr::Not(inner) => {
            assert!(matches!(inner.as_ref(), PolicyExpr::Var { .. }));
        }
        _ => panic!("Expected Not"),
    }
}

#[test]
fn test_policy_expr_implies() {
    let expr = PolicyExpr::Implies(
        Box::new(PolicyExpr::Var {
            name: "a".into(),
            span: crate::token::Span::default(),
        }),
        Box::new(PolicyExpr::Var {
            name: "b".into(),
            span: crate::token::Span::default(),
        }),
    );
    match expr {
        PolicyExpr::Implies(left, right) => {
            assert!(matches!(left.as_ref(), PolicyExpr::Var { .. }));
            assert!(matches!(right.as_ref(), PolicyExpr::Var { .. }));
        }
        _ => panic!("Expected Implies"),
    }
}

#[test]
fn test_policy_expr_sequential() {
    let expr = PolicyExpr::Sequential(vec![
        PolicyExpr::Var {
            name: "p1".into(),
            span: crate::token::Span::default(),
        },
        PolicyExpr::Var {
            name: "p2".into(),
            span: crate::token::Span::default(),
        },
        PolicyExpr::Var {
            name: "p3".into(),
            span: crate::token::Span::default(),
        },
    ]);
    match expr {
        PolicyExpr::Sequential(exprs) => assert_eq!(exprs.len(), 3),
        _ => panic!("Expected Sequential"),
    }
}

#[test]
fn test_policy_expr_concurrent() {
    let expr = PolicyExpr::Concurrent(vec![
        PolicyExpr::Var {
            name: "p1".into(),
            span: crate::token::Span::default(),
        },
        PolicyExpr::Var {
            name: "p2".into(),
            span: crate::token::Span::default(),
        },
    ]);
    match expr {
        PolicyExpr::Concurrent(exprs) => assert_eq!(exprs.len(), 2),
        _ => panic!("Expected Concurrent"),
    }
}

#[test]
fn test_policy_expr_forall() {
    let expr = PolicyExpr::ForAll {
        var: "x".into(),
        items: Box::new(Expr::Variable {
            name: "items".into(),
            span: crate::token::Span::default(),
        }),
        body: Box::new(PolicyExpr::Var {
            name: "policy".into(),
            span: crate::token::Span::default(),
        }),
        span: Span::new(0, 20, 1, 1),
    };
    match expr {
        PolicyExpr::ForAll { var, body, .. } => {
            assert_eq!(var.as_ref(), "x");
            assert!(matches!(body.as_ref(), PolicyExpr::Var { .. }));
        }
        _ => panic!("Expected ForAll"),
    }
}

#[test]
fn test_policy_expr_exists() {
    let expr = PolicyExpr::Exists {
        var: "x".into(),
        items: Box::new(Expr::Variable {
            name: "items".into(),
            span: crate::token::Span::default(),
        }),
        body: Box::new(PolicyExpr::Var {
            name: "policy".into(),
            span: crate::token::Span::default(),
        }),
        span: Span::new(0, 20, 1, 1),
    };
    match expr {
        PolicyExpr::Exists { var, body, .. } => {
            assert_eq!(var.as_ref(), "x");
            assert!(matches!(body.as_ref(), PolicyExpr::Var { .. }));
        }
        _ => panic!("Expected Exists"),
    }
}

#[test]
fn test_policy_expr_method_call() {
    let expr = PolicyExpr::MethodCall {
        receiver: Box::new(PolicyExpr::Var {
            name: "base".into(),
            span: crate::token::Span::default(),
        }),
        method: "and".into(),
        args: vec![Expr::Variable {
            name: "other".into(),
            span: crate::token::Span::default(),
        }],
        span: Span::new(0, 15, 1, 1),
    };
    match expr {
        PolicyExpr::MethodCall {
            receiver,
            method,
            args,
            ..
        } => {
            assert!(matches!(receiver.as_ref(), PolicyExpr::Var { .. }));
            assert_eq!(method.as_ref(), "and");
            assert_eq!(args.len(), 1);
        }
        _ => panic!("Expected MethodCall"),
    }
}

#[test]
fn test_policy_expr_call() {
    let expr = PolicyExpr::Call {
        func: "rate_limit".into(),
        args: vec![Expr::Literal(Literal::Int(100))],
        span: Span::new(0, 15, 1, 1),
    };
    match expr {
        PolicyExpr::Call { func, args, .. } => {
            assert_eq!(func.as_ref(), "rate_limit");
            assert_eq!(args.len(), 1);
        }
        _ => panic!("Expected Call"),
    }
}

#[test]
fn test_policy_expr_span() {
    let span = Span::new(10, 20, 1, 5);
    let expr = PolicyExpr::Call {
        func: "test".into(),
        args: vec![],
        span,
    };
    assert_eq!(expr.span(), span);
}

// =========================================================================
// Operator Tests
// =========================================================================

#[test]
fn test_unary_ops() {
    assert_eq!(UnaryOp::Not, UnaryOp::Not);
    assert_eq!(UnaryOp::Neg, UnaryOp::Neg);
    assert_ne!(UnaryOp::Not, UnaryOp::Neg);
}

#[test]
fn test_binary_ops() {
    let ops = vec![
        BinaryOp::Add,
        BinaryOp::Sub,
        BinaryOp::Mul,
        BinaryOp::Div,
        BinaryOp::Mod,
        BinaryOp::And,
        BinaryOp::Or,
        BinaryOp::Eq,
        BinaryOp::Neq,
        BinaryOp::Lt,
        BinaryOp::Gt,
        BinaryOp::Leq,
        BinaryOp::Geq,
        BinaryOp::In,
    ];

    // Ensure all ops are distinct
    for (i, op1) in ops.iter().enumerate() {
        for (j, op2) in ops.iter().enumerate() {
            if i != j {
                assert_ne!(op1, op2);
            }
        }
    }
}

// =========================================================================
// Pattern Tests
// =========================================================================

#[test]
fn test_pattern_variable() {
    let pat = Pattern::Variable {
        name: "x".into(),
        span: crate::token::Span::default(),
    };
    assert!(matches!(pat, Pattern::Variable { .. }));
    if let Pattern::Variable { name, .. } = pat {
        assert_eq!(name, "x".into());
    }
}

#[test]
fn test_pattern_wildcard() {
    let pat = Pattern::Wildcard;
    assert!(matches!(pat, Pattern::Wildcard));
}

#[test]
fn test_pattern_tuple() {
    let pat = Pattern::Tuple(vec![
        Pattern::Variable {
            name: "a".into(),
            span: crate::token::Span::default(),
        },
        Pattern::Variable {
            name: "b".into(),
            span: crate::token::Span::default(),
        },
    ]);

    match pat {
        Pattern::Tuple(patterns) => {
            assert_eq!(patterns.len(), 2);
        }
        _ => panic!("Expected Tuple pattern"),
    }
}

#[test]
fn test_pattern_record() {
    let pat = Pattern::Record(vec![
        (
            "x".into(),
            Pattern::Variable {
                name: "a".into(),
                span: crate::token::Span::default(),
            },
        ),
        (
            "y".into(),
            Pattern::Variable {
                name: "b".into(),
                span: crate::token::Span::default(),
            },
        ),
    ]);

    match pat {
        Pattern::Record(fields) => {
            assert_eq!(fields.len(), 2);
            assert_eq!(fields[0].0, "x".into());
            assert_eq!(fields[1].0, "y".into());
        }
        _ => panic!("Expected Record pattern"),
    }
}

#[test]
fn test_pattern_list() {
    let pat = Pattern::List {
        elements: vec![Pattern::Variable {
            name: "head".into(),
            span: crate::token::Span::default(),
        }],
        rest: Some("tail".into()),
    };

    match pat {
        Pattern::List { elements, rest } => {
            assert_eq!(elements.len(), 1);
            assert_eq!(rest, Some("tail".into()));
        }
        _ => panic!("Expected List pattern"),
    }
}

#[test]
fn test_pattern_literal() {
    let pat = Pattern::Literal(Literal::Int(42));
    assert!(matches!(pat, Pattern::Literal(Literal::Int(42))));
}

// =========================================================================
// Literal Tests
// =========================================================================

#[test]
fn test_literal_variants() {
    let int_lit = Literal::Int(42);
    let float_lit = Literal::Float(ordered_float::OrderedFloat(1.5));
    let string_lit = Literal::String("hello".into());
    let bool_lit = Literal::Bool(true);
    let null_lit = Literal::Null;

    assert_eq!(int_lit, Literal::Int(42));
    assert_eq!(float_lit, Literal::Float(ordered_float::OrderedFloat(1.5)));
    assert_eq!(string_lit, Literal::String("hello".into()));
    assert_eq!(bool_lit, Literal::Bool(true));
    assert_eq!(null_lit, Literal::Null);
}

// =========================================================================
// Effect Type Tests
// =========================================================================

#[test]
fn test_effect_types() {
    let effects = [
        EffectType::Observe,
        EffectType::Read,
        EffectType::Analyze,
        EffectType::Decide,
        EffectType::Act,
        EffectType::Write,
        EffectType::External,
    ];

    // Ensure all effect types are distinct
    for (i, e1) in effects.iter().enumerate() {
        for (j, e2) in effects.iter().enumerate() {
            if i != j {
                assert_ne!(e1, e2);
            }
        }
    }
}

// =========================================================================
// Decision Tests
// =========================================================================

#[test]
fn test_decision_variants() {
    let permit = Decision::Permit;
    let deny = Decision::Deny;
    let require = Decision::RequireApproval {
        role: "admin".into(),
    };
    let escalate = Decision::Escalate;

    assert!(matches!(permit, Decision::Permit));
    assert!(matches!(deny, Decision::Deny));
    assert!(matches!(require, Decision::RequireApproval { .. }));
    assert!(matches!(escalate, Decision::Escalate));
}

// =========================================================================
// Param Tests
// =========================================================================

#[test]
fn test_param() {
    let param = Param {
        name: "x".into(),
        ty: Type::Name("Int".into()),
    };

    assert_eq!(param.name, "x".into());
    assert!(matches!(param.ty, Type::Name(_)));
}

// =========================================================================
// Type Tests
// =========================================================================

#[test]
fn test_type_variants() {
    let name_ty = Type::Name("Int".into());
    let list_ty = Type::List(Box::new(Type::Name("Int".into())));
    let record_ty = Type::Record(vec![
        ("x".into(), Type::Name("Int".into())),
        ("y".into(), Type::Name("String".into())),
    ]);
    let cap_ty = Type::Capability("Read".into());

    assert!(matches!(name_ty, Type::Name(_)));
    assert!(matches!(list_ty, Type::List(_)));
    assert!(matches!(record_ty, Type::Record(_)));
    assert!(matches!(cap_ty, Type::Capability(_)));
}

// =========================================================================
// Predicate Tests
// =========================================================================

#[test]
fn test_predicate() {
    let pred = Predicate {
        name: "is_admin".into(),
        args: vec![Expr::Variable {
            name: "user".into(),
            span: crate::token::Span::default(),
        }],
    };

    assert_eq!(pred.name, "is_admin".into());
    assert_eq!(pred.args.len(), 1);
}

// =========================================================================
// Constraint Tests
// =========================================================================

#[test]
fn test_constraint() {
    let constraint = Constraint {
        predicate: Predicate {
            name: "is_positive".into(),
            args: vec![],
        },
    };

    assert_eq!(constraint.predicate.name, "is_positive".into());
}

// =========================================================================
// Spanned Trait Tests
// =========================================================================

#[test]
fn test_expr_spanned() {
    let span = Span::new(5, 15, 1, 3);
    let expr = Expr::FieldAccess {
        base: Box::new(Expr::Variable {
            name: "obj".into(),
            span: crate::token::Span::default(),
        }),
        field: "field".into(),
        span,
    };

    assert_eq!(expr.span(), span);

    // Literals and variables return default span
    let lit = Expr::Literal(Literal::Int(42));
    assert_eq!(lit.span(), Span::default());

    let var = Expr::Variable {
        name: "x".into(),
        span: crate::token::Span::default(),
    };
    assert_eq!(var.span(), Span::default());
}
