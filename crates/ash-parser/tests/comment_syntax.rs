use ash_parser::parse_surface_file;

#[test]
fn slash_comments_are_skipped_at_file_start() {
    let source = "// leading file comment\nworkflow main() { done; }\n";

    let module = parse_surface_file(source).expect("leading // comment should parse");

    assert!(module.workflow.is_some());
}

#[test]
fn slash_comments_are_skipped_inside_workflow_body() {
    let source =
        "workflow main() {\n  // before statement\n  let x = 1;\n  // before ret\n  ret x;\n}\n";

    let module = parse_surface_file(source).expect("in-body // comments should parse");

    assert!(module.workflow.is_some());
}

#[test]
fn slash_comments_are_skipped_after_statements() {
    let source = "workflow main() {\n  let x = 1; // trailing let comment\n  ret x; // trailing ret comment\n}\n";

    let module = parse_surface_file(source).expect("trailing // comments should parse");

    assert!(module.workflow.is_some());
}

#[test]
fn existing_dash_and_block_comments_still_parse() {
    let source = "-- leading dash comment\n/* block comment */\nworkflow main() {\n  let x = 1; -- trailing dash comment\n  /* body block */\n  ret x;\n}\n";

    let module = parse_surface_file(source).expect("existing comments should keep parsing");

    assert!(module.workflow.is_some());
}
