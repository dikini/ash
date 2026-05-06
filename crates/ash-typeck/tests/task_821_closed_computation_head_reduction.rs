use ash_core::kind::Kind;
use ash_core::module_graph::ModuleId;
use ash_core::semantic_summary::{
    DomainConstructorId, ModuleIdentity, ModuleSourceOrigin, SealedDomainId,
};
use ash_core::type_ir::{
    CanonicalTypeExpr, NormalFormBlockReason, NormalTypeExpr, TypeComputationHeadId,
};
use ash_typeck::TypeEnv;
use ash_typeck::normalizer::{
    FixtureDomainConstructorPattern, FixtureEquation, FixtureEquationRegistry, FixturePattern,
    FixtureResultExpr, NormalizationConfig, NormalizationError, NormalizationFuel,
    NormalizationMode, Normalizer,
};

fn module() -> ModuleIdentity {
    ModuleIdentity::new(
        None,
        ModuleId(821),
        vec!["task_821".to_string(), "closed_reduction".to_string()],
        ModuleSourceOrigin::Synthetic {
            reason: "TASK-821 closed computation-head reduction tests".to_string(),
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

fn var_result(name: &str) -> FixtureResultExpr {
    FixtureResultExpr::BoundVar(name.to_string())
}

fn ctor_pattern(name: &str, args: Vec<FixturePattern>) -> FixturePattern {
    FixturePattern::DomainConstructor(Box::new(FixtureDomainConstructorPattern {
        constructor: ctor(name),
        domain: domain(),
        args,
    }))
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

fn nil_expr() -> CanonicalTypeExpr {
    app("NilLiteral", vec![])
}

fn cons_expr(head: &str, tail: CanonicalTypeExpr) -> CanonicalTypeExpr {
    app(
        "ConsLiteral",
        vec![CanonicalTypeExpr::Primitive(head.to_string()), tail],
    )
}

fn nil_normal() -> NormalTypeExpr {
    NormalTypeExpr::DomainConstructorApp {
        constructor: ctor("Nil"),
        domain: domain(),
        args: vec![],
        kind: Kind::Type,
    }
}

fn cons_normal(head: &str, tail: NormalTypeExpr) -> NormalTypeExpr {
    NormalTypeExpr::DomainConstructorApp {
        constructor: ctor("Cons"),
        domain: domain(),
        args: vec![NormalTypeExpr::Primitive(head.to_string()), tail],
        kind: Kind::Type,
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

fn normalize(expr: &CanonicalTypeExpr) -> NormalTypeExpr {
    let env = TypeEnv::new();
    let registry = registry();
    Normalizer::with_registry(&env, &registry)
        .normalize(expr)
        .expect("closed reduction succeeds")
        .normal
}

#[test]
fn append_nil_reduces_to_closed_second_argument_domain_constructor() {
    let expr = app("Append", vec![nil_expr(), cons_expr("B", nil_expr())]);

    assert_eq!(normalize(&expr), cons_normal("B", nil_normal()));
}

#[test]
fn append_cons_nil_to_cons_nil_fully_reduces_recursively() {
    let expr = app(
        "Append",
        vec![cons_expr("A", nil_expr()), cons_expr("B", nil_expr())],
    );

    assert_eq!(
        normalize(&expr),
        cons_normal("A", cons_normal("B", nil_normal()))
    );
}

#[test]
fn no_matching_equation_keeps_neutral_computation_app_with_normalized_args() {
    let expr = app(
        "Append",
        vec![
            CanonicalTypeExpr::Primitive("NotAList".to_string()),
            nil_expr(),
        ],
    );

    assert_eq!(
        normalize(&expr),
        NormalTypeExpr::NeutralComputationApp {
            head: head("Append"),
            args: vec![
                NormalTypeExpr::Primitive("NotAList".to_string()),
                nil_normal()
            ],
            kind: Kind::Type,
            reason: NormalFormBlockReason::Unsupported,
        }
    );
}

#[test]
fn closed_reduction_consumes_fuel_and_reports_exhaustion() {
    let env = TypeEnv::new();
    let registry = registry();
    let normalizer = Normalizer::with_config_and_registry(
        &env,
        NormalizationConfig {
            mode: NormalizationMode::Full,
            fuel: NormalizationFuel::new(1),
            trace: false,
        },
        &registry,
    );

    let err = normalizer
        .normalize(&app("Append", vec![nil_expr(), nil_expr()]))
        .expect_err("fixture reduction must consume fuel rather than loop or succeed for free");

    assert_eq!(
        err,
        NormalizationError::FuelExhausted {
            mode: NormalizationMode::Full,
            remaining: 0,
        }
    );
}

#[test]
fn first_matching_equation_wins_before_later_fallback() {
    let append = head("Append");
    let registry = FixtureEquationRegistry::empty()
        .with_equation(
            FixtureEquation::new(head("NilLiteral"), vec![], ctor_result("Nil", vec![]))
                .expect("nil literal fixture"),
        )
        .expect("register nil literal")
        .with_equation(
            FixtureEquation::new(
                append.clone(),
                vec![ctor_pattern("Nil", vec![]), var_pattern("ys")],
                var_result("ys"),
            )
            .expect("specific nil equation"),
        )
        .expect("register specific nil equation")
        .with_equation(
            FixtureEquation::new(
                append.clone(),
                vec![var_pattern("xs"), var_pattern("ys")],
                ctor_result("Cons", vec![var_result("xs"), var_result("ys")]),
            )
            .expect("fallback equation"),
        )
        .expect("register fallback equation");
    let env = TypeEnv::new();
    let expr = app("Append", vec![nil_expr(), nil_expr()]);

    let normal = Normalizer::with_registry(&env, &registry)
        .normalize(&expr)
        .expect("first match reduction succeeds")
        .normal;

    assert_eq!(normal, nil_normal());
}
