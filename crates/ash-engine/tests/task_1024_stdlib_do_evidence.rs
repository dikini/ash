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
    ash_parser::parse_surface_file(&source)
        .unwrap_or_else(|errors| panic!("{relative} should parse: {errors:?}"))
}

fn option_int() -> Type {
    Type::Constructor {
        name: QualifiedName::root("Option"),
        args: vec![Type::Int],
        kind: Kind::Type,
    }
}

fn result_int() -> Type {
    Type::Constructor {
        name: QualifiedName::root("Result"),
        args: vec![Type::Int, Type::Var(TypeVar::fresh())],
        kind: Kind::Type,
    }
}

fn bind_stdlib_monad_helper_signatures(env: &mut TypeEnv) {
    let option_int = option_int();
    let result_int = result_int();
    let option_fn = Type::Constructor {
        name: QualifiedName::root("Option"),
        args: vec![Type::Fn(vec![Type::Int], Box::new(Type::Int))],
        kind: Kind::Type,
    };
    let result_fn = Type::Constructor {
        name: QualifiedName::root("Result"),
        args: vec![
            Type::Fn(vec![Type::Int], Box::new(Type::Int)),
            Type::Var(TypeVar::fresh()),
        ],
        kind: Kind::Type,
    };
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
                result_int.clone(),
                Type::Fn(vec![Type::Int], Box::new(Type::Int)),
            ],
            Box::new(result_int.clone()),
        ),
    );
    env.bind_variable(
        "result::pure",
        Type::Fn(vec![Type::Int], Box::new(result_int.clone())),
    );
    env.bind_variable(
        "result::apply",
        Type::Fn(
            vec![result_fn, result_int.clone()],
            Box::new(result_int.clone()),
        ),
    );
    env.bind_variable(
        "result::and_then",
        Type::Fn(
            vec![
                result_int.clone(),
                Type::Fn(vec![Type::Int], Box::new(result_int.clone())),
            ],
            Box::new(result_int),
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
