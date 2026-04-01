use ash_core::ast::{TypeBody, TypeDef, TypeExpr, VariantDef, Visibility};
use ash_parser::surface::{Expr, Literal};
use ash_parser::token::Span;
use ash_typeck::check_expr::check_expr;
use ash_typeck::type_env::TypeEnv;
use ash_typeck::types::Type;

fn runtime_error_type_def() -> TypeDef {
    TypeDef {
        name: "RuntimeError".to_string(),
        params: vec![],
        body: TypeBody::Enum(vec![VariantDef {
            name: "RuntimeError".to_string(),
            fields: vec![
                ("exit_code".to_string(), TypeExpr::Named("Int".to_string())),
                ("message".to_string(), TypeExpr::Named("String".to_string())),
            ],
        }]),
        visibility: Visibility::Public,
    }
}

fn runtime_error_expr(exit_code: i64, message: &str) -> Expr {
    Expr::Constructor {
        name: "RuntimeError".into(),
        fields: vec![
            ("exit_code".into(), Expr::Literal(Literal::Int(exit_code))),
            (
                "message".into(),
                Expr::Literal(Literal::String(message.into())),
            ),
        ],
        span: Span::default(),
    }
}

#[test]
fn runtime_error_constructor_typechecks() {
    let mut env = TypeEnv::with_builtin_types();
    env.register_type(&runtime_error_type_def()).unwrap();

    let result = check_expr(&env, &runtime_error_expr(7, "boom"));

    assert!(
        result.is_ok(),
        "expected RuntimeError constructor to typecheck"
    );
    match result.ty {
        Type::Constructor { name, args, .. } => {
            assert_eq!(name.to_string(), "RuntimeError");
            assert!(args.is_empty());
        }
        other => panic!("expected RuntimeError constructor type, got {other:?}"),
    }
}

#[test]
fn runtime_error_composes_inside_result_constructor() {
    let mut env = TypeEnv::with_builtin_types();
    env.register_type(&runtime_error_type_def()).unwrap();

    let expr = Expr::Constructor {
        name: "Err".into(),
        fields: vec![("error".into(), runtime_error_expr(42, "test"))],
        span: Span::default(),
    };

    let result = check_expr(&env, &expr);

    assert!(
        result.is_ok(),
        "expected Err {{ error: RuntimeError {{ ... }} }} to typecheck"
    );
    match result.ty {
        Type::Constructor { name, args, .. } => {
            assert_eq!(name.to_string(), "Result");
            assert_eq!(args.len(), 2);
            match &args[1] {
                Type::Constructor { name, args, .. } => {
                    assert_eq!(name.to_string(), "RuntimeError");
                    assert!(args.is_empty());
                }
                other => panic!("expected RuntimeError error payload type, got {other:?}"),
            }
        }
        other => panic!("expected Result constructor type, got {other:?}"),
    }
}
