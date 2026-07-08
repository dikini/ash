//! Integration tests for canonical REPL AST reporting.

use ash_repl::ast_display;

#[test]
fn ast_display_formats_expression_ast_in_spec_shape() {
    let output = ast_display("1 + 2").expect("expression AST");

    assert_eq!(
        output,
        "Binary {\n  op: Add,\n  left: Literal(Int(1)),\n  right: Literal(Int(2)),\n}"
    );
}

#[test]
fn ast_display_rejects_removed_workflow_declaration_shape() {
    let source = concat!("work", "flow demo { ret 42; }");
    assert!(ast_display(source).is_err());
}

#[test]
fn ast_display_pretty_indents_nested_nodes() {
    let output = ast_display("foo(1 + 2, bar)").expect("nested expression AST");

    assert!(output.contains("Call {\n"));
    assert!(output.contains("  func: \"foo\",\n"));
    assert!(output.contains("  args: [\n"));
    assert!(output.contains("    Binary {\n"));
    assert!(output.contains("      op: Add,\n"));
    assert!(output.contains("    Variable(\"bar\"),\n"));
}

#[test]
fn ast_display_omits_synthetic_workflows_and_debug_artifacts() {
    let output = ast_display("1 + 2").expect("expression AST");

    assert!(!output.contains("__ast__"));
    assert!(!output.contains(concat!("work", "flow __")));
    assert!(!output.contains("Expr {"));
    assert!(!output.contains("Workflow {"));
    assert!(!output.contains("span:"));
    assert!(!output.contains("NodeId"));
}
