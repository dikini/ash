//! TASK-836: type-function pattern coverage, overlap, and residual defaults.

use ash_core::ast::Visibility as CoreVisibility;
use ash_core::module_graph::{CrateId, ModuleId};
use ash_core::semantic_summary::{
    DomainConstructorId, DomainConstructorSummary, DomainFieldSummary, ModuleIdentity,
    ModuleSemanticSummary, ModuleSourceOrigin, SealedDomainId, SealedDomainSummary, SourceAnchor,
    SourceOrigin, SummaryVersion,
};
use ash_parser::surface::Definition;
use ash_typeck::TypeEnv;

fn module_identity(id: usize) -> ModuleIdentity {
    ModuleIdentity::new(
        Some(CrateId(836)),
        ModuleId(id),
        vec!["task836".to_string(), format!("m{id}")],
        ModuleSourceOrigin::Synthetic {
            reason: format!("task-836-{id}"),
        },
    )
}

fn anchor(label: &str) -> SourceAnchor {
    SourceAnchor::new(
        SourceOrigin::Synthetic {
            reason: "task-836-test".into(),
        },
        None,
        label,
    )
}

fn type_list_domain(
    module: &ModuleIdentity,
    name: &str,
    nil: &str,
    cons: &str,
) -> SealedDomainSummary {
    let domain = SealedDomainId::new(module.clone(), name);
    let nil = DomainConstructorSummary::new(
        DomainConstructorId::new(domain.clone(), nil),
        nil,
        vec![],
        anchor(nil),
    );
    let cons = DomainConstructorSummary::new(
        DomainConstructorId::new(domain.clone(), cons),
        cons,
        vec![
            DomainFieldSummary::unconstrained("head"),
            DomainFieldSummary::constrained_to("tail", &domain, domain.clone()),
        ],
        anchor(cons),
    );
    SealedDomainSummary::new(domain, name, CoreVisibility::Public, anchor(name))
        .with_constructor(nil)
        .with_constructor(cons)
}

fn lowercase_domain(module: &ModuleIdentity) -> SealedDomainSummary {
    let domain = SealedDomainId::new(module.clone(), "LowerList");
    let nil = DomainConstructorSummary::new(
        DomainConstructorId::new(domain.clone(), "nil"),
        "nil",
        vec![],
        anchor("nil"),
    );
    let cons = DomainConstructorSummary::new(
        DomainConstructorId::new(domain.clone(), "cons"),
        "cons",
        vec![
            DomainFieldSummary::unconstrained("head"),
            DomainFieldSummary::constrained_to("tail", &domain, domain.clone()),
        ],
        anchor("cons"),
    );
    SealedDomainSummary::new(
        domain,
        "LowerList",
        CoreVisibility::Public,
        anchor("LowerList"),
    )
    .with_constructor(nil)
    .with_constructor(cons)
}

fn register_domains(env: &mut TypeEnv, module: &ModuleIdentity) {
    let mut summary = ModuleSemanticSummary::new(module.clone())
        .with_exported_sealed_domain(type_list_domain(module, "TypeList", "Nil", "Cons"))
        .with_exported_sealed_domain(type_list_domain(
            module,
            "OtherList",
            "OtherNil",
            "OtherCons",
        ))
        .with_exported_sealed_domain(lowercase_domain(module));
    summary.version = SummaryVersion::SPEC059_SEALED_DOMAIN_V2;
    env.register_module_semantic_summary(&summary)
        .expect("domains register");
}

fn type_fns(source: &str) -> Vec<ash_parser::surface::TypeFnDef> {
    let parsed = ash_parser::parse_surface_file(source).expect("source parses");
    parsed
        .definitions
        .into_iter()
        .filter_map(|def| match def {
            Definition::TypeFn(type_fn) => Some(type_fn),
            _ => None,
        })
        .collect()
}

fn assert_accepts(source: &str) {
    let module = module_identity(source.len());
    let mut env = TypeEnv::new();
    register_domains(&mut env, &module);
    let defs = type_fns(source);
    env.register_local_type_functions(&module, &defs)
        .expect("definition should accept");
}

fn assert_rejects(source: &str, expected: &str) {
    let module = module_identity(source.len() + expected.len());
    let mut env = TypeEnv::new();
    register_domains(&mut env, &module);
    let defs = type_fns(source);
    let err = env
        .register_local_type_functions(&module, &defs)
        .expect_err("definition should reject");
    let actual = format!("{err}");
    assert!(
        actual.contains(expected),
        "expected diagnostic containing {expected:?}, got {actual}"
    );
}

#[test]
fn rejects_non_exhaustive_head_style_partial_definition() {
    assert_rejects(
        "type fn Head(xs: TypeList) -> Type { case Head<Cons<h, t>> = h; }",
        "non-exhaustive type function 'Head'",
    );
}

#[test]
fn rejects_overlapping_explicit_rows() {
    assert_rejects(
        r#"
        type fn F(xs: TypeList) -> TypeList {
            case F<Nil> = Nil;
            case F<Nil> = Nil;
            case F<Cons<h, t>> = t;
        }
        "#,
        "overlapping type function equation",
    );
}

#[test]
fn rejects_later_explicit_row_after_default() {
    assert_rejects(
        r#"
        type fn F(xs: TypeList) -> TypeList {
            case F<_> = Nil;
            case F<Cons<h, t>> = t;
        }
        "#,
        "unreachable type function equation",
    );
}

#[test]
fn rejects_empty_and_duplicate_defaults() {
    assert_rejects(
        r#"
        type fn F(xs: TypeList) -> TypeList {
            case F<Nil> = Nil;
            case F<Cons<h, t>> = t;
            case F<_> = Nil;
        }
        "#,
        "empty residual default",
    );
    assert_rejects(
        r#"
        type fn G(xs: TypeList) -> TypeList {
            case G<_> = Nil;
            case G<_> = Nil;
        }
        "#,
        "empty residual default",
    );
}

#[test]
fn accepts_positive_multiple_default_residual_rows() {
    assert_accepts(
        r#"
        type fn G(xs: TypeList, ys: TypeList) -> TypeList {
            case G<Nil, _> = Nil;
            case G<_, Nil> = Nil;
            case G<_, _> = Nil;
        }
        "#,
    );
}

#[test]
fn rejects_nested_pattern_coverage_gap() {
    assert_rejects(
        r#"
        type fn TailNil(xs: TypeList) -> TypeList {
            case TailNil<Nil> = Nil;
            case TailNil<Cons<h, Nil>> = Nil;
        }
        "#,
        "non-exhaustive type function 'TailNil'",
    );
}

#[test]
fn accepts_nested_default_residual_row() {
    assert_accepts(
        r#"
        type fn TailKind(xs: TypeList) -> TypeList {
            case TailKind<Nil> = Nil;
            case TailKind<Cons<h, Nil>> = Nil;
            case TailKind<Cons<h, _>> = Nil;
        }
        "#,
    );
}

#[test]
fn rejects_nested_constructor_pattern_inside_type_slot() {
    assert_rejects(
        "type fn F(xs: TypeList) -> TypeList { case F<Cons<Nil, t>> = t; case F<Nil> = Nil; }",
        "constructor pattern 'Nil' requires a sealed-domain position",
    );
}

#[test]
fn lowercase_constructor_names_resolve_as_constructors_not_variables() {
    assert_accepts(
        r#"
        type fn F(xs: LowerList) -> LowerList {
            case F<nil> = nil;
            case F<cons<h, t>> = t;
        }
        "#,
    );
}

#[test]
fn allows_same_pattern_variable_name_in_different_rows() {
    assert_accepts(
        r#"
        type fn F(xs: TypeList) -> TypeList {
            case F<Nil> = Nil;
            case F<Cons<x, xs>> = xs;
        }
        type fn G(xs: TypeList) -> TypeList {
            case G<Nil> = Nil;
            case G<Cons<x, xs>> = xs;
        }
        "#,
    );
}
