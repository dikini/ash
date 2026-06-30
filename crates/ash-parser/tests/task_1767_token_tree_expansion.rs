use ash_parser::surface::{Definition, ExpansionError, Expr, expand_surface_module};

fn function_body(module: &ash_parser::surface::ExpandedSurfaceModule, index: usize) -> &Expr {
    let Definition::Function(def) = &module.module.definitions[index] else {
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
fn bracket_token_tree_invocation_reparses_once_and_expands() {
    let module = ash_parser::parse_surface_file(
        r#"
macro inc(x) => add(x, 1);
fn add(x: Int, y: Int) -> Int { x + y }
fn use_macro(n: Int) -> Int { inc![n] }
"#,
    )
    .expect("module parses");

    let expanded = expand_surface_module(module).expect("bracket token-tree macro expands");
    let Expr::Call { func, args, .. } = function_body(&expanded, 2) else {
        panic!("expected expanded call body")
    };
    assert_eq!(func.as_ref(), "add");
    assert_eq!(args.len(), 2);
}

#[test]
fn brace_token_tree_invocation_reparses_once_and_expands() {
    let module = ash_parser::parse_surface_file(
        r#"
macro wrap(x) => Some(x);
fn use_macro(n: Int) -> Option<Int> { wrap!{n} }
"#,
    )
    .expect("module parses");

    let expanded = expand_surface_module(module).expect("brace token-tree macro expands");
    let Expr::Constructor { name, .. } = function_body(&expanded, 1) else {
        panic!("expected expanded constructor body")
    };
    assert_eq!(name.as_ref(), "Some");
}

#[test]
fn invalid_token_tree_reparse_is_macro_diagnostic() {
    let module = ash_parser::parse_surface_file(
        r#"
macro id(x) => x;
fn use_macro(n: Int) -> Int { id![n +] }
"#,
    )
    .expect("module parses");

    let err = expand_surface_module(module).expect_err("invalid token tree output rejects");
    assert!(
        matches!(err, ExpansionError::MacroTokenTreeReparseFailed { ref name, .. } if name.as_ref() == "id"),
        "unexpected error: {err:?}"
    );
    assert!(
        err.to_string()
            .contains("token-tree input failed to reparse")
    );
}

#[test]
fn reparsed_token_tree_output_cannot_bypass_residual_macro_validation() {
    let module = ash_parser::parse_surface_file(
        r#"
macro id(x) => x;
fn use_macro(n: Int) -> Int { id![missing!(n)] }
"#,
    )
    .expect("module parses");

    let err = expand_surface_module(module).expect_err("residual macro in reparsed output rejects");
    assert!(
        matches!(err, ExpansionError::UnknownMacroInvocation { ref name, .. } if name.as_ref() == "missing"),
        "unexpected error: {err:?}"
    );
}
