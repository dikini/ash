//! Integration tests for canonical REPL type reporting.

use ash_repl::infer_type_display;

#[test]
fn type_reporting_uses_canonical_type_names() {
    assert_eq!(infer_type_display("42").expect("type inference"), "Int");
    assert_eq!(
        infer_type_display("\"hello\"").expect("type inference"),
        "String"
    );
    assert_eq!(
        infer_type_display("[1, 2, 3]").expect("type inference"),
        "List<Int>"
    );
}

#[test]
fn type_reporting_flows_through_expression_inference() {
    assert_eq!(infer_type_display("1 + 2").expect("type inference"), "Int");
    assert_eq!(infer_type_display("!true").expect("type inference"), "Bool");
}
