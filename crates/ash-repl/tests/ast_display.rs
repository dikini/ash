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
fn ast_display_formats_canonical_on_handler_clauses_without_debug_leakage() {
    let output = ast_display(
        "on run(req) { TestClock::sleep(milliseconds, resume) => resume(milliseconds), done(value) => value, }",
    )
    .expect("canonical `on` handler AST");

    assert!(output.starts_with("On {\n"));
    assert!(output.contains("  computation: Call {\n"));
    assert!(output.contains("    func: \"run\",\n"));
    assert!(output.contains("    Operation {\n"));
    assert!(output.contains("      impl_type: \"TestClock\",\n"));
    assert!(output.contains("      operation: \"sleep\",\n"));
    assert!(output.contains("      pattern: Variable(\"milliseconds\"),\n"));
    assert!(output.contains("      resume: \"resume\",\n"));
    assert!(output.contains("    Done {\n"));
    assert!(output.contains("      binding: \"value\",\n"));
    assert_no_ast_debug_leakage(&output);
}

#[test]
fn ast_display_formats_composite_on_clause_patterns_without_debug_leakage() {
    let output = ast_display(
        "on run(req) {\
         TestClock::list([head, ..tail], resume) => null,\
         TestClock::record({ left: first, right: _ }, resume) => null,\
         TestClock::tuple((first, _), resume) => null,\
         TestClock::unit(None, resume) => null,\
         TestClock::variant_tuple(Pair(first, 7), resume) => null,\
         TestClock::variant_record(Some { value: value }, resume) => null,\
         TestClock::literal(42, resume) => null,\
         done(value) => value,\
         }",
    )
    .expect("composite canonical `on` handler AST");

    assert!(output.contains("pattern: ListPattern {\n"));
    assert!(output.contains("elements: [\n"));
    assert!(output.contains("Variable(\"head\")"));
    assert!(output.contains("rest: Some(\"tail\")"));
    assert!(output.contains("pattern: Record([\n"));
    assert!(output.contains("Field(\"left\", Variable(\"first\"))"));
    assert!(output.contains("Field(\"right\", Wildcard)"));
    assert!(output.contains("pattern: Tuple([\n"));
    assert!(output.contains("name: \"None\","));
    assert!(output.contains("payload: Unit,"));
    assert!(output.contains("name: \"Pair\","));
    assert!(output.contains("payload: Tuple([\n"));
    assert!(output.contains("Literal(Int(7))"));
    assert!(output.contains("name: \"Some\","));
    assert!(output.contains("payload: Record([\n"));
    assert!(output.contains("Field(\"value\", Variable(\"value\"))"));
    assert!(output.contains("pattern: Literal(Int(42)),"));
    assert_no_ast_debug_leakage(&output);
}

#[test]
fn ast_display_formats_canonical_handle_with_without_debug_leakage() {
    let output = ast_display("handle TestClock::sleep(0) with absorb_sleep")
        .expect("canonical `handle ... with` AST");

    assert!(output.starts_with("HandleWith {\n"));
    assert!(output.contains("  expression: Call {\n"));
    assert!(output.contains("    func: \"sleep\",\n"));
    assert!(output.contains("  handler: \"absorb_sleep\",\n"));
    assert_no_ast_debug_leakage(&output);
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

fn assert_no_ast_debug_leakage(output: &str) {
    for marker in ["span:", "Span {", "Expr {", "NodeId"] {
        assert!(
            !output.contains(marker),
            "AST display leaked parser/debug detail `{marker}`: {output}"
        );
    }
}
