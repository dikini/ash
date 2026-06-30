use ash_parser::surface::expand_surface_module;

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
fn typed_macro_template_result_mismatch_rejects_with_definition_context() {
    let module = ash_parser::parse_surface_file(
        r#"
macro bad(x: Int) -> String => x;
fn use_macro() -> String { bad!(1) }
"#,
    )
    .expect("module parses before typed macro check");

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
