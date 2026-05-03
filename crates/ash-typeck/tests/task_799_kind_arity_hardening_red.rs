use ash_core::ast::{TypeExpr, Visibility};
use ash_core::module_graph::{CrateId, ModuleId};
use ash_core::semantic_summary::{
    AssociatedMemberIdentityId, AssociatedMemberIdentitySummary, InterfaceIdentityId,
    InterfaceIdentitySummary, ModuleIdentity, ModuleSemanticSummary, ModuleSourceOrigin,
    RepresentationExposure, SourceAnchor, SourceOrigin, TypeDeclId, TypeDeclSummary,
    TypeRepresentationSummary,
};
use ash_parser::surface;
use ash_parser::surface::{
    AssociatedTypeDecl, InterfaceDef as SurfaceInterfaceDef, Visibility as SurfaceVisibility,
};
use ash_typeck::TypeEnv;

fn module_identity(id: usize, path: &[&str]) -> ModuleIdentity {
    ModuleIdentity::new(
        Some(CrateId(1)),
        ModuleId(id),
        path.iter().map(|part| (*part).to_string()).collect(),
        ModuleSourceOrigin::Synthetic {
            reason: format!("task-799-{id}"),
        },
    )
}

fn anchor(label: &str) -> SourceAnchor {
    SourceAnchor::new(
        SourceOrigin::Synthetic {
            reason: "task-799-test".into(),
        },
        None,
        label,
    )
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

fn pair_interface_identity(module: &ModuleIdentity) -> InterfaceIdentityId {
    InterfaceIdentityId::new(module.clone(), "Pair")
}

fn item_member_identity(interface: &InterfaceIdentityId) -> AssociatedMemberIdentityId {
    AssociatedMemberIdentityId::associated_type(
        interface.clone(),
        "Item",
        vec!["Pair".into(), "Item".into()],
    )
}

fn pair_interface_def() -> SurfaceInterfaceDef {
    SurfaceInterfaceDef {
        visibility: SurfaceVisibility::Inherited,
        name: "Pair".into(),
        type_params: vec!["A".into(), "B".into()],
        associated_types: vec![AssociatedTypeDecl {
            name: "Item".into(),
            span: ash_parser::token::Span::default(),
        }],
        methods: vec![],
        span: ash_parser::token::Span::default(),
    }
}

fn register_pair_projection_metadata(env: &mut TypeEnv, module: &ModuleIdentity) {
    let interface = pair_interface_identity(module);
    let member = item_member_identity(&interface);

    env.register_interface(&pair_interface_def())
        .expect("test precondition: Pair interface definition should register");
    env.register_interface_identity_summary(&InterfaceIdentitySummary::new(
        interface.clone(),
        "Pair",
        vec!["Pair".into()],
        anchor("interface Pair"),
    ))
    .expect("test precondition: Pair interface identity should register");
    env.register_associated_member_identity_summary(&AssociatedMemberIdentitySummary::new(
        member,
        "Item",
        anchor("associated type Item"),
    ))
    .expect("test precondition: Pair::Item identity should register");
}

#[test]
fn task799_nominal_wrong_arity_is_rejected_via_core_canonical_lowering() {
    let env = TypeEnv::with_builtin_types();

    let err = env
        .lower_core_type_expr_to_canonical(&TypeExpr::Constructor {
            name: "Result".into(),
            args: vec![TypeExpr::Named("Int".into())],
        })
        .expect_err("TASK-799 should reject wrong nominal arity before canonical IR escapes");

    let message = err.to_string();
    assert!(
        message.contains("Result") && message.contains("expected 2") && message.contains("found 1"),
        "expected nominal arity diagnostic, got: {message}"
    );
}

#[test]
fn task799_imported_nominal_wrong_arity_is_rejected_via_surface_canonical_lowering() {
    let module = module_identity(7993, &["dep", "types"]);
    let summary = ModuleSemanticSummary::new(module.clone()).with_exported_type(
        exported_type_summary(module, "RemotePair", "RemotePair", &["A", "B"]),
    );

    let mut env = TypeEnv::new();
    env.register_module_semantic_summary(&summary)
        .expect("test precondition: imported RemotePair summary should register");

    let err = env
        .lower_surface_type_to_canonical(&surface::Type::Constructor {
            name: "RemotePair".into(),
            args: vec![surface::Type::Name("Int".into())],
        })
        .expect_err(
            "TASK-799 should reject wrong imported nominal arity before canonical IR escapes",
        );

    let message = err.to_string();
    assert!(
        message.contains("RemotePair")
            && message.contains("expected 2")
            && message.contains("found 1"),
        "expected imported nominal arity diagnostic, got: {message}"
    );
}

#[test]
fn task799_projection_lowering_rejects_bare_base_when_registered_interface_arity_is_binary() {
    let module = module_identity(7994, &["pkg", "pair"]);

    let mut env = TypeEnv::new();
    register_pair_projection_metadata(&mut env, &module);

    let err = env
        .lower_core_type_expr_to_canonical(&TypeExpr::Associated {
            base: Box::new(TypeExpr::Named("T".into())),
            name: "Item".into(),
        })
        .expect_err(
            "TASK-799 should reject projection lowering when the registered Pair::Item spine gets only one argument",
        );

    let message = err.to_string();
    assert!(
        message.contains("Pair")
            && (message.contains("arity")
                || message.contains("kind")
                || message.contains("spine")
                || message.contains("expected 2")
                || message.contains("found 1")),
        "expected projection spine validation diagnostic, got: {message}"
    );
}
