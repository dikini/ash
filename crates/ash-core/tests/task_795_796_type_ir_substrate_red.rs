use ash_core::kind::Kind;
use ash_core::semantic_summary::{
    AssociatedMemberIdentityId, InterfaceIdentityId, ModuleIdentity, ModuleSourceOrigin, TypeDeclId,
};
use ash_core::type_ir::{CanonicalTypeExpr, ProjectionRigidity, TypeComputationHeadId};
use ash_core::{module_graph::CrateId, module_graph::ModuleId};

fn module_identity() -> ModuleIdentity {
    ModuleIdentity::new(
        Some(CrateId(7)),
        ModuleId(11),
        vec!["demo".to_string(), "types".to_string()],
        ModuleSourceOrigin::Synthetic {
            reason: "phase-110 red test".to_string(),
        },
    )
}

fn type_decl(name: &str) -> TypeDeclId {
    TypeDeclId::ordinary(module_identity(), name)
}

fn interface_identity(name: &str) -> InterfaceIdentityId {
    InterfaceIdentityId::new(module_identity(), name)
}

#[test]
fn shared_kind_in_ash_core_supports_existing_arity_helpers() {
    assert_eq!(Kind::Type.arity(), 0);
    assert_eq!(Kind::n_ary(2).to_string(), "* -> * -> *");
}

#[test]
fn canonical_type_expr_distinguishes_nominal_projection_and_computation_heads() {
    let nominal = CanonicalTypeExpr::NominalApp {
        origin: type_decl("List"),
        visible_name: "List".to_string(),
        args: vec![CanonicalTypeExpr::Var("T".to_string())],
        kind: Kind::Type,
    };
    let projection = CanonicalTypeExpr::Projection {
        interface: interface_identity("Iterable"),
        member: AssociatedMemberIdentityId::associated_type(
            interface_identity("Iterable"),
            "Item",
            vec!["Item".to_string()],
        ),
        args: vec![CanonicalTypeExpr::Var("S".to_string())],
        kind: Kind::Type,
        rigidity: ProjectionRigidity::Rigid,
    };
    let computation = CanonicalTypeExpr::ComputationHeadApp {
        head: TypeComputationHeadId::new(module_identity(), "MapResult"),
        args: vec![CanonicalTypeExpr::Var("S".to_string())],
        kind: Kind::Type,
    };

    assert_ne!(nominal, projection);
    assert_ne!(nominal, computation);
    assert_ne!(projection, computation);
}

#[test]
fn canonical_projection_preserves_declaring_interface_member_and_argument_spine() {
    let interface = interface_identity("MapLike");
    let projection = CanonicalTypeExpr::Projection {
        interface: interface.clone(),
        member: AssociatedMemberIdentityId::associated_type(
            interface.clone(),
            "Entry",
            vec!["Entry".to_string()],
        ),
        args: vec![
            CanonicalTypeExpr::Var("K".to_string()),
            CanonicalTypeExpr::Var("V".to_string()),
        ],
        kind: Kind::Type,
        rigidity: ProjectionRigidity::Rigid,
    };

    match projection {
        CanonicalTypeExpr::Projection {
            interface: actual_interface,
            member,
            args,
            rigidity,
            ..
        } => {
            assert_eq!(actual_interface, interface);
            assert_eq!(member.name, "Entry");
            assert_eq!(args.len(), 2);
            assert!(matches!(rigidity, ProjectionRigidity::Rigid));
        }
        other => panic!("expected canonical projection node, got {other:?}"),
    }
}
