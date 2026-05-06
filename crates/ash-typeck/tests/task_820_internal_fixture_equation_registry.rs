use ash_core::kind::Kind;
use ash_core::module_graph::ModuleId;
use ash_core::semantic_summary::{
    DomainConstructorId, ModuleIdentity, ModuleSemanticSummary, ModuleSourceOrigin, SealedDomainId,
};
use ash_core::type_ir::{
    CanonicalTypeExpr, NormalFormBlockReason, NormalTypeExpr, TypeComputationHeadId,
};
use ash_typeck::TypeEnv;
use ash_typeck::normalizer::{
    FixtureDomainConstructorPattern, FixtureEquation, FixtureEquationRegistry,
    FixtureEquationRegistryError, FixturePattern, FixtureResultExpr, Normalizer,
};

fn module() -> ModuleIdentity {
    ModuleIdentity::new(
        None,
        ModuleId(820),
        vec!["task_820".to_string(), "fixture_registry".to_string()],
        ModuleSourceOrigin::Synthetic {
            reason: "TASK-820 fixture registry tests".to_string(),
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

#[test]
fn default_registry_is_empty_and_normalizer_constructors_expose_explicit_registry() {
    let env = TypeEnv::new();
    let empty = FixtureEquationRegistry::empty();

    assert!(empty.is_empty());
    assert_eq!(empty.equations_for(&head("Append")).count(), 0);

    let normalizer = Normalizer::new(&env);
    assert!(normalizer.fixture_registry().is_empty());

    let with_registry = Normalizer::with_registry(&env, &empty);
    assert!(with_registry.fixture_registry().is_empty());
}

#[test]
fn registry_is_keyed_by_computation_head_and_preserves_insertion_order_and_arity() {
    let append = head("Append");
    let map = head("Map");
    let nil = ctor("Nil");
    let cons = ctor("Cons");

    let nil_case = FixtureEquation::new(
        append.clone(),
        vec![
            FixturePattern::DomainConstructor(Box::new(FixtureDomainConstructorPattern {
                constructor: nil.clone(),
                domain: domain(),
                args: vec![],
            })),
            var_pattern("ys"),
        ],
        var_result("ys"),
    )
    .expect("well-formed Nil case");
    let cons_case = FixtureEquation::new(
        append.clone(),
        vec![
            FixturePattern::DomainConstructor(Box::new(FixtureDomainConstructorPattern {
                constructor: cons.clone(),
                domain: domain(),
                args: vec![var_pattern("h"), var_pattern("t")],
            })),
            var_pattern("ys"),
        ],
        FixtureResultExpr::DomainConstructor {
            constructor: cons.clone(),
            domain: domain(),
            args: vec![
                var_result("h"),
                FixtureResultExpr::ComputationHeadApp {
                    head: append.clone(),
                    args: vec![var_result("t"), var_result("ys")],
                    kind: Kind::Type,
                },
            ],
            kind: Kind::Type,
        },
    )
    .expect("well-formed Cons case");
    let map_case = FixtureEquation::new(map.clone(), vec![var_pattern("xs")], var_result("xs"))
        .expect("well-formed Map case");

    let registry = FixtureEquationRegistry::empty()
        .with_equation(nil_case.clone())
        .expect("register Nil case")
        .with_equation(cons_case.clone())
        .expect("register Cons case")
        .with_equation(map_case.clone())
        .expect("register Map case");

    let append_equations = registry.equations_for(&append).collect::<Vec<_>>();
    assert_eq!(append_equations, vec![&nil_case, &cons_case]);
    assert_eq!(append_equations[0].arity(), 2);
    assert_eq!(append_equations[1].arity(), 2);
    assert_eq!(
        registry.equations_for(&map).collect::<Vec<_>>(),
        vec![&map_case]
    );
}

#[test]
fn registry_reports_deterministic_structural_pattern_matches_without_reducing() {
    let append = head("Append");
    let nil = ctor("Nil");
    let nil_case = FixtureEquation::new(
        append.clone(),
        vec![
            FixturePattern::DomainConstructor(Box::new(FixtureDomainConstructorPattern {
                constructor: nil.clone(),
                domain: domain(),
                args: vec![],
            })),
            var_pattern("ys"),
        ],
        var_result("ys"),
    )
    .expect("well-formed Nil case");
    let fallback = FixtureEquation::new(
        append.clone(),
        vec![var_pattern("xs"), var_pattern("ys")],
        var_result("ys"),
    )
    .expect("well-formed fallback");
    let registry = FixtureEquationRegistry::empty()
        .with_equation(nil_case.clone())
        .expect("register Nil case")
        .with_equation(fallback)
        .expect("register fallback");

    let matched = registry
        .first_match(
            &append,
            &[
                NormalTypeExpr::DomainConstructorApp {
                    constructor: nil,
                    domain: domain(),
                    args: vec![],
                    kind: Kind::Type,
                },
                NormalTypeExpr::Var("Ys".to_string()),
            ],
        )
        .expect("Nil case is first structural match");

    assert_eq!(matched.equation(), &nil_case);
    assert_eq!(
        matched.bindings().get("ys"),
        Some(&NormalTypeExpr::Var("Ys".to_string()))
    );
}

#[test]
fn registry_rejects_duplicate_patterns_malformed_variables_and_arity_mismatch() {
    let append = head("Append");
    let first = FixtureEquation::new(append.clone(), vec![var_pattern("xs")], var_result("xs"))
        .expect("well-formed first equation");
    let duplicate = FixtureEquation::new(append.clone(), vec![var_pattern("xs")], var_result("xs"))
        .expect("well-formed duplicate shape");

    let err = FixtureEquationRegistry::empty()
        .with_equation(first)
        .expect("register first")
        .with_equation(duplicate)
        .expect_err("duplicate pattern is rejected");
    assert_eq!(
        err,
        FixtureEquationRegistryError::DuplicateEquation {
            head: append.clone()
        }
    );

    let malformed = FixtureEquation::new(
        append.clone(),
        vec![var_pattern("xs")],
        var_result("missing"),
    )
    .expect_err("result may only reference bound pattern variables");
    assert_eq!(
        malformed,
        FixtureEquationRegistryError::UnboundResultVariable {
            variable: "missing".to_string()
        }
    );

    let arity_err = FixtureEquationRegistry::empty()
        .with_equation(
            FixtureEquation::new(append.clone(), vec![var_pattern("xs")], var_result("xs"))
                .unwrap(),
        )
        .expect("register arity one")
        .with_equation(
            FixtureEquation::new(
                append.clone(),
                vec![var_pattern("xs"), var_pattern("ys")],
                var_result("ys"),
            )
            .unwrap(),
        )
        .expect_err("same head must keep deterministic arity");
    assert_eq!(
        arity_err,
        FixtureEquationRegistryError::ArityMismatch {
            head: append,
            expected: 1,
            actual: 2,
        }
    );
}

#[test]
fn fixture_registry_is_not_serialized_in_module_summaries_and_does_not_change_normalization_yet() {
    let env = TypeEnv::new();
    let append = head("Append");
    let registry = FixtureEquationRegistry::empty()
        .with_equation(
            FixtureEquation::new(append.clone(), vec![var_pattern("xs")], var_result("xs"))
                .expect("well-formed identity fixture"),
        )
        .expect("register fixture");

    let summary = ModuleSemanticSummary::new(module());
    assert!(summary.exported_types.is_empty());
    assert!(summary.exported_constructors.is_empty());
    assert!(summary.exported_sealed_domains.is_empty());

    let expr = CanonicalTypeExpr::ComputationHeadApp {
        head: append.clone(),
        args: vec![CanonicalTypeExpr::Var("Xs".to_string())],
        kind: Kind::Type,
    };
    let normalizer = Normalizer::with_registry(&env, &registry);
    let outcome = normalizer
        .normalize(&expr)
        .expect("TASK-820 does not apply fixture reductions yet");

    assert_eq!(
        outcome.normal,
        NormalTypeExpr::NeutralComputationApp {
            head: append,
            args: vec![NormalTypeExpr::Var("Xs".to_string())],
            kind: Kind::Type,
            reason: NormalFormBlockReason::AbstractScrutinee,
        }
    );
}
