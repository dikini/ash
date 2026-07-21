//! TASK-1032 typechecker evidence tests for `std::algebra::Comonad`.

use std::collections::BTreeSet;
use std::path::PathBuf;

use ash_core::Kind;
use ash_parser::surface::{Definition, InterfaceDef, Type as SurfaceType};
use ash_parser::token::Span;
use ash_typeck::TypeEnv;

fn std_src_path(relative: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../std/src")
        .join(relative)
}

fn parse_std_module(relative: &str) -> ash_parser::surface::ModuleFile {
    let path = std_src_path(relative);
    let source = std::fs::read_to_string(&path)
        .unwrap_or_else(|error| panic!("read {}: {error}", path.display()));
    ash_parser::parse_surface_file(&source)
        .unwrap_or_else(|errors| panic!("{relative} should parse: {errors:?}"))
}

fn parse_interface(relative: &str, name: &str) -> InterfaceDef {
    parse_std_module(relative)
        .definitions
        .into_iter()
        .find_map(|definition| match definition {
            Definition::Interface(interface) if interface.name.as_ref() == name => Some(interface),
            _ => None,
        })
        .unwrap_or_else(|| panic!("{relative} should define interface {name}"))
}

fn hole() -> SurfaceType {
    SurfaceType::Hole {
        span: Span::default(),
    }
}

fn constructor(name: &str, args: Vec<SurfaceType>) -> SurfaceType {
    SurfaceType::Constructor {
        name: name.into(),
        args,
    }
}

#[test]
fn comonad_interface_registers_with_expected_methods_and_kind() {
    let interface = parse_interface("algebra/comonad.ash", "Comonad");
    let mut env = TypeEnv::with_builtin_types();

    env.register_interface(&interface)
        .unwrap_or_else(|error| panic!("Comonad should register: {error}"));

    let registered = env
        .lookup_interface("Comonad")
        .expect("Comonad should be registered");
    assert_eq!(
        registered.type_param_kinds,
        vec![Kind::n_ary(1)],
        "Comonad should bind its carrier parameter at kind * -> *"
    );

    let methods = registered.methods.keys().cloned().collect::<BTreeSet<_>>();
    assert_eq!(
        methods,
        BTreeSet::from(["extract".to_string(), "extend".to_string()]),
        "Comonad should expose the TASK-1031 method surface"
    );
    assert_eq!(registered.methods["extract"].params.len(), 1);
    assert_eq!(registered.methods["extend"].params.len(), 2);
}

#[test]
fn comonad_negative_instances_remain_absent_for_partial_and_opaque_carriers() {
    let interface = parse_interface("algebra/comonad.ash", "Comonad");
    let mut env = TypeEnv::with_builtin_types();
    env.register_interface(&interface)
        .unwrap_or_else(|error| panic!("Comonad should register: {error}"));

    let forbidden = [
        SurfaceType::Name("Option".into()),
        constructor("Result", vec![hole(), SurfaceType::Name("String".into())]),
        SurfaceType::Name("List".into()),
        SurfaceType::Name("Act".into()),
        SurfaceType::Name("Proc".into()),
    ];

    for carrier in forbidden {
        assert!(
            env.resolve_interface_evidence("Comonad", std::slice::from_ref(&carrier))
                .is_err(),
            "Comonad evidence must remain absent for forbidden carrier {carrier:?}"
        );
    }
}
