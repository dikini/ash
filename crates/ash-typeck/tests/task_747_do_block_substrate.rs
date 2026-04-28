//! TASK-747/TASK-749 regression tests for generalized do-block parser/typechecker boundaries.

use ash_parser::input::new_input;
use ash_parser::parse_expr::expr;
use ash_parser::surface::Expr;
use ash_typeck::check_expr::check_expr;
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

#[test]
fn do_block_typecheck_is_supported_after_typed_elaboration() {
    let expr = parse_expr_source("do:Act { return 1 }");
    let result = check_expr(&TypeEnv::with_builtin_types(), &expr);

    assert!(
        result.is_ok(),
        "expected typed do-block support, got {result:?}"
    );
    assert_eq!(
        result.ty,
        Type::Constructor {
            name: QualifiedName::root("Act"),
            args: vec![Type::Int],
            kind: Kind::Type,
        }
    );
}
