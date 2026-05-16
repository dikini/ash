//! TASK-748 integration tests for generalized do-target resolution through expression type checking.

use ash_core::ast::{TypeBody, TypeDef, Visibility};
use ash_parser::input::new_input;
use ash_parser::parse_expr::expr;
use ash_parser::surface::Expr;
use ash_typeck::check_expr::check_expr;
use ash_typeck::error::ConstructorError;
use ash_typeck::type_env::TypeEnv;
use ash_typeck::{Kind, QualifiedName, Type};
use winnow::prelude::*;

fn parse_expr_source(source: &str) -> Expr {
    let mut input = new_input(source);
    let parsed = expr.parse_next(&mut input).expect("expression parses");
    assert!(
        input.input.is_empty(),
        "parser left trailing input: {:?}",
        input.input
    );
    parsed
}

fn first_unsupported(source: &str) -> String {
    let env = TypeEnv::with_builtin_types();
    first_unsupported_with_env(&env, source)
}

fn first_unsupported_with_env(env: &TypeEnv, source: &str) -> String {
    let result = check_expr(env, &parse_expr_source(source));
    result
        .errors
        .iter()
        .find_map(|err| match err {
            ConstructorError::UnsupportedExpression { kind, .. } => Some(kind.clone()),
            _ => None,
        })
        .unwrap_or_else(|| {
            format!(
                "expected unsupported expression error, got {:?}",
                result.errors
            )
        })
}

fn env_with_result_constructor() -> TypeEnv {
    let mut env = TypeEnv::with_builtin_types();
    env.register_type(&TypeDef {
        name: "E".into(),
        params: vec![],
        body: TypeBody::Struct(vec![]),
        visibility: Visibility::Public,
        builtin: false,
    })
    .expect("register E type");
    env
}

fn computation_type(name: &str, inner: Type) -> Type {
    Type::Constructor {
        name: QualifiedName::root(name),
        args: vec![inner],
        kind: Kind::Type,
    }
}

#[test]
fn check_expr_resolves_act_target_before_typed_elaboration_boundary() {
    let result = check_expr(
        &TypeEnv::with_builtin_types(),
        &parse_expr_source("do:Act { return 1 }"),
    );

    assert!(result.is_ok(), "do:Act should now type-check: {result:?}");
    assert_eq!(result.ty, computation_type("Act", Type::Int));
}

#[test]
fn check_expr_resolves_proc_target_before_typed_elaboration_boundary() {
    let result = check_expr(
        &TypeEnv::with_builtin_types(),
        &parse_expr_source("do:Proc { return 1 }"),
    );

    assert!(result.is_ok(), "do:Proc should now type-check: {result:?}");
    assert_eq!(result.ty, computation_type("Proc", Type::Int));
}

#[test]
fn check_expr_reports_wrong_kind_for_proper_type_target() {
    let message = first_unsupported("do:Int { return 1 }");

    assert!(message.contains("do target Int has kind *"), "{message}");
    assert!(message.contains("expected * -> *"), "{message}");
    assert!(message.contains("Act, Proc, or Workflow"), "{message}");
}

#[test]
fn check_expr_reports_unknown_do_target() {
    let message = first_unsupported("do:Missing { return 1 }");

    assert!(message.contains("unknown do target 'Missing'"), "{message}");
    assert!(message.contains("Act, Proc, or Workflow"), "{message}");
}

#[test]
fn check_expr_reports_partial_result_target_missing_evidence() {
    let env = env_with_result_constructor();
    let message = first_unsupported_with_env(&env, "do:Result<_, E> { return 1 }");

    assert!(message.contains("Result"), "{message}");
    assert!(message.contains("missing Monad evidence"), "{message}");
    assert!(message.contains("Monad<Result<_, E>>"), "{message}");
    assert!(message.contains("SPEC-067 Monad<K>"), "{message}");
}
