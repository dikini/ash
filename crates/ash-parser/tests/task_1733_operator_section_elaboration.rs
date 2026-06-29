use ash_parser::surface::{Expr, Spanned, SurfaceOrigin, expand_surface_module};

fn first_function_body(module: &ash_parser::surface::ModuleFile) -> &Expr {
    let ash_parser::surface::Definition::Function(def) = &module.definitions[0] else {
        panic!("expected function definition")
    };
    unwrap_block_tail(&def.body)
}

fn unwrap_block_tail(expr: &Expr) -> &Expr {
    match expr {
        Expr::Block {
            tail_expr: Some(tail),
            ..
        } => tail.as_ref(),
        other => other,
    }
}

#[test]
fn built_in_bare_section_elaborates_to_eta_expanded_function() {
    let module =
        ash_parser::parse_surface_file("fn op() -> Int { (+) }").expect("bare section parses");
    let expanded = expand_surface_module(module).expect("built-in section elaborates");
    match first_function_body(&expanded.module) {
        Expr::FnDef { params, body, .. } => {
            assert_eq!(params.len(), 2);
            assert!(
                matches!(body.as_ref(), Expr::Binary { raw_operator: Some(raw), .. } if raw.spelling.as_ref() == "+")
            );
        }
        other => panic!("expected eta-expanded function, got {other:?}"),
    }
}

#[test]
fn built_in_section_records_operator_section_origin_sidecar() {
    let module =
        ash_parser::parse_surface_file("fn op() -> Int { (+) }").expect("bare section parses");
    let expanded = expand_surface_module(module).expect("built-in section elaborates");
    assert_eq!(expanded.origins.len(), 1);
    let origin = &expanded.origins[0];
    assert_eq!(
        origin.generated_span,
        first_function_body(&expanded.module).span()
    );
    assert!(matches!(
        &origin.origin,
        SurfaceOrigin::OperatorSection {
            section_span,
            operator_span
        } if section_span == &origin.generated_span && operator_span.start < operator_span.end
    ));
}

#[test]
fn built_in_left_and_right_sections_elaborate_to_unary_functions() {
    let left_module =
        ash_parser::parse_surface_file("fn inc() -> Int { (1 +) }").expect("left section parses");
    let left_expanded = expand_surface_module(left_module).expect("left section elaborates");
    assert!(
        matches!(first_function_body(&left_expanded.module), Expr::FnDef { params, .. } if params.len() == 1)
    );

    let right_module = ash_parser::parse_surface_file("fn add_to_one() -> Int { (+ 1) }")
        .expect("right section parses");
    let right_expanded = expand_surface_module(right_module).expect("right section elaborates");
    assert!(
        matches!(first_function_body(&right_expanded.module), Expr::FnDef { params, .. } if params.len() == 1)
    );
}

#[test]
fn local_notation_section_elaborates_to_declared_callable_target() {
    let module = ash_parser::parse_surface_file(
        r#"
        infixl 6 <+> = combine
        fn section(x: Int) -> Int { (x <+>) }
        "#,
    )
    .expect("local notation section parses");
    let expanded = expand_surface_module(module).expect("local notation section elaborates");
    let ash_parser::surface::Definition::Function(def) = &expanded.module.definitions[1] else {
        panic!("expected function definition")
    };
    match unwrap_block_tail(&def.body) {
        Expr::FnDef { body, .. } => {
            assert!(matches!(body.as_ref(), Expr::Call { func, .. } if func.as_ref() == "combine"));
        }
        other => panic!("expected local notation eta function, got {other:?}"),
    }
}

#[test]
fn local_notation_section_records_notation_expansion_origin_sidecar() {
    let module = ash_parser::parse_surface_file(
        r#"
        infixl 6 <+> = combine
        fn section(x: Int) -> Int { (x <+>) }
        "#,
    )
    .expect("local notation section parses");
    let expanded = expand_surface_module(module).expect("local notation section elaborates");
    let body = {
        let ash_parser::surface::Definition::Function(def) = &expanded.module.definitions[1] else {
            panic!("expected function definition")
        };
        unwrap_block_tail(&def.body)
    };
    assert_eq!(expanded.origins.len(), 1);
    let origin = &expanded.origins[0];
    assert_eq!(origin.generated_span, body.span());
    assert!(matches!(
        &origin.origin,
        SurfaceOrigin::NotationExpansion {
            notation_span,
            target
        } if target.as_ref() == "combine" && notation_span.start < notation_span.end
    ));
}

#[test]
fn unresolved_operator_section_still_fails_closed() {
    let module = ash_parser::parse_surface_file("fn bad(x: Int) -> Int { (x <??>) }")
        .expect("unknown symbolic section parses");
    let err = expand_surface_module(module).expect_err("unknown operator remains fail closed");
    assert!(err.to_string().contains("operator section `<??>`"));
}

#[test]
fn bare_local_notation_section_preserves_qualified_callable_target() {
    let module = ash_parser::parse_surface_file(
        r#"
        infixl 6 <+> = Math::combine
        fn section() -> Int { (<+>) }
        "#,
    )
    .expect("qualified local notation section parses");
    let expanded = expand_surface_module(module).expect("local notation section elaborates");
    let ash_parser::surface::Definition::Function(def) = &expanded.module.definitions[1] else {
        panic!("expected function definition")
    };
    match unwrap_block_tail(&def.body) {
        Expr::FnDef { body, params, .. } => {
            assert_eq!(params.len(), 2);
            assert!(matches!(
                body.as_ref(),
                Expr::Call { func, module: Some(module), .. }
                    if func.as_ref() == "combine" && module.as_ref() == "Math"
            ));
        }
        other => panic!("expected local notation eta function, got {other:?}"),
    }
}

#[test]
fn explicit_section_operands_named_like_generated_binders_do_not_change_arity() {
    let left_module = ash_parser::parse_surface_file(
        "fn collide(__section_lhs: Int) -> Int { (__section_lhs +) }",
    )
    .expect("left section parses");
    let left_expanded = expand_surface_module(left_module).expect("left section elaborates");
    assert!(
        matches!(first_function_body(&left_expanded.module), Expr::FnDef { params, .. } if params.len() == 1)
    );

    let right_module = ash_parser::parse_surface_file(
        "fn collide(__section_rhs: Int) -> Int { (+ __section_rhs) }",
    )
    .expect("right section parses");
    let right_expanded = expand_surface_module(right_module).expect("right section elaborates");
    assert!(
        matches!(first_function_body(&right_expanded.module), Expr::FnDef { params, .. } if params.len() == 1)
    );
}
