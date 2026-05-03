use ash_core::ast::{TypeBody, TypeDef, Visibility};
use ash_core::module_graph::{CrateId, ModuleId};
use ash_core::semantic_summary::{
    AssociatedMemberIdentityId, AssociatedMemberIdentitySummary, InterfaceIdentityId,
    InterfaceIdentitySummary, ModuleIdentity, ModuleSemanticSummary, ModuleSourceOrigin,
    SourceAnchor, SourceOrigin,
};
use ash_typeck::{Kind, QualifiedName, Type, TypeEnv, TypeVar};

fn anchor(label: &str) -> SourceAnchor {
    SourceAnchor::new(
        SourceOrigin::Synthetic {
            reason: "task-802-test".into(),
        },
        None,
        label,
    )
}

fn module_identity(id: usize, path: &[&str]) -> ModuleIdentity {
    ModuleIdentity::new(
        Some(CrateId(1)),
        ModuleId(id),
        path.iter().map(|part| (*part).to_string()).collect(),
        ModuleSourceOrigin::Synthetic {
            reason: format!("task-802-{id}"),
        },
    )
}

fn register_identity_alias(env: &mut TypeEnv) {
    env.register_type(&TypeDef {
        name: "Identity".into(),
        params: vec!["T".into()],
        body: TypeBody::Alias(ash_core::ast::TypeExpr::Named("T".into())),
        visibility: Visibility::Public,
        builtin: false,
    })
    .expect("test precondition: Identity alias should register");
}

fn register_user_id_alias(env: &mut TypeEnv) {
    env.register_type(&TypeDef {
        name: "UserId".into(),
        params: vec![],
        body: TypeBody::Alias(ash_core::ast::TypeExpr::Named("String".into())),
        visibility: Visibility::Public,
        builtin: false,
    })
    .expect("test precondition: UserId alias should register");
}

fn ok_member_identity(
    interface: &InterfaceIdentityId,
    interface_spelling: &str,
) -> AssociatedMemberIdentityId {
    AssociatedMemberIdentityId::associated_type(
        interface.clone(),
        "Ok",
        vec![interface_spelling.into(), "Ok".into()],
    )
}

fn register_serializer_projection_aliases(env: &mut TypeEnv) {
    let module = module_identity(8021, &["pkg", "serializer"]);
    let interface = InterfaceIdentityId::new(module.clone(), "Serializer");
    let canonical_member = ok_member_identity(&interface, "Serializer");
    let alias_member = ok_member_identity(&interface, "SerializerAlias");

    let summary = ModuleSemanticSummary::new(module.clone())
        .with_interface_identity(InterfaceIdentitySummary::new(
            interface.clone(),
            "Serializer",
            vec!["Serializer".into()],
            anchor("interface Serializer"),
        ))
        .with_associated_member_identity(AssociatedMemberIdentitySummary::new(
            canonical_member,
            "Ok",
            anchor("associated type Ok"),
        ));

    env.register_module_semantic_summary(&summary)
        .expect("test precondition: canonical projection summary should register");
    env.register_interface_identity_summary(&InterfaceIdentitySummary::new(
        interface.clone(),
        "SerializerAlias",
        vec!["SerializerAlias".into()],
        anchor("interface SerializerAlias"),
    ))
    .expect("test precondition: alias interface identity should register");
    env.register_associated_member_identity_summary(&AssociatedMemberIdentitySummary::new(
        alias_member,
        "Ok",
        anchor("associated type Ok alias"),
    ))
    .expect("test precondition: alias associated member identity should register");
}

#[test]
fn task802_unify_types_consumes_transparent_alias_canonical_heads_at_equality_boundary() {
    let mut env = TypeEnv::with_builtin_types();
    register_identity_alias(&mut env);
    register_user_id_alias(&mut env);

    let alias = Type::Constructor {
        name: QualifiedName::root("Identity"),
        args: vec![Type::Constructor {
            name: QualifiedName::root("UserId"),
            args: vec![],
            kind: Kind::Type,
        }],
        kind: Kind::Type,
    };

    env.unify_types(&alias, &Type::String).expect(
        "TASK-802 should adopt TASK-801 transparent alias canonical heads at unify_types without requiring callers to normalize manually",
    );
}

#[test]
fn task802_types_equivalent_for_equality_consumes_projection_canonical_forms_at_boundary() {
    let mut env = TypeEnv::new();
    register_serializer_projection_aliases(&mut env);

    let canonical = Type::Associated {
        interface: "Serializer".into(),
        base: Box::new(Type::Var(TypeVar(41))),
        name: "Ok".into(),
    };
    let alias = Type::Associated {
        interface: "SerializerAlias".into(),
        base: Box::new(Type::Var(TypeVar(41))),
        name: "Ok".into(),
    };

    assert!(
        env.types_equivalent_for_equality(&canonical, &alias),
        "TASK-802 should route equality comparison through TASK-800 canonical rigid projection identities"
    );
}

#[test]
fn task802_unify_types_consumes_projection_canonical_forms_at_boundary() {
    let mut env = TypeEnv::new();
    register_serializer_projection_aliases(&mut env);

    let canonical = Type::Associated {
        interface: "Serializer".into(),
        base: Box::new(Type::Var(TypeVar(99))),
        name: "Ok".into(),
    };
    let alias = Type::Associated {
        interface: "SerializerAlias".into(),
        base: Box::new(Type::Var(TypeVar(99))),
        name: "Ok".into(),
    };

    env.unify_types(&canonical, &alias).expect(
        "TASK-802 should adopt TASK-800 canonical rigid projection identities at unify_types as well as equality comparison",
    );
}

#[test]
fn task802_neutral_projection_heads_still_do_not_solve_against_concrete_types() {
    let env = TypeEnv::with_builtin_types();
    let neutral = Type::Associated {
        interface: "Serializer".into(),
        base: Box::new(Type::Var(TypeVar(7))),
        name: "Ok".into(),
    };

    assert!(
        env.unify_types(&neutral, &Type::String).is_err(),
        "TASK-802 must not introduce solving or normalization for unresolved neutral projection heads"
    );
    assert!(
        !env.types_equivalent_for_equality(&neutral, &Type::String),
        "TASK-802 equality adoption must still reject unresolved neutral projection heads against concrete targets"
    );
}

#[test]
fn task802_neutral_projection_heads_still_do_not_invert_through_nominal_constructors() {
    let env = TypeEnv::with_builtin_types();
    let neutral_list = Type::List(Box::new(Type::Associated {
        interface: "Serializer".into(),
        base: Box::new(Type::Var(TypeVar(8))),
        name: "Ok".into(),
    }));

    assert!(
        env.unify_types(&neutral_list, &Type::List(Box::new(Type::String)))
            .is_err(),
        "TASK-802 must not add inversion or solving beneath an unresolved neutral projection head just because the surrounding constructor decomposes"
    );
}

#[test]
fn task802_ordinary_nominal_constructor_decomposition_behavior_remains_unchanged() {
    let env = TypeEnv::with_builtin_types();
    let left = Type::Constructor {
        name: QualifiedName::root("Pair"),
        args: vec![Type::Int, Type::String],
        kind: Kind::Type,
    };
    let right = Type::Constructor {
        name: QualifiedName::root("Pair"),
        args: vec![Type::Int, Type::String],
        kind: Kind::Type,
    };
    let different = Type::Constructor {
        name: QualifiedName::root("Pair"),
        args: vec![Type::String, Type::Int],
        kind: Kind::Type,
    };

    env.unify_types(&left, &right).expect(
        "TASK-802 should preserve existing ordinary constructor decomposition for matching nominal heads",
    );
    assert!(
        !env.types_equivalent_for_equality(&left, &different),
        "TASK-802 should not change ordinary constructor decomposition outcomes unrelated to alias/projection canonicalization"
    );
}
