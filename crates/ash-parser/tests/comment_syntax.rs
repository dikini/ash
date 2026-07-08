use ash_parser::parse_surface_file;

#[test]
fn slash_comments_are_skipped_at_file_start() {
    let source = "// leading file comment\nfn main() -> Int { 0 }\n";

    let module = parse_surface_file(source).expect("leading // comment should parse");

    assert_eq!(module.definitions.len(), 1);
}

#[test]
fn slash_comments_are_skipped_inside_workflow_body() {
    let source = "fn main() -> Int {\n  do {\n    // before statement\n    let x = 1;\n    // before return\n    return x;\n  }\n}\n";

    let module = parse_surface_file(source).expect("in-body // comments should parse");

    assert_eq!(module.definitions.len(), 1);
}

#[test]
fn slash_comments_are_skipped_after_statements() {
    let source = "fn main() -> Int {\n  do {\n    let x = 1; // trailing let comment\n    return x; // trailing return comment\n  }\n}\n";

    let module = parse_surface_file(source).expect("trailing // comments should parse");

    assert_eq!(module.definitions.len(), 1);
}

#[test]
fn existing_dash_and_block_comments_still_parse() {
    let source = "-- leading dash comment\n/* block comment */\nfn main() -> Int {\n  do {\n    let x = 1; -- trailing dash comment\n    /* body block */\n    return x;\n  }\n}\n";

    let module = parse_surface_file(source).expect("existing comments should keep parsing");

    assert_eq!(module.definitions.len(), 1);
}
