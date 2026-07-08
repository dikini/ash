use ash_parser::lower::{LoweringError, lower_expr};
use ash_parser::parse_expr::expr;
use ash_parser::surface::{ExpansionError, Expr, expand_surface_module};

fn parse_expr_complete(source: &str) -> Expr {
    let mut input = ash_parser::input::new_input(source);
    let parsed = expr(&mut input).expect("expression should parse");
    ash_parser::parse_utils::skip_whitespace_and_comments(&mut input);
    assert!(
        input.input.is_empty(),
        "unconsumed input: {:?}",
        input.input
    );
    parsed
}

#[test]
fn ordinary_module_crosses_expanded_surface_boundary_as_no_op() {
    let module = ash_parser::parse_surface_file("fn id(x: Int) -> Int { x }")
        .expect("ordinary module should parse");
    let expanded =
        expand_surface_module(module.clone()).expect("ordinary module needs no expansion");
    assert_eq!(expanded.module, module);
    assert!(expanded.diagnostics.is_empty());
}

#[test]
fn operator_section_is_rejected_at_expanded_surface_boundary() {
    let module = ash_parser::parse_surface_file("fn plus_one(x: Int) -> Int { (<*> x) }")
        .expect("operator section parses as surface syntax");
    let err = expand_surface_module(module).expect_err("operator section must not silently expand");
    match err {
        ExpansionError::UnresolvedOperatorSection { operator, span } => {
            assert_eq!(operator.as_ref(), "<*>");
            assert!(span.start < span.end);
        }
        other => panic!("unexpected expansion error: {other:?}"),
    }
}

#[test]
fn operator_section_in_function_contract_is_rejected_at_expanded_surface_boundary() {
    let module =
        ash_parser::parse_surface_file("fn checked(x: Int) -> Int requires: (<*> x) { x }")
            .expect("function contract operator section parses as surface syntax");
    let err = expand_surface_module(module).expect_err("contract section must not expand");
    assert!(matches!(
        err,
        ExpansionError::UnresolvedOperatorSection { operator, .. } if operator.as_ref() == "<*>"
    ));
}

#[test]
fn operator_section_in_main_function_body_is_rejected_at_expanded_surface_boundary() {
    let module = ash_parser::parse_surface_file("fn main() -> Int { (<*> 1) }")
        .expect("target function operator section parses as surface syntax");
    let err = expand_surface_module(module).expect_err("function body section must not expand");
    assert!(matches!(
        err,
        ExpansionError::UnresolvedOperatorSection { operator, .. } if operator.as_ref() == "<*>"
    ));
}

#[test]
fn direct_operator_section_expr_remains_surface_only() {
    let parsed = parse_expr_complete("(<*> x)");
    assert!(matches!(parsed, Expr::OperatorSection { .. }));
}

#[test]
fn lower_expr_rejects_unresolved_operator_section() {
    let parsed = parse_expr_complete("(<*> x)");
    let err = lower_expr(&parsed).expect_err("operator section must not lower before expansion");
    assert!(matches!(
        err,
        LoweringError::UnsupportedFeature(message)
            if message.contains("operator section `<*>` must be resolved")
    ));
}
