use ash_parser::surface::{
    BinaryOp, ExpansionError, Expr, SurfaceOrigin, expand_surface_module, visit_expr,
};

fn first_function_body(module: &ash_parser::surface::ModuleFile) -> &Expr {
    let def = module
        .definitions
        .iter()
        .find_map(|definition| match definition {
            ash_parser::surface::Definition::Function(def) => Some(def),
            _ => None,
        })
        .expect("expected function definition");
    match &def.body {
        Expr::Block {
            tail_expr: Some(tail),
            ..
        } => tail.as_ref(),
        other => other,
    }
}

#[test]
fn local_parenthesized_expression_macro_expands_to_ordinary_call() {
    let module = ash_parser::parse_surface_file(
        r"
macro inc(x) => add(x, 1);
fn use_macro(n: Int) -> Int { inc!(n) }
",
    )
    .expect("module parses");

    let expanded = expand_surface_module(module).expect("supported macro expands");
    let Expr::Call { func, args, .. } = first_function_body(&expanded.module) else {
        panic!("expected expanded ordinary call")
    };
    assert_eq!(func.as_ref(), "add");
    assert_eq!(args.len(), 2);
    assert!(matches!(&args[0], Expr::Variable { name, .. } if name.as_ref() == "n"));
    assert!(matches!(&args[1], Expr::Literal(_)));
    assert!(
        expanded.origins.iter().any(|origin| matches!(
            origin.origin,
            SurfaceOrigin::MacroExpansion { ref expansion_id, .. } if expansion_id.as_ref() == "inc"
        )),
        "macro expansion origin sidecar should be recorded"
    );
}

#[test]
fn arity_mismatch_rejects_before_expansion() {
    let module = ash_parser::parse_surface_file(
        r"
macro pair(x, y) => add(x, y);
fn use_macro(n: Int) -> Int { pair!(n) }
",
    )
    .expect("module parses");

    let err = expand_surface_module(module).expect_err("arity mismatch rejects");
    assert!(matches!(
        err,
        ExpansionError::MacroArityMismatch { ref name, expected: 2, actual: 1, .. }
            if name.as_ref() == "pair"
    ));
}

#[test]
fn unsupported_binder_template_rejects_fail_closed() {
    let module = ash_parser::parse_surface_file(
        r"
macro bindy(x) => fail x;
fn use_macro(n: Int) -> Int { bindy!(n) }
",
    )
    .expect("module parses");

    let err = expand_surface_module(module).expect_err("binder template rejects");
    assert!(matches!(
        err,
        ExpansionError::UnsupportedMacroTemplate { ref name, ref reason, .. }
            if name.as_ref() == "bindy" && reason.contains("fail")
    ));
}

#[test]
fn recursive_macro_expansion_hits_explicit_depth_limit() {
    let module = ash_parser::parse_surface_file(
        r"
macro again(x) => again!(x);
fn use_macro(n: Int) -> Int { again!(n) }
",
    )
    .expect("module parses");

    let err = expand_surface_module(module).expect_err("recursive expansion rejects");
    assert!(matches!(
        err,
        ExpansionError::MacroExpansionDepthExceeded { ref name, .. } if name.as_ref() == "again"
    ));
}

#[test]
fn macro_output_reenters_notation_expansion() {
    let module = ash_parser::parse_surface_file(
        r"
infixl 6 <+> = add;
macro plus_one(x) => (<+> x);
fn use_macro(n: Int) -> Int { plus_one!(n) }
",
    )
    .expect("module parses");

    let expanded = expand_surface_module(module).expect("macro output notation resolves");
    let mut saw_operator_section = false;
    visit_expr(first_function_body(&expanded.module), &mut |expr| {
        saw_operator_section |= matches!(expr, Expr::OperatorSection { .. });
    });
    assert!(
        !saw_operator_section,
        "macro output operator section should be elaborated"
    );
    assert!(
        expanded.origins.iter().any(|origin| matches!(
            origin.origin,
            SurfaceOrigin::NotationExpansion { ref target, .. } if target.as_ref() == "add"
        )),
        "macro output should re-enter notation expansion"
    );
}

#[test]
fn macro_substitution_reaches_binary_and_constructor_template_positions() {
    let module = ash_parser::parse_surface_file(
        r"
macro sum(x, y) => x + y;
fn use_macro(a: Int, b: Int) -> Int { sum!(a, b) }
",
    )
    .expect("module parses");

    let expanded = expand_surface_module(module).expect("binary macro template expands");
    let Expr::Binary {
        op: BinaryOp::Add,
        left,
        right,
        ..
    } = first_function_body(&expanded.module)
    else {
        panic!("expected expanded binary expression")
    };
    assert!(matches!(left.as_ref(), Expr::Variable { name, .. } if name.as_ref() == "a"));
    assert!(matches!(right.as_ref(), Expr::Variable { name, .. } if name.as_ref() == "b"));
}
