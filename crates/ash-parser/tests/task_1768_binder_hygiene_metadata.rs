use ash_parser::surface::{Expr, IdentifierHygieneContext, SurfaceOrigin, expand_surface_module};

#[test]
fn hygiene_metadata_distinguishes_definition_call_site_and_generated_identifiers() {
    let module = ash_parser::parse_surface_file(
        r#"
macro plus_one(x) => (+);
macro id(x) => x;
fn use_macro(n: Int) -> Int { plus_one!(n) }
fn use_id(n: Int) -> Int { id!(n) }
"#,
    )
    .expect("module parses");

    let expanded = expand_surface_module(module).expect("macro and section expand");

    assert!(
        expanded.hygiene.iter().any(|item| {
            item.name.as_ref() == "n"
                && item.context == IdentifierHygieneContext::DefinitionSite
                && item.expansion_id.is_none()
        }),
        "source function parameter should be definition-site metadata"
    );
    assert!(
        expanded.hygiene.iter().any(|item| {
            item.name.as_ref() == "n"
                && item.context == IdentifierHygieneContext::CallSite
                && item.expansion_id.is_none()
        }),
        "macro argument variable use should be call-site metadata"
    );

    let generated = expanded
        .hygiene
        .iter()
        .filter(|item| item.context == IdentifierHygieneContext::Generated)
        .collect::<Vec<_>>();
    assert!(
        !generated.is_empty(),
        "operator section should generate binders"
    );
    for item in generated {
        assert!(item.name.as_ref().starts_with("$ash_generated_section_"));
        assert!(item.expansion_id.is_some());
    }
}

#[test]
fn source_bindings_cannot_capture_generated_binder_hygiene_metadata() {
    let module = ash_parser::parse_surface_file(
        r#"
macro plus_one(x) => (<+> x);
infixl 6 <+> = add;
fn use_macro(__ash_generated_section_1_lhs: Int) -> Int { plus_one!(__ash_generated_section_1_lhs) }
"#,
    )
    .expect("source helper-like binding parses");

    let expanded = expand_surface_module(module).expect("macro and notation expand");
    assert!(expanded.hygiene.iter().any(|item| {
        item.name.as_ref() == "__ash_generated_section_1_lhs"
            && item.context == IdentifierHygieneContext::DefinitionSite
            && item.expansion_id.is_none()
    }));
    assert!(expanded.hygiene.iter().any(|item| {
        item.name.as_ref().starts_with("$ash_generated_section_")
            && item.context == IdentifierHygieneContext::Generated
            && item.expansion_id.is_some()
    }));
}

#[test]
fn hygiene_metadata_is_syntax_side_and_origin_only() {
    let module = ash_parser::parse_surface_file("fn op() -> Int { (+) }").expect("section parses");
    let expanded = expand_surface_module(module).expect("section expands");

    assert!(
        expanded
            .origins
            .iter()
            .any(|origin| matches!(origin.origin, SurfaceOrigin::OperatorSection { .. }))
    );
    assert!(
        expanded
            .hygiene
            .iter()
            .any(|item| item.context == IdentifierHygieneContext::Generated)
    );

    let function = expanded
        .module
        .definitions
        .iter()
        .find_map(|definition| match definition {
            ash_parser::surface::Definition::Function(def) => Some(def),
            _ => None,
        })
        .expect("function exists");
    let Expr::FnDef { params, .. } = (match &function.body {
        Expr::Block {
            tail_expr: Some(tail),
            ..
        } => tail.as_ref(),
        other => other,
    }) else {
        panic!("expected generated function expression")
    };
    assert_eq!(params.len(), 2);
}
