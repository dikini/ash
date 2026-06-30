use ash_parser::surface::{Definition, Expr, MacroDelimiter, Visibility};

fn first_function_body(module: &ash_parser::surface::ModuleFile) -> &Expr {
    let Definition::Function(def) = &module.definitions[0] else {
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
fn macro_declaration_parses_with_params_body_visibility_and_span() {
    let module = ash_parser::parse_surface_file("pub macro inc(x) => add(x, 1);")
        .expect("macro declaration parses");

    let Definition::Macro(def) = &module.definitions[0] else {
        panic!("expected macro definition")
    };

    assert_eq!(def.visibility, Visibility::Public);
    assert_eq!(def.name.as_ref(), "inc");
    assert_eq!(def.params, vec!["x".into()]);
    assert!(def.span.start < def.span.end);
    assert!(matches!(def.body, Expr::Call { ref func, .. } if func.as_ref() == "add"));
}

#[test]
fn parenthesized_macro_invocation_preserves_structured_arguments() {
    let module = ash_parser::parse_surface_file("fn use_macro(n: Int) -> Int { inc!(n) }")
        .expect("macro invocation parses");

    let Expr::MacroInvocation { invocation } = first_function_body(&module) else {
        panic!("expected macro invocation")
    };

    assert_eq!(invocation.name.as_ref(), "inc");
    assert_eq!(invocation.delimiter, MacroDelimiter::Paren);
    assert_eq!(invocation.raw_body.as_ref(), "n");
    let args = invocation
        .args
        .as_ref()
        .expect("parenthesized invocation has structured args");
    assert_eq!(args.len(), 1);
    assert!(matches!(&args[0], Expr::Variable { name, .. } if name.as_ref() == "n"));
}

#[test]
fn bracket_and_brace_invocations_remain_non_executable_diagnostic_carriers() {
    let bracket = ash_parser::parse_surface_file("fn use_macro(n: Int) -> Int { inc![n] }")
        .expect("bracket invocation parses as diagnostic carrier");
    let Expr::MacroInvocation { invocation } = first_function_body(&bracket) else {
        panic!("expected bracket macro invocation")
    };
    assert_eq!(invocation.delimiter, MacroDelimiter::Bracket);
    assert!(
        invocation.args.is_none(),
        "bracket invocation must not get executable structured args"
    );

    let brace = ash_parser::parse_surface_file("fn use_macro(n: Int) -> Int { inc!{n} }")
        .expect("brace invocation parses as diagnostic carrier");
    let Expr::MacroInvocation { invocation } = first_function_body(&brace) else {
        panic!("expected brace macro invocation")
    };
    assert_eq!(invocation.delimiter, MacroDelimiter::Brace);
    assert!(
        invocation.args.is_none(),
        "brace invocation must not get executable structured args"
    );
}

#[test]
fn qualified_macro_like_invocation_still_rejects() {
    let err = ash_parser::parse_surface_file("fn use_macro(n: Int) -> Int { macros::inc!(n) }")
        .expect_err("qualified macro paths are not part of the MVP carrier");

    assert!(!err.is_empty(), "expected at least one parse error");
}
