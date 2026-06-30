use ash_parser::surface::{
    Definition, Expr, MacroDelimiter, MacroInvocation, MacroInvocationBody, MacroTokenTree,
    expand_surface_module,
};

fn macro_invocation_body(source: &str) -> MacroInvocation {
    let module = ash_parser::parse_surface_file(source).expect("module parses");
    let Definition::Function(def) = &module.definitions[0] else {
        panic!("expected function definition")
    };
    let body = match &def.body {
        Expr::Block {
            tail_expr: Some(tail),
            ..
        } => tail.as_ref(),
        other => other,
    };
    let Expr::MacroInvocation { invocation } = body else {
        panic!("expected macro invocation")
    };
    invocation.clone()
}

fn assert_token(tree: &MacroTokenTree, expected: &str) {
    let MacroTokenTree::Token { spelling, span } = tree else {
        panic!("expected token tree token")
    };
    assert_eq!(spelling.as_ref(), expected);
    assert!(span.start < span.end, "token span should be non-empty");
}

#[test]
fn bracket_invocation_uses_structured_token_tree_body() {
    let invocation =
        macro_invocation_body("fn use_macro(n: Int) -> Int { inc![n, (add one), {x [y]}] }");

    assert_eq!(invocation.delimiter, MacroDelimiter::Bracket);
    assert!(
        invocation.args.is_none(),
        "bracket invocation must not become expression args"
    );

    let MacroInvocationBody::TokenTrees(trees) = &invocation.body else {
        panic!("bracket body should use token-tree carrier")
    };
    assert_eq!(trees, &invocation.token_trees);
    assert!(trees.len() >= 4);
    assert_token(&trees[0], "n,");

    let MacroTokenTree::Group {
        delimiter,
        tokens,
        span,
    } = trees
        .iter()
        .find(|tree| {
            matches!(
                tree,
                MacroTokenTree::Group {
                    delimiter: MacroDelimiter::Paren,
                    ..
                }
            )
        })
        .expect("expected parenthesized nested token-tree group")
    else {
        panic!("expected parenthesized nested token-tree group")
    };
    assert_eq!(*delimiter, MacroDelimiter::Paren);
    assert!(span.start < span.end);
    assert_token(&tokens[0], "add");
    assert_token(&tokens[1], "one");

    let MacroTokenTree::Group {
        delimiter,
        tokens,
        span,
    } = trees
        .iter()
        .find(|tree| {
            matches!(
                tree,
                MacroTokenTree::Group {
                    delimiter: MacroDelimiter::Brace,
                    ..
                }
            )
        })
        .expect("expected braced nested token-tree group")
    else {
        panic!("expected braced nested token-tree group")
    };
    assert_eq!(*delimiter, MacroDelimiter::Brace);
    assert!(span.start < span.end);
    assert_token(&tokens[0], "x");
}

#[test]
fn brace_invocation_uses_structured_token_tree_body_and_preserves_nested_groups() {
    let invocation =
        macro_invocation_body("fn use_macro(n: Int) -> Int { inc!{let x = [n]; {x}} }");

    assert_eq!(invocation.delimiter, MacroDelimiter::Brace);
    assert!(
        invocation.args.is_none(),
        "brace invocation must not become expression args"
    );

    let MacroInvocationBody::TokenTrees(trees) = &invocation.body else {
        panic!("brace body should use token-tree carrier")
    };
    assert_eq!(trees, &invocation.token_trees);
    assert!(trees.len() >= 5);
    assert_token(&trees[0], "let");
    assert_token(&trees[1], "x");
    assert_token(&trees[2], "=");

    let MacroTokenTree::Group {
        delimiter,
        tokens,
        span,
    } = trees
        .iter()
        .find(|tree| {
            matches!(
                tree,
                MacroTokenTree::Group {
                    delimiter: MacroDelimiter::Bracket,
                    ..
                }
            )
        })
        .expect("expected nested bracket group")
    else {
        panic!("expected nested bracket group")
    };
    assert_eq!(*delimiter, MacroDelimiter::Bracket);
    assert!(span.start < span.end);
    assert_token(&tokens[0], "n");

    let MacroTokenTree::Group {
        delimiter,
        tokens,
        span,
    } = trees
        .iter()
        .find(|tree| {
            matches!(
                tree,
                MacroTokenTree::Group {
                    delimiter: MacroDelimiter::Brace,
                    ..
                }
            )
        })
        .expect("expected nested brace group")
    else {
        panic!("expected nested brace group")
    };
    assert_eq!(*delimiter, MacroDelimiter::Brace);
    assert!(span.start < span.end);
    assert_token(&tokens[0], "x");
}

#[test]
fn parenthesized_invocation_still_uses_structured_expression_args() {
    let invocation = macro_invocation_body("fn use_macro(n: Int) -> Int { inc!(n) }");

    let MacroInvocationBody::ExprArgs(args) = &invocation.body else {
        panic!("parenthesized MVP invocation should use expression args")
    };
    assert_eq!(args.len(), 1);
    assert_eq!(invocation.args.as_ref().expect("args preserved").len(), 1);
    assert_eq!(invocation.token_trees.len(), 1);
}

#[test]
fn bracket_and_brace_structured_carriers_execute_only_after_task_1767_reparse_boundary() {
    for source in [
        "macro inc(x) => add(x, 1); fn add(x: Int, y: Int) -> Int { x + y } fn use_macro(n: Int) -> Int { inc![n] }",
        "macro inc(x) => add(x, 1); fn add(x: Int, y: Int) -> Int { x + y } fn use_macro(n: Int) -> Int { inc!{n} }",
    ] {
        let module = ash_parser::parse_surface_file(source).expect("module parses");
        expand_surface_module(module)
            .expect("structured token-tree carriers expand through TASK-1767 boundary");
    }
}
