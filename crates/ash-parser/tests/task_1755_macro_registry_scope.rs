use ash_parser::surface::{ExpansionError, Expr, build_local_macro_table, expand_surface_module};

#[test]
fn local_macro_registry_contains_only_local_declarations() {
    let module = ash_parser::parse_surface_file(
        r"
macro inc(x) => add(x, 1);

mod inner {
    macro hidden(x) => add(x, 2);
}
",
    )
    .expect("module parses");

    let table = build_local_macro_table(&module).expect("local macro table builds");
    assert!(table.resolve("inc").is_some());
    assert!(
        table.resolve("hidden").is_none(),
        "inline-module macro must not be visible in parent module table"
    );
}

#[test]
fn duplicate_local_macro_names_reject_before_expansion() {
    let module = ash_parser::parse_surface_file(
        r"
macro inc(x) => add(x, 1);
macro inc(y) => add(y, 2);
",
    )
    .expect("module parses");

    let err = build_local_macro_table(&module).expect_err("duplicate local macro names reject");
    assert!(matches!(
        err,
        ExpansionError::DuplicateMacroDeclaration { ref name, .. } if name.as_ref() == "inc"
    ));
}

#[test]
fn missing_macro_invocation_rejects_explicitly() {
    let module = ash_parser::parse_surface_file("fn use_macro(n: Int) -> Int { inc!(n) }")
        .expect("module parses");

    let err = expand_surface_module(module).expect_err("missing macro remains fail-closed");
    assert!(matches!(
        err,
        ExpansionError::UnknownMacroInvocation { ref name, .. } if name.as_ref() == "inc"
    ));
}

#[test]
fn local_macro_invocation_is_resolved_and_expanded_after_task_1756() {
    let module = ash_parser::parse_surface_file(
        r"
macro inc(x) => add(x, 1);
fn use_macro(n: Int) -> Int { inc!(n) }
",
    )
    .expect("module parses");

    let expanded = expand_surface_module(module).expect("local macro invocation expands");
    let ash_parser::surface::Definition::Function(def) = &expanded.module.definitions[1] else {
        panic!("expected function")
    };
    let Expr::Block {
        tail_expr: Some(tail),
        ..
    } = &def.body
    else {
        panic!("expected function block")
    };
    assert!(matches!(tail.as_ref(), Expr::Call { func, .. } if func.as_ref() == "add"));
}

#[test]
fn bracket_and_brace_macro_invocations_reject_as_unsupported_mvp_forms() {
    for source in [
        "macro inc(x) => add(x, 1); fn use_macro(n: Int) -> Int { inc![n] }",
        "macro inc(x) => add(x, 1); fn use_macro(n: Int) -> Int { inc!{n} }",
    ] {
        let module = ash_parser::parse_surface_file(source).expect("module parses");
        let err = expand_surface_module(module).expect_err("unsupported macro form rejects");
        assert!(matches!(
            err,
            ExpansionError::UnsupportedMacroInvocation { ref name, .. } if name.as_ref() == "inc"
        ));
    }
}

#[test]
fn macro_declaration_without_invocation_may_cross_expanded_surface_boundary() {
    let module = ash_parser::parse_surface_file(
        r"
pub macro inc(x) => add(x, 1);

pub fn add(x: Int, y: Int) -> Int {
    x + y
}
",
    )
    .expect("module parses");
    let expanded =
        expand_surface_module(module).expect("macro declaration alone may cross boundary");

    assert!(
        expanded
            .module
            .definitions
            .iter()
            .any(|definition| matches!(definition, ash_parser::surface::Definition::Macro(_))),
        "macro declaration carrier remains syntax metadata, not callable export"
    );
}

#[test]
fn structured_args_are_required_for_supported_parenthesized_invocations() {
    let module = ash_parser::parse_surface_file(
        r"
macro inc(x) => add(x, 1);
fn use_macro(n: Int) -> Int { inc!(n,) }
",
    )
    .expect("diagnostic carrier parses even when structured args are unavailable");

    let Expr::MacroInvocation { invocation } = ({
        let ash_parser::surface::Definition::Function(def) = &module.definitions[1] else {
            panic!("expected function")
        };
        match &def.body {
            Expr::Block {
                tail_expr: Some(tail),
                ..
            } => tail.as_ref(),
            other => other,
        }
    }) else {
        panic!("expected macro invocation")
    };
    assert!(invocation.args.is_none());

    let err = expand_surface_module(module).expect_err("unstructured args reject before execution");
    assert!(matches!(
        err,
        ExpansionError::UnsupportedMacroInvocation { ref name, .. } if name.as_ref() == "inc"
    ));
}
