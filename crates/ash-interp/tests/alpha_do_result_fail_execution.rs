use ash_core::runtime::{FailureBoundary, FailureEntity};
use ash_core::{TypeBody, TypeDef, Value, Visibility};
use ash_interp::Context;
use ash_interp::error::EvalError;
use ash_interp::eval::eval_expr;
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

fn env_with_monad_result_intrinsics() -> TypeEnv {
    let mut env = TypeEnv::with_builtin_types();
    env.register_type(&TypeDef {
        name: "E".to_string(),
        params: Vec::new(),
        body: TypeBody::Struct(Vec::new()),
        visibility: Visibility::Public,
        builtin: false,
    })
    .expect("E fixture type should register");
    let module = parse(
        r#"
        interface Monad<M : * -> *> {}
        impl Monad<Result<_, E>> {}
        "#,
    );
    env.register_interface(&interface_named(&module, "Monad"))
        .expect("Monad interface should register");
    env.register_impl(&impl_named(&module, "Monad"))
        .expect("Result Monad implementation should register");
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

fn computation_type(name: &str, args: Vec<Type>) -> Type {
    Type::Constructor {
        name: QualifiedName::root(name),
        args,
        kind: Kind::Type,
    }
}

fn var(name: &str) -> ash_parser::surface::Expr {
    ash_parser::surface::Expr::Variable {
        name: name.into(),
        span: Span::default(),
    }
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

fn err_value(message: &str) -> Value {
    Value::variant("Err", vec![("error", Value::String(message.to_string()))])
}

fn ok_value(value: Value) -> Value {
    Value::variant("Ok", vec![("value", value)])
}

fn bind_stmt(name: &str, value: ash_parser::surface::Expr) -> DoStmt {
    DoStmt::Bind {
        name: name.into(),
        value: Box::new(value),
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

fn ret(value: ash_parser::surface::Expr) -> DoStmt {
    DoStmt::Return {
        value: Box::new(value),
        span: Span::default(),
    }
}

fn do_result(stmts: Vec<DoStmt>) -> ash_parser::surface::Expr {
    ash_parser::surface::Expr::DoBlock {
        target: target("Result", vec![hole(), named_type("E")]),
        stmts,
        span: Span::default(),
    }
}

#[test]
fn do_result_fail_executes_as_operational_bottom_not_domain_err() {
    let env = env_with_monad_result_intrinsics();
    let expr = do_result(vec![
        bind_stmt("value", var("parsed")),
        let_stmt("_bottom", fail_expr(string_lit("boom"))),
        ret(var("value")),
    ]);

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

    let mut ctx = Context::new();
    ctx.set("parsed".to_string(), ok_value(Value::Int(7)));
    let err = eval_expr(&elaborated.expr, &ctx)
        .expect_err("fail inside do:Result must remain operational bottom");
    let EvalError::OperationalFailure(failure) = err else {
        panic!("expected operational failure, got {err:?}");
    };

    assert_eq!(failure.payload, Value::String("boom".to_string()));
    assert_eq!(failure.boundary, FailureBoundary::Pure);
    assert!(matches!(failure.entity, FailureEntity::LexicalFrame(_)));
    assert_ne!(failure.payload, err_value("boom"));
}

#[test]
fn do_result_bind_return_success_still_returns_ok_value() {
    let env = env_with_monad_result_intrinsics();
    let expr = do_result(vec![bind_stmt("value", var("parsed")), ret(var("value"))]);
    let elaborated =
        elaborate_typed_do_block(&env, &expr).expect("do:Result<_, E> success body lowers");

    let mut ctx = Context::new();
    ctx.set("parsed".to_string(), ok_value(Value::Int(7)));
    let value =
        eval_expr(&elaborated.expr, &ctx).expect("successful do:Result bind/return should execute");

    assert_eq!(value, ok_value(Value::Int(7)));
}
