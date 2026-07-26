//! TASK-2003: Core text must not admit the CPS terminal observation as Core syntax.

use ash_core::core_ash_text::parse_core_expr;

#[test]
fn rejects_direct_return_as_a_core_expression_form() {
    let error = parse_core_expr("(return (lit-int 42))")
        .expect_err("Return is a CPS terminal observation, not a Core expression form");

    assert_eq!(error.position(), 8);
    assert_eq!(error.message(), "unsupported expression form `return`");
}

#[test]
fn rejects_recursive_terminal_value_as_a_direct_core_expression_form() {
    let error = parse_core_expr(
        "(return (record (tag (lit-string \"Err\")) (error (tuple (lit-int 42) (lit-string \"boom\")))))",
    )
    .expect_err("recursive Return values remain CPS terminal observations, never Core syntax");

    assert_eq!(error.message(), "unsupported expression form `return`");
}
