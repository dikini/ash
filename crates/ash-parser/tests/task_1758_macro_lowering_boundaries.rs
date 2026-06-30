use ash_parser::lower::{expand_and_lower_surface_module, lower_expanded_surface_module};
use ash_parser::surface::{
    ExpandedSurfaceModule, Expr, MacroDelimiter, MacroInvocation, MacroInvocationBody,
};
use ash_parser::token::Span;

#[test]
fn high_level_lowering_expands_supported_macro_before_core_boundary() {
    let module = ash_parser::parse_surface_file(
        r"
macro inc(x) => add(x, 1);
fn add(x: Int, y: Int) -> Int { x + y }
fn use_macro(n: Int) -> Int { inc!(n) }
",
    )
    .expect("module parses");

    expand_and_lower_surface_module(module).expect("supported macro expands before lowering");
}

#[test]
fn high_level_lowering_rejects_missing_macro_before_core_boundary() {
    let module = ash_parser::parse_surface_file(
        r"
fn use_macro(n: Int) -> Int { inc!(n) }
",
    )
    .expect("module parses");

    let err =
        expand_and_lower_surface_module(module).expect_err("unknown macro rejects before lowering");
    assert!(
        err.to_string()
            .contains("unknown local macro invocation `inc!`"),
        "unexpected error: {err}"
    );
}

#[test]
fn direct_expanded_gate_rejects_raw_macro_carriers_if_caller_constructs_invalid_boundary() {
    let mut module = ash_parser::parse_surface_file("fn bad(n: Int) -> Int { n }")
        .expect("baseline module parses");
    let ash_parser::surface::Definition::Function(def) = &mut module.definitions[0] else {
        panic!("expected function")
    };
    def.body = Expr::MacroInvocation {
        invocation: MacroInvocation {
            name: "inc".into(),
            delimiter: MacroDelimiter::Paren,
            raw_body: "n".into(),
            body: MacroInvocationBody::ExprArgs(Vec::new()),
            token_trees: Vec::new(),
            args: Some(Vec::new()),
            span: Span::new(0, 1, 1, 1),
        },
    };
    let raw = ExpandedSurfaceModule {
        module,
        diagnostics: Vec::new(),
        origins: Vec::new(),
    };

    let err = lower_expanded_surface_module(&raw)
        .expect_err("raw macro carrier at expanded boundary is rejected");
    assert!(
        err.to_string()
            .contains("unexpanded macro invocation carrier `inc!` reached lowering"),
        "unexpected error: {err}"
    );
}
