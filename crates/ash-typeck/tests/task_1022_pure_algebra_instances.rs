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
    let source = strip_import_lines(&source);
    ash_parser::parse_surface_file(&source)
        .unwrap_or_else(|errors| panic!("{relative} should parse: {errors:?}"))
}

fn strip_import_lines(source: &str) -> String {
    source
        .lines()
        .filter(|line| !line.trim_start().starts_with("use "))
        .collect::<Vec<_>>()
        .join("\n")
}

fn bind_stdlib_pure_helper_signatures(env: &mut TypeEnv) {
    let a = Type::Var(TypeVar::fresh());
    let b = Type::Var(TypeVar::fresh());
    let e = Type::Var(TypeVar::fresh());
    let list_a = Type::List(Box::new(a.clone()));
    let list_b = Type::List(Box::new(b.clone()));
    let option_a = Type::Constructor {
        name: QualifiedName::root("Option"),
        args: vec![a.clone()],
        kind: Kind::Type,
    };
    let option_b = Type::Constructor {
        name: QualifiedName::root("Option"),
        args: vec![b.clone()],
        kind: Kind::Type,
    };
    let option_fn = Type::Constructor {
        name: QualifiedName::root("Option"),
        args: vec![Type::Fn(vec![a.clone()], Box::new(b.clone()))],
        kind: Kind::Type,
    };
    let result_a = Type::Constructor {
        name: QualifiedName::root("Result"),
        args: vec![a.clone(), e.clone()],
        kind: Kind::Type,
    };
    let result_b = Type::Constructor {
        name: QualifiedName::root("Result"),
        args: vec![b.clone(), e.clone()],
        kind: Kind::Type,
    };
    let result_fn = Type::Constructor {
        name: QualifiedName::root("Result"),
        args: vec![Type::Fn(vec![a.clone()], Box::new(b.clone())), e],
        kind: Kind::Type,
    };

    env.bind_variable(
        "string::concat",
        Type::Fn(vec![Type::String, Type::String], Box::new(Type::String)),
    );
    env.bind_variable(
        "list::concat",
        Type::Fn(
            vec![list_a.clone(), list_a.clone()],
            Box::new(list_a.clone()),
        ),
    );
    env.bind_variable(
        "list::map",
        Type::Fn(
            vec![
                list_a.clone(),
                Type::Fn(vec![a.clone()], Box::new(b.clone())),
            ],
            Box::new(list_b),
        ),
    );
    env.bind_variable(
        "option::map",
        Type::Fn(
            vec![
                option_a.clone(),
                Type::Fn(vec![a.clone()], Box::new(b.clone())),
            ],
            Box::new(option_b.clone()),
        ),
    );
    env.bind_variable(
        "option::pure",
        Type::Fn(vec![a.clone()], Box::new(option_a.clone())),
    );
    env.bind_variable(
        "option::apply",
        Type::Fn(
            vec![option_fn, option_a.clone()],
            Box::new(option_b.clone()),
        ),
    );
    env.bind_variable(
        "option::and_then",
        Type::Fn(
            vec![
                option_a.clone(),
                Type::Fn(vec![a.clone()], Box::new(option_b.clone())),
            ],
            Box::new(option_b),
        ),
    );
    env.bind_variable(
        "result::map",
        Type::Fn(
            vec![
                result_a.clone(),
                Type::Fn(vec![a.clone()], Box::new(b.clone())),
            ],
            Box::new(result_b.clone()),
        ),
    );
    env.bind_variable(
        "result::pure",
        Type::Fn(vec![a.clone()], Box::new(result_a.clone())),
    );
    env.bind_variable(
        "result::apply",
        Type::Fn(
            vec![result_fn, result_a.clone()],
            Box::new(result_b.clone()),
        ),
    );
    env.bind_variable(
        "result::and_then",
        Type::Fn(
            vec![
                result_a.clone(),
                Type::Fn(vec![a], Box::new(result_b.clone())),
            ],
            Box::new(result_b),
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
        parse_std_module("option.ash"),
        parse_std_module("result.ash"),
        parse_std_module("list.ash"),
        parse_std_module("string.ash"),
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

fn interface_from_source(source: &str) -> ash_parser::surface::InterfaceDef {
    let module = ash_parser::parse_surface_file(source)
        .unwrap_or_else(|errors| panic!("interface source should parse: {source}\n{errors:?}"));
    module
        .definitions
        .into_iter()
        .find_map(|definition| match definition {
            Definition::Interface(interface) => Some(interface),
            _ => None,
        })
        .unwrap_or_else(|| panic!("interface source should contain an interface: {source}"))
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
fn pure_algebra_instances_typeck_uses_generic_method_payload_surface() {
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

#[test]
fn pure_algebra_instances_monomorphize_generic_functor_method_payloads_at_call_site() {
    let env = env_with_task1022_stdlib_algebra();
    let option_string = Type::Constructor {
        name: QualifiedName::root("Option"),
        args: vec![Type::String],
        kind: Kind::Type,
    };
    let string_to_bool = Type::Fn(vec![Type::String], Box::new(Type::Bool));

    let option_return_type = env
        .resolve_interface_method_call("Functor", "map", &[option_string, string_to_bool])
        .expect("Functor<Option>::map should monomorphize method payload vars from call args");

    assert_eq!(
        option_return_type,
        Type::Constructor {
            name: QualifiedName::root("Option"),
            args: vec![Type::Bool],
            kind: Kind::Type,
        }
    );

    let result_string_error = Type::Constructor {
        name: QualifiedName::root("Result"),
        args: vec![Type::String, Type::Int],
        kind: Kind::Type,
    };
    let string_to_bool = Type::Fn(vec![Type::String], Box::new(Type::Bool));
    let result_return_type = env
        .resolve_interface_method_call("Functor", "map", &[result_string_error, string_to_bool])
        .expect("Functor<Result<_, E>>::map should monomorphize value payload without losing E");

    assert_eq!(
        result_return_type,
        Type::Constructor {
            name: QualifiedName::root("Result"),
            args: vec![Type::Bool, Type::Int],
            kind: Kind::Type,
        }
    );
}

#[test]
fn generic_interface_impl_method_body_must_not_specialize_payload_vars() {
    let mut env = TypeEnv::with_builtin_types();
    let interface = interface_from_source(
        r#"
            interface Functor<F : * -> *> {
                map(F<A>, (A) -> B) -> F<B>
            }
            "#,
    );
    env.register_interface(&interface)
        .expect("generic Functor interface should register");
    env.bind_variable(
        "option::pure",
        Type::Fn(
            vec![Type::Int],
            Box::new(Type::Constructor {
                name: QualifiedName::root("Option"),
                args: vec![Type::Int],
                kind: Kind::Type,
            }),
        ),
    );
    let implementation = impl_from_source(
        r#"
            impl Functor<Option> {
                map(value, f) = option::pure(1)
            }
            "#,
    );

    let err = env
        .register_impl(&implementation)
        .expect_err("impl body that specializes generic payload vars must be rejected");
    let message = err.to_string();
    assert!(
        message.contains("must keep method payload type variable")
            && message.contains("constrains"),
        "expected generic payload specialization diagnostic, got: {message}"
    );
}

#[test]
fn generic_interface_impl_method_body_must_not_collapse_payload_vars() {
    let mut env = TypeEnv::with_builtin_types();
    let interface = interface_from_source(
        r#"
            interface Functor<F : * -> *> {
                map(F<A>, (A) -> B) -> F<B>
            }
            "#,
    );
    env.register_interface(&interface)
        .expect("generic Functor interface should register");
    let implementation = impl_from_source(
        r#"
            impl Functor<Option> {
                map(value, f) = value
            }
            "#,
    );

    let err = env
        .register_impl(&implementation)
        .expect_err("impl body that collapses A and B must be rejected");
    let message = err.to_string();
    assert!(
        message.contains("must keep method payload type variable")
            && message.contains("constrains"),
        "expected independent payload diagnostic, got: {message}"
    );
}
