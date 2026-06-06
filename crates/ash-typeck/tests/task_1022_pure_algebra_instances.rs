//! TASK-1022 RED tests for pure stdlib algebra instances.

use std::path::PathBuf;

use ash_parser::surface::{Definition, Type as SurfaceType};
use ash_parser::token::Span;
use ash_typeck::{Kind, QualifiedName, Type, TypeEnv, TypeVar};

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

fn bind_stdlib_pure_helper_signatures(env: &mut TypeEnv) {
    let a = Type::Var(TypeVar::fresh());
    let b = Type::Var(TypeVar::fresh());
    let e = Type::Var(TypeVar::fresh());
    let list_a = Type::List(Box::new(a.clone()));
    let list_b = Type::List(Box::new(b.clone()));
    let option_int = Type::Constructor {
        name: QualifiedName::root("Option"),
        args: vec![Type::Int],
        kind: Kind::Type,
    };
    let option_fn = Type::Constructor {
        name: QualifiedName::root("Option"),
        args: vec![Type::Fn(vec![Type::Int], Box::new(Type::Int))],
        kind: Kind::Type,
    };
    let string_error_result_int = Type::Constructor {
        name: QualifiedName::root("Result"),
        args: vec![Type::Int, e.clone()],
        kind: Kind::Type,
    };
    let string_error_result_fn = Type::Constructor {
        name: QualifiedName::root("Result"),
        args: vec![Type::Fn(vec![Type::Int], Box::new(Type::Int)), e],
        kind: Kind::Type,
    };

    env.bind_variable(
        "string::concat",
        Type::Fn(vec![Type::String, Type::String], Box::new(Type::String)),
    );
    env.bind_variable(
        "list::concat",
        Type::Fn(vec![list_a.clone(), list_a.clone()], Box::new(list_a)),
    );
    env.bind_variable(
        "list::map",
        Type::Fn(
            vec![list_b, Type::Fn(vec![b], Box::new(a.clone()))],
            Box::new(Type::List(Box::new(a))),
        ),
    );
    env.bind_variable(
        "option::map",
        Type::Fn(
            vec![
                option_int.clone(),
                Type::Fn(vec![Type::Int], Box::new(Type::Int)),
            ],
            Box::new(option_int.clone()),
        ),
    );
    env.bind_variable(
        "option::pure",
        Type::Fn(vec![Type::Int], Box::new(option_int.clone())),
    );
    env.bind_variable(
        "option::apply",
        Type::Fn(
            vec![option_fn, option_int.clone()],
            Box::new(option_int.clone()),
        ),
    );
    env.bind_variable(
        "option::and_then",
        Type::Fn(
            vec![
                option_int.clone(),
                Type::Fn(vec![Type::Int], Box::new(option_int.clone())),
            ],
            Box::new(option_int),
        ),
    );
    env.bind_variable(
        "result::map",
        Type::Fn(
            vec![
                string_error_result_int.clone(),
                Type::Fn(vec![Type::Int], Box::new(Type::Int)),
            ],
            Box::new(string_error_result_int.clone()),
        ),
    );
    env.bind_variable(
        "result::pure",
        Type::Fn(vec![Type::Int], Box::new(string_error_result_int.clone())),
    );
    env.bind_variable(
        "result::apply",
        Type::Fn(
            vec![string_error_result_fn, string_error_result_int.clone()],
            Box::new(string_error_result_int.clone()),
        ),
    );
    env.bind_variable(
        "result::and_then",
        Type::Fn(
            vec![
                string_error_result_int.clone(),
                Type::Fn(vec![Type::Int], Box::new(string_error_result_int.clone())),
            ],
            Box::new(string_error_result_int),
        ),
    );
}

fn env_with_task1022_stdlib_algebra() -> TypeEnv {
    let modules = [
        parse_std_module("algebra/semigroup.ash"),
        parse_std_module("algebra/monoid.ash"),
        parse_std_module("algebra/functor.ash"),
        parse_std_module("algebra/applicative.ash"),
        parse_std_module("algebra/monad.ash"),
    ];
    let mut env = TypeEnv::with_builtin_types();
    bind_stdlib_pure_helper_signatures(&mut env);

    for module in &modules {
        for definition in &module.definitions {
            if let Definition::Interface(interface) = definition {
                env.register_interface(interface)
                    .unwrap_or_else(|error| panic!("register interface: {error}"));
            }
        }
    }
    for module in &modules {
        for definition in &module.definitions {
            if let Definition::Impl(implementation) = definition {
                env.register_impl(implementation)
                    .unwrap_or_else(|error| panic!("register impl: {error}"));
            }
        }
    }

    env
}

fn impl_from_source(source: &str) -> ash_parser::surface::ImplDef {
    let module = ash_parser::parse_surface_file(source)
        .unwrap_or_else(|errors| panic!("impl source should parse: {source}\n{errors:?}"));
    module
        .definitions
        .into_iter()
        .find_map(|definition| match definition {
            Definition::Impl(implementation) => Some(implementation),
            _ => None,
        })
        .unwrap_or_else(|| panic!("impl source should contain an impl: {source}"))
}

fn constructor(name: &str, args: Vec<SurfaceType>) -> SurfaceType {
    SurfaceType::Constructor {
        name: name.into(),
        args,
    }
}

fn hole() -> SurfaceType {
    SurfaceType::Hole {
        span: Span::default(),
    }
}

#[test]
fn pure_algebra_instances_typeck_resolves_option_result_and_list_functor_evidence() {
    let env = env_with_task1022_stdlib_algebra();

    for (interface, target) in [
        ("Functor", SurfaceType::Name("Option".into())),
        ("Applicative", SurfaceType::Name("Option".into())),
        ("Monad", SurfaceType::Name("Option".into())),
        (
            "Functor",
            constructor("Result", vec![hole(), SurfaceType::Name("String".into())]),
        ),
        (
            "Applicative",
            constructor("Result", vec![hole(), SurfaceType::Name("String".into())]),
        ),
        (
            "Monad",
            constructor("Result", vec![hole(), SurfaceType::Name("String".into())]),
        ),
        ("Functor", SurfaceType::Name("List".into())),
    ] {
        let evidence = env
            .resolve_interface_evidence(interface, std::slice::from_ref(&target))
            .unwrap_or_else(|error| {
                panic!("expected {interface} evidence for {target:?}: {error}")
            });
        assert_eq!(evidence.interface, interface);
        assert!(
            !evidence.methods.is_empty(),
            "expected source method bodies for {interface} evidence over {target:?}"
        );
    }
}

#[test]
fn pure_algebra_instances_typeck_resolves_string_and_list_semigroup_monoid_evidence() {
    let env = env_with_task1022_stdlib_algebra();

    for (interface, target) in [
        ("Semigroup", SurfaceType::Name("String".into())),
        ("Monoid", SurfaceType::Name("String".into())),
        (
            "Semigroup",
            constructor("List", vec![SurfaceType::Name("Int".into())]),
        ),
        (
            "Monoid",
            constructor("List", vec![SurfaceType::Name("Int".into())]),
        ),
    ] {
        let evidence = env
            .resolve_interface_evidence(interface, std::slice::from_ref(&target))
            .unwrap_or_else(|error| {
                panic!("expected {interface} evidence for {target:?}: {error}")
            });
        assert_eq!(evidence.interface, interface);
        assert!(
            !evidence.methods.is_empty(),
            "expected source method bodies for {interface} evidence over {target:?}"
        );
    }
}

#[test]
fn pure_algebra_instances_typeck_does_not_overclaim_list_applicative_or_monad() {
    let env = env_with_task1022_stdlib_algebra();

    for interface in ["Applicative", "Monad"] {
        assert!(
            env.resolve_interface_evidence(interface, &[SurfaceType::Name("List".into())])
                .is_err(),
            "TASK-1022 should not register {interface}<List> until honest list semantics/helpers exist"
        );
    }
}

#[test]
fn pure_algebra_instances_typeck_ambiguous_result_evidence_fails_closed() {
    let mut env = env_with_task1022_stdlib_algebra();
    let overlapping = impl_from_source(
        r#"
        pub impl Functor<Result<_, String>> {
            map(value, f) = result::map(value, f)
        }
        "#,
    );

    let err = env
        .register_impl(&overlapping)
        .expect_err("ambiguous generic and specific Result Functor evidence must fail closed");
    let message = err.to_string();

    assert!(
        message.contains("overlapping") && message.contains("Functor"),
        "expected overlapping evidence diagnostic, got: {message}"
    );
}

#[test]
fn pure_algebra_instances_typeck_uses_task1021_monomorphic_method_surface() {
    let env = env_with_task1022_stdlib_algebra();
    let evidence = env
        .resolve_interface_evidence("Monad", &[SurfaceType::Name("Option".into())])
        .expect("expected Monad<Option> evidence");

    assert_eq!(
        evidence.head,
        Type::Constructor {
            name: QualifiedName::root("Monad"),
            args: vec![Type::Constructor {
                name: QualifiedName::root("Option"),
                args: vec![],
                kind: Kind::n_ary(1),
            }],
            kind: Kind::Type,
        }
    );
    assert!(
        evidence
            .methods
            .iter()
            .any(|method| method.name == "unit" && method.params.len() == 1),
        "Monad<Option> should provide the canonical unit method"
    );
    assert!(
        evidence
            .methods
            .iter()
            .any(|method| method.name == "bind" && method.params.len() == 2),
        "Monad<Option> should provide the canonical bind method"
    );
}
