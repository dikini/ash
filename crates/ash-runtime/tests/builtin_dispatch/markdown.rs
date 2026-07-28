use ash_runtime::builtin_dispatch_table;

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
