#![allow(missing_docs)]
use ash_core::ast::{Expr, Workflow};
use ash_parser::surface::{
    Expr as SurfaceExpr, ImplDef, ImplMethodDef, InterfaceDef, InterfaceMethodSig, Literal,
    Type as SurfaceType, Visibility, WhereBound,
};
use ash_parser::token::Span;
use ash_typeck::type_env::TypeEnv;

fn test_span() -> Span {
    Span::default()
}

fn serialize_interface_def() -> InterfaceDef {
    InterfaceDef {
        visibility: Visibility::Public,
        name: "Serialize".into(),
        type_params: vec!["T".into()],
        evidence_constraints: vec![],
        associated_types: vec![],
        methods: vec![InterfaceMethodSig {
            name: "serialize".into(),
            params: vec![SurfaceType::Name("T".into())],
            return_type: SurfaceType::Name("String".into()),
            span: test_span(),
        }],
        span: test_span(),
    }
}

fn serialize_int_impl() -> ImplDef {
    ImplDef {
        visibility: Visibility::Public,
        interface: "Serialize".into(),
        type_params: vec![],
        type_args: vec![SurfaceType::Name("Int".into())],
        where_bounds: vec![],
        associated_type_bindings: vec![],
        methods: vec![ImplMethodDef {
            name: "serialize".into(),
            params: vec!["x".into()],
            body: SurfaceExpr::Literal(Literal::String("int".into())),
            span: test_span(),
        }],
        span: test_span(),
    }
}

fn serialize_list_generic_impl() -> ImplDef {
    ImplDef {
        visibility: Visibility::Public,
        interface: "Serialize".into(),
        type_params: vec!["T".into()],
        type_args: vec![SurfaceType::Constructor {
            name: "List".into(),
            args: vec![SurfaceType::Name("T".into())],
        }],
        where_bounds: vec![WhereBound {
            param: "T".into(),
            bound: "Serialize".into(),
            span: test_span(),
        }],
        associated_type_bindings: vec![],
        methods: vec![ImplMethodDef {
            name: "serialize".into(),
            params: vec!["items".into()],
            body: SurfaceExpr::Literal(Literal::String("list".into())),
            span: test_span(),
        }],
        span: test_span(),
    }
}

#[test]
fn task566_monomorphize_replaces_interface_call_with_impl_body() {
    let mut env = TypeEnv::with_builtin_types();
    env.register_interface(&serialize_interface_def()).unwrap();
    env.register_impl(&serialize_int_impl()).unwrap();

    let mut workflow = Workflow::Ret {
        expr: Expr::Call {
            func: "serialize".into(),
            module: Some("Serialize".into()),
            arguments: vec![Expr::Literal(ash_core::Value::Int(42))],
        },
    };

    ash_engine::monomorphize::monomorphize_workflow(&mut workflow, &env).unwrap();

    // After monomorphization, the interface call should be replaced by the impl body.
    match &workflow {
        Workflow::Ret { expr } => {
            assert!(
                matches!(expr, Expr::Literal(ash_core::Value::String(s)) if s == "int"),
                "expected monomorphized body literal, got {expr:?}"
            );
        }
        _ => panic!("unexpected workflow shape"),
    }
}

#[test]
fn task566_monomorphize_recursive_generic_impl() {
    let mut env = TypeEnv::with_builtin_types();
    env.register_interface(&serialize_interface_def()).unwrap();
    env.register_impl(&serialize_int_impl()).unwrap();
    env.register_impl(&serialize_list_generic_impl()).unwrap();

    let list_arg = Expr::Literal(ash_core::Value::List(Box::new(vec![
        ash_core::Value::Int(1),
        ash_core::Value::Int(2),
    ])));

    let mut workflow = Workflow::Ret {
        expr: Expr::Call {
            func: "serialize".into(),
            module: Some("Serialize".into()),
            arguments: vec![list_arg],
        },
    };

    ash_engine::monomorphize::monomorphize_workflow(&mut workflow, &env).unwrap();

    match &workflow {
        Workflow::Ret { expr } => {
            assert!(
                matches!(expr, Expr::Literal(ash_core::Value::String(s)) if s == "list"),
                "expected monomorphized generic impl body literal, got {expr:?}"
            );
        }
        _ => panic!("unexpected workflow shape"),
    }
}

#[test]
fn task566_monomorphize_errors_on_missing_impl() {
    let env = TypeEnv::with_builtin_types();
    // Intentionally omitting any impl registration

    let mut workflow = Workflow::Ret {
        expr: Expr::Call {
            func: "serialize".into(),
            module: Some("Serialize".into()),
            arguments: vec![Expr::Literal(ash_core::Value::Int(42))],
        },
    };

    let result = ash_engine::monomorphize::monomorphize_workflow(&mut workflow, &env);
    assert!(
        result.is_err(),
        "monomorphization should fail when no impl is found"
    );
}
