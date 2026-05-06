use ash_core::kind::Kind;
use ash_core::module_graph::ModuleId;
use ash_core::semantic_summary::{
    AssociatedMemberIdentityId, InterfaceIdentityId, ModuleIdentity, ModuleSourceOrigin, TypeDeclId,
};
use ash_core::type_ir::{
    CanonicalTypeExpr, NormalFormBlockReason, NormalTypeExpr, ProjectionRigidity,
    TypeComputationHeadId,
};
use ash_typeck::TypeEnv;
use ash_typeck::normalizer::{
    NormalizationConfig, NormalizationError, NormalizationFuel, NormalizationMode, Normalizer,
};

fn module() -> ModuleIdentity {
    ModuleIdentity::new(
        None,
        ModuleId(819),
        vec!["task_819".to_string(), "normalizer".to_string()],
        ModuleSourceOrigin::Synthetic {
            reason: "TASK-819 normalizer tests".to_string(),
        },
    )
}

fn normalize(expr: &CanonicalTypeExpr) -> NormalTypeExpr {
    let env = TypeEnv::new();
    let normalizer = Normalizer::new(&env);
    normalizer
        .normalize(expr)
        .expect("identity normalization succeeds")
        .normal
}

#[test]
fn identity_normalizes_primitive_and_var() {
    assert_eq!(
        normalize(&CanonicalTypeExpr::Primitive("Int".to_string())),
        NormalTypeExpr::Primitive("Int".to_string())
    );
    assert_eq!(
        normalize(&CanonicalTypeExpr::Var("T".to_string())),
        NormalTypeExpr::Var("T".to_string())
    );
}

#[test]
fn identity_normalizes_nominal_argument_spine() {
    let origin = TypeDeclId::ordinary(module(), "Box");
    let expr = CanonicalTypeExpr::NominalApp {
        origin: origin.clone(),
        visible_name: "VisibleBox".to_string(),
        args: vec![CanonicalTypeExpr::Var("T".to_string())],
        kind: Kind::Type,
    };

    assert_eq!(
        normalize(&expr),
        NormalTypeExpr::NominalApp {
            origin,
            visible_name: "VisibleBox".to_string(),
            args: vec![NormalTypeExpr::Var("T".to_string())],
            kind: Kind::Type,
        }
    );
}

#[test]
fn computation_heads_remain_neutral_without_fixture_equations() {
    let head = TypeComputationHeadId::new(module(), "Append");
    let expr = CanonicalTypeExpr::ComputationHeadApp {
        head: head.clone(),
        args: vec![CanonicalTypeExpr::Var("Xs".to_string())],
        kind: Kind::Type,
    };

    assert_eq!(
        normalize(&expr),
        NormalTypeExpr::NeutralComputationApp {
            head,
            args: vec![NormalTypeExpr::Var("Xs".to_string())],
            kind: Kind::Type,
            reason: NormalFormBlockReason::Unsupported,
        }
    );
}

#[test]
fn projections_preserve_rigidity_and_normalize_arguments() {
    let interface = InterfaceIdentityId::new(module(), "Iterable");
    let member = AssociatedMemberIdentityId::associated_type(interface.clone(), "Item", vec![]);
    let expr = CanonicalTypeExpr::Projection {
        interface: interface.clone(),
        member: member.clone(),
        args: vec![CanonicalTypeExpr::Primitive("String".to_string())],
        kind: Kind::Type,
        rigidity: ProjectionRigidity::Rigid,
    };

    assert_eq!(
        normalize(&expr),
        NormalTypeExpr::Projection {
            interface,
            member,
            args: vec![NormalTypeExpr::Primitive("String".to_string())],
            kind: Kind::Type,
            rigidity: ProjectionRigidity::Rigid,
            reason: Some(NormalFormBlockReason::RigidProjection),
        }
    );
}

#[test]
fn fuel_exhaustion_is_error_not_neutral_stuckness() {
    let env = TypeEnv::new();
    let config = NormalizationConfig {
        mode: NormalizationMode::Full,
        fuel: NormalizationFuel::new(0),
        trace: false,
    };
    let normalizer = Normalizer::with_config(&env, config);

    let err = normalizer
        .normalize(&CanonicalTypeExpr::Var("T".to_string()))
        .expect_err("zero fuel rejects before classifying stuckness");

    assert_eq!(
        err,
        NormalizationError::FuelExhausted {
            mode: NormalizationMode::Full,
            remaining: 0,
        }
    );
}

#[test]
fn config_surface_exposes_weak_head_full_and_demand_modes() {
    assert_eq!(NormalizationConfig::default().mode, NormalizationMode::Full);

    let env = TypeEnv::new();
    for mode in [
        NormalizationMode::WeakHead,
        NormalizationMode::Full,
        NormalizationMode::Demand,
    ] {
        let normalizer = Normalizer::with_config(
            &env,
            NormalizationConfig {
                mode,
                ..NormalizationConfig::default()
            },
        );
        let outcome = normalizer
            .normalize(&CanonicalTypeExpr::Primitive("Bool".to_string()))
            .expect("all skeleton modes use identity conversion for now");
        assert_eq!(outcome.mode, mode);
        assert_eq!(
            outcome.normal,
            NormalTypeExpr::Primitive("Bool".to_string())
        );
    }
}
