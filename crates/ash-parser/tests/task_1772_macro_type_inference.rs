use ash_parser::surface::{Type, collect_public_macro_summaries};

fn assert_name_type(ty: &Type, expected: &str) {
    assert!(
        matches!(ty, Type::Name(name) if name.as_ref() == expected),
        "expected {expected}, got {ty:?}"
    );
}

fn public_macro_signature(
    source: &str,
    name: &str,
) -> ash_parser::surface::MacroTypeSignatureSummary {
    let module = ash_parser::parse_surface_file(source).expect("module parses");
    let summaries = collect_public_macro_summaries(&module, "test").expect("summaries collect");
    summaries
        .into_iter()
        .find(|summary| summary.name.as_ref() == name)
        .and_then(|summary| summary.typed_signature)
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
fn infers_bounded_builtin_call_result_from_annotated_parameter() {
    let signature = public_macro_signature("pub macro inc(x: Int) => add(x, 1);", "inc");

    assert_name_type(&signature.return_type.expect("return inferred"), "Int");
}

#[test]
fn ambiguous_unannotated_identity_does_not_fabricate_summary() {
    let module = ash_parser::parse_surface_file("pub macro id(x) => x;").expect("module parses");
    let summaries = collect_public_macro_summaries(&module, "test").expect("summaries collect");
    let summary = summaries
        .into_iter()
        .find(|summary| summary.name.as_ref() == "id")
        .expect("macro summary exists");

    assert!(summary.typed_signature.is_none());
}
