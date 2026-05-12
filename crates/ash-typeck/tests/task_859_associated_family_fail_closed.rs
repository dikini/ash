use ash_parser::surface::{
    AssociatedFamilyDecreases, AssociatedTypeDecl, AssociatedTypeKind, ImplDef, InterfaceDef,
    InterfaceTypeParam, Type as SurfaceType, Visibility,
};
use ash_parser::token::Span;
use ash_typeck::TypeEnv;
use ash_typeck::error::TypeEnvError;

fn span() -> Span {
    Span::default()
}

#[test]
fn task_859_typeenv_rejects_domain_annotated_interface_params_until_registration_task() {
    let mut env = TypeEnv::new();

    let err = env
        .register_interface(&InterfaceDef {
            visibility: Visibility::Inherited,
            name: "Append".into(),
            type_params: vec![InterfaceTypeParam {
                name: "Xs".into(),
                domain: Some(SurfaceType::Name("TypeList".into())),
                span: span(),
            }],
            associated_types: vec![],
            methods: vec![],
            span: span(),
        })
        .expect_err("domain-annotated family params must fail closed before TASK-861");

    assert!(matches!(err, TypeEnvError::InvalidDefinition(_, _)));
    assert!(
        err.to_string()
            .contains("domain annotation on interface parameter 'Xs'")
    );
}

#[test]
fn task_859_typeenv_rejects_sealed_associated_family_as_ordinary_associated_type() {
    let mut env = TypeEnv::new();

    let err = env
        .register_interface(&InterfaceDef {
            visibility: Visibility::Inherited,
            name: "Iterator".into(),
            type_params: vec!["T".into()],
            associated_types: vec![AssociatedTypeDecl {
                name: "Item".into(),
                kind: AssociatedTypeKind::SealedFamily {
                    result_domain: SurfaceType::Name("Type".into()),
                    decreases: Some(AssociatedFamilyDecreases {
                        param: "T".into(),
                        span: span(),
                    }),
                    span: span(),
                },
                span: span(),
            }],
            methods: vec![],
            span: span(),
        })
        .expect_err("sealed associated families must fail closed before TASK-861");

    assert!(matches!(err, TypeEnvError::InvalidDefinition(_, _)));
    assert!(err.to_string().contains("sealed associated family 'Item'"));
}

#[test]
fn task_859_typeenv_rejects_domain_annotated_impl_params_until_registration_task() {
    let mut env = TypeEnv::new();
    env.register_interface(&InterfaceDef {
        visibility: Visibility::Inherited,
        name: "Iterator".into(),
        type_params: vec!["T".into()],
        associated_types: vec![],
        methods: vec![],
        span: span(),
    })
    .expect("ordinary interface should register as test precondition");

    let err = env
        .register_impl(&ImplDef {
            visibility: Visibility::Inherited,
            interface: "Iterator".into(),
            type_params: vec![InterfaceTypeParam {
                name: "Xs".into(),
                domain: Some(SurfaceType::Name("TypeList".into())),
                span: span(),
            }],
            type_args: vec![SurfaceType::Name("Xs".into())],
            where_bounds: vec![],
            associated_type_bindings: vec![],
            methods: vec![],
            span: span(),
        })
        .expect_err("domain-annotated impl params must fail closed before TASK-861");

    assert!(matches!(err, TypeEnvError::InvalidDefinition(_, _)));
    assert!(
        err.to_string()
            .contains("domain annotation on impl parameter 'Xs'")
    );
}
