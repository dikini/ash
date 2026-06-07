//! TASK-1041: interface evidence constraints survive core lowering and summary transport.

use ash_core::ast::TypeExpr;
use ash_engine::module_loader::load_ordinary_file;
use ash_parser::surface::Definition;

fn write_file(path: &std::path::Path, source: &str) {
    std::fs::write(path, source).expect("write ash source");
}

const fn provider_with_constrained_interface() -> &'static str {
    r"
pub interface Applicative<F> {}
pub interface Monad<M> where M: Applicative {}
"
}

fn first_interface(source: &str, name: &str) -> ash_parser::surface::InterfaceDef {
    ash_parser::parse_surface_file(source)
        .unwrap_or_else(|errors| panic!("module file should parse: {source}\nerrors: {errors:?}"))
        .definitions
        .into_iter()
        .find_map(|definition| match definition {
            Definition::Interface(interface) if interface.name.as_ref() == name => Some(interface),
            _ => None,
        })
        .unwrap_or_else(|| panic!("interface {name} should be parsed"))
}

#[test]
fn core_lowering_preserves_interface_owned_evidence_constraints() {
    let interface = first_interface(provider_with_constrained_interface(), "Monad");

    let lowered = ash_parser::lower::lower_interface_def(&interface)
        .expect("interface evidence constraints should lower to core metadata");

    assert_eq!(lowered.evidence_constraints.len(), 1);
    let constraint = &lowered.evidence_constraints[0];
    assert_eq!(constraint.subject, TypeExpr::Named("M".to_string()));
    assert_eq!(
        constraint.required_evidence,
        TypeExpr::Named("Applicative".to_string())
    );
}

#[test]
fn named_interface_import_transports_required_evidence_constraints() {
    let dir = tempfile::tempdir().expect("tempdir");
    let provider = dir.path().join("provider.ash");
    let caller = dir.path().join("caller.ash");
    write_file(&provider, provider_with_constrained_interface());
    write_file(
        &caller,
        r"use provider::{Monad}
workflow main { ret 0 }
",
    );

    let loaded = load_ordinary_file(&caller).expect("named interface import should load provider");
    let monad = loaded
        .imported_semantic_summaries
        .iter()
        .flat_map(|summary| summary.interface_identities.iter())
        .find(|identity| identity.name.as_str() == "Monad")
        .expect("named import should transport Monad interface identity");

    assert_eq!(monad.evidence_constraints.len(), 1);
    let constraint = &monad.evidence_constraints[0];
    assert_eq!(constraint.subject, TypeExpr::Named("M".to_string()));
    assert_eq!(
        constraint.required_evidence,
        TypeExpr::Named("Applicative".to_string())
    );
}

#[test]
fn glob_interface_import_keeps_interface_constraints_distinct_from_impl_where_constraints() {
    let dir = tempfile::tempdir().expect("tempdir");
    let provider = dir.path().join("provider.ash");
    let caller = dir.path().join("caller.ash");
    write_file(
        &provider,
        r"
pub interface Applicative<F> {}
pub interface Monad<M> where M: Applicative {}
impl<T> Monad<T> where T: Applicative {}
",
    );
    write_file(
        &caller,
        r"use provider::*
workflow main { ret 0 }
",
    );

    let loaded = load_ordinary_file(&caller).expect("glob interface import should load provider");
    let constraints = loaded
        .imported_semantic_summaries
        .iter()
        .flat_map(|summary| summary.interface_identities.iter())
        .filter(|identity| identity.name.as_str() == "Monad")
        .flat_map(|identity| identity.evidence_constraints.iter())
        .collect::<Vec<_>>();

    assert_eq!(
        constraints.len(),
        1,
        "summary transport must carry only interface-owned evidence constraints: {constraints:?}"
    );
    assert_eq!(constraints[0].subject, TypeExpr::Named("M".to_string()));
}
