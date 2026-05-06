use ash_core::kind::Kind;
use ash_core::module_graph::ModuleId;
use ash_core::semantic_summary::{
    DomainConstructorId, ModuleIdentity, ModuleSourceOrigin, SealedDomainId,
};
use ash_core::type_ir::{
    CanonicalTypeExpr, NormalFormBlockReason, NormalTypeExpr, TypeComputationHeadId,
};
use ash_typeck::normalizer::{
    DefinitionalEqualityResult, FixtureDomainConstructorPattern, FixtureEquation,
    FixtureEquationRegistry, FixturePattern, FixtureResultExpr, Normalizer,
};
use ash_typeck::types::{Type, TypeVar, unify};
use ash_typeck::{QualifiedName, TypeEnv};

fn module() -> ModuleIdentity {
    ModuleIdentity::new(
        None,
        ModuleId(825),
        vec!["task_825".to_string(), "non_inverting_boundary".to_string()],
        ModuleSourceOrigin::Synthetic {
            reason: "TASK-825 non-inverting unification boundary tests".to_string(),
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

fn primitive(name: &str) -> CanonicalTypeExpr {
    CanonicalTypeExpr::Primitive(name.to_string())
}

fn nil_expr() -> CanonicalTypeExpr {
    app("NilLiteral", vec![])
}

fn cons_expr(head_expr: CanonicalTypeExpr, tail: CanonicalTypeExpr) -> CanonicalTypeExpr {
    app("ConsLiteral", vec![head_expr, tail])
}

fn nil_normal() -> NormalTypeExpr {
    NormalTypeExpr::DomainConstructorApp {
        constructor: ctor("Nil"),
        domain: domain(),
        args: vec![],
        kind: Kind::Type,
    }
}

fn cons_normal(head_expr: NormalTypeExpr, tail: NormalTypeExpr) -> NormalTypeExpr {
    NormalTypeExpr::DomainConstructorApp {
        constructor: ctor("Cons"),
        domain: domain(),
        args: vec![head_expr, tail],
        kind: Kind::Type,
    }
}

fn neutral_append_normal(xs: &str, ys: NormalTypeExpr) -> NormalTypeExpr {
    NormalTypeExpr::NeutralComputationApp {
        head: head("Append"),
        args: vec![NormalTypeExpr::Var(xs.to_string()), ys],
        kind: Kind::Type,
        reason: Some(NormalFormBlockReason::AbstractScrutinee),
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

fn defeq(lhs: &CanonicalTypeExpr, rhs: &CanonicalTypeExpr) -> DefinitionalEqualityResult {
    let env = TypeEnv::new();
    let registry = registry();
    Normalizer::with_registry(&env, &registry)
        .definitional_equality(lhs, rhs)
        .expect("definitional equality normalizes")
}

#[test]
fn task_825_same_headed_neutral_apps_do_not_unify_by_inverting_arguments() {
    // `CanonicalTypeExpr::Var(String)` is an abstract canonical variable. It is
    // not today's inference meta (`Type::Var(TypeVar)`) and definitional equality
    // must not solve it merely because it appears under a neutral computation head.
    let lhs = app("F", vec![var("X")]);
    let rhs = app("F", vec![var("Y")]);

    let result = defeq(&lhs, &rhs);

    match result {
        DefinitionalEqualityResult::BlockedByNeutrality {
            lhs_norm,
            rhs_norm,
            neutral_subterms,
            no_inversion_note,
        } => {
            assert_eq!(
                lhs_norm,
                NormalTypeExpr::NeutralComputationApp {
                    head: head("F"),
                    args: vec![NormalTypeExpr::Var("X".to_string())],
                    kind: Kind::Type,
                    reason: Some(NormalFormBlockReason::Unsupported),
                }
            );
            assert_eq!(
                rhs_norm,
                NormalTypeExpr::NeutralComputationApp {
                    head: head("F"),
                    args: vec![NormalTypeExpr::Var("Y".to_string())],
                    kind: Kind::Type,
                    reason: Some(NormalFormBlockReason::Unsupported),
                }
            );
            assert_eq!(neutral_subterms, vec![lhs_norm, rhs_norm]);
            assert!(no_inversion_note.contains("does not invert"));
            assert!(no_inversion_note.contains("neutral computation heads"));
        }
        other => panic!("expected neutrality-blocked non-inversion evidence, got {other:?}"),
    }
}

#[test]
fn task_825_append_output_shape_does_not_solve_open_inputs() {
    // `Append<Xs, Ys> == Cons<A, Nil>` is intentionally blocked: deciding it
    // would require inverting Append's output to infer facts about Xs/Ys.
    let lhs = app("Append", vec![var("Xs"), var("Ys")]);
    let rhs = cons_expr(primitive("A"), nil_expr());

    let result = defeq(&lhs, &rhs);

    match result {
        DefinitionalEqualityResult::BlockedByNeutrality {
            lhs_norm,
            rhs_norm,
            neutral_subterms,
            no_inversion_note,
        } => {
            assert_eq!(
                lhs_norm,
                neutral_append_normal("Xs", NormalTypeExpr::Var("Ys".to_string()))
            );
            assert_eq!(
                rhs_norm,
                cons_normal(NormalTypeExpr::Primitive("A".to_string()), nil_normal())
            );
            assert_eq!(neutral_subterms, vec![lhs_norm]);
            assert!(no_inversion_note.contains("does not invert"));
        }
        other => panic!("expected Append mismatch to be blocked by neutrality, got {other:?}"),
    }
}

#[test]
fn task_825_same_headed_neutral_apps_compare_equal_only_with_equal_argument_spines() {
    // Same-headed neutral applications are structural data for comparison, not a
    // solver boundary. Equal spines compare equal; differing spines do not cause
    // argument inference beneath the neutral head.
    let lhs = app("F", vec![var("X"), nil_expr()]);
    let rhs = app("F", vec![var("X"), nil_expr()]);

    assert_eq!(defeq(&lhs, &rhs), DefinitionalEqualityResult::Equal);
}

#[test]
fn task_825_legacy_nominal_unification_boundary_remains_unchanged() {
    // Existing `Type` unification still owns inference-meta solving and ordinary
    // nominal constructor decomposition. This is intentionally separate from the
    // canonical normalizer API above: `Type::Var(TypeVar)` is the current inference
    // meta carrier, unlike `CanonicalTypeExpr::Var(String)`.
    let meta = TypeVar(825);
    let lhs = Type::Constructor {
        name: QualifiedName::root("Box"),
        args: vec![Type::Var(meta)],
        kind: Kind::Type,
    };
    let rhs = Type::Constructor {
        name: QualifiedName::root("Box"),
        args: vec![Type::Int],
        kind: Kind::Type,
    };

    let substitution = unify(&lhs, &rhs).expect("same-headed nominal constructors decompose");

    assert_eq!(substitution.get(meta), Some(&Type::Int));
}
