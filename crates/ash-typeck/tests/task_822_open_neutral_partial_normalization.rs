use ash_core::kind::Kind;
use ash_core::module_graph::ModuleId;
use ash_core::semantic_summary::{
    AssociatedMemberIdentityId, DomainConstructorId, InterfaceIdentityId, ModuleIdentity,
    ModuleSourceOrigin, SealedDomainId,
};
use ash_core::type_ir::{
    CanonicalTypeExpr, NormalFormBlockReason, NormalTypeExpr, ProjectionRigidity,
    TypeComputationHeadId,
};
use ash_typeck::TypeEnv;
use ash_typeck::normalizer::{
    FixtureDomainConstructorPattern, FixtureEquation, FixtureEquationRegistry, FixturePattern,
    FixtureResultExpr, Normalizer,
};

fn module() -> ModuleIdentity {
    ModuleIdentity::new(
        None,
        ModuleId(822),
        vec!["task_822".to_string(), "open_neutral".to_string()],
        ModuleSourceOrigin::Synthetic {
            reason: "TASK-822 open neutral and partial normalization tests".to_string(),
        },
    )
}

fn head(name: &str) -> TypeComputationHeadId {
    TypeComputationHeadId::new(module(), name)
}

fn domain() -> SealedDomainId {
    SealedDomainId::new(module(), "List")
}

fn ctor(name: &str) -> DomainConstructorId {
    DomainConstructorId::new(domain(), name)
}

fn var_pattern(name: &str) -> FixturePattern {
    FixturePattern::Var(name.to_string())
}

fn ctor_pattern(name: &str, args: Vec<FixturePattern>) -> FixturePattern {
    FixturePattern::DomainConstructor(Box::new(FixtureDomainConstructorPattern {
        constructor: ctor(name),
        domain: domain(),
        args,
    }))
}

fn var_result(name: &str) -> FixtureResultExpr {
    FixtureResultExpr::BoundVar(name.to_string())
}

fn ctor_result(name: &str, args: Vec<FixtureResultExpr>) -> FixtureResultExpr {
    FixtureResultExpr::DomainConstructor {
        constructor: ctor(name),
        domain: domain(),
        args,
        kind: Kind::Type,
    }
}

fn app_result(name: &str, args: Vec<FixtureResultExpr>) -> FixtureResultExpr {
    FixtureResultExpr::ComputationHeadApp {
        head: head(name),
        args,
        kind: Kind::Type,
    }
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

fn nil_expr() -> CanonicalTypeExpr {
    app("NilLiteral", vec![])
}

fn cons_expr(head: CanonicalTypeExpr, tail: CanonicalTypeExpr) -> CanonicalTypeExpr {
    app("ConsLiteral", vec![head, tail])
}

fn nil_normal() -> NormalTypeExpr {
    NormalTypeExpr::DomainConstructorApp {
        constructor: ctor("Nil"),
        domain: domain(),
        args: vec![],
        kind: Kind::Type,
    }
}

fn cons_normal(head: NormalTypeExpr, tail: NormalTypeExpr) -> NormalTypeExpr {
    NormalTypeExpr::DomainConstructorApp {
        constructor: ctor("Cons"),
        domain: domain(),
        args: vec![head, tail],
        kind: Kind::Type,
    }
}

fn primitive(name: &str) -> NormalTypeExpr {
    NormalTypeExpr::Primitive(name.to_string())
}

fn normal_var(name: &str) -> NormalTypeExpr {
    NormalTypeExpr::Var(name.to_string())
}

fn neutral_append(args: Vec<NormalTypeExpr>, reason: NormalFormBlockReason) -> NormalTypeExpr {
    NormalTypeExpr::NeutralComputationApp {
        head: head("Append"),
        args,
        kind: Kind::Type,
        reason,
    }
}

fn interface() -> InterfaceIdentityId {
    InterfaceIdentityId::new(module(), "Iterable")
}

fn member() -> AssociatedMemberIdentityId {
    AssociatedMemberIdentityId::associated_type(interface(), "Item", vec![])
}

fn projection_arg() -> CanonicalTypeExpr {
    CanonicalTypeExpr::Projection {
        interface: interface(),
        member: member(),
        args: vec![var("T")],
        kind: Kind::Type,
        rigidity: ProjectionRigidity::Rigid,
    }
}

fn normalized_projection_arg() -> NormalTypeExpr {
    NormalTypeExpr::Projection {
        interface: interface(),
        member: member(),
        args: vec![normal_var("T")],
        kind: Kind::Type,
        rigidity: ProjectionRigidity::Rigid,
        reason: Some(NormalFormBlockReason::RigidProjection),
    }
}

fn registry() -> FixtureEquationRegistry {
    FixtureEquationRegistry::empty()
        .with_equation(
            FixtureEquation::new(head("NilLiteral"), vec![], ctor_result("Nil", vec![]))
                .expect("nil literal fixture"),
        )
        .expect("register nil literal")
        .with_equation(
            FixtureEquation::new(
                head("ConsLiteral"),
                vec![var_pattern("h"), var_pattern("t")],
                ctor_result("Cons", vec![var_result("h"), var_result("t")]),
            )
            .expect("cons literal fixture"),
        )
        .expect("register cons literal")
        .with_equation(
            FixtureEquation::new(
                head("Append"),
                vec![ctor_pattern("Nil", vec![]), var_pattern("ys")],
                var_result("ys"),
            )
            .expect("append nil fixture"),
        )
        .expect("register append nil")
        .with_equation(
            FixtureEquation::new(
                head("Append"),
                vec![
                    ctor_pattern("Cons", vec![var_pattern("h"), var_pattern("t")]),
                    var_pattern("ys"),
                ],
                ctor_result(
                    "Cons",
                    vec![
                        var_result("h"),
                        app_result("Append", vec![var_result("t"), var_result("ys")]),
                    ],
                ),
            )
            .expect("append cons fixture"),
        )
        .expect("register append cons")
}

fn registry_with_late_catch_all() -> FixtureEquationRegistry {
    registry()
        .with_equation(
            FixtureEquation::new(
                head("Append"),
                vec![var_pattern("xs"), var_pattern("ys")],
                ctor_result("Cons", vec![var_result("xs"), var_result("ys")]),
            )
            .expect("catch-all fixture"),
        )
        .expect("register catch-all fixture")
}

fn registry_with_singleton_catch_all() -> FixtureEquationRegistry {
    FixtureEquationRegistry::empty()
        .with_equation(
            FixtureEquation::new(
                head("Append"),
                vec![var_pattern("xs"), var_pattern("ys")],
                ctor_result("Cons", vec![var_result("xs"), var_result("ys")]),
            )
            .expect("singleton catch-all fixture"),
        )
        .expect("register singleton catch-all fixture")
}

fn registry_with_singleton_constructor_pattern() -> FixtureEquationRegistry {
    FixtureEquationRegistry::empty()
        .with_equation(
            FixtureEquation::new(
                head("Append"),
                vec![ctor_pattern(
                    "Cons",
                    vec![var_pattern("h"), var_pattern("t")],
                )],
                var_result("t"),
            )
            .expect("singleton constructor fixture"),
        )
        .expect("register singleton constructor fixture")
}

fn normalize_with(registry: &FixtureEquationRegistry, expr: &CanonicalTypeExpr) -> NormalTypeExpr {
    let env = TypeEnv::new();
    Normalizer::with_registry(&env, registry)
        .normalize(expr)
        .expect("normalization succeeds")
        .normal
}

fn normalize(expr: &CanonicalTypeExpr) -> NormalTypeExpr {
    let registry = registry();
    normalize_with(&registry, expr)
}

#[test]
fn append_open_variables_stays_neutral_with_abstract_scrutinee_reason() {
    let expr = app("Append", vec![var("Xs"), var("Ys")]);

    assert_eq!(
        normalize(&expr),
        neutral_append(
            vec![normal_var("Xs"), normal_var("Ys")],
            NormalFormBlockReason::AbstractScrutinee,
        )
    );
}

#[test]
fn nested_open_inside_domain_constructor_reduces_known_prefix_and_keeps_neutral_tail() {
    let expr = app(
        "Append",
        vec![
            cons_expr(CanonicalTypeExpr::Primitive("A".to_string()), var("Xs")),
            var("Ys"),
        ],
    );

    assert_eq!(
        normalize(&expr),
        cons_normal(
            primitive("A"),
            neutral_append(
                vec![normal_var("Xs"), normal_var("Ys")],
                NormalFormBlockReason::AbstractScrutinee,
            ),
        )
    );
}

#[test]
fn open_neutral_apps_normalize_argument_spines_before_sticking() {
    let expr = app(
        "Append",
        vec![var("Xs"), cons_expr(projection_arg(), nil_expr())],
    );

    assert_eq!(
        normalize(&expr),
        neutral_append(
            vec![
                normal_var("Xs"),
                cons_normal(normalized_projection_arg(), nil_normal())
            ],
            NormalFormBlockReason::AbstractScrutinee,
        )
    );
}

#[test]
fn partial_prefix_normalization_reduces_append_nil_with_open_suffix_to_suffix() {
    let expr = app("Append", vec![nil_expr(), var("Ys")]);

    assert_eq!(normalize(&expr), normal_var("Ys"));
}

#[test]
fn open_catch_all_equation_does_not_reduce_open_neutral_app() {
    let registry = registry_with_late_catch_all();
    let expr = app("Append", vec![var("Xs"), var("Ys")]);

    assert_eq!(
        normalize_with(&registry, &expr),
        neutral_append(
            vec![normal_var("Xs"), normal_var("Ys")],
            NormalFormBlockReason::AbstractScrutinee,
        )
    );
}

#[test]
fn singleton_open_catch_all_equation_does_not_reduce_open_neutral_app() {
    let registry = registry_with_singleton_catch_all();
    let expr = app("Append", vec![var("Xs"), var("Ys")]);

    assert_eq!(
        normalize_with(&registry, &expr),
        neutral_append(
            vec![normal_var("Xs"), normal_var("Ys")],
            NormalFormBlockReason::AbstractScrutinee,
        )
    );
}

#[test]
fn singleton_constructor_pattern_preserves_abstract_scrutinee_reason() {
    let registry = registry_with_singleton_constructor_pattern();
    let expr = app("Append", vec![var("Xs")]);

    assert_eq!(
        normalize_with(&registry, &expr),
        neutral_append(
            vec![normal_var("Xs")],
            NormalFormBlockReason::AbstractScrutinee,
        )
    );
}
