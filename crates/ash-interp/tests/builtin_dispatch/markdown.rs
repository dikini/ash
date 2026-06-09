use super::support::*;

#[test]
fn dispatch_table_contains_markdown_parse() {
    let table = builtin_dispatch_table();
    let entry = table
        .get("markdown::parse")
        .expect("markdown::parse should be in the dispatch table");
    assert_eq!(entry.arity, 1);
    assert!(!entry.variadic);
    assert!(entry.implemented);
}

#[test]
fn is_known_builtin_markdown_parse() {
    assert!(is_known_builtin("parse", Some("markdown")));
}

#[test]
fn eval_function_call_markdown_parse_dispatch() {
    let ctx = Context::new();
    let args = [Value::String("# Title\n\nParagraph text".to_string())];
    let result = dispatch_builtin("markdown::parse", &args, &ctx)
        .expect("dispatch should find markdown::parse")
        .expect("markdown::parse should succeed");
    let json_str = match result {
        Value::String(s) => s,
        other => panic!("expected String, got {other:?}"),
    };
    let val: serde_json::Value = serde_json::from_str(&json_str).expect("should be valid JSON");
    let blocks = val["blocks"].as_array().expect("blocks should be array");
    assert_eq!(blocks.len(), 2);
    assert_eq!(blocks[0]["type"], "heading");
    assert_eq!(blocks[1]["type"], "paragraph");
}
