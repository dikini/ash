use ash_parser::surface::{ComputationRowItem, Definition, Type};

fn first_function(source: &str) -> ash_parser::surface::FnDef {
    let module = ash_parser::parse_surface_file(source).expect("module parses");
    let Definition::Function(function) = module
        .definitions
        .into_iter()
        .next()
        .expect("module has a function")
    else {
        panic!("expected function definition");
    };
    function
}

#[test]
fn inline_row_items_keep_source_spans_before_validation() {
    let function =
        first_function("fn guarded(f: (Int) -> {PosixFs::read | r} String) -> Int { 0 }");
    let Type::Fn(_, Some(row), _) = &function.params[0].ty else {
        panic!("expected row-bearing callable parameter");
    };

    assert!(
        row.span.start < row.span.end,
        "row should carry a non-empty source span"
    );
    assert_eq!(row.items.len(), 2);
    for item in &row.items {
        let span = match item {
            ComputationRowItem::Operation { span, .. } | ComputationRowItem::Tail { span, .. } => {
                span
            }
            other => panic!("unexpected row item {other:?}"),
        };
        assert!(
            span.start < span.end,
            "row item should carry a non-empty source span: {item:?}"
        );
    }
}

#[test]
fn expanded_row_block_span_survives_surface_parsing() {
    let function = first_function(
        r#"
        fn guarded(req: Int) -> Int
        where
            row {
                resource fs,
                fail RuntimeError,
                evidence response_contract,
                | r
            }
        {
            req
        }
        "#,
    );
    let row = function
        .proposition_tail
        .as_ref()
        .and_then(|tail| tail.row.as_ref())
        .expect("where row is parsed");

    assert!(row.row_keyword_span.start < row.row_keyword_span.end);
    assert!(row.span.start < row.span.end);
    assert_eq!(row.row.items.len(), 4);
}
