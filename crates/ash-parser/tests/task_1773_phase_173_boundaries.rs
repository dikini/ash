//! TASK-1773 closeout regressions for Phase 173 parser-side macro boundaries.

use ash_parser::surface::{
    Definition, Expr, Type, collect_public_macro_summaries, expand_surface_module,
};

fn first_function_body(module: &ash_parser::surface::ModuleFile) -> &Expr {
    let def = module
        .definitions
        .iter()
        .find_map(|definition| match definition {
            Definition::Function(def) => Some(def),
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

fn assert_name_type(ty: &Type, expected: &str) {
    assert!(
        matches!(ty, Type::Name(name) if name.as_ref() == expected),
        "expected {expected}, got {ty:?}"
    );
}

#[test]
fn inferred_macro_summary_remains_syntax_phase_while_expansion_removes_invocation_carrier() {
    let module = ash_parser::parse_surface_file(
        r"
pub macro inc(x: Int) => x + 1;
fn use_macro(n: Int) -> Int { inc!(n) }
",
    )
    .expect("module parses");

    let summaries = collect_public_macro_summaries(&module, "provider").expect("summaries collect");
    let signature = summaries
        .iter()
        .find(|summary| summary.name.as_ref() == "inc")
        .and_then(|summary| summary.typed_signature.as_ref())
        .expect("public macro summary carries inferred syntax-phase signature");
    assert_name_type(
        signature.param_types[0]
            .as_ref()
            .expect("source param annotation preserved"),
        "Int",
    );
    assert_name_type(
        &signature.return_type.clone().expect("return inferred"),
        "Int",
    );

    let expanded = expand_surface_module(module).expect("supported macro expands");
    let Expr::Binary { .. } = first_function_body(&expanded.module) else {
        panic!("expected macro invocation to expand before lowerable boundary")
    };
}

#[test]
fn ambiguous_macro_summary_does_not_fabricate_typed_metadata_at_closeout_boundary() {
    let module = ash_parser::parse_surface_file("pub macro id(x) => x;").expect("module parses");
    let summaries = collect_public_macro_summaries(&module, "provider").expect("summaries collect");
    let summary = summaries
        .iter()
        .find(|summary| summary.name.as_ref() == "id")
        .expect("macro summary exists");

    assert!(
        summary.typed_signature.is_none(),
        "unannotated identity macros must remain untyped rather than fabricating a polymorphic summary"
    );
}
