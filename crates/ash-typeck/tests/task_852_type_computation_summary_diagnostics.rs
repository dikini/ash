//! TASK-852: private-opacity and unavailable-reduction diagnostics for type-computation summaries.

use ash_core::ast::{Span as CoreSpan, Visibility as CoreVisibility};
use ash_core::kind::Kind;
use ash_core::module_graph::{CrateId, ModuleId};
use ash_core::semantic_summary::{
    DomainConstructorId, DomainConstructorSummary, DomainFieldSummary, ModuleIdentity,
    ModuleSemanticSummary, ModuleSourceOrigin, SealedDomainId, SealedDomainSummary, SourceAnchor,
    SourceOrigin, SummaryVersion,
};
use ash_core::type_ir::{
    CanonicalTypeExpr, NormalTypeExpr, TypeComputationHeadId, TypeFunctionResultExpr,
};
use ash_parser::surface::Definition;
use ash_parser::token::Span;
use ash_typeck::TypeEnv;
use ash_typeck::error::TypeEnvError;
use ash_typeck::normalizer::{Normalizer, NormalizerDiagnosticKind};

fn module() -> ModuleIdentity {
    ModuleIdentity::new(
        Some(CrateId(852)),
        ModuleId(1),
        vec!["task852".into(), "producer".into()],
        ModuleSourceOrigin::Synthetic {
            reason: "task-852 diagnostics tests".into(),
        },
    )
}

fn span(start: usize, end: usize) -> Span {
    Span::new(start, end, 0, 0)
}

fn core_span(start: usize, end: usize) -> CoreSpan {
    CoreSpan { start, end }
}

fn anchor(label: &str, start: usize, end: usize) -> SourceAnchor {
    SourceAnchor::new(
        SourceOrigin::Synthetic {
            reason: "task-852-test".into(),
        },
        Some(core_span(start, end)),
        label,
    )
}

fn domain() -> SealedDomainId {
    SealedDomainId::new(module(), "TypeList")
}

fn ctor(name: &str) -> DomainConstructorId {
    DomainConstructorId::new(domain(), name)
}

fn typelist_summary(visibility: CoreVisibility) -> ModuleSemanticSummary {
    let domain = domain();
    let nil = DomainConstructorSummary::new(ctor("Nil"), "Nil", vec![], anchor("Nil", 80, 83));
    let cons = DomainConstructorSummary::new(
        ctor("Cons"),
        "Cons",
        vec![
            DomainFieldSummary::unconstrained("head"),
            DomainFieldSummary::constrained_to("tail", &domain, domain.clone()),
        ],
        anchor("Cons", 85, 89),
    );
    ModuleSemanticSummary::new(module())
        .with_version(SummaryVersion::SPEC062_TYPE_COMPUTATION_V3)
        .with_exported_sealed_domain(
            SealedDomainSummary::new(domain, "TypeList", visibility, anchor("TypeList", 20, 28))
                .with_constructor(nil)
                .with_constructor(cons),
        )
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

fn public_type_fn_using_private_helper_source() -> Vec<ash_parser::surface::TypeFnDef> {
    parse_type_fns(
        r#"
        type fn Secret(xs: TypeList) -> TypeList {
            case Secret<xs> = xs;
        }

        pub type fn Public(xs: TypeList) -> TypeList {
            case Public<xs> = Secret<xs>;
        }
        "#,
    )
}

fn public_identity_summary() -> ModuleSemanticSummary {
    let mut producer = TypeEnv::new();
    producer
        .register_module_semantic_summary(&typelist_summary(CoreVisibility::Public))
        .expect("domain summary registers");
    producer
        .register_local_type_functions(
            &module(),
            &parse_type_fns(
                r#"
                pub type fn Id(xs: TypeList) -> TypeList {
                    case Id<xs> = xs;
                }
                "#,
            ),
        )
        .expect("type fn registers");

    let mut summary = typelist_summary(CoreVisibility::Public);
    for type_fn in producer
        .export_public_type_function_summaries(&module())
        .expect("public summary exports")
    {
        summary = summary.with_exported_type_function(type_fn);
    }
    summary
}

fn normalize_known_id(env: &TypeEnv, head: &TypeComputationHeadId) -> NormalTypeExpr {
    Normalizer::new(env)
        .normalize_known_computation_app(
            head,
            vec![NormalTypeExpr::Primitive("Unit".into())],
            &Kind::Type,
        )
        .expect("normalization succeeds")
}

#[test]
fn private_dependency_export_failure_is_structured_and_anchored() {
    let mut producer = TypeEnv::new();
    producer
        .register_module_semantic_summary(&typelist_summary(CoreVisibility::Public))
        .expect("domain summary registers");

    let err = producer
        .register_local_type_functions(&module(), &public_type_fn_using_private_helper_source())
        .expect_err("public type fn depending on private helper is rejected at export validation");

    match err {
        TypeEnvError::PrivateDependencyExportFailure {
            public_item,
            dependency,
            dependency_kind,
            span: got_span,
        } => {
            assert_eq!(public_item, "Public");
            assert_eq!(dependency, "Secret");
            assert_eq!(dependency_kind, "type function");
            assert_ne!(
                got_span,
                Span::default(),
                "source anchor span must not be default"
            );
        }
        other => panic!("expected structured private dependency diagnostic, got {other:?}"),
    }
}

#[test]
fn private_domain_export_failure_uses_domain_anchor_span() {
    let mut producer = TypeEnv::new();
    producer
        .register_local_sealed_domain_summary(
            &typelist_summary(CoreVisibility::Private).exported_sealed_domains[0],
        )
        .expect("private local domain registers");

    let err = producer
        .register_local_type_functions(
            &module(),
            &parse_type_fns(
                r#"
                pub type fn Bad(xs: TypeList) -> TypeList {
                    case Bad<xs> = xs;
                }
                "#,
            ),
        )
        .expect_err("public type fn depending on private domain is rejected");

    match err {
        TypeEnvError::PrivateDependencyExportFailure {
            public_item,
            dependency,
            dependency_kind,
            span: got_span,
        } => {
            assert_eq!(public_item, "Bad");
            assert_eq!(dependency, "TypeList");
            assert_eq!(dependency_kind, "sealed domain");
            assert_eq!(got_span, span(20, 28));
        }
        other => panic!("expected structured private domain diagnostic, got {other:?}"),
    }
}

#[test]
fn summary_version_and_malformed_imports_are_rejected_before_partial_registration() {
    let good = public_identity_summary();
    let imported_head = good.exported_type_functions[0].head.clone();
    for version in [
        SummaryVersion::SPEC057_ORDINARY_TYPE_V1,
        SummaryVersion::SPEC059_SEALED_DOMAIN_V2,
    ] {
        let malformed = ModuleSemanticSummary {
            version,
            ..good.clone()
        };
        let mut env = TypeEnv::new();
        let err = env
            .register_module_semantic_summary(&malformed)
            .expect_err("unsupported summary carrying computation fields is malformed");
        assert!(matches!(
            err,
            TypeEnvError::MalformedImportedComputationSummary { .. }
        ));
        assert!(matches!(
            normalize_known_id(&env, &imported_head),
            NormalTypeExpr::NeutralComputationApp { .. }
        ));
    }

    let future = ModuleSemanticSummary {
        version: SummaryVersion(99),
        ..good.clone()
    };
    let mut env = TypeEnv::new();
    let err = env
        .register_module_semantic_summary(&future)
        .expect_err("unsupported version rejected");
    assert!(matches!(
        err,
        TypeEnvError::UnsupportedSummaryVersion {
            version: SummaryVersion(99),
            ..
        }
    ));
    assert!(matches!(
        normalize_known_id(&env, &imported_head),
        NormalTypeExpr::NeutralComputationApp { .. }
    ));
}

#[test]
fn import_order_conflict_is_structured_and_transactional() {
    let first = public_identity_summary();
    let head = first.exported_type_functions[0].head.clone();
    let mut conflicting = first.clone();
    conflicting.exported_type_functions[0].equations[0].result =
        TypeFunctionResultExpr::Primitive {
            name: "Bool".into(),
            kind: Kind::Type,
            constraint: first.exported_type_functions[0].result_constraint.clone(),
            source_anchor: anchor("conflicting result", 200, 204),
        };

    let mut env = TypeEnv::new();
    let err = env
        .register_module_semantic_summaries(&[first, conflicting])
        .expect_err("conflicting duplicate computation summaries are rejected atomically");

    match err {
        TypeEnvError::ImportOrderConflict {
            family,
            name,
            span: got_span,
        } => {
            assert_eq!(family, "type-function summary");
            assert_eq!(name, "Id");
            assert_ne!(got_span, Span::default());
        }
        other => panic!("expected structured import-order conflict, got {other:?}"),
    }
    assert!(matches!(
        normalize_known_id(&env, &head),
        NormalTypeExpr::NeutralComputationApp { .. }
    ));
}

#[test]
fn unavailable_private_reduction_boundary_is_named_for_required_concrete_normal_form() {
    let head = TypeComputationHeadId::new(module(), "HiddenReduce");
    let expr = CanonicalTypeExpr::ComputationHeadApp {
        head,
        args: vec![CanonicalTypeExpr::Primitive("Unit".into())],
        kind: Kind::Type,
    };

    let err = Normalizer::new(&TypeEnv::new())
        .require_concrete_normal_form(&expr)
        .expect_err("required reduction is unavailable at public boundary");

    assert_eq!(
        err.kind,
        NormalizerDiagnosticKind::UnavailablePrivateReduction
    );
    assert!(err.message.contains("unavailable-private-reduction"));
    assert!(matches!(
        err.normal_slice,
        Some(NormalTypeExpr::NeutralComputationApp { .. })
    ));
}
