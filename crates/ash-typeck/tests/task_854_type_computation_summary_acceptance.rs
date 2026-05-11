//! TASK-854: SPEC-062 acceptance/non-interference focused typeck checks.

use ash_core::ast::{TypeBody, TypeDef, Visibility as CoreVisibility};
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
use ash_typeck::normalizer::Normalizer;

fn module(id: usize) -> ModuleIdentity {
    ModuleIdentity::new(
        Some(CrateId(854)),
        ModuleId(id),
        vec!["task854".to_string(), format!("m{id}")],
        ModuleSourceOrigin::Synthetic {
            reason: format!("task-854-{id}"),
        },
    )
}

fn anchor(label: &str) -> SourceAnchor {
    SourceAnchor::new(
        SourceOrigin::Synthetic {
            reason: "task-854-acceptance".into(),
        },
        None,
        label,
    )
}

fn domain(module: &ModuleIdentity, name: &str) -> SealedDomainId {
    SealedDomainId::new(module.clone(), name)
}

fn ctor(domain: &SealedDomainId, name: &str) -> DomainConstructorId {
    DomainConstructorId::new(domain.clone(), name)
}

fn list_domain_summary(
    module: &ModuleIdentity,
    name: &str,
    nil: &str,
    cons: &str,
    visibility: CoreVisibility,
) -> SealedDomainSummary {
    let domain = domain(module, name);
    SealedDomainSummary::new(domain.clone(), name, visibility, anchor(name))
        .with_constructor(DomainConstructorSummary::new(
            ctor(&domain, nil),
            nil,
            vec![],
            anchor(nil),
        ))
        .with_constructor(DomainConstructorSummary::new(
            ctor(&domain, cons),
            cons,
            vec![
                DomainFieldSummary::unconstrained("head"),
                DomainFieldSummary::constrained_to("tail", &domain, domain.clone()),
            ],
            anchor(cons),
        ))
}

fn typelist_summary(module: &ModuleIdentity) -> ModuleSemanticSummary {
    ModuleSemanticSummary::new(module.clone())
        .with_version(SummaryVersion::SPEC062_TYPE_COMPUTATION_V3)
        .with_exported_sealed_domain(list_domain_summary(
            module,
            "TypeList",
            "Nil",
            "Cons",
            CoreVisibility::Public,
        ))
}

fn parse_type_fns(source: &str) -> Vec<ash_parser::surface::TypeFnDef> {
    ash_parser::parse_surface_file(source)
        .expect("source parses")
        .definitions
        .into_iter()
        .filter_map(|def| match def {
            Definition::TypeFn(type_fn) => Some(type_fn),
            _ => None,
        })
        .collect()
}

fn exported_public_append_summary(module: &ModuleIdentity) -> ModuleSemanticSummary {
    let mut producer = TypeEnv::new();
    producer
        .register_module_semantic_summary(&typelist_summary(module))
        .expect("producer public domain registers");
    producer
        .register_local_type_functions(
            module,
            &parse_type_fns(
                r#"
                pub type fn Append(xs: TypeList, ys: TypeList) -> TypeList decreases xs {
                    case Append<Nil, ys> = ys;
                    case Append<Cons<h, t>, ys> = Cons<h, Append<t, ys>>;
                }
                "#,
            ),
        )
        .expect("export-closed Append registers");

    let mut summary = typelist_summary(module);
    for type_fn in producer
        .export_public_type_function_summaries(module)
        .expect("public Append summary exports")
    {
        summary = summary.with_exported_type_function(type_fn);
    }
    summary
}

fn summary_head(summary: &ModuleSemanticSummary, name: &str) -> TypeComputationHeadId {
    summary
        .exported_type_functions
        .iter()
        .find(|type_fn| type_fn.exported_name == name)
        .expect("summary head exists")
        .head
        .clone()
}

fn nil(module: &ModuleIdentity) -> NormalTypeExpr {
    let domain = domain(module, "TypeList");
    NormalTypeExpr::DomainConstructorApp {
        constructor: ctor(&domain, "Nil"),
        domain,
        args: vec![],
        kind: Kind::Type,
    }
}

fn cons(module: &ModuleIdentity, head: NormalTypeExpr, tail: NormalTypeExpr) -> NormalTypeExpr {
    let domain = domain(module, "TypeList");
    NormalTypeExpr::DomainConstructorApp {
        constructor: ctor(&domain, "Cons"),
        domain,
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

#[test]
fn downstream_imported_public_summary_reduces_closed_apps_and_preserves_abstract_neutrality() {
    let module = module(1);
    let summary = exported_public_append_summary(&module);
    let head = summary_head(&summary, "Append");
    let mut downstream = TypeEnv::new();
    downstream
        .register_module_semantic_summaries(std::slice::from_ref(&summary))
        .expect("downstream imports public computation summary");

    let ys = cons(&module, prim("B"), nil(&module));
    let closed = Normalizer::new(&downstream)
        .normalize_known_computation_app(&head, vec![nil(&module), ys.clone()], &Kind::Type)
        .expect("closed downstream normalization succeeds");
    assert_eq!(closed, ys);

    let open = Normalizer::new(&downstream)
        .normalize_known_computation_app(&head, vec![var("Xs"), var("Ys")], &Kind::Type)
        .expect("open downstream normalization succeeds");
    assert_eq!(
        open,
        NormalTypeExpr::NeutralComputationApp {
            head,
            args: vec![var("Xs"), var("Ys")],
            kind: Kind::Type,
            reason: NormalFormBlockReason::AbstractScrutinee,
        }
    );
}

fn assert_export_rejects(
    module: &ModuleIdentity,
    setup: impl FnOnce(&mut TypeEnv),
    source: &str,
    expected: &str,
) {
    let mut env = TypeEnv::new();
    setup(&mut env);
    let err = env
        .register_local_type_functions(module, &parse_type_fns(source))
        .expect_err("public type function must be rejected before export");
    let actual = err.to_string();
    assert!(
        actual.contains(expected),
        "expected diagnostic containing {expected:?}, got {actual}"
    );
}

#[test]
fn public_export_validation_rejects_private_helper_domain_marker_and_ordinary_type_dependencies() {
    let public_module = module(2);
    assert_export_rejects(
        &public_module,
        |env| {
            env.register_module_semantic_summary(&typelist_summary(&public_module))
                .expect("public domain registers");
        },
        r#"
        type fn Helper(xs: TypeList) -> TypeList { case Helper<xs> = xs; }
        pub type fn UseHelper(xs: TypeList) -> TypeList { case UseHelper<xs> = Helper<xs>; }
        "#,
        "public type function 'UseHelper' depends on private type function 'Helper'",
    );

    let private_domain_module = module(3);
    assert_export_rejects(
        &private_domain_module,
        |env| {
            env.register_local_sealed_domain_summary(&list_domain_summary(
                &private_domain_module,
                "PrivateList",
                "PrivateNil",
                "PrivateCons",
                CoreVisibility::Private,
            ))
            .expect("private local domain registers");
        },
        r#"
        pub type fn Leak(xs: PrivateList) -> PrivateList { case Leak<xs> = xs; }
        "#,
        "public type function 'Leak' depends on private sealed domain 'PrivateList'",
    );

    let private_marker_module = module(4);
    assert_export_rejects(
        &private_marker_module,
        |env| {
            env.register_local_sealed_domain_summary(&list_domain_summary(
                &private_marker_module,
                "PrivateList",
                "PrivateNil",
                "PrivateCons",
                CoreVisibility::Private,
            ))
            .expect("private local domain registers");
        },
        r#"
        pub type fn Make(xs: PrivateList) -> PrivateList {
            case Make<PrivateNil> = PrivateNil;
            case Make<PrivateCons<h, t>> = PrivateNil;
        }
        "#,
        "public type function 'Make' depends on private marker constructor 'PrivateNil'",
    );

    let private_ordinary_module = module(5);
    assert_export_rejects(
        &private_ordinary_module,
        |env| {
            env.register_module_semantic_summary(&typelist_summary(&private_ordinary_module))
                .expect("public domain registers");
            env.register_type(&TypeDef {
                name: "Secret".to_string(),
                params: vec![],
                body: TypeBody::Struct(vec![]),
                visibility: CoreVisibility::Private,
                builtin: false,
            })
            .expect("private ordinary type registers");
        },
        r#"
        pub type fn Leak(xs: TypeList, secret: Secret) -> TypeList { case Leak<xs, secret> = xs; }
        "#,
        "public type function 'Leak' depends on private ordinary type 'Secret'",
    );
}
