use ash_core::kind::Kind;
use ash_core::module_graph::ModuleId;
use ash_core::semantic_summary::{
    AssociatedMemberIdentityId, InterfaceIdentityId, ModuleIdentity, ModuleSourceOrigin,
};
use ash_core::type_ir::{CanonicalTypeExpr, ProjectionRigidity, TypeComputationHeadId};
use ash_typeck::TypeEnv;
use ash_typeck::normalizer::{DefinitionalEqualityResult, Normalizer};

fn module() -> ModuleIdentity {
    ModuleIdentity::new(
        None,
        ModuleId(829),
        vec!["task_829".to_string(), "review_remediation".to_string()],
        ModuleSourceOrigin::Synthetic {
            reason: "TASK-829 Phase 112 review remediation tests".to_string(),
        },
    )
}

fn head(name: &str) -> TypeComputationHeadId {
    TypeComputationHeadId::new(module(), name)
}

fn app(name: &str, args: Vec<CanonicalTypeExpr>) -> CanonicalTypeExpr {
    CanonicalTypeExpr::ComputationHeadApp {
        head: head(name),
        args,
        kind: Kind::Type,
    }
}

fn var(name: &str) -> CanonicalTypeExpr {
    CanonicalTypeExpr::Var(name.to_string())
}

fn interface(name: &str) -> InterfaceIdentityId {
    InterfaceIdentityId::new(module(), name)
}

fn member(interface: InterfaceIdentityId, name: &str) -> AssociatedMemberIdentityId {
    AssociatedMemberIdentityId::associated_type(interface, name, vec![])
}

fn projection(interface_name: &str, member_name: &str) -> CanonicalTypeExpr {
    let interface = interface(interface_name);
    CanonicalTypeExpr::Projection {
        member: member(interface.clone(), member_name),
        interface,
        args: vec![var("T")],
        kind: Kind::Type,
        rigidity: ProjectionRigidity::Neutral,
    }
}

fn defeq(lhs: &CanonicalTypeExpr, rhs: &CanonicalTypeExpr) -> DefinitionalEqualityResult {
    let env = TypeEnv::new();
    Normalizer::new(&env)
        .definitional_equality(lhs, rhs)
        .expect("normalization succeeds")
}

#[test]
fn task_829_different_neutral_computation_heads_are_structural_mismatches_not_blocked() {
    let lhs = app("F", vec![var("X")]);
    let rhs = app("G", vec![var("X")]);

    let result = defeq(&lhs, &rhs);

    assert!(
        matches!(result, DefinitionalEqualityResult::NotEqual { .. }),
        "different neutral computation heads have known unequal identities and do not require inversion: {result:?}"
    );
}

#[test]
fn task_829_different_projection_identities_are_structural_mismatches_not_blocked() {
    let lhs = projection("Iterable", "Item");
    let rhs = projection("Stream", "Element");

    let result = defeq(&lhs, &rhs);

    assert!(
        matches!(result, DefinitionalEqualityResult::NotEqual { .. }),
        "different projection identities are structurally unequal rather than neutrality-blocked: {result:?}"
    );
}

#[test]
fn task_829_different_closed_data_heads_do_not_become_blocked_due_to_neutral_arguments() {
    let box_id = ash_core::semantic_summary::TypeDeclId::ordinary(module(), "Box");
    let option_id = ash_core::semantic_summary::TypeDeclId::ordinary(module(), "Option");
    let lhs = CanonicalTypeExpr::NominalApp {
        origin: box_id,
        visible_name: "Box".to_string(),
        args: vec![app("F", vec![var("X")])],
        kind: Kind::Type,
    };
    let rhs = CanonicalTypeExpr::NominalApp {
        origin: option_id,
        visible_name: "Option".to_string(),
        args: vec![app("F", vec![var("X")])],
        kind: Kind::Type,
    };

    let result = defeq(&lhs, &rhs);

    assert!(
        matches!(result, DefinitionalEqualityResult::NotEqual { .. }),
        "different data heads are known unequal before inspecting neutral arguments: {result:?}"
    );
}
