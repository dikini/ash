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

fn primitive(name: &str) -> CanonicalTypeExpr {
    CanonicalTypeExpr::Primitive(name.to_string())
}

fn interface(name: &str) -> InterfaceIdentityId {
    InterfaceIdentityId::new(module(), name)
}

fn member(interface: InterfaceIdentityId, name: &str) -> AssociatedMemberIdentityId {
    AssociatedMemberIdentityId::associated_type(interface, name, vec![])
}

fn projection_with_args(
    interface_name: &str,
    member_name: &str,
    rigidity: ProjectionRigidity,
    args: Vec<CanonicalTypeExpr>,
) -> CanonicalTypeExpr {
    let interface = interface(interface_name);
    CanonicalTypeExpr::Projection {
        member: member(interface.clone(), member_name),
        interface,
        args,
        kind: Kind::Type,
        rigidity,
    }
}

fn projection(interface_name: &str, member_name: &str) -> CanonicalTypeExpr {
    projection_with_args(
        interface_name,
        member_name,
        ProjectionRigidity::Neutral,
        vec![var("T")],
    )
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

#[test]
fn task_829_projection_rigidity_mismatch_is_structural_not_blocked() {
    let lhs = projection_with_args(
        "Iterable",
        "Item",
        ProjectionRigidity::Rigid,
        vec![var("T")],
    );
    let rhs = projection_with_args(
        "Iterable",
        "Item",
        ProjectionRigidity::Neutral,
        vec![var("T")],
    );

    let result = defeq(&lhs, &rhs);

    assert!(
        matches!(result, DefinitionalEqualityResult::NotEqual { .. }),
        "same projection identity with different rigidity is structurally unequal: {result:?}"
    );
}

#[test]
fn task_829_same_neutral_head_with_closed_arg_mismatch_is_structural_not_blocked() {
    let lhs = app("F", vec![primitive("Int")]);
    let rhs = app("F", vec![primitive("String")]);

    let result = defeq(&lhs, &rhs);

    assert!(
        matches!(result, DefinitionalEqualityResult::NotEqual { .. }),
        "same neutral head with known unequal normalized argument spines is not blocked: {result:?}"
    );
}

#[test]
fn task_829_same_projection_identity_with_closed_arg_mismatch_is_structural_not_blocked() {
    let lhs = projection_with_args(
        "Iterable",
        "Item",
        ProjectionRigidity::Neutral,
        vec![primitive("Int")],
    );
    let rhs = projection_with_args(
        "Iterable",
        "Item",
        ProjectionRigidity::Neutral,
        vec![primitive("String")],
    );

    let result = defeq(&lhs, &rhs);

    assert!(
        matches!(result, DefinitionalEqualityResult::NotEqual { .. }),
        "same projection identity with known unequal normalized argument spines is not blocked: {result:?}"
    );
}

#[test]
fn task_829_same_neutral_head_with_open_vs_closed_arg_remains_blocked() {
    let lhs = app("F", vec![var("X")]);
    let rhs = app("F", vec![primitive("Int")]);

    let result = defeq(&lhs, &rhs);

    assert!(
        matches!(
            result,
            DefinitionalEqualityResult::BlockedByNeutrality { .. }
        ),
        "same neutral head with an open argument mismatch must not solve or classify as known unequal: {result:?}"
    );
}

#[test]
fn task_829_same_projection_identity_with_open_vs_closed_arg_remains_blocked() {
    let lhs = projection_with_args(
        "Iterable",
        "Item",
        ProjectionRigidity::Neutral,
        vec![var("X")],
    );
    let rhs = projection_with_args(
        "Iterable",
        "Item",
        ProjectionRigidity::Neutral,
        vec![primitive("Int")],
    );

    let result = defeq(&lhs, &rhs);

    assert!(
        matches!(
            result,
            DefinitionalEqualityResult::BlockedByNeutrality { .. }
        ),
        "same projection identity with an open argument mismatch must not solve or classify as known unequal: {result:?}"
    );
}
