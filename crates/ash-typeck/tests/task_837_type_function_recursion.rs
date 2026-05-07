//! TASK-837: type-function structural recursion and decreases validation.

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
        Some(CrateId(837)),
        ModuleId(id),
        vec!["task837".to_string(), format!("m{id}")],
        ModuleSourceOrigin::Synthetic {
            reason: format!("task-837-{id}"),
        },
    )
}

fn anchor(label: &str) -> SourceAnchor {
    SourceAnchor::new(
        SourceOrigin::Synthetic {
            reason: "task-837-test".into(),
        },
        None,
        label,
    )
}

fn type_list_domain(module: &ModuleIdentity) -> SealedDomainSummary {
    let domain = SealedDomainId::new(module.clone(), "TypeList");
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
    SealedDomainSummary::new(
        domain,
        "TypeList",
        CoreVisibility::Public,
        anchor("TypeList"),
    )
    .with_constructor(nil)
    .with_constructor(cons)
}

fn flat_domain(module: &ModuleIdentity) -> SealedDomainSummary {
    let domain = SealedDomainId::new(module.clone(), "Flat");
    let z = DomainConstructorSummary::new(
        DomainConstructorId::new(domain.clone(), "Z"),
        "Z",
        vec![],
        anchor("Z"),
    );
    let s = DomainConstructorSummary::new(
        DomainConstructorId::new(domain.clone(), "S"),
        "S",
        vec![DomainFieldSummary::unconstrained("payload")],
        anchor("S"),
    );
    SealedDomainSummary::new(domain, "Flat", CoreVisibility::Public, anchor("Flat"))
        .with_constructor(z)
        .with_constructor(s)
}

fn register_domains(env: &mut TypeEnv, module: &ModuleIdentity) {
    let mut summary = ModuleSemanticSummary::new(module.clone())
        .with_exported_sealed_domain(type_list_domain(module))
        .with_exported_sealed_domain(flat_domain(module));
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
    for def in defs {
        assert!(
            env.lookup_local_type_function(def.name.as_ref()).is_none(),
            "invalid type function {} must not be published",
            def.name
        );
    }
}

#[test]
fn rejects_missing_decreases_on_recursive_function() {
    assert_rejects(
        r#"
        type fn Append(xs: TypeList, ys: TypeList) -> TypeList {
            case Append<Nil, ys> = ys;
            case Append<Cons<h, t>, ys> = Cons<h, Append<t, ys>>;
        }
        "#,
        "missing decreases clause for recursive type function 'Append'",
    );
}

#[test]
fn rejects_non_sealed_decreasing_param() {
    assert_rejects(
        r#"
        type fn Bad(x: Type, xs: TypeList) -> Type decreases x {
            case Bad<x, Nil> = Int;
            case Bad<x, Cons<h, t>> = Bad<x, t>;
        }
        "#,
        "invalid decreases parameter 'x' in type function 'Bad'",
    );
}

#[test]
fn rejects_unknown_decreases_param() {
    assert_rejects(
        "type fn F(xs: TypeList) -> TypeList decreases missing { case F<xs> = xs; }",
        "unknown decreases parameter 'missing' in type function 'F'",
    );
}

#[test]
fn rejects_decreases_param_without_structural_subcomponent_metadata() {
    assert_rejects(
        r#"
        type fn F(xs: Flat) -> Flat decreases xs {
            case F<Z> = Z;
            case F<S<x>> = Z;
        }
        "#,
        "invalid decreases parameter 'xs' in type function 'F'",
    );
}

#[test]
fn accepts_append_like_recursion_on_tail_subcomponent() {
    assert_accepts(
        r#"
        type fn Append(xs: TypeList, ys: TypeList) -> TypeList decreases xs {
            case Append<Nil, ys> = ys;
            case Append<Cons<h, t>, ys> = Cons<h, Append<t, ys>>;
        }
        "#,
    );
}

#[test]
fn rejects_same_argument_recursion() {
    assert_rejects(
        r#"
        type fn Bad(xs: TypeList) -> TypeList decreases xs {
            case Bad<xs> = Bad<xs>;
        }
        "#,
        "non-decreasing recursive call in type function 'Bad'",
    );
}

#[test]
fn rejects_rebuilt_argument_recursion() {
    assert_rejects(
        r#"
        type fn Bad(xs: TypeList) -> TypeList decreases xs {
            case Bad<Nil> = Nil;
            case Bad<Cons<h, t>> = Bad<Cons<h, t>>;
        }
        "#,
        "non-decreasing recursive call in type function 'Bad'",
    );
}

#[test]
fn rejects_alias_or_computed_argument_recursion() {
    assert_rejects(
        r#"
        type fn Bad(xs: TypeList) -> TypeList decreases xs {
            case Bad<Nil> = Nil;
            case Bad<Cons<h, t>> = Bad<Id<t>>;
        }
        type fn Id(xs: TypeList) -> TypeList { case Id<xs> = xs; }
        "#,
        "forward reference to later type function 'Id'",
    );

    assert_rejects(
        r#"
        type fn Id(xs: TypeList) -> TypeList { case Id<xs> = xs; }
        type fn Bad(xs: TypeList) -> TypeList decreases xs {
            case Bad<Nil> = Nil;
            case Bad<Cons<h, t>> = Bad<Id<t>>;
        }
        "#,
        "non-decreasing recursive call in type function 'Bad'",
    );
}

#[test]
fn detects_nested_recursive_calls_under_rhs_children() {
    assert_rejects(
        r#"
        type fn Bad(arg: TypeList) -> TypeList decreases arg {
            case Bad<Nil> = Nil;
            case Bad<Cons<h, t>> = Cons<h, Cons<h, Bad<Cons<h, t>>>>;
        }
        "#,
        "non-decreasing recursive call in type function 'Bad'",
    );
}

#[test]
fn rejects_mutual_recursion_by_source_order() {
    assert_rejects(
        r#"
        type fn A(xs: TypeList) -> TypeList decreases xs {
            case A<Nil> = Nil;
            case A<Cons<h, t>> = B<t>;
        }
        type fn B(xs: TypeList) -> TypeList decreases xs {
            case B<Nil> = Nil;
            case B<Cons<h, t>> = A<t>;
        }
        "#,
        "forward reference to later type function 'B'",
    );
}

#[test]
fn invalid_recursive_heads_are_unpublished_after_structural_failure() {
    let module = module_identity(999);
    let mut env = TypeEnv::new();
    register_domains(&mut env, &module);
    let bad = type_fns(
        r#"
        type fn Bad(xs: TypeList) -> TypeList decreases xs {
            case Bad<xs> = Bad<xs>;
        }
        "#,
    );
    let err = env
        .register_local_type_functions(&module, &bad)
        .expect_err("bad recursion rejects");
    assert!(format!("{err}").contains("non-decreasing recursive call"));
    assert!(env.lookup_local_type_function("Bad").is_none());

    let later = type_fns("type fn Later(xs: TypeList) -> TypeList { case Later<xs> = Bad<xs>; }");
    let err = env
        .register_local_type_functions(&module, &later)
        .expect_err("invalid recursive head remains unpublished");
    assert!(format!("{err}").contains("unresolved type function or type head 'Bad'"));
}
