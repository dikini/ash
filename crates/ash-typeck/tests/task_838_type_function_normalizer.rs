//! TASK-838: source-backed type-function equations feed the normalizer.

use ash_core::ast::Visibility as CoreVisibility;
use ash_core::kind::Kind;
use ash_core::module_graph::{CrateId, ModuleId};
use ash_core::semantic_summary::{
    DomainConstructorId, DomainConstructorSummary, DomainFieldSummary, ModuleIdentity,
    ModuleSemanticSummary, ModuleSourceOrigin, SealedDomainId, SealedDomainSummary, SourceAnchor,
    SourceOrigin, SummaryVersion,
};
use ash_core::type_ir::{NormalFormBlockReason, NormalTypeExpr, TypeComputationHeadId};
use ash_parser::surface::Definition;
use ash_typeck::TypeEnv;
use ash_typeck::normalizer::{DefinitionalEqualityResult, Normalizer};

fn module() -> ModuleIdentity {
    ModuleIdentity::new(
        Some(CrateId(838)),
        ModuleId(1),
        vec!["task838".to_string(), "source_normalizer".to_string()],
        ModuleSourceOrigin::Synthetic {
            reason: "task-838 source-backed normalizer tests".to_string(),
        },
    )
}

fn anchor(label: &str) -> SourceAnchor {
    SourceAnchor::new(
        SourceOrigin::Synthetic {
            reason: "task-838-test".into(),
        },
        None,
        label,
    )
}

fn domain() -> SealedDomainId {
    SealedDomainId::new(module(), "TypeList")
}

fn ctor(name: &str) -> DomainConstructorId {
    DomainConstructorId::new(domain(), name)
}

fn typelist_summary() -> ModuleSemanticSummary {
    let domain = domain();
    let nil = DomainConstructorSummary::new(
        DomainConstructorId::new(domain.clone(), "Nil"),
        "Nil",
        vec![],
        anchor("Nil"),
    );
    let cons = DomainConstructorSummary::new(
        DomainConstructorId::new(domain.clone(), "Cons"),
        "Cons",
        vec![
            DomainFieldSummary::unconstrained("head"),
            DomainFieldSummary::constrained_to("tail", &domain, domain.clone()),
        ],
        anchor("Cons"),
    );
    let mut summary = ModuleSemanticSummary::new(module()).with_exported_sealed_domain(
        SealedDomainSummary::new(
            domain,
            "TypeList",
            CoreVisibility::Public,
            anchor("TypeList"),
        )
        .with_constructor(nil)
        .with_constructor(cons),
    );
    summary.version = SummaryVersion::SPEC059_SEALED_DOMAIN_V2;
    summary
}

fn source_type_fns() -> Vec<ash_parser::surface::TypeFnDef> {
    let parsed = ash_parser::parse_surface_file(
        r#"
        type fn Append(xs: TypeList, ys: TypeList) -> TypeList decreases xs {
            case Append<Nil, ys> = ys;
            case Append<Cons<h, t>, ys> = Cons<h, Append<t, ys>>;
        }
        "#,
    )
    .expect("source parses");
    parsed
        .definitions
        .into_iter()
        .filter_map(|def| match def {
            Definition::TypeFn(type_fn) => Some(type_fn),
            _ => None,
        })
        .collect()
}

fn env_with_source_append() -> TypeEnv {
    let mut env = TypeEnv::new();
    env.register_module_semantic_summary(&typelist_summary())
        .expect("sealed domain registers");
    env.register_local_type_functions(&module(), &source_type_fns())
        .expect("source type functions validate and publish");
    env
}

fn head(env: &TypeEnv, name: &str) -> TypeComputationHeadId {
    env.lookup_local_type_function(name)
        .expect("type function exists")
        .head
        .clone()
}

fn nil() -> NormalTypeExpr {
    NormalTypeExpr::DomainConstructorApp {
        constructor: ctor("Nil"),
        domain: domain(),
        args: vec![],
        kind: Kind::Type,
    }
}

fn cons(head: NormalTypeExpr, tail: NormalTypeExpr) -> NormalTypeExpr {
    NormalTypeExpr::DomainConstructorApp {
        constructor: ctor("Cons"),
        domain: domain(),
        args: vec![head, tail],
        kind: Kind::Type,
    }
}

fn var(name: &str) -> NormalTypeExpr {
    NormalTypeExpr::Var(name.to_string())
}

fn prim(name: &str) -> NormalTypeExpr {
    NormalTypeExpr::Primitive(name.to_string())
}

fn append(env: &TypeEnv, xs: NormalTypeExpr, ys: NormalTypeExpr) -> NormalTypeExpr {
    Normalizer::new(env)
        .normalize_known_computation_app(&head(env, "Append"), vec![xs, ys], &Kind::Type)
        .expect("normalization succeeds")
}

#[test]
fn source_append_nil_case_reduces_and_substitutes_ys() {
    let env = env_with_source_append();
    let ys = cons(prim("B"), nil());

    assert_eq!(append(&env, nil(), ys.clone()), ys);
}

#[test]
fn source_append_cons_case_reduces_and_substitutes_h_t_and_ys() {
    let env = env_with_source_append();
    let ys = cons(prim("B"), nil());

    assert_eq!(
        append(&env, cons(prim("A"), nil()), ys.clone()),
        cons(prim("A"), ys)
    );
}

#[test]
fn source_append_nested_recursive_known_reduction() {
    let env = env_with_source_append();
    let xs = cons(prim("A"), cons(prim("B"), nil()));
    let ys = cons(prim("C"), nil());

    assert_eq!(
        append(&env, xs, ys),
        cons(prim("A"), cons(prim("B"), cons(prim("C"), nil())))
    );
}

#[test]
fn source_append_open_scrutinee_stays_neutral() {
    let env = env_with_source_append();

    assert_eq!(
        append(&env, var("Xs"), var("Ys")),
        NormalTypeExpr::NeutralComputationApp {
            head: head(&env, "Append"),
            args: vec![var("Xs"), var("Ys")],
            kind: Kind::Type,
            reason: NormalFormBlockReason::AbstractScrutinee,
        }
    );
}

#[test]
fn source_append_partial_open_reduction_preserves_neutral_tail() {
    let env = env_with_source_append();

    assert_eq!(
        append(&env, cons(prim("A"), var("Xs")), var("Ys")),
        cons(
            prim("A"),
            NormalTypeExpr::NeutralComputationApp {
                head: head(&env, "Append"),
                args: vec![var("Xs"), var("Ys")],
                kind: Kind::Type,
                reason: NormalFormBlockReason::AbstractScrutinee,
            }
        )
    );
}

#[test]
fn definitional_equality_uses_source_declaration_reduction() {
    let env = env_with_source_append();
    let lhs = append(&env, cons(prim("A"), nil()), cons(prim("B"), nil()));
    let rhs = cons(prim("A"), cons(prim("B"), nil()));

    assert_eq!(
        Normalizer::new(&env).definitional_equality_normal_forms(&lhs, &rhs),
        DefinitionalEqualityResult::Equal
    );
}
