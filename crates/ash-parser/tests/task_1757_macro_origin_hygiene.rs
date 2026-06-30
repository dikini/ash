use ash_parser::surface::{ExpansionId, Expr, SurfaceOrigin, expand_surface_module, visit_expr};

#[test]
fn macro_expansion_origin_records_call_span_and_stable_id() {
    let module = ash_parser::parse_surface_file(
        r"
macro inc(x) => add(x, 1);
fn use_macro(n: Int) -> Int { inc!(n) }
",
    )
    .expect("module parses");

    let expanded = expand_surface_module(module).expect("macro expands");
    let macro_origin = expanded
        .origins
        .iter()
        .find(|origin| matches!(origin.origin, SurfaceOrigin::MacroExpansion { .. }))
        .expect("macro expansion origin recorded");
    assert_eq!(macro_origin.expansion_id, ExpansionId(1));
    assert!(matches!(
        macro_origin.origin,
        SurfaceOrigin::MacroExpansion { ref expansion_id, call_span }
            if expansion_id.as_ref() == "inc" && call_span.start < call_span.end
    ));
}

#[test]
fn notation_generated_inside_macro_records_macro_parent_origin() {
    let module = ash_parser::parse_surface_file(
        r"
infixl 6 <+> = add;
macro plus_one(x) => (<+> x);
fn use_macro(n: Int) -> Int { plus_one!(n) }
",
    )
    .expect("module parses");

    let expanded = expand_surface_module(module).expect("macro and notation expand");
    let notation_origin = expanded
        .origins
        .iter()
        .find(|origin| matches!(origin.origin, SurfaceOrigin::NotationExpansion { .. }))
        .expect("notation origin recorded");
    assert!(matches!(
        notation_origin.parent.as_deref(),
        Some(SurfaceOrigin::MacroExpansion { expansion_id, .. }) if expansion_id.as_ref() == "plus_one"
    ));
}

#[test]
fn nested_macro_expansion_records_parent_macro_origin() {
    let module = ash_parser::parse_surface_file(
        r"
macro inner(x) => add(x, 1);
macro outer(x) => inner!(x);
fn use_macro(n: Int) -> Int { outer!(n) }
",
    )
    .expect("module parses");

    let expanded = expand_surface_module(module).expect("nested macros expand");
    let inner_origin = expanded
        .origins
        .iter()
        .find(|origin| matches!(
            origin.origin,
            SurfaceOrigin::MacroExpansion { ref expansion_id, .. } if expansion_id.as_ref() == "inner"
        ))
        .expect("inner macro expansion origin recorded");

    assert!(matches!(
        inner_origin.parent.as_deref(),
        Some(SurfaceOrigin::MacroExpansion { expansion_id, .. }) if expansion_id.as_ref() == "outer"
    ));

    let function = expanded
        .module
        .definitions
        .iter()
        .find_map(|definition| match definition {
            ash_parser::surface::Definition::Function(def) => Some(def),
            _ => None,
        })
        .expect("function exists");
    let body = match &function.body {
        Expr::Block {
            tail_expr: Some(tail),
            ..
        } => tail.as_ref(),
        other => other,
    };
    let Expr::Call { args, .. } = body else {
        panic!("expected inner macro to expand to ordinary call")
    };
    assert!(matches!(&args[0], Expr::Variable { name, .. } if name.as_ref() == "n"));
}

#[test]
fn free_template_variables_are_rejected_instead_of_capturing_call_site_bindings() {
    let module = ash_parser::parse_surface_file(
        r"
macro leaky(x) => add(x, y);
fn use_macro(x: Int, y: Int) -> Int { leaky!(x) }
",
    )
    .expect("module parses");

    let err = expand_surface_module(module).expect_err("free macro template variable rejects");
    assert!(matches!(
        err,
        ash_parser::surface::ExpansionError::UnsupportedMacroTemplate { ref name, ref reason, .. }
            if name.as_ref() == "leaky" && reason.contains("free variable")
    ));
}

#[test]
fn source_names_cannot_capture_generated_operator_section_helpers_inside_macro() {
    let module = ash_parser::parse_surface_file(
        r"
infixl 6 <+> = add;
macro plus_one(x) => (<+> x);
fn use_macro(__ash_generated_section_2_lhs: Int) -> Int { plus_one!(__ash_generated_section_2_lhs) }
",
    )
    .expect("module parses");

    let expanded = expand_surface_module(module).expect("macro and notation expand");
    let mut saw_generated_helper = false;
    let mut saw_source_param = false;
    let function = expanded
        .module
        .definitions
        .iter()
        .find_map(|definition| match definition {
            ash_parser::surface::Definition::Function(def) => Some(def),
            _ => None,
        })
        .expect("function exists");
    visit_expr(&function.body, &mut |expr| {
        if let Expr::Variable { name, .. } = expr {
            saw_generated_helper |= name.as_ref().starts_with("__ash_generated_section_");
            saw_source_param |= name.as_ref() == "__ash_generated_section_2_lhs";
        }
    });
    assert!(
        saw_generated_helper,
        "operator-section expansion should create fenced helper names"
    );
    assert!(
        saw_source_param,
        "source parameter use should remain distinct and visible"
    );
}

#[test]
fn macro_template_attempting_operational_bottom_is_rejected_not_hygienized() {
    let module = ash_parser::parse_surface_file(
        r"
macro bad(x) => fail x;
fn use_macro(n: Int) -> Int { bad!(n) }
",
    )
    .expect("module parses");

    let err = expand_surface_module(module).expect_err("unsupported template rejects");
    assert!(matches!(
        err,
        ash_parser::surface::ExpansionError::UnsupportedMacroTemplate { ref name, ref reason, .. }
            if name.as_ref() == "bad" && reason.contains("fail")
    ));
}
