use ash_core::ast::Visibility;
use ash_core::module_graph::{CrateId, ModuleId};
use ash_core::semantic_summary::{
    AssociatedMemberIdentityId, AssociatedMemberIdentitySummary, InterfaceIdentityId,
    InterfaceIdentitySummary, ModuleIdentity, ModuleSemanticSummary, ModuleSourceOrigin,
    RepresentationExposure, SourceAnchor, SourceOrigin, TypeDeclId, TypeDeclSummary,
    TypeRepresentationSummary,
};
use ash_typeck::TypeEnv;
use ash_typeck::types::{Substitution, Type, TypeVar, unify};

fn module_identity() -> ModuleIdentity {
    ModuleIdentity::new(
        Some(CrateId(1)),
        ModuleId(788),
        vec!["pkg".into(), "iface".into()],
        ModuleSourceOrigin::Synthetic {
            reason: "task-788-test".into(),
        },
    )
}

fn anchor(label: &str) -> SourceAnchor {
    SourceAnchor::new(
        SourceOrigin::Synthetic {
            reason: "task-788-test".into(),
        },
        None,
        label,
    )
}

#[test]
fn task788_simple_associated_type_substitution_updates_only_base_type() {
    let base = TypeVar(788);
    let associated = Type::Associated {
        interface: "Serializer".into(),
        base: Box::new(Type::Var(base)),
        name: "Ok".into(),
    };
    let mut substitution = Substitution::new();
    substitution.insert(base, Type::String);

    assert_eq!(
        substitution.apply(&associated),
        Type::Associated {
            interface: "Serializer".into(),
            base: Box::new(Type::String),
            name: "Ok".into(),
        }
    );
}

#[test]
fn task788_associated_projection_does_not_normalize_or_unify_with_concrete_type() {
    let associated = Type::Associated {
        interface: "Serializer".into(),
        base: Box::new(Type::String),
        name: "Ok".into(),
    };

    assert!(
        unify(&associated, &Type::String).is_err(),
        "associated identity metadata must not introduce projection normalization or definitional equality"
    );
}

#[test]
fn task788_summary_identity_slots_remain_opaque_metadata_for_typeenv_registration() {
    let module = module_identity();
    let interface_id = InterfaceIdentityId::new(module.clone(), "Serializer");
    let associated_id = AssociatedMemberIdentityId::associated_type(
        interface_id.clone(),
        "Ok",
        vec!["Serializer".into(), "Ok".into()],
    );
    let payload_id = TypeDeclId::ordinary(module.clone(), "Payload");
    let payload = TypeDeclSummary::new(
        payload_id.clone(),
        "Payload",
        Visibility::Public,
        RepresentationExposure::Opaque,
        TypeRepresentationSummary::opaque(false),
        anchor("type Payload"),
    );
    let summary = ModuleSemanticSummary::new(module)
        .with_exported_type(payload)
        .with_interface_identity(InterfaceIdentitySummary::new(
            interface_id.clone(),
            "Serializer",
            vec!["Serializer".into()],
            anchor("interface Serializer"),
        ))
        .with_associated_member_identity(AssociatedMemberIdentitySummary::new(
            associated_id.clone(),
            "Ok",
            anchor("associated type Ok"),
        ));

    let mut env = TypeEnv::new();
    env.register_module_semantic_summary(&summary)
        .expect("opaque interface metadata must not block ordinary type summary registration");

    assert_eq!(env.type_identity_for_name("Payload"), Some(&payload_id));
    assert_eq!(summary.interface_identities[0].id, interface_id);
    assert_eq!(summary.associated_member_identities[0].id, associated_id);
    assert!(
        unify(
            &Type::Associated {
                interface: "Serializer".into(),
                base: Box::new(Type::String),
                name: "Ok".into(),
            },
            &Type::Constructor {
                name: ash_typeck::QualifiedName::root("Payload"),
                args: vec![],
                kind: ash_typeck::Kind::Type,
            },
        )
        .is_err(),
        "interface metadata slots must remain uninterpreted and must not add projection equality"
    );
}
