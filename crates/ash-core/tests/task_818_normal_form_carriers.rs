use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

use ash_core::kind::Kind;
use ash_core::module_graph::{CrateId, ModuleId};
use ash_core::semantic_summary::{
    AssociatedMemberIdentityId, ConstructorId, ConstructorPayloadKind, DomainConstructorId,
    InterfaceIdentityId, ModuleIdentity, ModuleSourceOrigin, SealedDomainId, TypeDeclId,
};
use ash_core::type_ir::{
    CanonicalTypeExpr, NormalFormBlockReason, NormalTypeExpr, ProjectionRigidity,
    TypeComputationHeadId,
};

fn module_identity() -> ModuleIdentity {
    ModuleIdentity::new(
        Some(CrateId(818)),
        ModuleId(112),
        vec!["phase112".to_string(), "task818".to_string()],
        ModuleSourceOrigin::Synthetic {
            reason: "task-818 normal-form carrier test".to_string(),
        },
    )
}

fn domain_id(name: &str) -> SealedDomainId {
    SealedDomainId::new(module_identity(), name)
}

fn domain_constructor(domain: SealedDomainId, name: &str) -> DomainConstructorId {
    DomainConstructorId::new(domain, name)
}

fn ordinary_type(name: &str) -> TypeDeclId {
    TypeDeclId::ordinary(module_identity(), name)
}

fn hash_of<T: Hash>(value: &T) -> u64 {
    let mut hasher = DefaultHasher::new();
    value.hash(&mut hasher);
    hasher.finish()
}

#[test]
fn domain_constructor_normal_forms_are_identity_equal_hashable_and_serde_roundtrip() {
    let domain = domain_id("TypeList");
    let cons = domain_constructor(domain.clone(), "Cons");
    let lhs = NormalTypeExpr::DomainConstructorApp {
        constructor: cons.clone(),
        domain: domain.clone(),
        args: vec![
            NormalTypeExpr::Primitive("Int".to_string()),
            NormalTypeExpr::Var("Tail".to_string()),
        ],
        kind: Kind::Type,
    };
    let rhs = NormalTypeExpr::DomainConstructorApp {
        constructor: cons,
        domain,
        args: vec![
            NormalTypeExpr::Primitive("Int".to_string()),
            NormalTypeExpr::Var("Tail".to_string()),
        ],
        kind: Kind::Type,
    };

    assert_eq!(lhs, rhs);
    assert_eq!(hash_of(&lhs), hash_of(&rhs));

    let json = serde_json::to_string(&lhs).expect("normal form serializes");
    assert!(json.contains("DomainConstructorApp") || json.contains("domain_constructor"));
    let decoded: NormalTypeExpr = serde_json::from_str(&json).expect("normal form deserializes");
    assert_eq!(decoded, lhs);
}

#[test]
fn sealed_domain_constructor_apps_do_not_use_ordinary_constructor_identity() {
    let domain = domain_id("TypeList");
    let nil = domain_constructor(domain.clone(), "Nil");
    let ordinary_nil = ConstructorId::variant(
        ordinary_type("TypeList"),
        "Nil",
        ConstructorPayloadKind::Unit,
    );

    let domain_nf = NormalTypeExpr::DomainConstructorApp {
        constructor: nil,
        domain: domain.clone(),
        args: vec![],
        kind: Kind::Type,
    };
    let nominal_nf = NormalTypeExpr::NominalApp {
        origin: ordinary_nil.parent.clone(),
        visible_name: ordinary_nil.name.to_string(),
        args: vec![],
        kind: Kind::Type,
    };

    assert_ne!(domain_nf, nominal_nf);
    match domain_nf {
        NormalTypeExpr::DomainConstructorApp {
            constructor,
            domain: actual_domain,
            ..
        } => {
            assert_eq!(constructor.domain, actual_domain);
            assert_eq!(actual_domain, domain);
        }
        other => panic!("expected sealed-domain constructor normal form, got {other:?}"),
    }
}

#[test]
fn neutral_computation_apps_are_distinct_from_domain_constructor_heads() {
    let domain = domain_id("TypeList");
    let nil = NormalTypeExpr::DomainConstructorApp {
        constructor: domain_constructor(domain.clone(), "Nil"),
        domain,
        args: vec![],
        kind: Kind::Type,
    };
    let append = NormalTypeExpr::NeutralComputationApp {
        head: TypeComputationHeadId::new(module_identity(), "Append"),
        args: vec![
            NormalTypeExpr::Var("Xs".to_string()),
            NormalTypeExpr::Var("Ys".to_string()),
        ],
        kind: Kind::Type,
        reason: Some(NormalFormBlockReason::AbstractScrutinee),
    };

    assert_ne!(nil, append);
    match append {
        NormalTypeExpr::NeutralComputationApp {
            head,
            args,
            kind,
            reason,
        } => {
            assert_eq!(head.name, "Append");
            assert_eq!(args.len(), 2);
            assert_eq!(kind, Kind::Type);
            assert_eq!(reason, Some(NormalFormBlockReason::AbstractScrutinee));
        }
        other => panic!("expected neutral computation normal form, got {other:?}"),
    }
}

#[test]
fn neutral_and_rigid_projection_normal_forms_preserve_rigidity_in_equality_hash_and_serde() {
    let interface = InterfaceIdentityId::new(module_identity(), "Iterable");
    let member = AssociatedMemberIdentityId::associated_type(
        interface.clone(),
        "Item",
        vec!["Item".to_string()],
    );
    let args = vec![NormalTypeExpr::Var("Collection".to_string())];
    let neutral = NormalTypeExpr::Projection {
        interface: interface.clone(),
        member: member.clone(),
        args: args.clone(),
        kind: Kind::Type,
        rigidity: ProjectionRigidity::Neutral,
        reason: Some(NormalFormBlockReason::AbstractScrutinee),
    };
    let rigid = NormalTypeExpr::Projection {
        interface,
        member,
        args,
        kind: Kind::Type,
        rigidity: ProjectionRigidity::Rigid,
        reason: Some(NormalFormBlockReason::MissingAssociatedEvidence),
    };

    assert_ne!(neutral, rigid);
    assert_ne!(hash_of(&neutral), hash_of(&rigid));

    let decoded: NormalTypeExpr =
        serde_json::from_str(&serde_json::to_string(&neutral).expect("projection serializes"))
            .expect("projection deserializes");
    assert_eq!(decoded, neutral);
}

#[test]
fn canonical_type_expr_serde_shape_remains_backward_compatible() {
    let canonical = CanonicalTypeExpr::ComputationHeadApp {
        head: TypeComputationHeadId::new(module_identity(), "Append"),
        args: vec![CanonicalTypeExpr::Var("Xs".to_string())],
        kind: Kind::Type,
    };

    let json = serde_json::to_string(&canonical).expect("canonical expression serializes");
    assert!(json.contains("ComputationHeadApp"));
    let decoded: CanonicalTypeExpr =
        serde_json::from_str(&json).expect("canonical expression deserializes");
    assert_eq!(decoded, canonical);
}
