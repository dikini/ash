//! TASK-851: imported public type-function summaries register computation heads for normalization.

use ash_core::ast::Visibility as CoreVisibility;
use ash_core::kind::Kind;
use ash_core::module_graph::{CrateId, ModuleId};
use ash_core::semantic_summary::{
    DomainConstructorId, DomainConstructorSummary, DomainFieldSummary, ModuleIdentity,
    ModuleSemanticSummary, ModuleSourceOrigin, RepresentationExposure, SealedDomainId,
    SealedDomainSummary, SourceAnchor, SourceOrigin, SummaryVersion, TypeDeclId, TypeDeclSummary,
    TypeRepresentationSummary,
};
use ash_core::type_ir::{
    CanonicalTypeExpr, NormalFormBlockReason, NormalTypeExpr, TypeComputationHeadId,
    TypeFunctionResultConstraint, TypeFunctionResultExpr,
};
use ash_parser::surface::Definition;
use ash_typeck::TypeEnv;
use ash_typeck::normalizer::{DefinitionalEqualityResult, Normalizer};

fn module() -> ModuleIdentity {
    ModuleIdentity::new(
        Some(CrateId(851)),
        ModuleId(1),
        vec!["task851".to_string(), "producer".to_string()],
        ModuleSourceOrigin::Synthetic {
            reason: "task-851 imported type-function normalizer tests".to_string(),
        },
    )
}

fn anchor(label: &str) -> SourceAnchor {
    SourceAnchor::new(
        SourceOrigin::Synthetic {
            reason: "task-851-test".into(),
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
    ModuleSemanticSummary::new(module())
        .with_version(SummaryVersion::SPEC062_TYPE_COMPUTATION_V3)
        .with_exported_sealed_domain(
            SealedDomainSummary::new(
                domain,
                "TypeList",
                CoreVisibility::Public,
                anchor("TypeList"),
            )
            .with_constructor(nil)
            .with_constructor(cons),
        )
}

fn source_type_fns() -> Vec<ash_parser::surface::TypeFnDef> {
    let parsed = ash_parser::parse_surface_file(
        r#"
        pub type fn Id(xs: TypeList) -> TypeList {
            case Id<xs> = xs;
        }

        pub type fn UseId(xs: TypeList) -> TypeList {
            case UseId<xs> = Id<xs>;
        }

        pub type fn Append(xs: TypeList, ys: TypeList) -> TypeList decreases xs {
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

fn source_type_fns_with_late_decreases() -> Vec<ash_parser::surface::TypeFnDef> {
    let parsed = ash_parser::parse_surface_file(
        r#"
        pub type fn DropSecond(skip: TypeList, xs: TypeList) -> TypeList decreases xs {
            case DropSecond<skip, Nil> = skip;
            case DropSecond<skip, Cons<h, t>> = DropSecond<skip, t>;
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

fn exported_late_decreases_summary() -> ModuleSemanticSummary {
    let mut producer = TypeEnv::new();
    producer
        .register_module_semantic_summary(&typelist_summary())
        .expect("producer domain registers");
    producer
        .register_local_type_functions(&module(), &source_type_fns_with_late_decreases())
        .expect("producer type functions register");

    let mut summary = typelist_summary();
    for type_fn in producer
        .export_public_type_function_summaries(&module())
        .expect("public summaries export")
    {
        summary = summary.with_exported_type_function(type_fn);
    }
    summary
}

fn exported_summary() -> ModuleSemanticSummary {
    let mut producer = TypeEnv::new();
    producer
        .register_module_semantic_summary(&typelist_summary())
        .expect("producer domain registers");
    producer
        .register_local_type_functions(&module(), &source_type_fns())
        .expect("producer type functions register");

    let mut summary = typelist_summary();
    for type_fn in producer
        .export_public_type_function_summaries(&module())
        .expect("public summaries export")
    {
        summary = summary.with_exported_type_function(type_fn);
    }
    summary
}

fn imported_env() -> (TypeEnv, ModuleSemanticSummary) {
    let summary = exported_summary();
    let mut env = TypeEnv::new();
    env.register_module_semantic_summaries(std::slice::from_ref(&summary))
        .expect("imported summary batch registers");
    (env, summary)
}

fn summary_head(summary: &ModuleSemanticSummary, name: &str) -> TypeComputationHeadId {
    summary
        .exported_type_functions
        .iter()
        .find(|type_fn| type_fn.exported_name == name)
        .expect("exported type fn exists")
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

fn prim(name: &str) -> NormalTypeExpr {
    NormalTypeExpr::Primitive(name.to_string())
}

fn var(name: &str) -> NormalTypeExpr {
    NormalTypeExpr::Var(name.to_string())
}

fn append(
    env: &TypeEnv,
    summary: &ModuleSemanticSummary,
    xs: NormalTypeExpr,
    ys: NormalTypeExpr,
) -> NormalTypeExpr {
    Normalizer::new(env)
        .normalize_known_computation_app(
            &summary_head(summary, "Append"),
            vec![xs, ys],
            &Kind::Type,
        )
        .expect("normalization succeeds")
}

#[test]
fn imported_public_transparent_type_fn_reduces_closed_sealed_constructor_argument() {
    let (env, summary) = imported_env();
    let ys = cons(prim("B"), nil());

    assert_eq!(append(&env, &summary, nil(), ys.clone()), ys);
}

#[test]
fn definitional_equality_uses_imported_public_reductions() {
    let (env, summary) = imported_env();
    let lhs = append(
        &env,
        &summary,
        cons(prim("A"), nil()),
        cons(prim("B"), nil()),
    );
    let rhs = cons(prim("A"), cons(prim("B"), nil()));

    assert_eq!(
        Normalizer::new(&env).definitional_equality_normal_forms(&lhs, &rhs),
        DefinitionalEqualityResult::Equal
    );
}

#[test]
fn imported_open_argument_remains_neutral_and_stable() {
    let (env, summary) = imported_env();

    assert_eq!(
        append(&env, &summary, var("Xs"), var("Ys")),
        NormalTypeExpr::NeutralComputationApp {
            head: summary_head(&summary, "Append"),
            args: vec![var("Xs"), var("Ys")],
            kind: Kind::Type,
            reason: NormalFormBlockReason::AbstractScrutinee,
        }
    );
}

#[test]
fn imported_dependency_helper_head_reduces_for_public_head_but_is_not_source_visible() {
    let (env, summary) = imported_env();
    assert!(env.lookup_local_type_function("Id").is_none());

    let value = cons(prim("A"), nil());
    let reduced = Normalizer::new(&env)
        .normalize_known_computation_app(
            &summary_head(&summary, "UseId"),
            vec![value.clone()],
            &Kind::Type,
        )
        .expect("normalization succeeds");

    assert_eq!(reduced, value);
}

#[test]
fn selected_imported_type_function_visible_name_lowers_source_rhs_without_helper_leakage() {
    let full = exported_summary();
    let mut selected = full.clone();
    selected.exported_type_functions = full
        .exported_type_functions
        .iter()
        .filter(|summary| matches!(summary.exported_name.as_str(), "Id" | "UseId"))
        .cloned()
        .map(|mut summary| {
            if summary.exported_name == "UseId" {
                summary.exported_name = "AliasUseId".to_string();
            } else {
                summary.exported_name = "$ash_dependency$Id".to_string();
            }
            summary
        })
        .collect();

    let mut env = TypeEnv::new();
    env.register_module_semantic_summaries(std::slice::from_ref(&selected))
        .expect("selected summary registers");
    env.expose_imported_type_function_name("AliasUseId", summary_head(&full, "UseId"))
        .expect("selected imported head becomes source visible");

    assert!(env.lookup_local_type_function("AliasUseId").is_some());
    assert!(env.lookup_local_type_function("Id").is_none());

    let local_module = ModuleIdentity::new(
        Some(CrateId(851)),
        ModuleId(2),
        vec!["task851".to_string(), "consumer".to_string()],
        ModuleSourceOrigin::Synthetic {
            reason: "task-851 imported visible alias consumer".to_string(),
        },
    );
    let parsed = ash_parser::parse_surface_file(
        r#"
        pub type fn Downstream(xs: TypeList) -> TypeList {
            case Downstream<xs> = AliasUseId<xs>;
        }
        "#,
    )
    .expect("consumer source parses");
    let downstream_defs = parsed
        .definitions
        .into_iter()
        .filter_map(|def| match def {
            Definition::TypeFn(type_fn) => Some(type_fn),
            _ => None,
        })
        .collect::<Vec<_>>();
    env.register_local_type_functions(&local_module, &downstream_defs)
        .expect("downstream RHS resolves imported visible type function");

    let downstream_head = env
        .lookup_local_type_function("Downstream")
        .expect("downstream source head is visible")
        .head
        .clone();
    let value = cons(prim("A"), nil());
    let reduced = Normalizer::new(&env)
        .normalize_known_computation_app(&downstream_head, vec![value.clone()], &Kind::Type)
        .expect("downstream normalization succeeds through imported alias");

    assert_eq!(reduced, value);
}

#[test]
fn malformed_v3_unbound_result_variable_is_rejected() {
    let mut malformed = exported_summary();
    let id = malformed
        .exported_type_functions
        .iter_mut()
        .find(|type_fn| type_fn.exported_name == "Id")
        .expect("Id summary exists");
    id.equations[0].result = TypeFunctionResultExpr::Var {
        name: "ghost".to_string(),
        kind: Kind::Type,
        constraint: TypeFunctionResultConstraint::Domain(domain()),
        source_anchor: anchor("forged unbound var"),
    };

    let mut env = TypeEnv::new();
    assert!(
        env.register_module_semantic_summaries(std::slice::from_ref(&malformed))
            .is_err()
    );
}

#[test]
fn malformed_v3_unknown_nominal_dependency_is_rejected() {
    let mut malformed = exported_summary();
    let missing_id = TypeDeclId::ordinary(module(), "Missing");
    let id = malformed
        .exported_type_functions
        .iter_mut()
        .find(|type_fn| type_fn.exported_name == "Id")
        .expect("Id summary exists");
    id.equations[0].result = TypeFunctionResultExpr::NominalApp {
        origin: missing_id,
        visible_name: "Missing".to_string(),
        args: vec![],
        kind: Kind::Type,
        constraint: TypeFunctionResultConstraint::Kind(Kind::Type),
        source_anchor: anchor("forged nominal"),
    };

    let mut env = TypeEnv::new();
    assert!(
        env.register_module_semantic_summaries(std::slice::from_ref(&malformed))
            .is_err()
    );
}

#[test]
fn malformed_v3_unknown_nominal_signature_return_is_rejected() {
    let mut malformed = exported_summary();
    let missing_id = TypeDeclId::ordinary(module(), "Missing");
    let id = malformed
        .exported_type_functions
        .iter_mut()
        .find(|type_fn| type_fn.exported_name == "Id")
        .expect("Id summary exists");
    id.return_type = CanonicalTypeExpr::NominalApp {
        origin: missing_id,
        visible_name: "Missing".to_string(),
        args: vec![],
        kind: Kind::Type,
    };

    let mut env = TypeEnv::new();
    assert!(
        env.register_module_semantic_summaries(std::slice::from_ref(&malformed))
            .is_err(),
        "imported summary with unknown nominal return signature must reject before normalizer use"
    );
}

#[test]
fn malformed_v3_nominal_signature_param_arity_is_rejected() {
    let mut malformed = exported_summary();
    let box_id = TypeDeclId::ordinary(module(), "Box");
    malformed = malformed.with_exported_type(
        TypeDeclSummary::new(
            box_id.clone(),
            "Box",
            CoreVisibility::Public,
            RepresentationExposure::Opaque,
            TypeRepresentationSummary::opaque(false),
            anchor("Box"),
        )
        .with_params(vec!["T".to_string()]),
    );
    let id = malformed
        .exported_type_functions
        .iter_mut()
        .find(|type_fn| type_fn.exported_name == "Id")
        .expect("Id summary exists");
    id.params[0].ty = CanonicalTypeExpr::NominalApp {
        origin: box_id,
        visible_name: "Box".to_string(),
        args: vec![],
        kind: Kind::Type,
    };

    let mut env = TypeEnv::new();
    assert!(
        env.register_module_semantic_summaries(std::slice::from_ref(&malformed))
            .is_err(),
        "imported summary with malformed nominal parameter signature must reject before normalizer use"
    );
}

#[test]
fn malformed_v3_nominal_result_arity_is_rejected() {
    let mut malformed = exported_summary();
    let box_id = TypeDeclId::ordinary(module(), "Box");
    malformed = malformed.with_exported_type(
        TypeDeclSummary::new(
            box_id.clone(),
            "Box",
            CoreVisibility::Public,
            RepresentationExposure::Opaque,
            TypeRepresentationSummary::opaque(false),
            anchor("Box"),
        )
        .with_params(vec!["T".to_string()]),
    );
    let id = malformed
        .exported_type_functions
        .iter_mut()
        .find(|type_fn| type_fn.exported_name == "Id")
        .expect("Id summary exists");
    id.equations[0].result = TypeFunctionResultExpr::NominalApp {
        origin: box_id,
        visible_name: "Box".to_string(),
        args: vec![
            TypeFunctionResultExpr::Primitive {
                name: "Int".to_string(),
                kind: Kind::Type,
                constraint: TypeFunctionResultConstraint::Kind(Kind::Type),
                source_anchor: anchor("Int"),
            },
            TypeFunctionResultExpr::Primitive {
                name: "String".to_string(),
                kind: Kind::Type,
                constraint: TypeFunctionResultConstraint::Kind(Kind::Type),
                source_anchor: anchor("String"),
            },
        ],
        kind: Kind::Type,
        constraint: TypeFunctionResultConstraint::Kind(Kind::Type),
        source_anchor: anchor("forged arity"),
    };

    let mut env = TypeEnv::new();
    assert!(
        env.register_module_semantic_summaries(std::slice::from_ref(&malformed))
            .is_err()
    );
}

#[test]
fn imported_recursive_summary_preserves_late_decreases_parameter() {
    let summary = exported_late_decreases_summary();
    let exported = summary
        .exported_type_functions
        .iter()
        .find(|type_fn| type_fn.exported_name == "DropSecond")
        .expect("DropSecond summary exists");
    assert_eq!(
        exported.revalidation_metadata.decreases_param.as_deref(),
        Some("xs")
    );

    let mut env = TypeEnv::new();
    env.register_module_semantic_summaries(std::slice::from_ref(&summary))
        .expect("late decreases parameter remains valid after import");
}

#[test]
fn malformed_or_unsupported_summary_is_rejected_without_partial_computation_registration() {
    for version in [
        SummaryVersion::SPEC057_ORDINARY_TYPE_V1,
        SummaryVersion::SPEC059_SEALED_DOMAIN_V2,
        SummaryVersion(99),
    ] {
        let good = exported_summary();
        let head = summary_head(&good, "Id");
        let malformed = ModuleSemanticSummary { version, ..good };
        let mut env = TypeEnv::new();

        assert!(
            env.register_module_semantic_summaries(std::slice::from_ref(&malformed))
                .is_err()
        );

        assert_eq!(
            Normalizer::new(&env)
                .normalize_known_computation_app(&head, vec![nil()], &Kind::Type)
                .expect("normalization succeeds"),
            NormalTypeExpr::NeutralComputationApp {
                head,
                args: vec![nil()],
                kind: Kind::Type,
                reason: NormalFormBlockReason::Unsupported,
            }
        );
    }
}
