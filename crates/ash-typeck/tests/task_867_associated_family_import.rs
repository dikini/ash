use ash_core::kind::Kind;
use ash_core::module_graph::{CrateId, ModuleId};
use ash_core::semantic_summary::{
    AssociatedFamilyClosureMetadata, AssociatedFamilyDependencyClosure,
    AssociatedFamilyDependencySummaryRef, AssociatedFamilyExportMode,
    AssociatedFamilyRevalidationMetadata, AssociatedFamilySummary, AssociatedMemberIdentityId,
    AssociatedMemberIdentitySummary, DomainConstructorId, InterfaceIdentityId,
    InterfaceIdentitySummary, ModuleIdentity, ModuleSemanticSummary, ModuleSourceOrigin,
    ModuleSummaryRef, SealedDomainId, SourceAnchor, SourceOrigin, SummaryVersion, TypeDeclId,
};
use ash_core::type_ir::{
    AssociatedFamilyEquation, AssociatedFamilyHeadId, AssociatedFamilyPattern,
    AssociatedFamilyProjection, AssociatedFamilyProjectionMode, AssociatedFamilyResultConstraint,
    AssociatedFamilyResultExpr, AssociatedFamilyScheme, AssociatedFamilySchemeParam,
    CanonicalTypeExpr, NormalTypeExpr, ProjectionRigidity,
};
use ash_parser::surface::{
    AssociatedFamilyDecreases, AssociatedTypeDecl, AssociatedTypeKind, InterfaceDef,
    InterfaceTypeParam, Type as SurfaceType, Visibility,
};
use ash_parser::token::Span;
use ash_typeck::TypeEnv;
use ash_typeck::normalizer::{NormalizationEvidence, Normalizer};

fn module(name: &str, id: usize) -> ModuleIdentity {
    ModuleIdentity::new(
        Some(CrateId(867)),
        ModuleId(id),
        vec!["task867".into(), name.into()],
        ModuleSourceOrigin::Synthetic {
            reason: format!("TASK-867 typeck test module {name}"),
        },
    )
}

fn anchor(label: &str) -> SourceAnchor {
    SourceAnchor::new(
        SourceOrigin::Synthetic {
            reason: "task-867-typeck-test".into(),
        },
        None,
        label,
    )
}

fn head(module: &ModuleIdentity, interface: &str, member: &str) -> AssociatedFamilyHeadId {
    let interface_identity = InterfaceIdentityId::new(module.clone(), interface);
    let member_identity = AssociatedMemberIdentityId::associated_type(
        interface_identity.clone(),
        member,
        vec![interface.into(), member.into()],
    );
    AssociatedFamilyHeadId {
        interface: interface_identity,
        member: member_identity,
    }
}

fn closure_metadata(family_count: usize, helper_count: usize) -> AssociatedFamilyClosureMetadata {
    AssociatedFamilyClosureMetadata {
        public_closure_checked: true,
        public_ordinary_type_count: 0,
        public_sealed_domain_count: 0,
        public_domain_constructor_count: 0,
        public_type_function_count: 0,
        public_associated_family_count: family_count,
        public_projection_count: 0,
        helper_family_count: helper_count,
    }
}

fn dependency_closure() -> AssociatedFamilyDependencyClosure {
    AssociatedFamilyDependencyClosure {
        ordinary_types: vec![],
        sealed_domains: vec![],
        domain_constructors: vec![],
        type_functions: vec![],
        associated_projections: vec![],
        associated_families: vec![],
        type_function_summaries: vec![],
        closure_metadata: closure_metadata(1, 0),
    }
}

fn identity_result(name: &str) -> AssociatedFamilyResultExpr {
    AssociatedFamilyResultExpr::Var {
        name: name.into(),
        kind: Kind::Type,
        constraint: AssociatedFamilyResultConstraint::Kind(Kind::Type),
        source_anchor: anchor(name),
    }
}

fn primitive_result(name: &str) -> AssociatedFamilyResultExpr {
    AssociatedFamilyResultExpr::Primitive {
        name: name.into(),
        kind: Kind::Type,
        constraint: AssociatedFamilyResultConstraint::Kind(Kind::Type),
        source_anchor: anchor(name),
    }
}

fn projection_result(head: AssociatedFamilyHeadId, arg_name: &str) -> AssociatedFamilyResultExpr {
    projection_result_with_args(head, vec![identity_result(arg_name)])
}

fn projection_result_with_args(
    head: AssociatedFamilyHeadId,
    interface_args: Vec<AssociatedFamilyResultExpr>,
) -> AssociatedFamilyResultExpr {
    AssociatedFamilyResultExpr::AssociatedFamilyProjection {
        head,
        interface_args,
        kind: Kind::Type,
        constraint: AssociatedFamilyResultConstraint::Kind(Kind::Type),
        rigidity: ProjectionRigidity::Neutral,
        source_anchor: anchor("helper projection"),
    }
}

fn projection_dependency(
    head: AssociatedFamilyHeadId,
    interface_args: Vec<CanonicalTypeExpr>,
) -> AssociatedFamilyProjection {
    AssociatedFamilyProjection {
        head,
        interface_args,
        kind: Kind::Type,
        rigidity: ProjectionRigidity::Neutral,
        mode: AssociatedFamilyProjectionMode::ReducibleSealedFamilyHead,
    }
}

fn helper_dependency(
    module: &ModuleIdentity,
    family: AssociatedFamilyHeadId,
) -> AssociatedFamilyDependencySummaryRef {
    AssociatedFamilyDependencySummaryRef {
        summary_ref: ModuleSummaryRef {
            module: module.clone(),
            version: SummaryVersion::SPEC063_ASSOCIATED_FAMILY_V4,
        },
        family,
        digest: None,
        compiler_algorithm_version: Some("spec-063-mvp".into()),
        source_visible: false,
        normalizer_available: true,
    }
}

fn scheme_for_head(
    family_head: AssociatedFamilyHeadId,
    result: AssociatedFamilyResultExpr,
) -> AssociatedFamilyScheme {
    let constraint = AssociatedFamilyResultConstraint::Kind(Kind::Type);
    AssociatedFamilyScheme {
        head: family_head.clone(),
        params: vec![AssociatedFamilySchemeParam {
            name: "T".into(),
            ty: CanonicalTypeExpr::Var("T".into()),
            kind: Kind::Type,
            domain_constraint: None,
            source_anchor: anchor("T"),
        }],
        result_domain: CanonicalTypeExpr::Primitive("Type".into()),
        result_kind: Kind::Type,
        equations: vec![AssociatedFamilyEquation {
            head: family_head,
            ordinal: 0,
            interface_arg_patterns: vec![AssociatedFamilyPattern::Var {
                name: "T".into(),
                constraint,
                source_anchor: anchor("pattern T"),
            }],
            result,
            decreases: None,
            source_anchor: anchor("equation"),
            case_head_anchor: anchor("case"),
        }],
        source_anchor: anchor("scheme"),
    }
}

fn public_family_interface(interface: &str, member: &str) -> InterfaceDef {
    InterfaceDef {
        visibility: Visibility::Public,
        name: interface.into(),
        type_params: vec![InterfaceTypeParam {
            name: "T".into(),
            domain: None,
            kind: None,
            span: Span::default(),
        }],
        associated_types: vec![AssociatedTypeDecl {
            name: member.into(),
            kind: AssociatedTypeKind::SealedFamily {
                result_domain: SurfaceType::Name("Type".into()),
                decreases: None::<AssociatedFamilyDecreases>,
                span: Span::default(),
            },
            span: Span::default(),
        }],
        methods: vec![],
        span: Span::default(),
    }
}

fn associated_family_summary(
    module: &ModuleIdentity,
    interface: &str,
    member: &str,
    visible_name: &str,
    result: AssociatedFamilyResultExpr,
) -> AssociatedFamilySummary {
    let family_head = head(module, interface, member);
    let constraint = AssociatedFamilyResultConstraint::Kind(Kind::Type);
    let equation = AssociatedFamilyEquation {
        head: family_head.clone(),
        ordinal: 0,
        interface_arg_patterns: vec![AssociatedFamilyPattern::Var {
            name: "T".into(),
            constraint: constraint.clone(),
            source_anchor: anchor("pattern T"),
        }],
        result,
        decreases: None,
        source_anchor: anchor("equation"),
        case_head_anchor: anchor("case"),
    };

    AssociatedFamilySummary {
        head: family_head.clone(),
        interface_identity: family_head.interface.clone(),
        member_identity: family_head.member.clone(),
        visible_name: visible_name.into(),
        result_domain: CanonicalTypeExpr::Primitive("Type".into()),
        result_kind: Kind::Type,
        export_mode: AssociatedFamilyExportMode::TransparentEquations,
        schemes: vec![AssociatedFamilyScheme {
            head: family_head,
            params: vec![AssociatedFamilySchemeParam {
                name: "T".into(),
                ty: CanonicalTypeExpr::Var("T".into()),
                kind: Kind::Type,
                domain_constraint: None,
                source_anchor: anchor("T"),
            }],
            result_domain: CanonicalTypeExpr::Primitive("Type".into()),
            result_kind: Kind::Type,
            equations: vec![equation],
            source_anchor: anchor("scheme"),
        }],
        dependency_closure: dependency_closure(),
        source_anchor: anchor("family"),
        revalidation_metadata: AssociatedFamilyRevalidationMetadata {
            spec_version: SummaryVersion::SPEC063_ASSOCIATED_FAMILY_V4,
            kind_and_domain_checked: true,
            coverage_and_overlap_checked: true,
            coherence_checked: true,
            recursion_checked: true,
            decreases: vec![],
        },
    }
}

fn semantic_summary(
    module: &ModuleIdentity,
    families: Vec<AssociatedFamilySummary>,
) -> ModuleSemanticSummary {
    let mut summary = ModuleSemanticSummary::new(module.clone())
        .with_version(SummaryVersion::SPEC063_ASSOCIATED_FAMILY_V4);
    for family in families {
        summary = summary
            .with_interface_identity(InterfaceIdentitySummary::new(
                family.interface_identity.clone(),
                family.interface_identity.name.clone(),
                vec![family.interface_identity.name.to_string()],
                anchor("interface identity"),
            ))
            .with_associated_member_identity(AssociatedMemberIdentitySummary::new(
                family.member_identity.clone(),
                family.member_identity.name.clone(),
                anchor("member identity"),
            ))
            .with_exported_associated_family(family);
    }
    summary
}

fn normalize_imported_identity_projection(
    env: &TypeEnv,
    family_head: &AssociatedFamilyHeadId,
) -> NormalTypeExpr {
    let expr = CanonicalTypeExpr::Projection {
        interface: family_head.interface.clone(),
        member: family_head.member.clone(),
        args: vec![CanonicalTypeExpr::Primitive("String".into())],
        kind: Kind::Type,
        rigidity: ProjectionRigidity::Neutral,
    };
    let outcome = Normalizer::new(env)
        .normalize(&expr)
        .expect("normalization should not be a syntax/import crash");
    assert_eq!(
        outcome.evidence,
        NormalizationEvidence::AssociatedFamilyProjectionReduced,
        "validated imported V4 family summaries must be normalizer-available"
    );
    outcome.normal
}

#[test]
fn task_867_validated_v4_import_declares_family_and_reduces_downstream() {
    let provider = module("provider", 1);
    let family =
        associated_family_summary(&provider, "Iterator", "Item", "Item", identity_result("T"));
    let family_head = family.head.clone();
    let summary = semantic_summary(&provider, vec![family]);
    let mut env = TypeEnv::new();

    env.register_module_semantic_summary(&summary)
        .expect("well-formed V4 family summary should import");
    env.set_current_module_identity(module("downstream", 2));

    assert_eq!(
        normalize_imported_identity_projection(&env, &family_head),
        NormalTypeExpr::Primitive("String".into())
    );
}

#[test]
fn task_867_import_rejects_malformed_metadata_before_registration() {
    let provider = module("metadata", 10);
    let base =
        associated_family_summary(&provider, "Iterator", "Item", "Item", identity_result("T"));

    let mut wrong_spec = base.clone();
    wrong_spec.revalidation_metadata.spec_version = SummaryVersion::SPEC062_TYPE_COMPUTATION_V3;

    let mut missing_kind_domain = base.clone();
    missing_kind_domain
        .revalidation_metadata
        .kind_and_domain_checked = false;

    let mut missing_coverage = base.clone();
    missing_coverage
        .revalidation_metadata
        .coverage_and_overlap_checked = false;

    let mut missing_coherence = base.clone();
    missing_coherence.revalidation_metadata.coherence_checked = false;

    for (label, family) in [
        ("wrong SPEC version", wrong_spec),
        ("missing kind/domain flag", missing_kind_domain),
        ("missing coverage/overlap flag", missing_coverage),
        ("missing coherence flag", missing_coherence),
    ] {
        let summary = semantic_summary(&provider, vec![family]);
        let mut env = TypeEnv::new();
        let err = env
            .register_module_semantic_summary(&summary)
            .expect_err(label);
        assert!(
            err.to_string().contains("associated") || err.to_string().contains("SPEC-063"),
            "{label} should produce an associated-family import diagnostic, got {err}"
        );
        assert!(
            env.lookup_associated_family_declaration("Iterator", "Item")
                .is_none(),
            "failed import must not leave a staged associated-family declaration behind"
        );
    }
}

#[test]
fn task_867_import_rejects_result_domain_mismatch_overlap_and_unknown_dependency_transactionally() {
    let provider = module("bad-shapes", 20);
    let base =
        associated_family_summary(&provider, "Iterator", "Item", "Item", identity_result("T"));

    let mut bad_domain = base.clone();
    bad_domain.result_domain = CanonicalTypeExpr::Primitive("NotType".into());

    let mut ambiguous = base.clone();
    let mut duplicate = ambiguous.schemes[0].equations[0].clone();
    duplicate.ordinal = 1;
    duplicate.result = primitive_result("Int");
    ambiguous.schemes[0].equations.push(duplicate);

    let mut unknown_dependency = base.clone();
    unknown_dependency
        .dependency_closure
        .ordinary_types
        .push(TypeDeclId::ordinary(
            module("unknown-dependency", 21),
            "Hidden",
        ));
    unknown_dependency
        .dependency_closure
        .closure_metadata
        .public_ordinary_type_count = 1;

    for (label, family) in [
        ("bad result domain", bad_domain),
        ("overlap/ambiguity", ambiguous),
        ("unknown dependency", unknown_dependency),
    ] {
        let summary = semantic_summary(&provider, vec![family]);
        let mut env = TypeEnv::new();
        let err = env
            .register_module_semantic_summary(&summary)
            .expect_err(label);
        assert!(
            err.to_string().contains("associated")
                || err.to_string().contains("domain")
                || err.to_string().contains("dependency"),
            "{label} should produce a focused import diagnostic, got {err}"
        );
        assert!(
            env.lookup_associated_family_declaration("Iterator", "Item")
                .is_none(),
            "{label} must reject transactionally before declaration/normalizer registration"
        );
    }
}

#[test]
fn task_867_import_rejects_omitted_associated_family_dependency_closure() {
    let provider = module("omitted-helper", 25);
    let helper = associated_family_summary(
        &provider,
        "HelperInterface",
        "Out",
        "$ash_dependency$HelperOut",
        identity_result("T"),
    );
    let main = associated_family_summary(
        &provider,
        "MainInterface",
        "Out",
        "Out",
        projection_result(helper.head.clone(), "T"),
    );
    let summary = semantic_summary(&provider, vec![main, helper]);
    let mut env = TypeEnv::new();

    let err = env
        .register_module_semantic_summary(&summary)
        .expect_err("RHS helper projection omitted from dependency_closure must be rejected");

    assert!(
        err.to_string().contains("dependency closure"),
        "omitted dependency should produce a closure diagnostic, got {err}"
    );
    assert!(
        env.lookup_associated_family_declaration("MainInterface", "Out")
            .is_none(),
        "malformed summary must reject transactionally before source-visible registration"
    );
    assert!(
        env.lookup_associated_family_declaration("HelperInterface", "Out")
            .is_none(),
        "malformed summary must not leave hidden helper declarations staged"
    );
}

#[test]
fn task_867_import_rejects_lossy_associated_projection_argument_spine() {
    let provider = module("lossy-import", 28);
    let helper = associated_family_summary(
        &provider,
        "HelperInterface",
        "Out",
        "$ash_dependency$HelperOut",
        identity_result("T"),
    );
    let nested_arg = projection_result(helper.head.clone(), "T");
    let mut main = associated_family_summary(
        &provider,
        "MainInterface",
        "Out",
        "Out",
        projection_result_with_args(helper.head.clone(), vec![nested_arg]),
    );
    main.dependency_closure
        .associated_families
        .push(helper_dependency(&provider, helper.head.clone()));
    main.dependency_closure
        .associated_projections
        .push(projection_dependency(helper.head.clone(), vec![]));
    main.dependency_closure.closure_metadata = AssociatedFamilyClosureMetadata {
        public_projection_count: 1,
        ..closure_metadata(2, 1)
    };

    let summary = semantic_summary(&provider, vec![main, helper]);
    let mut env = TypeEnv::new();

    let err = env
        .register_module_semantic_summary(&summary)
        .expect_err("shortened projection argument spine must not import lossily");

    assert!(
        err.to_string().contains("argument") || err.to_string().contains("losslessly"),
        "lossy associated projection closure should produce an argument-spine diagnostic, got {err}"
    );
    assert!(
        env.lookup_associated_family_declaration("MainInterface", "Out")
            .is_none(),
        "lossy import must reject transactionally before registration"
    );
}

#[test]
fn task_867_import_rejects_lossy_domain_constructor_argument_spine() {
    let provider = module("lossy-domain-import", 34);
    let helper = associated_family_summary(
        &provider,
        "HelperInterface",
        "Out",
        "$ash_dependency$HelperOut",
        identity_result("T"),
    );
    let domain = SealedDomainId::new(provider.clone(), "Nat");
    let zero = DomainConstructorId::new(domain.clone(), "Zero");
    let constructor_arg = AssociatedFamilyResultExpr::DomainConstructorApp {
        constructor: zero,
        domain,
        args: vec![],
        kind: Kind::Type,
        constraint: AssociatedFamilyResultConstraint::Kind(Kind::Type),
        source_anchor: anchor("domain constructor argument"),
    };
    let mut main = associated_family_summary(
        &provider,
        "MainInterface",
        "Out",
        "Out",
        projection_result_with_args(helper.head.clone(), vec![constructor_arg]),
    );
    main.dependency_closure
        .associated_families
        .push(helper_dependency(&provider, helper.head.clone()));
    main.dependency_closure
        .associated_projections
        .push(projection_dependency(
            helper.head.clone(),
            vec![CanonicalTypeExpr::Var("T".into())],
        ));
    main.dependency_closure.closure_metadata = AssociatedFamilyClosureMetadata {
        public_projection_count: 1,
        ..closure_metadata(2, 1)
    };

    let summary = semantic_summary(&provider, vec![main, helper]);
    let mut env = TypeEnv::new();

    let err = env
        .register_module_semantic_summary(&summary)
        .expect_err("domain-constructor projection argument must not import lossily");

    assert!(
        err.to_string().contains("domain-constructor") || err.to_string().contains("losslessly"),
        "lossy domain-constructor projection argument should produce a lossless-closure diagnostic, got {err}"
    );
    assert!(
        env.lookup_associated_family_declaration("MainInterface", "Out")
            .is_none(),
        "lossy domain-constructor import must reject transactionally before registration"
    );
}

#[test]
fn task_867_export_rejects_unrepresentable_nested_projection_argument() {
    let provider = module("lossy-export", 29);
    let mut env = TypeEnv::with_builtin_types();
    env.set_current_module_identity(provider.clone());
    env.register_interface(&public_family_interface("HelperInterface", "Out"))
        .expect("helper family interface registers");
    env.register_interface(&public_family_interface("MainInterface", "Out"))
        .expect("main family interface registers");
    let helper_head = env
        .lookup_associated_family_declaration("HelperInterface", "Out")
        .expect("helper family declaration exists")
        .head
        .clone();
    let main_head = env
        .lookup_associated_family_declaration("MainInterface", "Out")
        .expect("main family declaration exists")
        .head
        .clone();

    env.register_associated_family_scheme(
        scheme_for_head(helper_head.clone(), identity_result("T")),
        provider.clone(),
    )
    .expect("helper identity scheme registers");
    let nested_arg = projection_result(helper_head.clone(), "T");
    env.register_associated_family_scheme(
        scheme_for_head(
            main_head,
            projection_result_with_args(helper_head, vec![nested_arg]),
        ),
        provider.clone(),
    )
    .expect("main scheme with nested projection argument registers before export validation");

    let err = env
        .export_public_associated_family_summaries(&provider)
        .expect_err("export must fail closed instead of shortening nested projection arguments");

    assert!(
        err.to_string().contains("losslessly") || err.to_string().contains("projection argument"),
        "lossy associated projection export should be rejected explicitly, got {err}"
    );
}

#[test]
fn task_867_export_preserves_representable_projection_argument_spine() {
    let provider = module("lossless-export", 33);
    let mut env = TypeEnv::with_builtin_types();
    env.set_current_module_identity(provider.clone());
    env.register_interface(&public_family_interface("HelperInterface", "Out"))
        .expect("helper family interface registers");
    env.register_interface(&public_family_interface("MainInterface", "Out"))
        .expect("main family interface registers");
    let helper_head = env
        .lookup_associated_family_declaration("HelperInterface", "Out")
        .expect("helper family declaration exists")
        .head
        .clone();
    let main_head = env
        .lookup_associated_family_declaration("MainInterface", "Out")
        .expect("main family declaration exists")
        .head
        .clone();

    env.register_associated_family_scheme(
        scheme_for_head(helper_head.clone(), identity_result("T")),
        provider.clone(),
    )
    .expect("helper identity scheme registers");
    env.register_associated_family_scheme(
        scheme_for_head(main_head, projection_result(helper_head.clone(), "T")),
        provider.clone(),
    )
    .expect("representable main scheme registers");

    let summaries = env
        .export_public_associated_family_summaries(&provider)
        .expect("representable projection argument spine should export losslessly");
    let main = summaries
        .iter()
        .find(|family| {
            family.visible_name == "Out" && family.head.interface.name == "MainInterface"
        })
        .expect("main family summary exported");

    assert_eq!(main.dependency_closure.associated_projections.len(), 1);
    assert_eq!(
        main.dependency_closure.associated_projections[0].interface_args,
        vec![CanonicalTypeExpr::Var("T".into())]
    );
}

#[test]
fn task_867_batch_import_is_order_stable_for_associated_family_dependencies() {
    let provider = module("order", 30);
    let helper = associated_family_summary(
        &provider,
        "HelperInterface",
        "Out",
        "$ash_dependency$HelperOut",
        identity_result("T"),
    );
    let mut main = associated_family_summary(
        &provider,
        "MainInterface",
        "Out",
        "Out",
        projection_result(helper.head.clone(), "T"),
    );
    main.dependency_closure
        .associated_families
        .push(helper_dependency(&provider, helper.head.clone()));
    main.dependency_closure
        .associated_projections
        .push(projection_dependency(
            helper.head.clone(),
            vec![CanonicalTypeExpr::Var("T".into())],
        ));
    main.dependency_closure.closure_metadata = AssociatedFamilyClosureMetadata {
        public_projection_count: 1,
        ..closure_metadata(2, 1)
    };

    let summary_main = semantic_summary(&provider, vec![main.clone()]);
    let summary_helper = semantic_summary(&provider, vec![helper.clone()]);

    let mut main_then_helper = TypeEnv::new();
    main_then_helper
        .register_module_semantic_summaries(&[summary_main.clone(), summary_helper.clone()])
        .expect("batch declaration should make helper heads available before validation");
    main_then_helper.set_current_module_identity(module("downstream-a", 31));

    let mut helper_then_main = TypeEnv::new();
    helper_then_main
        .register_module_semantic_summaries(&[summary_helper, summary_main])
        .expect("opposite import order should validate identically");
    helper_then_main.set_current_module_identity(module("downstream-b", 32));

    assert!(
        helper_then_main
            .lookup_associated_family_declaration("HelperInterface", "Out")
            .is_none(),
        "dependency helper family must be normalizer-available without becoming source-visible"
    );
    assert_eq!(
        normalize_imported_identity_projection(&main_then_helper, &main.head),
        NormalTypeExpr::Primitive("String".into())
    );
    assert_eq!(
        normalize_imported_identity_projection(&helper_then_main, &main.head),
        NormalTypeExpr::Primitive("String".into())
    );
}
