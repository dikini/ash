//! TASK-1749 cross-boundary validation for Phase 171 hygiene/origin/scope behavior.

use ash_engine::Engine;
use ash_engine::module_loader::load_ordinary_file;
use ash_parser::surface::{Expr, SurfaceOrigin, expand_surface_module};

fn write_pair(
    provider_source: &str,
    caller_source: &str,
) -> (tempfile::TempDir, std::path::PathBuf) {
    let dir = tempfile::tempdir().expect("tempdir");
    let provider = dir.path().join("provider.ash");
    let caller = dir.path().join("caller.ash");
    std::fs::write(&provider, provider_source).expect("write provider");
    std::fs::write(&caller, caller_source).expect("write caller");
    (dir, caller)
}

fn write_module(source: &str) -> (tempfile::TempDir, std::path::PathBuf) {
    let dir = tempfile::tempdir().expect("tempdir");
    let path = dir.path().join("module.ash");
    std::fs::write(&path, source).expect("write module");
    (dir, path)
}

#[test]
fn parser_origin_and_generated_hygiene_match_engine_acceptance_for_local_notation() {
    let source = r"
infixl 6 <+> = combine;

pub fn combine(x: Int, y: Int) -> Int {
    x + y
}

pub fn local_section() {
    (<+>)
}
";
    let parsed = ash_parser::parse_surface_file(source).expect("module parses");
    let expanded = expand_surface_module(parsed).expect("local notation expands");
    assert_eq!(expanded.origins.len(), 1);
    let origin = &expanded.origins[0];
    assert_eq!(origin.expansion_id.0, 0);
    assert!(matches!(
        &origin.origin,
        SurfaceOrigin::NotationExpansion { target, .. } if target.as_ref() == "combine"
    ));

    let ash_parser::surface::Definition::Function(def) = &expanded.module.definitions[2] else {
        panic!("expected local_section function")
    };
    let Expr::Block {
        tail_expr: Some(body),
        ..
    } = &def.body
    else {
        panic!("expected block body")
    };
    let Expr::FnDef { params, .. } = body.as_ref() else {
        panic!("expected eta-expanded section")
    };
    assert!(
        params
            .iter()
            .all(|(name, _)| name.starts_with("$ash_generated_section_0_"))
    );

    let (_dir, path) = write_module(source);
    Engine::new()
        .build()
        .expect("engine builds")
        .check_module_file(&path)
        .expect("high-level module validation accepts expanded local notation");
}

#[tokio::test]
async fn imported_notation_stays_inactive_while_callable_import_remains_usable() {
    let provider = r"
pub infixl 6 <+> = combine;

pub fn combine(x: Int, y: Int) -> Int {
    x + y
}
";

    let (_dir, callable_caller) = write_pair(
        provider,
        r"
use provider::{combine}
fn main() { combine(1, 2) }
",
    );
    load_ordinary_file(&callable_caller).expect("ordinary callable import remains usable");

    let (_dir, notation_caller) = write_pair(
        provider,
        r"
use provider::{combine}
fn main() { (<+>) }
",
    );
    let err = Engine::new()
        .build()
        .expect("engine builds")
        .run_file(&notation_caller)
        .await
        .expect_err("imported notation must remain inactive");
    assert!(
        err.to_string().contains("<+>")
            && (err.to_string().contains("operator section")
                || err.to_string().contains("unsupported feature")),
        "unexpected error: {err}"
    );
}

#[test]
fn macro_invocation_is_rejected_by_engine_and_typechecker_boundaries() {
    let source = r"
pub fn use_macro() -> Int {
    make_id!(1)
}
";
    let (_dir, path) = write_module(source);
    let err = Engine::new()
        .build()
        .expect("engine builds")
        .check_module_file(&path)
        .expect_err("engine rejects macro invocation before acceptance");
    assert!(
        err.to_string()
            .contains("unknown local macro invocation `make_id!`")
    );

    let parsed = ash_parser::parse_surface_file(source).expect("macro carrier parses");
    let ash_parser::surface::Definition::Function(def) = &parsed.definitions[0] else {
        panic!("expected function")
    };
    let result = ash_typeck::check_expr::check_expr(&ash_typeck::TypeEnv::new(), &def.body);
    assert!(
        result.errors.iter().any(|error| error
            .to_string()
            .contains("unexpanded macro invocation carrier `make_id!` reached type checking")),
        "typechecker diagnostics should reject macro invocation: {:?}",
        result.errors
    );
}
