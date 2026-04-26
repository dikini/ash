use ash_parser::surface::{ActStmt, Expr as SurfaceExpr, Literal};
use ash_parser::token::Span;
use ash_typeck::check_expr::check_expr;
use ash_typeck::type_env::TypeEnv;
use ash_typeck::types::Type;
use ash_typeck::{Kind, QualifiedName};

fn span() -> Span {
    Span::default()
}

fn act_of(inner: Type) -> Type {
    Type::Constructor {
        name: QualifiedName::root("Act"),
        args: vec![inner],
        kind: Kind::Type,
    }
}

fn proc_of(inner: Type) -> Type {
    Type::Constructor {
        name: QualifiedName::root("Proc"),
        args: vec![inner],
        kind: Kind::Type,
    }
}

fn int_act_block() -> SurfaceExpr {
    SurfaceExpr::ActBlock {
        stmts: vec![ActStmt::Return {
            value: Box::new(SurfaceExpr::Literal(Literal::Int(1))),
            span: span(),
        }],
        span: span(),
    }
}

#[test]
fn proc_from_act_signature_is_registered_as_explicit_act_to_proc_embedding_only() {
    let env = TypeEnv::with_builtin_types();

    let ty = env
        .lookup_variable("proc::from_act")
        .expect("TypeEnv should register proc::from_act");
    let Type::Fn(params, ret) = ty else {
        panic!("expected proc::from_act to be a function type");
    };

    assert_eq!(params.len(), 1);
    match (&params[0], ret.as_ref()) {
        (
            Type::Constructor {
                name: act_name,
                args: act_args,
                ..
            },
            Type::Constructor {
                name: proc_name,
                args: proc_args,
                ..
            },
        ) => {
            assert_eq!(act_name.name, "Act");
            assert_eq!(proc_name.name, "Proc");
            assert_eq!(act_args.len(), 1);
            assert_eq!(proc_args.len(), 1);
            assert_eq!(act_args[0], proc_args[0]);
        }
        other => panic!("expected Act<A> -> Proc<A>, got {other:?}"),
    }
}

#[test]
fn proc_from_act_is_explicit_embedding_and_proc_unit_does_not_flatten_act_payloads() {
    let env = TypeEnv::with_builtin_types();

    let embedded = SurfaceExpr::Call {
        func: "from_act".into(),
        module: Some("proc".into()),
        args: vec![int_act_block()],
        span: span(),
    };
    let lifted_without_embedding = SurfaceExpr::Call {
        func: "unit".into(),
        module: Some("proc".into()),
        args: vec![int_act_block()],
        span: span(),
    };

    let embedded_result = check_expr(&env, &embedded);
    assert!(
        embedded_result.is_ok(),
        "proc::from_act(act {{ ret 1; }}) should typecheck as an explicit embedding: {embedded_result:?}"
    );
    assert_eq!(
        embedded_result.substitution.apply(&embedded_result.ty),
        proc_of(Type::Int)
    );

    let lifted_result = check_expr(&env, &lifted_without_embedding);
    assert!(
        lifted_result.is_ok(),
        "proc::unit(act {{ ret 1; }}) should keep the Act payload rather than flattening it: {lifted_result:?}"
    );
    assert_eq!(
        lifted_result.substitution.apply(&lifted_result.ty),
        proc_of(act_of(Type::Int))
    );
}
