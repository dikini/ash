use ash_core::ast::Expr as CoreExpr;
use ash_parser::surface::{
    Definition, DoStmt, DoTarget, ImplDef, InterfaceDef, Literal, Type as SurfaceType,
};
use ash_parser::token::Span;
use ash_typeck::check_expr::elaborate_typed_do_block;
use ash_typeck::{QualifiedName, SelectedDoOperation, TypeEnv};

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

fn impl_named(module: &ash_parser::surface::ModuleFile, name: &str, index: usize) -> ImplDef {
    module
        .definitions
        .iter()
        .filter_map(|definition| match definition {
            Definition::Impl(implementation) if implementation.interface.as_ref() == name => {
                Some(implementation.clone())
            }
            _ => None,
        })
        .nth(index)
        .unwrap_or_else(|| panic!("impl {name} at index {index} should be present"))
}

fn monad_option_module() -> ash_parser::surface::ModuleFile {
    parse(
        r#"
        interface Monad<M : * -> *> {
            return(Int) -> M<Int>
            bind(M<Int>, Fn(Int) -> M<Int>) -> M<Int>
        }

        impl Monad<Option> {
            return(value) = Some { value: value }
            bind(value, _f) = value
        }
        "#,
    )
}

fn malformed_monad_module() -> ash_parser::surface::ModuleFile {
    parse(
        r#"
        interface Monad<M : * -> *> {
        }

        impl Monad<Option> {
        }
        "#,
    )
}

fn env_with_empty_monad_option_impl() -> TypeEnv {
    let module = malformed_monad_module();
    let interface = interface_named(&module, "Monad");
    let implementation = impl_named(&module, "Monad", 0);

    let mut env = TypeEnv::with_builtin_types();
    env.register_interface(&interface)
        .expect("empty Monad interface should register for malformed evidence regression");
    env.register_impl(&implementation)
        .expect("empty Monad<Option> impl should register against empty interface");
    env
}

fn env_with_monad_option_methods() -> TypeEnv {
    let module = monad_option_module();
    let interface = interface_named(&module, "Monad");
    let implementation = impl_named(&module, "Monad", 0);

    let mut env = TypeEnv::with_builtin_types();
    env.register_interface(&interface)
        .expect("Monad interface should register");
    env.register_impl(&implementation)
        .expect("Monad<Option> implementation should register");
    env
}

fn target(name: &str, args: Vec<SurfaceType>) -> DoTarget {
    DoTarget {
        name: name.into(),
        args,
        span: Span::default(),
    }
}

fn do_return_option_int(value: i64) -> ash_parser::surface::Expr {
    ash_parser::surface::Expr::DoBlock {
        target: target("Option", Vec::new()),
        stmts: vec![DoStmt::Return {
            value: Box::new(ash_parser::surface::Expr::Literal(Literal::Int(value))),
            span: Span::default(),
        }],
        span: Span::default(),
    }
}

#[test]
fn monad_evidence_records_return_and_bind_method_bodies() {
    let env = env_with_monad_option_methods();
    let expr = do_return_option_int(1);

    let evidence = elaborate_typed_do_block(&env, &expr)
        .expect("do:Option should resolve selected Monad<Option> evidence")
        .selected_evidence
        .expect("selected evidence should be preserved on elaboration result");

    assert_eq!(evidence.target, QualifiedName::root("Option"));
    assert_eq!(evidence.value_constructor, QualifiedName::root("Option"));
    let SelectedDoOperation::EvidenceMethod {
        evidence_key,
        method,
        params,
        body,
    } = evidence.return_op
    else {
        panic!("return op should carry the selected method body");
    };
    assert_eq!(evidence_key, "Monad<Option>");
    assert_eq!(method, "return");
    assert_eq!(params, vec!["value".to_string()]);
    assert!(
        matches!(body, CoreExpr::Constructor { ref name, .. } if name == "Some"),
        "return body should be the selected Option implementation body, got {body:?}"
    );

    let SelectedDoOperation::EvidenceMethod {
        evidence_key,
        method,
        params,
        body,
    } = evidence.bind_op
    else {
        panic!("bind op should carry the selected method body");
    };
    assert_eq!(evidence_key, "Monad<Option>");
    assert_eq!(method, "bind");
    assert_eq!(params, vec!["value".to_string(), "_f".to_string()]);
    assert!(
        matches!(body, CoreExpr::Variable { ref name, .. } if name == "value"),
        "bind body should be the selected Option implementation body, got {body:?}"
    );
}

#[test]
fn do_option_return_only_lowers_through_selected_evidence_body() {
    let env = env_with_monad_option_methods();
    let expr = do_return_option_int(1);

    let elaborated = elaborate_typed_do_block(&env, &expr).expect("return-only do:Option lowers");

    assert_eq!(
        elaborated
            .selected_evidence
            .as_ref()
            .map(|e| e.return_op.clone()),
        Some(SelectedDoOperation::EvidenceMethod {
            evidence_key: "Monad<Option>".to_string(),
            method: "return".to_string(),
            params: vec!["value".to_string()],
            body: CoreExpr::Constructor {
                name: "Some".into(),
                fields: vec![(
                    "value".into(),
                    CoreExpr::Variable {
                        name: "value".into(),
                        span: ash_core::ast::Span::default(),
                    },
                )],
            },
        })
    );
    assert!(
        matches!(
            elaborated.expr,
            CoreExpr::FnApply { ref func, ref args }
                if matches!(func.as_ref(), CoreExpr::FnDef { params, body, .. }
                    if params.len() == 1
                        && params[0].0 == "value"
                        && matches!(body.as_ref(), CoreExpr::Constructor { name, .. } if name == "Some"))
                    && args.len() == 1
        ),
        "do:Option return must apply the selected method body directly without a rendered evidence key as semantic identity: {:?}",
        elaborated.expr
    );
}

#[test]
fn ambiguous_monad_evidence_rejected_before_lowering() {
    let module = parse(
        r#"
        interface Monad<M : * -> *> {
            return(Int) -> M<Int>
            bind(M<Int>, Fn(Int) -> M<Int>) -> M<Int>
        }

        impl Monad<Option> {
            return(value) = Some { value: value }
            bind(value, _f) = value
        }

        impl Monad<Option> {
            return(value) = Some { value: value }
            bind(value, _f) = value
        }
        "#,
    );
    let interface = interface_named(&module, "Monad");
    let first = impl_named(&module, "Monad", 0);
    let duplicate = impl_named(&module, "Monad", 1);

    let mut env = TypeEnv::with_builtin_types();
    env.register_interface(&interface)
        .expect("Monad interface should register");
    env.register_impl(&first)
        .expect("first Monad<Option> evidence should register");
    let err = env
        .register_impl(&duplicate)
        .expect_err("overlapping Monad<Option> evidence must be rejected before lowering");
    let message = err.to_string();

    assert!(message.contains("Monad"), "{message}");
    assert!(message.contains("Option"), "{message}");
    assert!(
        message.contains("duplicate") || message.contains("overlap"),
        "{message}"
    );
}

#[test]
fn malformed_monad_evidence_without_return_or_bind_fails_closed() {
    let env = env_with_empty_monad_option_impl();

    let err = elaborate_typed_do_block(&env, &do_return_option_int(1))
        .expect_err("malformed Monad<Option> evidence must fail closed instead of panicking");
    let message = format!("{err:?}");

    assert!(message.contains("Monad<Option>"), "{message}");
    assert!(message.contains("return"), "{message}");
    assert!(message.contains("selected"), "{message}");
}
