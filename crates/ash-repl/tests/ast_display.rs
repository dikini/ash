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
fn ast_display_formats_workflow_ast_in_spec_shape() {
    let output = ast_display("workflow demo { ret 42; }").expect("workflow AST");

    assert_eq!(
        output,
        concat!(
            "WorkflowDef {\n",
            "  name: \"demo\",\n",
            "  params: [],\n",
            "  plays_roles: [],\n",
            "  capabilities: [],\n",
            "  body: Ret {\n",
            "    expr: Literal(Int(42)),\n",
            "  },\n",
            "  contract: None,\n",
            "}"
        )
    );
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
    assert!(!output.contains("workflow __"));
    assert!(!output.contains("Expr {"));
    assert!(!output.contains("Workflow {"));
    assert!(!output.contains("span:"));
    assert!(!output.contains("NodeId"));
}
