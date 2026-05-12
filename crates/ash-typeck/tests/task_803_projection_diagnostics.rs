use ash_core::ast::{TypeExpr, Visibility};
use ash_core::module_graph::{CrateId, ModuleId};
use ash_core::semantic_summary::{
    AssociatedMemberIdentityId, AssociatedMemberIdentitySummary, InterfaceIdentityId,
    InterfaceIdentitySummary, ModuleIdentity, ModuleSemanticSummary, ModuleSourceOrigin,
    RepresentationExposure, SourceAnchor, SourceOrigin, TypeDeclId, TypeDeclSummary,
    TypeRepresentationSummary,
};
use ash_parser::surface::{
    AssociatedTypeBinding, AssociatedTypeDecl, ImplDef, InterfaceDef as SurfaceInterfaceDef,
    Type as SurfaceType, Visibility as SurfaceVisibility, WhereBound,
};
use ash_parser::token::Span;
use ash_typeck::TypeEnv;

fn module_identity(id: usize, path: &[&str]) -> ModuleIdentity {
    ModuleIdentity::new(
        Some(CrateId(1)),
        ModuleId(id),
        path.iter().map(|part| (*part).to_string()).collect(),
        ModuleSourceOrigin::Synthetic {
            reason: format!("task-803-{id}"),
        },
    )
}

fn anchor(label: &str) -> SourceAnchor {
    SourceAnchor::new(
        SourceOrigin::Synthetic {
            reason: "task-803-test".into(),
        },
        None,
        label,
    )
}

fn span() -> Span {
    Span::default()
}

fn exported_type_summary(
    module: ModuleIdentity,
    origin_name: &str,
    exported_name: &str,
    params: &[&str],
) -> TypeDeclSummary {
    TypeDeclSummary::new(
        TypeDeclId::ordinary(module, origin_name),
        exported_name,
        Visibility::Public,
        RepresentationExposure::Opaque,
        TypeRepresentationSummary::opaque(false),
        anchor(exported_name),
    )
    .with_params(params.iter().map(|param| (*param).to_string()).collect())
}

fn interface_identity(module: &ModuleIdentity, name: &str) -> InterfaceIdentityId {
    InterfaceIdentityId::new(module.clone(), name)
}

fn member_identity(
    interface: &InterfaceIdentityId,
    member_name: &str,
    interface_spelling: &str,
) -> AssociatedMemberIdentityId {
    AssociatedMemberIdentityId::associated_type(
        interface.clone(),
        member_name,
        vec![interface_spelling.into(), member_name.into()],
    )
}

fn register_pair_projection_metadata(env: &mut TypeEnv, module: &ModuleIdentity) {
    let interface = interface_identity(module, "Pair");
    let member = member_identity(&interface, "Item", "Pair");
    let summary = ModuleSemanticSummary::new(module.clone()).with_exported_type(
        exported_type_summary(module.clone(), "Pair", "Pair", &["A", "B"]),
    );

    env.register_module_semantic_summary(&summary)
        .expect("test precondition: Pair carrier type summary should register");
    env.register_interface_identity_summary(&InterfaceIdentitySummary::new(
        interface.clone(),
        "Pair",
        vec!["Pair".into()],
        anchor("interface Pair"),
    ))
    .expect("test precondition: Pair interface identity should register");
    env.register_interface(&SurfaceInterfaceDef {
        visibility: SurfaceVisibility::Inherited,
        name: "Pair".into(),
        type_params: vec!["A".into(), "B".into()],
        associated_types: vec![AssociatedTypeDecl {
            name: "Item".into(),
            kind: ash_parser::surface::AssociatedTypeKind::Ordinary,
            span: span(),
        }],
        methods: vec![],
        span: span(),
    })
    .expect("test precondition: Pair interface definition should register");
    env.register_associated_member_identity_summary(&AssociatedMemberIdentitySummary::new(
        member,
        "Item",
        anchor("associated type Item"),
    ))
    .expect("test precondition: Pair::Item identity should register");
}

fn register_serializer_projection_metadata(env: &mut TypeEnv, module: &ModuleIdentity, name: &str) {
    let interface = interface_identity(module, name);
    let member = member_identity(&interface, "Ok", name);
    let summary = ModuleSemanticSummary::new(module.clone())
        .with_exported_type(exported_type_summary(module.clone(), name, name, &["T"]))
        .with_interface_identity(InterfaceIdentitySummary::new(
            interface.clone(),
            name,
            vec![name.into()],
            anchor(&format!("interface {name}")),
        ))
        .with_associated_member_identity(AssociatedMemberIdentitySummary::new(
            member,
            "Ok",
            anchor(&format!("associated type {name}::Ok")),
        ));

    env.register_module_semantic_summary(&summary)
        .expect("test precondition: projection summary should register");
    env.register_interface(&SurfaceInterfaceDef {
        visibility: SurfaceVisibility::Inherited,
        name: name.into(),
        type_params: vec!["T".into()],
        associated_types: vec![AssociatedTypeDecl {
            name: "Ok".into(),
            kind: ash_parser::surface::AssociatedTypeKind::Ordinary,
            span: span(),
        }],
        methods: vec![],
        span: span(),
    })
    .expect("test precondition: interface definition should register");
}

fn serializer_impl_with_ambiguous_ok_projection() -> ImplDef {
    ImplDef {
        visibility: SurfaceVisibility::Inherited,
        interface: "Serializer".into(),
        type_args: vec![SurfaceType::Name("String".into())],
        type_params: vec!["T".into()],
        where_bounds: vec![
            WhereBound {
                param: "T".into(),
                bound: "Serializer".into(),
                span: span(),
            },
            WhereBound {
                param: "T".into(),
                bound: "Formatter".into(),
                span: span(),
            },
        ],
        associated_type_bindings: vec![AssociatedTypeBinding {
            name: "Ok".into(),
            ty: SurfaceType::Associated {
                base: Box::new(SurfaceType::Name("T".into())),
                name: "Ok".into(),
            },
            span: span(),
        }],
        methods: vec![],
        span: span(),
    }
}

#[test]
fn task803_ambiguous_unary_projection_reports_member_and_candidate_bounds() {
    let serializer_module = module_identity(8031, &["pkg", "serializer"]);
    let formatter_module = module_identity(8032, &["pkg", "formatter"]);

    let mut env = TypeEnv::new();
    register_serializer_projection_metadata(&mut env, &serializer_module, "Serializer");
    register_serializer_projection_metadata(&mut env, &formatter_module, "Formatter");

    let err = env
        .register_impl(&serializer_impl_with_ambiguous_ok_projection())
        .expect_err(
            "TASK-803 should reject ambiguous unary projection members in impl-local bounds",
        );

    let message = err.to_string();
    assert!(
        message.contains("ambiguous associated type 'Ok'"),
        "got: {message}"
    );
    assert!(
        message.contains("T::Ok")
            && message.contains("Serializer")
            && message.contains("Formatter"),
        "expected ambiguity diagnostic to mention the source projection and candidate bounds, got: {message}"
    );
}

#[test]
fn task803_tuple_projection_base_reports_projection_specific_unsupported_shape() {
    let module = module_identity(8033, &["pkg", "pair"]);
    let mut env = TypeEnv::new();
    register_pair_projection_metadata(&mut env, &module);

    let err = env
        .lower_core_type_expr_to_canonical(&TypeExpr::Associated {
            base: Box::new(TypeExpr::Tuple(vec![
                TypeExpr::Named("Left".into()),
                TypeExpr::Named("Right".into()),
            ])),
            name: "Item".into(),
        })
        .expect_err(
            "TASK-803 should reject tuple projection bases with a projection-specific diagnostic",
        );

    let message = err.to_string();
    assert!(message.contains("Tuple(2)"), "got: {message}");
    assert!(
        message.contains("projection") && message.contains("unsupported"),
        "expected unsupported projection-shape wording, got: {message}"
    );
}

#[test]
fn task803_surface_capability_projection_base_reports_projection_specific_unsupported_shape() {
    let module = module_identity(8034, &["pkg", "pair"]);
    let mut env = TypeEnv::new();
    register_pair_projection_metadata(&mut env, &module);

    let err = env
        .lower_surface_type_to_canonical(&SurfaceType::Associated {
            base: Box::new(SurfaceType::Capability("Clock".into())),
            name: "Item".into(),
        })
        .expect_err("TASK-803 should reject capability projection bases with a projection-specific diagnostic");

    let message = err.to_string();
    assert!(message.contains("Capability(Clock)"), "got: {message}");
    assert!(
        message.contains("projection") && message.contains("unsupported"),
        "expected unsupported projection-shape wording, got: {message}"
    );
}

#[test]
fn task803_multi_parameter_projection_wrong_arity_mentions_full_projection_spelling() {
    let module = module_identity(8035, &["pkg", "pair"]);
    let mut env = TypeEnv::new();
    register_pair_projection_metadata(&mut env, &module);

    let err = env
        .lower_core_type_expr_to_canonical(&TypeExpr::Associated {
            base: Box::new(TypeExpr::Named("T".into())),
            name: "Item".into(),
        })
        .expect_err("TASK-803 should reject bare unary bases for binary projection spines");

    let message = err.to_string();
    assert!(
        message.contains("Pair") && message.contains("expected 2") && message.contains("found 1"),
        "got: {message}"
    );
    assert!(
        message.contains("T::Item") || message.contains("Pair::Item"),
        "expected arity diagnostic to mention the projection spelling, got: {message}"
    );
}

#[test]
fn task803_unknown_projection_member_mentions_interface_and_full_projection_spelling() {
    let module = module_identity(8036, &["pkg", "pair"]);
    let mut env = TypeEnv::new();
    register_pair_projection_metadata(&mut env, &module);

    let err = env
        .lower_core_type_expr_to_canonical(&TypeExpr::Associated {
            base: Box::new(TypeExpr::Constructor {
                name: "Pair".into(),
                args: vec![
                    TypeExpr::Named("Left".into()),
                    TypeExpr::Named("Right".into()),
                ],
            }),
            name: "Missing".into(),
        })
        .expect_err("TASK-803 should reject unknown members on otherwise-valid projection spines");

    let message = err.to_string();
    assert!(
        message.contains("registered member on interface Pair"),
        "got: {message}"
    );
    assert!(
        message.contains("Pair<Left, Right>::Missing") || message.contains("Pair::Missing"),
        "expected diagnostic to mention the full projection spelling, got: {message}"
    );
}

#[test]
fn task803_function_projection_base_reports_projection_specific_unsupported_shape() {
    let module = module_identity(8038, &["pkg", "pair"]);
    let mut env = TypeEnv::new();
    register_pair_projection_metadata(&mut env, &module);

    let err = env
        .lower_surface_type_to_canonical(&SurfaceType::Associated {
            base: Box::new(SurfaceType::Fn(
                vec![SurfaceType::Name("Int".into())],
                Box::new(SurfaceType::Name("String".into())),
            )),
            name: "Item".into(),
        })
        .expect_err("TASK-803 should reject function projection bases with a projection-specific diagnostic");

    let message = err.to_string();
    assert!(message.contains("Fn"), "got: {message}");
    assert!(
        message.contains("projection") && message.contains("unsupported"),
        "expected unsupported projection-shape wording for wrong-kind/function bases, got: {message}"
    );
}

#[test]
fn task803_non_projection_constructor_shape_diagnostics_remain_unchanged() {
    let module = module_identity(8039, &["pkg", "pair"]);
    let mut env = TypeEnv::new();
    register_pair_projection_metadata(&mut env, &module);

    let err = env
        .lower_core_type_expr_to_canonical(&TypeExpr::Tuple(vec![TypeExpr::Named("Only".into())]))
        .expect_err("non-projection constructor-shape rejection should remain explicit");

    let message = err.to_string();
    assert!(message.contains("Tuple(1)"), "got: {message}");
    assert!(
        !message.contains("unsupported projection base"),
        "TASK-803 diagnostics work must not relabel unrelated non-projection constructor-shape failures as projection diagnostics: {message}"
    );
}
