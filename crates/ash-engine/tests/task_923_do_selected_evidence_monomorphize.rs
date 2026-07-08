#![allow(missing_docs)]

use ash_core::Value;
use ash_core::ast::{Expr as CoreExpr, Workflow};
use ash_interp::{Context, eval_expr};
use ash_parser::surface::{Definition, DoStmt, DoTarget, ImplDef, InterfaceDef};
use ash_typeck::check_expr::elaborate_typed_do_block;
use ash_typeck::{Kind, QualifiedName, Type, TypeEnv};
use std::collections::HashMap;

fn parse(source: &str) -> ash_parser::surface::ModuleFile {
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

fn env_with_executable_option_monad() -> TypeEnv {
    let module = parse(
        r"
        interface Monad<M : * -> *> {
            unit(Int) -> M<Int>
            bind(M<Int>, (Int) -> M<Int>) -> M<Int>
        }

        impl Monad<Option> {
            unit(value) = Some { value: value }
            bind(_value, _f) = _f(1)
        }
        ",
    );
    let interface = interface_named(&module, "Monad");
    let implementation = impl_named(&module, "Monad");

    let mut env = TypeEnv::with_builtin_types();
    env.register_interface(&interface)
        .expect("Monad interface should register");
    env.register_impl(&implementation)
        .expect("Monad<Option> implementation should register");
    env.bind_variable(
        "selected",
        Type::Constructor {
            name: QualifiedName::root("Option"),
            args: vec![Type::Int],
            kind: Kind::Type,
        },
    );
    env
}

fn target(name: &str) -> DoTarget {
    DoTarget {
        name: name.into(),
        args: Vec::new(),
        span: ash_parser::token::Span::default(),
    }
}

fn var(name: &str) -> ash_parser::surface::Expr {
    ash_parser::surface::Expr::Variable {
        name: name.into(),
        span: ash_parser::token::Span::default(),
    }
}

fn do_option_bind_return() -> ash_parser::surface::Expr {
    ash_parser::surface::Expr::DoBlock {
        target: target("Option"),
        stmts: vec![
            DoStmt::Bind {
                name: "value".into(),
                value: Box::new(var("selected")),
                span: ash_parser::token::Span::default(),
            },
            DoStmt::Return {
                value: Box::new(var("value")),
                span: ash_parser::token::Span::default(),
            },
        ],
        span: ash_parser::token::Span::default(),
    }
}

fn some(value: Value) -> Value {
    Value::Variant {
        name: "Some".to_string(),
        fields: Box::new(vec![("value".to_string(), value)]),
    }
}

fn assert_no_selected_evidence_dispatch(expr: &CoreExpr) {
    match expr {
        CoreExpr::Call {
            module, arguments, ..
        } => {
            assert_ne!(
                module.as_deref(),
                Some("__ash_selected_evidence::Monad"),
                "selected evidence must not survive as a synthetic dispatch call: {expr:?}"
            );
            for argument in arguments {
                assert_no_selected_evidence_dispatch(argument);
            }
        }
        CoreExpr::FnApply { func, args } => {
            assert_no_selected_evidence_dispatch(func);
            for arg in args {
                assert_no_selected_evidence_dispatch(arg);
            }
        }
        CoreExpr::FnDef { body, .. } => assert_no_selected_evidence_dispatch(body),
        CoreExpr::Let { expr, body, .. } => {
            assert_no_selected_evidence_dispatch(expr);
            assert_no_selected_evidence_dispatch(body);
        }
        CoreExpr::Constructor { fields, .. } => {
            for (_, field) in fields {
                assert_no_selected_evidence_dispatch(field);
            }
        }
        _ => {}
    }
}

#[test]
fn task_923_selected_bind_survives_engine_monomorphize_and_executes() {
    let env = env_with_executable_option_monad();
    let elaborated = elaborate_typed_do_block(&env, &do_option_bind_return())
        .expect("selected Monad<Option> do bind should elaborate");
    let mut workflow = Workflow::Ret {
        expr: elaborated.expr,
    };

    ash_engine::monomorphize::monomorphize_workflow(&mut workflow, &env)
        .expect("selected evidence closure lowering should survive monomorphization");

    let Workflow::Ret { expr } = workflow else {
        panic!("monomorphization should preserve Ret workflow shape");
    };
    assert!(matches!(
        expr,
        CoreExpr::FnApply { ref func, ref args }
            if matches!(func.as_ref(), CoreExpr::FnDef { params, body, .. }
                if params.iter().map(|(name, _)| name.as_str()).eq(["_value", "_f"])
                    && matches!(body.as_ref(), CoreExpr::FnApply { .. }))
                && args.len() == 2
    ));
    assert_no_selected_evidence_dispatch(&expr);

    let mut bindings = HashMap::new();
    bindings.insert("selected".to_string(), some(Value::Int(7)));
    let value = eval_expr(&expr, &Context::with_bindings(bindings))
        .expect("monomorphized selected bind closure should execute");

    assert_eq!(value, some(Value::Int(1)));
}
