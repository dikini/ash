use ash_parser::surface::{Definition, MacroInputKind, Type, collect_public_macro_summaries};

#[test]
fn typed_macro_signatures_parse_into_surface_carriers() {
    let module = ash_parser::parse_surface_file(
        r#"
pub macro inc(x: Int, label: String) -> Int => add(x, 1);
"#,
    )
    .expect("typed macro parses");

    let macro_def = module
        .definitions
        .iter()
        .find_map(|definition| match definition {
            Definition::Macro(def) => Some(def),
            _ => None,
        })
        .expect("macro definition exists");

    assert_eq!(macro_def.params, vec!["x".into(), "label".into()]);
    let signature = macro_def
        .typed_signature
        .as_ref()
        .expect("typed signature carrier exists");
    assert_eq!(
        signature.param_types,
        vec![
            Some(Type::Name("Int".into())),
            Some(Type::Name("String".into()))
        ]
    );
    assert_eq!(signature.return_type, Some(Type::Name("Int".into())));
    assert!(signature.span.start < signature.span.end);
}

#[test]
fn public_macro_summaries_preserve_typed_signatures() {
    let module = ash_parser::parse_surface_file(
        r#"
pub macro inc(x: Int) -> Int => add(x, 1);
"#,
    )
    .expect("typed public macro parses");

    let summaries = collect_public_macro_summaries(&module, "provider")
        .expect("typed public macro summary collects");
    assert_eq!(summaries.len(), 1);
    let summary = &summaries[0];
    assert_eq!(summary.input_kind, MacroInputKind::ExprArgs);
    let signature = summary
        .typed_signature
        .as_ref()
        .expect("summary preserves typed signature");
    assert_eq!(signature.param_types, vec![Some(Type::Name("Int".into()))]);
    assert_eq!(signature.return_type, Some(Type::Name("Int".into())));
}

#[test]
fn untyped_phase_172_macros_remain_accepted() {
    let module = ash_parser::parse_surface_file("pub macro inc(x) => add(x, 1);")
        .expect("untyped macro still parses");
    let summaries = collect_public_macro_summaries(&module, "provider")
        .expect("untyped public macro summary still collects");
    assert_eq!(summaries.len(), 1);
    assert!(summaries[0].typed_signature.is_none());
}
