//! Lowering tests.

use super::*;
use crate::surface::{
    BinaryOp, Contract as SurfaceContract, DoStmt, DoTarget, EffectType, EnsuresClause,
    Expr as SurfaceExpr, Literal as SurfaceLiteral, Pattern, Requirement as SurfaceRequirement,
};
use crate::token::Span;

fn dummy_span() -> Span {
    Span::new(0, 0, 1, 1)
}

fn int_expr(value: i64) -> SurfaceExpr {
    SurfaceExpr::Literal(SurfaceLiteral::Int(value))
}

fn var_expr(name: &str) -> SurfaceExpr {
    SurfaceExpr::Variable {
        name: name.into(),
        span: crate::token::Span::default(),
    }
}

#[test]
fn test_lower_do_block_act_return_rejects_parser_only_lowering() {
    let surface = SurfaceExpr::DoBlock {
        target: DoTarget {
            name: "Act".into(),
            args: vec![],
            span: Span::default(),
        },
        stmts: vec![DoStmt::Return {
            value: Box::new(int_expr(1)),
            span: Span::default(),
        }],
        span: Span::default(),
    };

    let err = lower_expr(&surface).expect_err("generic do block must require typed elaboration");
    assert!(matches!(
        err,
        LoweringError::ExprNotLowerable { kind }
            if kind.contains("typed do elaboration")
    ));
}

#[test]
fn test_lower_do_block_proc_bind_rejects_parser_only_lowering() {
    let surface = SurfaceExpr::DoBlock {
        target: DoTarget {
            name: "Proc".into(),
            args: vec![],
            span: Span::default(),
        },
        stmts: vec![
            DoStmt::Bind {
                name: "x".into(),
                value: Box::new(SurfaceExpr::Call {
                    func: "unit".into(),
                    module: Some("proc".into()),
                    args: vec![int_expr(1)],
                    span: Span::default(),
                }),
                span: Span::default(),
            },
            DoStmt::Return {
                value: Box::new(var_expr("x")),
                span: Span::default(),
            },
        ],
        span: Span::default(),
    };

    let err = lower_expr(&surface).expect_err("generic do block must require typed elaboration");
    assert!(matches!(
        err,
        LoweringError::ExprNotLowerable { kind }
            if kind.contains("typed do elaboration")
    ));
}

#[test]
fn test_lower_expr_literal() {
    let surface = SurfaceExpr::Literal(SurfaceLiteral::Int(42));
    let core = lower_expr(&surface).unwrap();
    assert!(matches!(core, CoreExpr::Literal(ash_core::Value::Int(42))));
}

#[test]
fn test_lower_expr_variable() {
    let surface = SurfaceExpr::Variable {
        name: "my_var".into(),
        span: crate::token::Span::default(),
    };
    let core = lower_expr(&surface).unwrap();
    assert!(matches!(core, CoreExpr::Variable { name, .. } if name == "my_var"));
}

#[test]
fn test_lower_expr_binary() {
    let surface = SurfaceExpr::Binary {
        op: BinaryOp::Add,
        raw_operator: None,
        left: Box::new(SurfaceExpr::Literal(SurfaceLiteral::Int(1))),
        right: Box::new(SurfaceExpr::Literal(SurfaceLiteral::Int(2))),
        span: dummy_span(),
    };
    let core = lower_expr(&surface).unwrap();
    assert!(matches!(
        core,
        CoreExpr::Binary {
            op: ash_core::BinaryOp::Add,
            ..
        }
    ));
}

#[test]
#[allow(clippy::approx_constant)]
fn test_lower_expr_float_literal_error() {
    let surface = SurfaceExpr::Literal(SurfaceLiteral::Float(ordered_float::OrderedFloat(3.14)));
    let result = lower_expr(&surface);
    assert!(matches!(result, Err(LoweringError::FloatNotSupported)));
}

#[test]
fn test_interface_method_call_lowers_as_call() {
    // After TASK-561, interface method calls use Expr::Call with module qualifier
    let surface = SurfaceExpr::Call {
        func: "explain".into(),
        module: Some("Explain".into()),
        args: vec![SurfaceExpr::Variable {
            name: "value".into(),
            span: crate::token::Span::default(),
        }],
        span: crate::token::Span::new(0, 22, 1, 1),
    };

    let result = lower_expr(&surface);
    assert!(result.is_ok());
    let core = result.unwrap();
    match &core {
        CoreExpr::Call {
            func,
            module,
            arguments,
        } => {
            assert_eq!(func, "explain");
            assert_eq!(module.as_deref(), Some("Explain"));
            assert_eq!(arguments.len(), 1);
        }
        other => panic!("expected CoreExpr::Call, got {other:?}"),
    }
}

#[test]
fn test_lower_pattern_variable() {
    let surface = Pattern::Variable {
        name: "x".into(),
        span: crate::token::Span::default(),
    };
    let core = lower_pattern(&surface).unwrap();
    assert!(matches!(core, CorePattern::Variable { name, .. } if name == "x"));
}

#[test]
fn test_lower_pattern_wildcard() {
    let surface = Pattern::Wildcard;
    let core = lower_pattern(&surface).unwrap();
    assert!(matches!(core, CorePattern::Wildcard));
}

#[test]
fn test_lower_pattern_tuple() {
    let surface = Pattern::Tuple(vec![
        Pattern::Variable {
            name: "a".into(),
            span: crate::token::Span::default(),
        },
        Pattern::Variable {
            name: "b".into(),
            span: crate::token::Span::default(),
        },
    ]);
    let core = lower_pattern(&surface).unwrap();
    assert!(matches!(core, CorePattern::Tuple(pats) if pats.len() == 2));
}

#[test]
fn test_lower_literal_int() {
    let surface = SurfaceLiteral::Int(42);
    let core = lower_literal(&surface).unwrap();
    assert!(matches!(core, ash_core::Value::Int(42)));
}

#[test]
fn test_lower_literal_string() {
    let surface = SurfaceLiteral::String("hello".into());
    let core = lower_literal(&surface).unwrap();
    assert!(matches!(core, ash_core::Value::String(s) if s == "hello"));
}

#[test]
fn test_lower_unary_op() {
    assert!(matches!(
        lower_unary_op(UnaryOp::Not),
        ash_core::UnaryOp::Not
    ));
    assert!(matches!(
        lower_unary_op(UnaryOp::Neg),
        ash_core::UnaryOp::Neg
    ));
}

#[test]
fn test_lower_binary_op() {
    assert!(matches!(
        lower_binary_op(BinaryOp::Add).unwrap(),
        ash_core::BinaryOp::Add
    ));
    assert!(matches!(
        lower_binary_op(BinaryOp::Sub).unwrap(),
        ash_core::BinaryOp::Sub
    ));
    assert!(matches!(
        lower_binary_op(BinaryOp::Mul).unwrap(),
        ash_core::BinaryOp::Mul
    ));
    assert!(matches!(
        lower_binary_op(BinaryOp::Div).unwrap(),
        ash_core::BinaryOp::Div
    ));
    assert!(matches!(
        lower_binary_op(BinaryOp::Mod).unwrap(),
        ash_core::BinaryOp::Mod
    ));
    assert!(matches!(
        lower_binary_op(BinaryOp::Eq).unwrap(),
        ash_core::BinaryOp::Eq
    ));
    assert!(matches!(
        lower_binary_op(BinaryOp::And).unwrap(),
        ash_core::BinaryOp::And
    ));
    assert!(matches!(
        lower_binary_op(BinaryOp::Or).unwrap(),
        ash_core::BinaryOp::Or
    ));
}

#[test]
fn test_lower_fn_contract_stage1_predicates() {
    let contract = SurfaceContract {
        requires: vec![
            SurfaceRequirement::Arithmetic {
                expr: SurfaceExpr::Binary {
                    op: BinaryOp::Geq,
                    raw_operator: None,
                    left: Box::new(var_expr("n")),
                    right: Box::new(int_expr(0)),
                    span: dummy_span(),
                },
            },
            SurfaceRequirement::Arithmetic {
                expr: SurfaceExpr::Binary {
                    op: BinaryOp::Neq,
                    raw_operator: None,
                    left: Box::new(var_expr("d")),
                    right: Box::new(int_expr(0)),
                    span: dummy_span(),
                },
            },
            SurfaceRequirement::Arithmetic {
                expr: SurfaceExpr::Binary {
                    op: BinaryOp::Eq,
                    raw_operator: None,
                    left: Box::new(SurfaceExpr::Binary {
                        op: BinaryOp::Mod,
                        raw_operator: None,
                        left: Box::new(var_expr("n")),
                        right: Box::new(int_expr(2)),
                        span: dummy_span(),
                    }),
                    right: Box::new(int_expr(1)),
                    span: dummy_span(),
                },
            },
        ],
        ensures: vec![EnsuresClause {
            expr: SurfaceExpr::Binary {
                op: BinaryOp::Geq,
                raw_operator: None,
                left: Box::new(var_expr("result")),
                right: Box::new(int_expr(0)),
                span: dummy_span(),
            },
            span: dummy_span(),
        }],
    };

    let ctx = FnContractLoweringContext {
        name: "safe_div",
        params: &[
            (
                "n".to_string(),
                ash_core::core_ash::CoreType::Base("Int".to_string()),
            ),
            (
                "d".to_string(),
                ash_core::core_ash::CoreType::Base("Int".to_string()),
            ),
        ],
        param_name_spans: &[],
        result: Some(ash_core::core_ash::CoreType::Base("Int".to_string())),
        callable_span: None,
    };

    let lowered = lower_fn_contract(Some(&contract), &ctx).expect("fn contract should lower");
    assert_eq!(lowered.contract.requires.len(), 3);
    assert_eq!(lowered.runtime_postconditions.predicates.len(), 1);
    assert!(matches!(
        &lowered.contract.requires[0],
        ash_core::contract::Requirement::Arithmetic { var, constraint }
            if var == "n"
                && matches!(constraint, ash_core::contract::ArithConstraint::Gte(0))
    ));
    assert!(matches!(
        &lowered.contract.requires[1],
        ash_core::contract::Requirement::Arithmetic { var, constraint }
            if var == "d"
                && matches!(constraint, ash_core::contract::ArithConstraint::NotEq(0))
    ));
    assert!(matches!(
        &lowered.contract.requires[2],
        ash_core::contract::Requirement::Arithmetic { var, constraint }
            if var == "n"
                && matches!(
                    constraint,
                    ash_core::contract::ArithConstraint::Modulo { div: 2, rem: 1 }
                )
    ));
    assert!(matches!(
        &lowered.runtime_postconditions.predicates[0],
        ash_core::contract::PostPredicate::ResultSatisfies(
            ash_core::contract::ArithConstraint::Gte(0)
        )
    ));
}

#[test]
fn test_lower_fn_contract_rejects_non_value_ensures() {
    let contract = SurfaceContract {
        requires: vec![],
        ensures: vec![EnsuresClause {
            expr: SurfaceExpr::Binary {
                op: BinaryOp::Geq,
                raw_operator: None,
                left: Box::new(var_expr("state")),
                right: Box::new(int_expr(0)),
                span: dummy_span(),
            },
            span: dummy_span(),
        }],
    };

    let ctx = FnContractLoweringContext {
        name: "_test",
        params: &[(
            "state".to_string(),
            ash_core::core_ash::CoreType::Base("Int".to_string()),
        )],
        param_name_spans: &[],
        result: Some(ash_core::core_ash::CoreType::Base("Int".to_string())),
        callable_span: None,
    };

    let error = lower_fn_contract(Some(&contract), &ctx).expect_err("invalid ensures should fail");
    assert!(matches!(
        error,
        FnContractLoweringError::InvalidEnsures { .. }
    ));
}

#[test]
fn test_lower_effect_type() {
    assert!(matches!(
        lower_effect_type(EffectType::Observe),
        Effect::Epistemic
    ));
    assert!(matches!(
        lower_effect_type(EffectType::Read),
        Effect::Epistemic
    ));
    assert!(matches!(
        lower_effect_type(EffectType::Analyze),
        Effect::Deliberative
    ));
    assert!(matches!(
        lower_effect_type(EffectType::Decide),
        Effect::Evaluative
    ));
    assert!(matches!(
        lower_effect_type(EffectType::Act),
        Effect::Operational
    ));
    assert!(matches!(
        lower_effect_type(EffectType::Write),
        Effect::Operational
    ));
    assert!(matches!(
        lower_effect_type(EffectType::External),
        Effect::Operational
    ));
}

// =========================================================================
// Module-Owned Capability Resolution Tests (TASK-475)
// =========================================================================

// --- BuiltinFnDef lowering tests ---

#[test]
fn test_lower_builtin_fn_simple() {
    use crate::surface::{BuiltinFnDef, Param, Type, Visibility};

    let def = BuiltinFnDef {
        visibility: Visibility::Inherited,
        name: "foo".into(),
        type_params: vec![],
        params: vec![Param {
            name: "x".into(),
            name_span: dummy_span(),
            ty: Type::Name("Int".into()),
        }],
        return_type: Type::Name("Int".into()),
        proposition_tail: None,
        span: dummy_span(),
    };

    let core = lower_builtin_fn_def(&def).expect("builtin fn should lower");

    assert_eq!(core.name, "foo");
    assert!(core.type_params.is_empty());
    assert_eq!(core.params.len(), 1);
    assert_eq!(core.params[0].0, "x");
    assert_eq!(
        core.params[0].1,
        ash_core::ast::TypeExpr::Named("Int".to_string())
    );
    assert_eq!(
        core.return_type,
        ash_core::ast::TypeExpr::Named("Int".to_string())
    );
    assert_eq!(core.visibility, ash_core::ast::Visibility::Private);
}

#[test]
fn test_lower_builtin_fn_with_type_params() {
    use crate::surface::{BuiltinFnDef, Param, Type, Visibility};

    let def = BuiltinFnDef {
        visibility: Visibility::Public,
        name: "id".into(),
        type_params: vec!["T".into()],
        params: vec![Param {
            name: "value".into(),
            name_span: dummy_span(),
            ty: Type::Name("T".into()),
        }],
        return_type: Type::Name("T".into()),
        proposition_tail: None,
        span: dummy_span(),
    };

    let core = lower_builtin_fn_def(&def).expect("builtin fn should lower");

    assert_eq!(core.name, "id");
    assert_eq!(core.type_params, vec!["T".to_string()]);
    assert_eq!(core.params.len(), 1);
    assert_eq!(core.params[0].0, "value");
    assert_eq!(core.visibility, ash_core::ast::Visibility::Public);
}

#[test]
fn test_lower_builtin_fn_rejects_kinded_type_params() {
    use crate::surface::{BuiltinFnDef, KindAnnotation, Param, Type, TypeParam, Visibility};

    let def = BuiltinFnDef {
        visibility: Visibility::Public,
        name: "pure".into(),
        type_params: vec![TypeParam {
            name: "M".into(),
            kind: Some(KindAnnotation {
                kind: ash_core::Kind::arrow(ash_core::Kind::Type, ash_core::Kind::Type),
                span: dummy_span(),
            }),
            bounds: Vec::new(),
            span: dummy_span(),
        }],
        params: vec![Param {
            name: "value".into(),
            name_span: dummy_span(),
            ty: Type::Name("Int".into()),
        }],
        return_type: Type::Constructor {
            name: "M".into(),
            args: vec![Type::Name("Int".into())],
        },
        proposition_tail: None,
        span: dummy_span(),
    };

    let err = lower_builtin_fn_def(&def).expect_err("kinded builtin fn should not lower yet");

    assert_eq!(
        err,
        LoweringError::UnsupportedFeature(
            "kinded builtin function type parameters are parsed by TASK-906 but lowered by TASK-907"
                .to_string()
        )
    );
}

#[test]
fn test_lower_builtin_fn_multi_param() {
    use crate::surface::{BuiltinFnDef, Param, Type, Visibility};

    let def = BuiltinFnDef {
        visibility: Visibility::Crate,
        name: "add".into(),
        type_params: vec![],
        params: vec![
            Param {
                name: "a".into(),
                name_span: dummy_span(),
                ty: Type::Name("Int".into()),
            },
            Param {
                name: "b".into(),
                name_span: dummy_span(),
                ty: Type::Name("Int".into()),
            },
        ],
        return_type: Type::Name("Int".into()),
        proposition_tail: None,
        span: dummy_span(),
    };

    let core = lower_builtin_fn_def(&def).expect("builtin fn should lower");

    assert_eq!(core.name, "add");
    assert_eq!(core.params.len(), 2);
    assert_eq!(core.params[0].0, "a");
    assert_eq!(core.params[1].0, "b");
    assert_eq!(core.visibility, ash_core::ast::Visibility::Crate);
}

#[test]
fn test_lower_builtin_fn_complex_return_type() {
    use crate::surface::{BuiltinFnDef, Param, Type, Visibility};

    let def = BuiltinFnDef {
        visibility: Visibility::Inherited,
        name: "make_list".into(),
        type_params: vec!["T".into()],
        params: vec![Param {
            name: "x".into(),
            name_span: dummy_span(),
            ty: Type::Name("T".into()),
        }],
        return_type: Type::Constructor {
            name: "List".into(),
            args: vec![Type::Name("T".into())],
        },
        proposition_tail: None,
        span: dummy_span(),
    };

    let core = lower_builtin_fn_def(&def).expect("builtin fn should lower");

    assert_eq!(core.name, "make_list");
    assert_eq!(
        core.return_type,
        ash_core::ast::TypeExpr::Constructor {
            name: "List".to_string(),
            args: vec![ash_core::ast::TypeExpr::Named("T".to_string())],
        }
    );
}

#[test]
fn test_lower_builtin_fn_no_params() {
    // Zero-parameter builtin fn (e.g., builtin fn get_time() -> Int;)
    use crate::surface::{BuiltinFnDef, Type, Visibility};

    let def = BuiltinFnDef {
        visibility: Visibility::Public,
        name: "get_time".into(),
        type_params: vec![],
        params: vec![],
        return_type: Type::Name("Int".into()),
        proposition_tail: None,
        span: dummy_span(),
    };

    let core = lower_builtin_fn_def(&def).expect("builtin fn should lower");

    assert_eq!(core.name, "get_time");
    assert!(core.params.is_empty());
    assert_eq!(core.visibility, ash_core::ast::Visibility::Public);
}

#[test]
fn test_lower_builtin_fn_parse_and_lower_roundtrip() {
    // Parse a builtin fn from source and lower it
    let source = "builtin fn foo(x: Int) -> Int;";
    let parsed = crate::parse_surface_file(source).expect("parse should succeed");

    // Find the BuiltinFn definition
    let builtin_def = parsed
        .definitions
        .iter()
        .find_map(|d| match d {
            crate::surface::Definition::BuiltinFn(b) => Some(b.clone()),
            _ => None,
        })
        .expect("should find a BuiltinFn definition");

    assert_eq!(builtin_def.name.as_ref(), "foo");

    let core = lower_builtin_fn_def(&builtin_def).expect("builtin fn should lower");

    assert_eq!(core.name, "foo");
    assert!(core.type_params.is_empty());
    assert_eq!(core.params.len(), 1);
    assert_eq!(core.params[0].0, "x");
    assert_eq!(
        core.params[0].1,
        ash_core::ast::TypeExpr::Named("Int".to_string())
    );
    assert_eq!(
        core.return_type,
        ash_core::ast::TypeExpr::Named("Int".to_string())
    );
}

#[test]
fn test_lower_builtin_fn_parse_generic_roundtrip() {
    // Parse a generic builtin fn from source and lower it
    let source = "pub builtin fn map<T>(f: T, x: Int) -> T;";
    let parsed = crate::parse_surface_file(source).expect("parse should succeed");

    let builtin_def = parsed
        .definitions
        .iter()
        .find_map(|d| match d {
            crate::surface::Definition::BuiltinFn(b) => Some(b.clone()),
            _ => None,
        })
        .expect("should find a BuiltinFn definition");

    assert_eq!(builtin_def.name.as_ref(), "map");

    let core = lower_builtin_fn_def(&builtin_def).expect("builtin fn should lower");

    assert_eq!(core.name, "map");
    assert_eq!(core.type_params, vec!["T".to_string()]);
    assert_eq!(core.params.len(), 2);
    assert_eq!(core.params[0].0, "f");
    assert_eq!(core.params[1].0, "x");
    assert_eq!(core.visibility, ash_core::ast::Visibility::Public);
}
