use ash_parser::surface::{Expr, expand_surface_module};

fn first_function_body(module: &ash_parser::surface::ModuleFile) -> &Expr {
    let ash_parser::surface::Definition::Function(def) = &module.definitions[0] else {
        panic!("expected function definition")
    };
    match &def.body {
        Expr::Block {
            tail_expr: Some(tail),
            ..
        } => tail.as_ref(),
        other => other,
    }
}

#[test]
fn generated_section_parameters_are_not_source_spellable_identifiers() {
    let module =
        ash_parser::parse_surface_file("fn op() -> Int { (+) }").expect("bare section parses");
    let expanded = expand_surface_module(module).expect("built-in section elaborates");

    let Expr::FnDef { params, .. } = first_function_body(&expanded.module) else {
        panic!("expected eta-expanded function")
    };

    assert_eq!(params.len(), 2);
    assert!(
        params
            .iter()
            .all(|(name, _)| name.starts_with("$ash_generated_section_"))
    );
    for (name, _) in params {
        assert!(
            ash_parser::parse_surface_file(&format!("fn leak() -> Int {{ {name} }}")).is_err(),
            "generated helper name {name} must not be source-spellable"
        );
    }
}

#[test]
fn source_bindings_named_like_generated_helper_placeholders_do_not_capture_generated_params() {
    let module = ash_parser::parse_surface_file(
        "fn collide(__section_lhs: Int, __section_rhs: Int) -> Int { (+) }",
    )
    .expect("source helper-like bindings and bare section parse");
    let expanded = expand_surface_module(module).expect("section elaborates");

    let Expr::FnDef { params, body, .. } = first_function_body(&expanded.module) else {
        panic!("expected eta-expanded function")
    };

    let names: Vec<&str> = params.iter().map(|(name, _)| name.as_ref()).collect();
    assert!(!names.contains(&"__section_lhs"));
    assert!(!names.contains(&"__section_rhs"));
    assert!(matches!(body.as_ref(), Expr::Binary { left, right, .. }
        if matches!(left.as_ref(), Expr::Variable { name, .. } if name.starts_with("$ash_generated_section_"))
            && matches!(right.as_ref(), Expr::Variable { name, .. } if name.starts_with("$ash_generated_section_"))));
}

#[test]
fn generated_identifier_spelling_carries_expansion_context() {
    let module =
        ash_parser::parse_surface_file("fn op() -> Int { (+) }").expect("bare section parses");
    let expanded = expand_surface_module(module).expect("built-in section elaborates");

    let Expr::FnDef { params, .. } = first_function_body(&expanded.module) else {
        panic!("expected eta-expanded function")
    };

    assert_eq!(expanded.origins.len(), 1);
    let expansion_id = expanded.origins[0].expansion_id.0;
    assert!(
        params
            .iter()
            .all(|(name, _)| name.contains(&format!("_{expansion_id}_")))
    );
}
