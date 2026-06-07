use ash_core::ast::{Expr as CoreExpr, TypeBody, TypeDef, Visibility};
use ash_parser::surface::{
    Definition, DoStmt, DoTarget, ImplDef, InterfaceDef, Literal, Type as SurfaceType,
};
use ash_parser::token::Span;
use ash_typeck::check_expr::elaborate_typed_do_block;
use ash_typeck::{Kind, QualifiedName, SelectedDoOperation, Type, TypeEnv};

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

fn register_module_evidence(env: &mut TypeEnv, source: &str) {
    let module = parse(source);
    let interface = interface_named(&module, "Monad");
    let implementation = impl_named(&module, "Monad", 0);

    env.register_interface(&interface)
        .expect("Monad interface should register");
    env.register_impl(&implementation)
        .expect("Monad implementation should register");
}

fn env_with_monad_option_methods() -> TypeEnv {
    let mut env = TypeEnv::with_builtin_types();
    register_module_evidence(
        &mut env,
        r#"
        interface Monad<M : * -> *> {
            unit(Int) -> M<Int>
            bind(M<Int>, Fn(Int) -> M<Int>) -> M<Int>
        }

        impl Monad<Option> {
            unit(value) = Some { value: value }
            bind(value, _f) = value
        }
        "#,
    );
    env.bind_variable("selected", computation_type("Option", vec![Type::Int]));
    env
}

fn env_with_monad_result_intrinsics() -> TypeEnv {
    let mut env = TypeEnv::with_builtin_types();
    env.register_type(&TypeDef {
        name: "E".to_string(),
        params: Vec::new(),
        body: TypeBody::Struct(Vec::new()),
        visibility: Visibility::Public,
        builtin: false,
    })
    .expect("ParseError fixture type should register");
    register_module_evidence(
        &mut env,
        r#"
        interface Monad<M : * -> *> {}
        impl Monad<Result<_, E>> {}
        "#,
    );
    env.bind_variable(
        "parsed",
        computation_type("Result", vec![Type::Int, computation_type("E", Vec::new())]),
    );
    env
}

fn target(name: &str, args: Vec<SurfaceType>) -> DoTarget {
    DoTarget {
        name: name.into(),
        args,
        span: Span::default(),
    }
}

fn hole() -> SurfaceType {
    SurfaceType::Hole {
        span: Span::default(),
    }
}

fn named_type(name: &str) -> SurfaceType {
    SurfaceType::Name(name.into())
}

fn var(name: &str) -> ash_parser::surface::Expr {
    ash_parser::surface::Expr::Variable {
        name: name.into(),
        span: Span::default(),
    }
}

fn int_lit(value: i64) -> ash_parser::surface::Expr {
    ash_parser::surface::Expr::Literal(Literal::Int(value))
}

fn string_lit(value: &str) -> ash_parser::surface::Expr {
    ash_parser::surface::Expr::Literal(Literal::String(value.into()))
}

fn fail_expr(payload: ash_parser::surface::Expr) -> ash_parser::surface::Expr {
    ash_parser::surface::Expr::Fail {
        payload: Box::new(payload),
        span: Span::default(),
    }
}

fn let_stmt(name: &str, value: ash_parser::surface::Expr) -> DoStmt {
    DoStmt::Let {
        name: name.into(),
        value: Box::new(value),
        span: Span::default(),
    }
}

fn bind_stmt(name: &str, value: ash_parser::surface::Expr) -> DoStmt {
    DoStmt::Bind {
        name: name.into(),
        value: Box::new(value),
        span: Span::default(),
    }
}

fn ret(value: ash_parser::surface::Expr) -> DoStmt {
    DoStmt::Return {
        value: Box::new(value),
        span: Span::default(),
    }
}

fn do_block(target: DoTarget, stmts: Vec<DoStmt>) -> ash_parser::surface::Expr {
    ash_parser::surface::Expr::DoBlock {
        target,
        stmts,
        span: Span::default(),
    }
}

fn computation_type(name: &str, args: Vec<Type>) -> Type {
    Type::Constructor {
        name: QualifiedName::root(name),
        args,
        kind: Kind::Type,
    }
}

fn assert_no_hidden_or_generic_bind(expr: &CoreExpr) {
    match expr {
        CoreExpr::Call {
            func,
            module,
            arguments,
        } => {
            assert!(
                !(func == "bind" && module.as_deref().is_none()),
                "selected evidence bind must not lower to hidden Act bind: {expr:?}"
            );
            assert!(
                !(func == "bind" && module.as_deref() == Some("Monad")),
                "selected evidence bind must not lower to generic Monad::bind: {expr:?}"
            );
            assert!(
                !(func == "bind" && module.as_deref().is_some_and(|m| m.contains('<'))),
                "rendered evidence keys are snapshots only, not dispatch modules: {expr:?}"
            );
            for argument in arguments {
                assert_no_hidden_or_generic_bind(argument);
            }
        }
        CoreExpr::FnDef { body, .. } => assert_no_hidden_or_generic_bind(body),
        CoreExpr::Let { expr, body, .. } => {
            assert_no_hidden_or_generic_bind(expr);
            assert_no_hidden_or_generic_bind(body);
        }
        CoreExpr::FnApply { func, args } => {
            assert_no_hidden_or_generic_bind(func);
            for arg in args {
                assert_no_hidden_or_generic_bind(arg);
            }
        }
        CoreExpr::Constructor { fields, .. } => {
            for (_, field) in fields {
                assert_no_hidden_or_generic_bind(field);
            }
        }
        _ => {}
    }
}

fn assert_no_selected_evidence_dispatch_module(expr: &CoreExpr) {
    match expr {
        CoreExpr::Call {
            module, arguments, ..
        } => {
            assert_ne!(
                module.as_deref(),
                Some("__ash_selected_evidence::Monad"),
                "selected method bodies must be applied directly instead of dispatched through a synthetic shared module: {expr:?}"
            );
            for argument in arguments {
                assert_no_selected_evidence_dispatch_module(argument);
            }
        }
        CoreExpr::FnDef { body, .. } => assert_no_selected_evidence_dispatch_module(body),
        CoreExpr::FnApply { func, args } => {
            assert_no_selected_evidence_dispatch_module(func);
            for arg in args {
                assert_no_selected_evidence_dispatch_module(arg);
            }
        }
        CoreExpr::Let { expr, body, .. } => {
            assert_no_selected_evidence_dispatch_module(expr);
            assert_no_selected_evidence_dispatch_module(body);
        }
        CoreExpr::Constructor { fields, .. } => {
            for (_, field) in fields {
                assert_no_selected_evidence_dispatch_module(field);
            }
        }
        _ => {}
    }
}

#[test]
fn mismatched_result_error_evidence_is_rejected() {
    let mut env = env_with_monad_result_intrinsics();
    env.register_type(&TypeDef {
        name: "OtherE".to_string(),
        params: Vec::new(),
        body: TypeBody::Struct(Vec::new()),
        visibility: Visibility::Public,
        builtin: false,
    })
    .expect("OtherE fixture type should register");
    env.bind_variable(
        "other_parsed",
        computation_type(
            "Result",
            vec![Type::Int, computation_type("OtherE", Vec::new())],
        ),
    );
    let expr = do_block(
        target("Result", vec![hole(), named_type("OtherE")]),
        vec![bind_stmt("value", var("other_parsed")), ret(var("value"))],
    );

    let err = elaborate_typed_do_block(&env, &expr)
        .expect_err("Result<_, OtherE> must not reuse Monad<Result<_, E>> evidence");
    let message = format!("{err:?}");

    assert!(message.contains("Monad<Result<_, OtherE>>"), "{message}");
    assert!(message.contains("missing"), "{message}");
}

#[test]
fn do_result_bind_lowers_through_monad_bind_evidence() {
    let env = env_with_monad_result_intrinsics();
    let expr = do_block(
        target("Result", vec![hole(), named_type("E")]),
        vec![bind_stmt("value", var("parsed")), ret(var("value"))],
    );

    let elaborated = elaborate_typed_do_block(&env, &expr).expect("do:Result<_, E> bind lowers");

    let evidence = elaborated
        .selected_evidence
        .expect("selected Result evidence should be preserved");
    assert_eq!(evidence.target, QualifiedName::root("Result"));
    assert_eq!(evidence.value_constructor, QualifiedName::root("Result"));
    assert_eq!(
        evidence.bind_op,
        SelectedDoOperation::EvidenceIntrinsic {
            evidence_key: "Monad<Result<_, E>>".to_string(),
            method: "bind".to_string(),
            shim: QualifiedName::qualified(vec!["result".to_string()], "and_then"),
        }
    );
    assert!(matches!(
        elaborated.expr,
        CoreExpr::Call { ref func, module: Some(ref module), ref arguments }
            if func == "and_then" && module == "result" && arguments.len() == 2
    ));
    assert_no_hidden_or_generic_bind(&elaborated.expr);
}

#[test]
fn do_result_fail_body_preserves_monad_evidence_and_concrete_result_type() {
    let env = env_with_monad_result_intrinsics();
    let expr = do_block(
        target("Result", vec![hole(), named_type("E")]),
        vec![
            bind_stmt("value", var("parsed")),
            let_stmt("_bottom", fail_expr(string_lit("boom"))),
            ret(var("value")),
        ],
    );

    let elaborated =
        elaborate_typed_do_block(&env, &expr).expect("do:Result<_, E> fail body lowers");

    assert_eq!(
        elaborated.ty,
        computation_type("Result", vec![Type::Int, computation_type("E", Vec::new())],)
    );
    let evidence = elaborated
        .selected_evidence
        .expect("selected Result evidence should be preserved");
    assert_eq!(
        evidence.bind_op,
        SelectedDoOperation::EvidenceIntrinsic {
            evidence_key: "Monad<Result<_, E>>".to_string(),
            method: "bind".to_string(),
            shim: QualifiedName::qualified(vec!["result".to_string()], "and_then"),
        }
    );
    assert_no_hidden_or_generic_bind(&elaborated.expr);
}

#[test]
fn user_option_do_bind_uses_selected_monad_evidence() {
    let env = env_with_monad_option_methods();
    let expr = do_block(
        target("Option", Vec::new()),
        vec![bind_stmt("value", var("selected")), ret(var("value"))],
    );

    let elaborated = elaborate_typed_do_block(&env, &expr).expect("do:Option bind lowers");

    let evidence = elaborated
        .selected_evidence
        .expect("selected Option evidence should be preserved");
    assert!(matches!(
        evidence.bind_op,
        SelectedDoOperation::EvidenceMethod {
            ref evidence_key,
            ref method,
            ref params,
            ..
        } if evidence_key == "Monad<Option>" && method == "bind" && params == &vec!["value".to_string(), "_f".to_string()]
    ));
    assert!(matches!(
        elaborated.expr,
        CoreExpr::FnApply { ref func, ref args }
            if args.len() == 2
                && matches!(func.as_ref(), CoreExpr::FnDef { params, body, .. }
                    if params.len() == 2
                        && params[0].0 == "value"
                        && params[1].0 == "_f"
                        && matches!(body.as_ref(), CoreExpr::Variable { name, .. } if name == "value"))
                && matches!(args[1], CoreExpr::FnDef { .. })
    ));
    assert_no_hidden_or_generic_bind(&elaborated.expr);
    assert_no_selected_evidence_dispatch_module(&elaborated.expr);
}

#[test]
fn generic_monad_do_specializes_before_execution() {
    let env = env_with_monad_option_methods();
    let expr = do_block(
        target("Option", Vec::new()),
        vec![bind_stmt("value", var("selected")), ret(int_lit(7))],
    );

    let elaborated = elaborate_typed_do_block(&env, &expr)
        .expect("selected Monad<Option> bind should elaborate before execution");

    assert!(matches!(
        elaborated.expr,
        CoreExpr::FnApply {
            func: ref outer_func,
            args: ref outer_args,
        } if matches!(outer_func.as_ref(), CoreExpr::FnDef { params, body, .. }
                if params.len() == 2
                    && params[0].0 == "value"
                    && params[1].0 == "_f"
                    && matches!(body.as_ref(), CoreExpr::Variable { name, .. } if name == "value"))
            && matches!(
                outer_args.as_slice(),
                [
                    CoreExpr::Variable { name, .. },
                    CoreExpr::FnDef { params, body, .. },
                ] if name == "selected"
                    && params.len() == 1
                    && params[0].0 == "value"
                    && matches!(
                        body.as_ref(),
                        CoreExpr::FnApply { func, args }
                            if matches!(func.as_ref(), CoreExpr::FnDef { params, body, .. }
                                if params.len() == 1
                                    && params[0].0 == "value"
                                    && matches!(body.as_ref(), CoreExpr::Constructor { name, .. } if name == "Some"))
                                && args.len() == 1
                    )
            )
    ));
    assert_no_hidden_or_generic_bind(&elaborated.expr);
    assert_no_selected_evidence_dispatch_module(&elaborated.expr);
}
