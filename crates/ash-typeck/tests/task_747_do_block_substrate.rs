//! TASK-747/TASK-749 regression tests for generalized do-block parser/typechecker boundaries.

use ash_parser::input::new_input;
use ash_parser::parse_expr::expr;
use ash_parser::surface::{Definition, Expr, ImplDef, InterfaceDef};
use ash_typeck::check_expr::check_expr;
use ash_typeck::type_env::TypeEnv;
use ash_typeck::{Kind, QualifiedName, Type};
use winnow::prelude::*;

fn parse_expr_source(source: &str) -> Expr {
    let mut input = new_input(source);
    let parsed = expr.parse_next(&mut input).expect("expression parses");
    assert!(
        input.input.is_empty(),
        "parser left trailing input: {:?}",
        input.input
    );
    parsed
}

fn parse_module(source: &str) -> ash_parser::surface::ModuleFile {
    ash_parser::parse_surface_file(source)
        .unwrap_or_else(|errors| panic!("module file should parse: {source}\nerrors: {errors:?}"))
}

fn interface_named(module: &ash_parser::surface::ModuleFile, name: &str) -> InterfaceDef {
    module
        .definitions
        .iter()
        .find_map(|definition| match definition {
            Definition::Interface(interface) if interface.name.as_ref() == name => {
                Some(interface.clone())
            }
            _ => None,
        })
        .unwrap_or_else(|| panic!("interface {name} should be present"))
}

fn impl_named(module: &ash_parser::surface::ModuleFile, name: &str) -> ImplDef {
    module
        .definitions
        .iter()
        .find_map(|definition| match definition {
            Definition::Impl(implementation) if implementation.interface.as_ref() == name => {
                Some(implementation.clone())
            }
            _ => None,
        })
        .unwrap_or_else(|| panic!("impl {name} should be present"))
}

fn env_with_monad_option_evidence() -> TypeEnv {
    let module = parse_module(
        r#"
        interface Monad<M : * -> *> {
            unit(Int) -> M<Int>
            bind(M<Int>, (Int) -> M<Int>) -> M<Int>
        }
        impl Monad<Option> {
            unit(value) = Some { value: value }
            bind(value, f) = value
        }
        "#,
    );
    let interface = interface_named(&module, "Monad");
    let implementation = impl_named(&module, "Monad");

    let mut env = TypeEnv::with_builtin_types();
    env.register_interface(&interface)
        .expect("Monad<M : * -> *> should register");
    env.register_impl(&implementation)
        .expect("impl Monad<Option> should register explicit evidence");
    env
}

#[test]
fn do_block_typecheck_is_supported_after_typed_elaboration() {
    let expr = parse_expr_source("do:Option { return 1 }");
    let result = check_expr(&env_with_monad_option_evidence(), &expr);

    assert!(
        result.is_ok(),
        "expected typed do-block support, got {result:?}"
    );
    assert_eq!(
        result.ty,
        Type::Constructor {
            name: QualifiedName::root("Option"),
            args: vec![Type::Int],
            kind: Kind::Type,
        }
    );
}
