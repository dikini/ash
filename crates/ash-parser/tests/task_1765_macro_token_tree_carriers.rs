use ash_parser::surface::{
    Definition, Expr, MacroDelimiter, MacroTokenTree, expand_surface_module,
};

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

fn macro_invocation_body(source: &str) -> ash_parser::surface::MacroInvocation {
    let module = ash_parser::parse_surface_file(source).expect("source parses");
    let Expr::MacroInvocation { invocation } = first_function_body(&module) else {
        panic!("expected macro invocation")
    };
    invocation.clone()
}

#[test]
fn bracket_macro_invocation_preserves_nested_token_tree_delimiters_and_spellings() {
    let invocation =
        macro_invocation_body("fn use_macro(n: Int) -> Int { inc![n (add one) {x [y]}] }");

    assert_eq!(invocation.delimiter, MacroDelimiter::Bracket);
    assert_eq!(invocation.raw_body.as_ref(), "n (add one) {x [y]}");
    assert!(
        invocation.args.is_none(),
        "bracket token trees are not executable expression args"
    );
    assert_eq!(invocation.token_trees.len(), 3);

    let MacroTokenTree::Token { spelling, span } = &invocation.token_trees[0] else {
        panic!("expected leading token")
    };
    assert_eq!(spelling.as_ref(), "n");
    assert!(span.start < span.end);

    let MacroTokenTree::Group {
        delimiter,
        tokens,
        span,
    } = &invocation.token_trees[1]
    else {
        panic!("expected nested parenthesized group")
    };
    assert_eq!(*delimiter, MacroDelimiter::Paren);
    assert_eq!(tokens.len(), 2);
    assert!(span.start < span.end);
    assert!(
        matches!(&tokens[0], MacroTokenTree::Token { spelling, .. } if spelling.as_ref() == "add")
    );
    assert!(
        matches!(&tokens[1], MacroTokenTree::Token { spelling, .. } if spelling.as_ref() == "one")
    );

    let MacroTokenTree::Group {
        delimiter,
        tokens,
        span,
    } = &invocation.token_trees[2]
    else {
        panic!("expected nested braced group")
    };
    assert_eq!(*delimiter, MacroDelimiter::Brace);
    assert_eq!(tokens.len(), 2);
    assert!(span.start < span.end);
    assert!(
        matches!(&tokens[0], MacroTokenTree::Token { spelling, .. } if spelling.as_ref() == "x")
    );
    let MacroTokenTree::Group {
        delimiter, tokens, ..
    } = &tokens[1]
    else {
        panic!("expected nested bracket group")
    };
    assert_eq!(*delimiter, MacroDelimiter::Bracket);
    assert_eq!(tokens.len(), 1);
    assert!(
        matches!(&tokens[0], MacroTokenTree::Token { spelling, .. } if spelling.as_ref() == "y")
    );
}

#[test]
fn parenthesized_macro_invocation_preserves_token_trees_alongside_structured_args() {
    let invocation = macro_invocation_body("fn use_macro(n: Int) -> Int { inc!(n) }");

    assert_eq!(invocation.delimiter, MacroDelimiter::Paren);
    assert!(
        invocation.args.is_some(),
        "parenthesized MVP subset keeps structured args"
    );
    assert_eq!(invocation.token_trees.len(), 1);
    assert!(
        matches!(&invocation.token_trees[0], MacroTokenTree::Token { spelling, .. } if spelling.as_ref() == "n")
    );
}

#[test]
fn token_tree_carriers_require_the_expansion_boundary_before_lowering() {
    let module = ash_parser::parse_surface_file(
        "macro inc(x) => add(x, 1); fn use_macro(n: Int) -> Int { inc![n] }",
    )
    .expect("source parses");

    let expanded = expand_surface_module(module).expect("token-tree input reparses and expands");
    assert!(
        expanded
            .module
            .definitions
            .iter()
            .any(|definition| matches!(definition, Definition::Function(_)))
    );
}
