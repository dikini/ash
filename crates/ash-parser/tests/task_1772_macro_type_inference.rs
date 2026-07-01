use ash_parser::surface::{MacroSummary, Type, collect_public_macro_summaries};

fn assert_name_type(ty: &Type, expected: &str) {
    assert!(
        matches!(ty, Type::Name(name) if name.as_ref() == expected),
        "expected {expected}, got {ty:?}"
    );
}

fn public_macro_summary(source: &str, name: &str) -> MacroSummary {
    let module = ash_parser::parse_surface_file(source).expect("module parses");
    let summaries = collect_public_macro_summaries(&module, "test").expect("summaries collect");
    summaries
        .into_iter()
        .find(|summary| summary.name.as_ref() == name)
        .expect("macro summary exists")
}

fn public_macro_signature(
    source: &str,
    name: &str,
) -> ash_parser::surface::MacroTypeSignatureSummary {
    public_macro_summary(source, name)
        .typed_signature
        .expect("macro has inferred typed signature")
}

#[test]
fn infers_literal_macro_result_summary() {
    let signature = public_macro_signature("pub macro answer() => 1;", "answer");

    assert!(signature.param_types.is_empty());
    assert_name_type(&signature.return_type.expect("return inferred"), "Int");
}

#[test]
fn infers_identity_result_from_annotated_parameter() {
    let signature = public_macro_signature("pub macro id(x: Int) => x;", "id");

    assert_eq!(signature.param_types.len(), 1);
    assert_name_type(
        signature.param_types[0]
            .as_ref()
            .expect("param type preserved"),
        "Int",
    );
    assert_name_type(&signature.return_type.expect("return inferred"), "Int");
}

#[test]
fn infers_bounded_binary_operator_result_from_annotated_parameter() {
    let signature = public_macro_signature("pub macro inc(x: Int) => x + 1;", "inc");

    assert_name_type(&signature.return_type.expect("return inferred"), "Int");
}

#[test]
fn ordinary_calls_do_not_fabricate_inferred_results() {
    let signature = public_macro_signature("pub macro inc(x: Int) => add(x, 1);", "inc");

    assert_eq!(signature.param_types.len(), 1);
    assert_name_type(
        signature.param_types[0]
            .as_ref()
            .expect("param type preserved"),
        "Int",
    );
    assert!(signature.return_type.is_none());
}

#[test]
fn infers_result_through_unique_local_callable_identity() {
    let signature = public_macro_signature(
        "pub fn add(a: Int, b: Int) -> Int { a + b }\npub macro inc(x: Int) => add(x, 1);",
        "inc",
    );

    assert_name_type(&signature.return_type.expect("return inferred"), "Int");
}

#[test]
fn ambiguous_local_callable_identity_stays_uninferred() {
    let signature = public_macro_signature(
        "pub fn add(a: Int, b: Int) -> Int { a + b }\npub fn add(a: Int, b: Int) -> Int { a + b }\npub macro inc(x: Int) => add(x, 1);",
        "inc",
    );

    assert!(signature.return_type.is_none());
}

#[test]
fn wrong_arity_call_stays_uninferred_even_with_typed_arguments() {
    let signature = public_macro_signature(
        "pub fn add(a: Int, b: Int) -> Int { a + b }\npub macro bad_add(x: Int) => add(x);",
        "bad_add",
    );

    assert_eq!(signature.param_types.len(), 1);
    assert!(signature.return_type.is_none());
}

#[test]
fn wrong_argument_type_to_proven_callable_stays_uninferred() {
    let signature = public_macro_signature(
        "pub fn len(x: String) -> Int { 1 }\npub macro bad_len(x: Int) => len(x);",
        "bad_len",
    );

    assert_eq!(signature.param_types.len(), 1);
    assert!(signature.return_type.is_none());
}

#[test]
fn private_callable_identity_stays_uninferred_for_public_macro_summary() {
    let signature = public_macro_signature(
        "fn add(a: Int, b: Int) -> Int { a + b }\npub macro inc(x: Int) => add(x, 1);",
        "inc",
    );

    assert!(signature.return_type.is_none());
}

#[test]
fn macro_summary_name_is_not_callable_identity_proof() {
    let signature = public_macro_signature(
        "pub macro add(x: Int) -> Int => x;\npub macro inc(x: Int) => add(x, 1);",
        "inc",
    );

    assert_eq!(signature.param_types.len(), 1);
    assert!(signature.return_type.is_none());
}

#[test]
fn module_qualified_calls_stay_uninferred() {
    let signature = public_macro_signature(
        "pub macro call_math(x: Int) => math::add(x, 1);",
        "call_math",
    );

    assert_eq!(signature.param_types.len(), 1);
    assert!(signature.return_type.is_none());
}

#[test]
fn ambiguous_unannotated_identity_does_not_fabricate_summary() {
    let summary = public_macro_summary("pub macro id(x) => x;", "id");

    assert!(summary.typed_signature.is_none());
}
