use ash_parser::surface::{
    ExpansionError, Type, build_local_macro_table, expand_surface_module,
    expand_surface_module_with_imported_macros,
};

#[test]
fn typed_macro_argument_mismatch_rejects_before_expansion_acceptance() {
    let module = ash_parser::parse_surface_file(
        r#"
macro inc(x: Int) -> Int => x;
fn use_macro() -> Int { inc!("not-int") }
"#,
    )
    .expect("module parses before typed macro check");

    let err = expand_surface_module(module).expect_err("typed macro arg mismatch rejects");
    let message = err.to_string();
    assert!(
        message.contains("macro `inc` typed signature mismatch at argument 1 at call site"),
        "unexpected error: {message}"
    );
    assert!(
        message.contains("expected Int"),
        "unexpected error: {message}"
    );
    assert!(
        message.contains("got String"),
        "unexpected error: {message}"
    );
}

#[test]
fn typed_macro_unknown_argument_type_rejects_fail_closed() {
    let module = ash_parser::parse_surface_file(
        r#"
macro inc(x: Int) -> Int => x;
fn use_macro(n: Int) -> Int { inc!(n) }
"#,
    )
    .expect("module parses before typed macro check");

    let err = expand_surface_module(module).expect_err("unknown typed macro arg rejects");
    let message = err.to_string();
    assert!(
        message.contains("macro `inc` typed signature mismatch at argument 1 at call site"),
        "unexpected error: {message}"
    );
    assert!(
        message.contains("expected Int"),
        "unexpected error: {message}"
    );
    assert!(
        message.contains("got unknown argument type"),
        "unexpected error: {message}"
    );
}

#[test]
fn malformed_imported_typed_signature_rejects_instead_of_panicking() {
    let provider = ash_parser::parse_surface_file("pub macro inc(x: Int) -> Int => x;")
        .expect("provider parses");
    let table = build_local_macro_table(&provider).expect("provider macro table builds");
    let mut imported = table
        .resolve("inc")
        .expect("provider macro entry exists")
        .clone();
    imported
        .typed_signature
        .as_mut()
        .expect("typed signature exists")
        .param_types
        .push(Some(Type::Name("Int".into())));

    let caller =
        ash_parser::parse_surface_file("fn use_macro() -> Int { inc!(1) }").expect("caller parses");
    let err = expand_surface_module_with_imported_macros(caller, vec![imported])
        .expect_err("malformed imported typed signature rejects");
    let message = err.to_string();
    assert!(
        message.contains("typed signature has 2 parameter(s), but macro declares 1 parameter(s)"),
        "unexpected error: {message}"
    );
}

#[test]
fn typed_macro_template_result_mismatch_rejects_with_definition_context() {
    let source = r#"
macro bad(x: Int) -> String => x;
fn use_macro() -> String { bad!(1) }
"#;
    let module =
        ash_parser::parse_surface_file(source).expect("module parses before typed macro check");

    let err = expand_surface_module(module).expect_err("typed macro result mismatch rejects");
    let message = err.to_string();
    assert!(
        message.contains(
            "macro `bad` typed signature mismatch at template result at macro definition"
        ),
        "unexpected error: {message}"
    );
    assert!(
        message.contains("expected String"),
        "unexpected error: {message}"
    );
    assert!(message.contains("got Int"), "unexpected error: {message}");

    match err {
        ExpansionError::MacroTypeMismatch { span, position, .. } => {
            assert_eq!(position.as_ref(), "template result at macro definition");
            let template_body_start = source
                .find("=> x")
                .expect("fixture contains macro template body")
                + "=> ".len();
            assert_eq!(
                span.start, template_body_start,
                "result mismatch should point at the template body, not the call site"
            );
        }
        other => panic!("unexpected expansion error: {other:?}"),
    }
}

#[test]
fn typed_macro_matching_signature_expands_normally() {
    let module = ash_parser::parse_surface_file(
        r#"
macro id_int(x: Int) -> Int => x;
fn use_macro() -> Int { id_int!(1) }
"#,
    )
    .expect("module parses before typed macro check");

    expand_surface_module(module).expect("matching typed macro signature expands");
}
