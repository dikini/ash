use ash_core::ast::{TypeBody, TypeDef, TypeExpr, Visibility};
use ash_core::module_graph::{CrateId, ModuleId};
use ash_core::semantic_summary::{
    AssociatedMemberIdentityId, AssociatedMemberIdentitySummary, InterfaceIdentityId,
    InterfaceIdentitySummary, ModuleIdentity, ModuleSemanticSummary, ModuleSourceOrigin,
    SourceAnchor, SourceOrigin,
};
use ash_core::type_ir::{CanonicalTypeExpr, ProjectionRigidity};
use ash_parser::surface;
use ash_typeck::{Kind, TypeEnv};

fn module_identity(id: usize, path: &[&str]) -> ModuleIdentity {
    ModuleIdentity::new(
        Some(CrateId(1)),
        ModuleId(id),
        path.iter().map(|part| (*part).to_string()).collect(),
        ModuleSourceOrigin::Synthetic {
            reason: format!("task-798-{id}"),
        },
    )
}

fn anchor(label: &str) -> SourceAnchor {
    SourceAnchor::new(
        SourceOrigin::Synthetic {
            reason: "task-798-test".into(),
        },
        None,
        label,
    )
}

fn serializer_interface_identity(module: &ModuleIdentity) -> InterfaceIdentityId {
    InterfaceIdentityId::new(module.clone(), "Serializer")
}

fn ok_member_identity(interface: &InterfaceIdentityId) -> AssociatedMemberIdentityId {
    AssociatedMemberIdentityId::associated_type(
        interface.clone(),
        "Ok",
        vec!["Serializer".into(), "Ok".into()],
    )
}

fn error_member_identity(interface: &InterfaceIdentityId) -> AssociatedMemberIdentityId {
    AssociatedMemberIdentityId::associated_type(
        interface.clone(),
        "Error",
        vec![interface.name.to_string(), "Error".into()],
    )
}

fn synthetic_identityless_type(name: &str) -> TypeDef {
    TypeDef {
        name: name.into(),
        params: vec![],
        body: TypeBody::Struct(vec![]),
        visibility: Visibility::Public,
        builtin: false,
    }
}

#[test]
fn task798_ast_type_expr_lowering_entry_point_produces_canonical_projection_ids() {
    let module = module_identity(7981, &["pkg", "source"]);
    let interface = serializer_interface_identity(&module);
    let member = ok_member_identity(&interface);

    let mut env = TypeEnv::new();
    env.register_interface_identity_summary(&InterfaceIdentitySummary::new(
        interface.clone(),
        "Serializer",
        vec!["Serializer".into()],
        anchor("interface Serializer"),
    ))
    .expect("source-local interface identities should register before lowering");
    env.register_associated_member_identity_summary(&AssociatedMemberIdentitySummary::new(
        member.clone(),
        "Ok",
        anchor("associated type Ok"),
    ))
    .expect("source-local associated member identities should register before lowering");

    let lowered = env
        .lower_core_type_expr_to_canonical(&TypeExpr::Associated {
            base: Box::new(TypeExpr::Named("T".into())),
            name: "Ok".into(),
        })
        .expect("core associated type syntax should lower into canonical type IR");

    assert_eq!(
        lowered,
        CanonicalTypeExpr::Projection {
            interface,
            member,
            args: vec![CanonicalTypeExpr::Var("T".into())],
            kind: Kind::Type,
            rigidity: ProjectionRigidity::Neutral,
        }
    );
}

#[test]
fn task798_surface_type_lowering_entry_point_preserves_nominal_arity_checks() {
    let env = TypeEnv::with_builtin_types();

    let err = env
        .lower_surface_type_to_canonical(&surface::Type::Constructor {
            name: "Result".into(),
            args: vec![surface::Type::Name("Int".into())],
        })
        .expect_err("canonical lowering should reuse existing arity validation entry points");

    let message = err.to_string();
    assert!(
        message.contains("Result") && message.contains('2'.to_string().as_str()),
        "expected reused kind/arity diagnostics to mention Result arity, got: {message}"
    );
}

#[test]
fn task798_imported_and_source_registered_projection_identities_share_one_typeenv_registry() {
    let imported_module = module_identity(7982, &["dep", "wire"]);
    let imported_interface = serializer_interface_identity(&imported_module);
    let imported_member = ok_member_identity(&imported_interface);
    let imported_summary = ModuleSemanticSummary::new(imported_module)
        .with_interface_identity(InterfaceIdentitySummary::new(
            imported_interface.clone(),
            "Serializer",
            vec!["Serializer".into()],
            anchor("imported interface Serializer"),
        ))
        .with_associated_member_identity(AssociatedMemberIdentitySummary::new(
            imported_member.clone(),
            "Ok",
            anchor("imported associated type Ok"),
        ));

    let source_module = module_identity(7983, &["pkg", "source"]);
    let source_interface = serializer_interface_identity(&source_module);
    let source_member = ok_member_identity(&source_interface);

    let mut env = TypeEnv::new();
    env.register_module_semantic_summary(&imported_summary)
        .expect("imported summary identities should stage into the canonical registry");
    env.register_interface_identity_summary(&InterfaceIdentitySummary::new(
        source_interface.clone(),
        "Serializer",
        vec!["Serializer".into()],
        anchor("source interface Serializer"),
    ))
    .expect("source-local identity registration should share the imported registry");
    env.register_associated_member_identity_summary(&AssociatedMemberIdentitySummary::new(
        source_member.clone(),
        "Ok",
        anchor("source associated type Ok"),
    ))
    .expect("source-local associated members should share the imported registry");

    assert_eq!(
        env.interface_identity_for_name("Serializer"),
        Some(&source_interface),
        "source-local declarations should become the preferred visible identity"
    );
    assert_eq!(
        env.associated_member_identity_for_interface_member("Serializer", "Ok"),
        Some(&source_member),
        "source-local associated members should become the preferred visible identity"
    );
    assert!(
        env.interface_identity_known(&imported_interface)
            && env.interface_identity_known(&source_interface),
        "imported and source-local interface identities should coexist in one registry"
    );
    assert!(
        env.associated_member_identity_known(&imported_member)
            && env.associated_member_identity_known(&source_member),
        "imported and source-local associated member identities should coexist in one registry"
    );
}

#[test]
fn task798_canonical_nominal_lowering_rejects_missing_identity_after_resolve_succeeds() {
    let mut env = TypeEnv::new();
    env.register_type_identity(&synthetic_identityless_type("Ghost"))
        .expect("synthetic type should resolve without registering a canonical identity");

    assert!(
        env.resolve_type("Ghost").is_ok(),
        "precondition: Ghost should resolve successfully before canonical lowering"
    );
    assert!(
        env.type_identity_for_name("Ghost").is_none(),
        "precondition: Ghost intentionally lacks a registered canonical identity"
    );

    let err = env
        .lower_core_type_expr_to_canonical(&TypeExpr::Named("Ghost".into()))
        .expect_err(
            "canonical lowering must reject resolved nominal types that still lack canonical identity registration"
        );

    let message = err.to_string();
    assert!(
        message.contains("Ghost")
            && (message.contains("identity")
                || message.contains("canonical")
                || message.contains("registered")),
        "expected missing canonical identity diagnostic, got: {message}"
    );
}

#[test]
fn task798_unbounded_associated_projection_stays_neutral_at_lowering_boundary() {
    let module = module_identity(7984, &["pkg", "source"]);
    let interface = serializer_interface_identity(&module);
    let member = ok_member_identity(&interface);

    let mut env = TypeEnv::new();
    env.register_interface_identity_summary(&InterfaceIdentitySummary::new(
        interface.clone(),
        "Serializer",
        vec!["Serializer".into()],
        anchor("interface Serializer"),
    ))
    .expect("interface identity should register before lowering");
    env.register_associated_member_identity_summary(&AssociatedMemberIdentitySummary::new(
        member.clone(),
        "Ok",
        anchor("associated type Ok"),
    ))
    .expect("associated member identity should register before lowering");

    let lowered = env
        .lower_core_type_expr_to_canonical(&TypeExpr::Associated {
            base: Box::new(TypeExpr::Named("T".into())),
            name: "Ok".into(),
        })
        .expect("entry-point lowering may preserve unresolved associated syntax without inference");

    assert_eq!(
        lowered,
        CanonicalTypeExpr::Projection {
            interface,
            member,
            args: vec![CanonicalTypeExpr::Var("T".into())],
            kind: Kind::Type,
            rigidity: ProjectionRigidity::Neutral,
        },
        "TASK-798 lowering should not rigidify unbounded T::Ok before TASK-800 resolution/inference exists"
    );
}

#[test]
fn task798_interface_identity_registration_rejects_conflicting_visible_aliases() {
    let source_module = module_identity(7985, &["pkg", "source"]);
    let imported_module = module_identity(7986, &["dep", "wire"]);
    let source_interface = serializer_interface_identity(&source_module);
    let imported_interface = serializer_interface_identity(&imported_module);

    let mut env = TypeEnv::new();
    env.register_interface_identity_summary(&InterfaceIdentitySummary::new(
        source_interface,
        "Serializer",
        vec!["Serializer".into()],
        anchor("source interface Serializer"),
    ))
    .expect("first visible alias should register");

    let err = env
        .register_interface_identity_summary(&InterfaceIdentitySummary::new(
            imported_interface,
            "Serializer",
            vec!["Serializer".into()],
            anchor("conflicting imported interface Serializer"),
        ))
        .expect_err("conflicting visible interface aliases must be rejected, not overwritten");

    let message = err.to_string();
    assert!(
        message.contains("Serializer")
            && (message.contains("conflict")
                || message.contains("duplicate")
                || message.contains("already defined")
                || message.contains("Invalid type definition")),
        "expected conflicting interface alias diagnostic, got: {message}"
    );
}

#[test]
fn task798_associated_member_registration_rejects_conflicting_visible_aliases() {
    let serializer_module = module_identity(7987, &["pkg", "serializer"]);
    let source_interface = serializer_interface_identity(&serializer_module);
    let imported_interface =
        serializer_interface_identity(&module_identity(7988, &["dep", "serializer"]));
    let source_member = error_member_identity(&source_interface);
    let imported_member = error_member_identity(&imported_interface);

    let mut env = TypeEnv::new();
    env.register_interface_identity_summary(&InterfaceIdentitySummary::new(
        source_interface.clone(),
        "Serializer",
        vec!["Serializer".into()],
        anchor("source interface Serializer"),
    ))
    .expect("source interface should register");
    env.register_associated_member_identity_summary(&AssociatedMemberIdentitySummary::new(
        source_member,
        "Error",
        anchor("source Serializer::Error"),
    ))
    .expect("first associated member alias should register");

    let err = env
        .register_associated_member_identity_summary(&AssociatedMemberIdentitySummary::new(
            imported_member,
            "Error",
            anchor("imported Serializer::Error"),
        ))
        .expect_err(
            "conflicting visible associated-member aliases for the same interface/member pair must be rejected",
        );

    let message = err.to_string();
    assert!(
        message.contains("Serializer")
            && message.contains("Error")
            && (message.contains("conflict")
                || message.contains("duplicate")
                || message.contains("already defined")
                || message.contains("Invalid type definition")),
        "expected conflicting associated-member alias diagnostic, got: {message}"
    );
}

#[test]
fn task798_surface_lowering_rejects_deferred_non_nominal_shapes_instead_of_stringifying_them() {
    let env = TypeEnv::with_builtin_types();

    let err = env
        .lower_surface_type_to_canonical(&surface::Type::List(Box::new(surface::Type::Name(
            "Int".into(),
        ))))
        .expect_err(
            "TASK-798 should reject deferred non-nominal shapes instead of lowering them into lossy Primitive strings",
        );

    let message = err.to_string();
    assert!(
        message.contains("List")
            && (message.contains("unsupported")
                || message.contains("deferred")
                || message.contains("canonical")
                || message.contains("Constructor name mismatch")),
        "expected unsupported deferred-shape diagnostic, got: {message}"
    );
}
