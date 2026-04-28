//! TASK-748 integration tests for generalized do-target resolution through expression type checking.

use ash_parser::input::new_input;
use ash_parser::parse_expr::expr;
use ash_parser::surface::Expr;
use ash_typeck::check_expr::check_expr;
use ash_typeck::error::ConstructorError;
use ash_typeck::type_env::TypeEnv;
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
    let result = check_expr(&TypeEnv::with_builtin_types(), &parse_expr_source(source));
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

#[test]
fn check_expr_resolves_act_target_before_typed_elaboration_boundary() {
    let message = first_unsupported("do:Act { return 1 }");

    assert!(
        message.contains("statement type checking") && message.contains("TASK-749"),
        "{message}"
    );
}

#[test]
fn check_expr_resolves_proc_target_before_typed_elaboration_boundary() {
    let message = first_unsupported("do:Proc { return 1 }");

    assert!(
        message.contains("statement type checking") && message.contains("TASK-749"),
        "{message}"
    );
}

#[test]
fn check_expr_reports_wrong_kind_for_proper_type_target() {
    let message = first_unsupported("do:Int { return 1 }");

    assert!(message.contains("do target Int has kind *"), "{message}");
    assert!(message.contains("expected * -> *"), "{message}");
}

#[test]
fn check_expr_reports_unknown_do_target() {
    let message = first_unsupported("do:Missing { return 1 }");

    assert!(message.contains("unknown do target 'Missing'"), "{message}");
}

#[test]
fn check_expr_reports_result_target_deferred() {
    let message = first_unsupported("do:Result { return 1 }");

    assert!(message.contains("Result"), "{message}");
    assert!(message.contains("deferred"), "{message}");
    assert!(message.contains("Monad<K>"), "{message}");
}
