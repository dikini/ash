#![allow(missing_docs)]
use ash_core::ast::Expr;
use ash_parser::surface::{
    AssociatedTypeBinding, AssociatedTypeDecl, Expr as SurfaceExpr, ImplDef, ImplMethodDef,
    InterfaceDef, InterfaceMethodSig, Literal, Type as SurfaceType, Visibility,
};
use ash_parser::token::Span;
use ash_typeck::Type;
use ash_typeck::type_env::TypeEnv;

fn test_span() -> Span {
    Span::default()
}

fn serializer_interface_def() -> InterfaceDef {
    InterfaceDef {
        visibility: Visibility::Public,
        name: "Serializer".into(),
        type_params: vec!["S".into()],
        evidence_constraints: vec![],
        associated_types: vec![AssociatedTypeDecl {
            name: "Ok".into(),
            kind: ash_parser::surface::AssociatedTypeKind::Ordinary,
            span: test_span(),
        }],
        methods: vec![InterfaceMethodSig {
            name: "serialize_bool".into(),
            params: vec![
                SurfaceType::Name("S".into()),
                SurfaceType::Name("Bool".into()),
            ],
            return_type: SurfaceType::Associated {
                base: Box::new(SurfaceType::Name("S".into())),
                name: "Ok".into(),
            },
            span: test_span(),
        }],
        laws: Vec::new(),
        span: test_span(),
    }
}

fn serializer_string_impl() -> ImplDef {
    ImplDef {
        visibility: Visibility::Public,
        interface: "Serializer".into(),
        type_params: vec![],
        type_args: vec![SurfaceType::Name("String".into())],
        where_bounds: vec![],
        associated_type_bindings: vec![AssociatedTypeBinding {
            name: "Ok".into(),
            ty: SurfaceType::Name("String".into()),
            span: test_span(),
        }],
        methods: vec![ImplMethodDef {
            name: "serialize_bool".into(),
            params: vec!["writer".into(), "value".into()],
            body: SurfaceExpr::Literal(Literal::String("serialized".into())),
            span: test_span(),
        }],
        handlers: Vec::new(),
        derived_handlers: Vec::new(),
        proofs: Vec::new(),
        span: test_span(),
    }
}

#[test]
fn task568_associated_type_replaced_in_monomorphized_body() {
    let mut env = TypeEnv::with_builtin_types();
    env.register_interface(&serializer_interface_def()).unwrap();
    env.register_impl(&serializer_string_impl()).unwrap();

    let mut expr = Expr::Call {
        func: "serialize_bool".into(),
        module: Some("Serializer".into()),
        arguments: vec![
            Expr::Literal(ash_core::Value::String("writer".into())),
            Expr::Literal(ash_core::Value::Bool(true)),
        ],
    };

    ash_engine::monomorphize::monomorphize_expr(&mut expr, &env).unwrap();

    assert!(
        matches!(&expr, Expr::Literal(ash_core::Value::String(s)) if s == "serialized"),
        "expected monomorphized body literal, got {expr:?}"
    );

    // Also verify that the selected scheme's method signature normalizes correctly.
    let (_, scheme) = env
        .select_impl_scheme("Serializer", "serialize_bool", &[Type::String, Type::Bool])
        .unwrap();
    let method = scheme
        .methods
        .iter()
        .find(|m| m.name == "serialize_bool")
        .unwrap();
    let normalized_return = env
        .normalize_associated_types(
            &method.return_type,
            scheme,
            &ash_typeck::types::Substitution::new(),
        )
        .unwrap();
    assert_eq!(normalized_return, Type::String);
}
