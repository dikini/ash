#![allow(missing_docs)]

use std::path::PathBuf;

use ash_parser::surface::{Definition, DoStmt, DoTarget, Expr, Literal};
use ash_parser::token::Span;
use ash_typeck::check_expr::elaborate_typed_do_block;
use ash_typeck::{Kind, QualifiedName, SelectedDoOperation, Type, TypeEnv, TypeVar};

fn span() -> Span {
    Span::default()
}

fn parse_std_module(relative: &str) -> ash_parser::surface::ModuleFile {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../std/src")
        .join(relative);
    let source = std::fs::read_to_string(&path)
        .unwrap_or_else(|error| panic!("read {}: {error}", path.display()));
    let source_without_imports = source
        .lines()
        .filter(|line| !line.trim_start().starts_with("use "))
        .collect::<Vec<_>>()
        .join("\n");
    ash_parser::parse_surface_file(&source_without_imports)
        .unwrap_or_else(|errors| panic!("{relative} should parse: {errors:?}"))
}

fn bind_stdlib_monad_helper_signatures(env: &mut TypeEnv) {
    let a = Type::Var(TypeVar::fresh());
    let b = Type::Var(TypeVar::fresh());
    let e = Type::Var(TypeVar::fresh());
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
                option_a,
                Type::Fn(vec![a.clone()], Box::new(option_b.clone())),
            ],
            Box::new(option_b),
        ),
    );
    env.bind_variable(
        "result::map",
        Type::Fn(
            vec![result_a.clone(), Type::Fn(vec![a.clone()], Box::new(b))],
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
            vec![result_a, Type::Fn(vec![a], Box::new(result_b.clone()))],
            Box::new(result_b),
        ),
    );
}

fn env_with_stdlib_monad_evidence() -> TypeEnv {
    let modules = [
        parse_std_module("algebra/functor.ash"),
        parse_std_module("algebra/applicative.ash"),
        parse_std_module("algebra/monad.ash"),
        parse_std_module("option.ash"),
    ];
    let mut env = TypeEnv::with_builtin_types();
    bind_stdlib_monad_helper_signatures(&mut env);

    for module in &modules {
        for definition in &module.definitions {
            if let Definition::Interface(interface) = definition {
                env.register_interface(interface)
                    .unwrap_or_else(|error| panic!("register Monad interface: {error}"));
            }
        }
    }
    for module in &modules {
        for definition in &module.definitions {
            if let Definition::Impl(implementation) = definition {
                env.register_impl(implementation)
                    .unwrap_or_else(|error| panic!("register Monad impl: {error}"));
            }
        }
    }

    env
}

fn target(name: &str) -> DoTarget {
    DoTarget {
        name: name.into(),
        args: Vec::new(),
        span: span(),
    }
}

const fn int_lit(value: i64) -> Expr {
    Expr::Literal(Literal::Int(value))
}

fn do_option_return() -> Expr {
    Expr::DoBlock {
        target: target("Option"),
        stmts: vec![DoStmt::Return {
            value: Box::new(int_lit(1)),
            span: span(),
        }],
        span: span(),
    }
}

#[test]
fn stdlib_do_evidence_engine_registers_stdlib_unit_method() {
    let env = env_with_stdlib_monad_evidence();
    let elaborated = elaborate_typed_do_block(&env, &do_option_return())
        .expect("do:Option should elaborate through stdlib Monad<Option>");
    let selected = elaborated
        .selected_evidence
        .expect("do:Option should preserve selected evidence");
    assert!(matches!(
        selected.return_op,
        SelectedDoOperation::EvidenceMethod { ref evidence_key, ref method, ref params, .. }
            if evidence_key == "Monad<Option>" && method == "unit" && params == &["value"]
    ));
    assert!(matches!(
        selected.bind_op,
        SelectedDoOperation::EvidenceMethod { ref evidence_key, ref method, ref params, .. }
            if evidence_key == "Monad<Option>" && method == "bind" && params == &["value", "f"]
    ));
}
