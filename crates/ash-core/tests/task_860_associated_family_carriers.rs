//! TASK-860 RED tests for core associated-family identity, projection, and V4 summary carriers.
//!
//! These tests intentionally name the TASK-860 public API before production code exists.
//! The first run is expected to fail at compile time with missing carrier/API symbols.

use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

use ash_core::ast::Span;
use ash_core::kind::Kind;
use ash_core::module_graph::{CrateId, ModuleId};
use ash_core::semantic_summary::{
    AssociatedFamilyClosureMetadata, AssociatedFamilyDependencyClosure,
    AssociatedFamilyDependencySummaryRef, AssociatedFamilyExportMode,
    AssociatedFamilyRevalidationMetadata, AssociatedFamilySummary, AssociatedMemberIdentityId,
    DomainConstructorId, InterfaceIdentityId, ModuleIdentity, ModuleSemanticSummary,
    ModuleSemanticSummaryValidationError, ModuleSourceOrigin, ModuleSummaryRef, SealedDomainId,
    SourceAnchor, SourceOrigin, SummaryVersion, TypeDeclId, TypeFunctionDependencySummaryRef,
    ValidatedDecreasesSummary,
};
use ash_core::type_ir::{
    AssociatedFamilyEquation, AssociatedFamilyHeadId, AssociatedFamilyPattern,
    AssociatedFamilyProjection, AssociatedFamilyProjectionKind, AssociatedFamilyProjectionMode,
    AssociatedFamilyResultConstraint, AssociatedFamilyResultExpr, AssociatedFamilyScheme,
    AssociatedFamilySchemeParam, CanonicalTypeExpr, ProjectionRigidity, TypeComputationHeadId,
};

fn hash_of<T: Hash>(value: &T) -> u64 {
    let mut hasher = DefaultHasher::new();
    value.hash(&mut hasher);
    hasher.finish()
}

fn module_identity(module_id: usize, name: &str) -> ModuleIdentity {
    ModuleIdentity::new(
        Some(CrateId(860)),
        ModuleId(module_id),
        vec!["task860".to_string(), name.to_string()],
        ModuleSourceOrigin::Synthetic {
            reason: format!("task-860 {name}"),
        },
    )
}

fn anchor(label: &str) -> SourceAnchor {
    SourceAnchor::new(
        SourceOrigin::Synthetic {
            reason: "task-860 associated-family carrier test".to_string(),
        },
        Some(Span { start: 10, end: 20 }),
        label,
    )
}

fn append_head() -> AssociatedFamilyHeadId {
    let module = module_identity(1, "append");
    let interface = InterfaceIdentityId::new(module, "Append");
    let member = AssociatedMemberIdentityId::associated_type(
        interface.clone(),
        "Out",
        vec!["Append".to_string(), "Out".to_string()],
    );

    AssociatedFamilyHeadId { interface, member }
}

fn type_list_domain() -> SealedDomainId {
    SealedDomainId::new(module_identity(2, "type_list_domain"), "TypeList")
}

fn nil_constructor() -> DomainConstructorId {
    DomainConstructorId::new(type_list_domain(), "Nil")
}

fn cons_constructor() -> DomainConstructorId {
    DomainConstructorId::new(type_list_domain(), "Cons")
}

fn dependency_ref(digest: &str) -> AssociatedFamilyDependencySummaryRef {
    AssociatedFamilyDependencySummaryRef {
        summary_ref: ModuleSummaryRef {
            module: module_identity(900, "dependency"),
            version: SummaryVersion::SPEC063_ASSOCIATED_FAMILY_V4,
        },
        family: append_head(),
        digest: Some(digest.to_string()),
        compiler_algorithm_version: Some("spec-063-task-860-red".to_string()),
        source_visible: false,
        normalizer_available: true,
    }
}

fn type_function_dependency_ref(digest: &str) -> TypeFunctionDependencySummaryRef {
    TypeFunctionDependencySummaryRef {
        summary_ref: ModuleSummaryRef {
            module: module_identity(901, "type_fn_dependency"),
            version: SummaryVersion::SPEC062_TYPE_COMPUTATION_V3,
        },
        digest: Some(digest.to_string()),
        compiler_algorithm_version: Some("spec-062-compatible".to_string()),
    }
}

fn public_append_family_summary(result_name: &str, digest: &str) -> AssociatedFamilySummary {
    let head = append_head();
    let domain = type_list_domain();
    let decreases = ValidatedDecreasesSummary {
        parameter: "Xs".to_string(),
        parameter_index: 0,
        domain: domain.clone(),
        structural_recursion_checked: true,
        source_anchor: anchor("decreases Xs"),
    };

    AssociatedFamilySummary {
        head: head.clone(),
        interface_identity: head.interface.clone(),
        member_identity: head.member.clone(),
        visible_name: "Append::Out".to_string(),
        result_domain: CanonicalTypeExpr::Primitive("TypeList".to_string()),
        result_kind: Kind::Type,
        export_mode: AssociatedFamilyExportMode::TransparentEquations,
        schemes: vec![append_nil_scheme(
            head.clone(),
            result_name,
            Some(decreases.clone()),
        )],
        dependency_closure: AssociatedFamilyDependencyClosure {
            ordinary_types: vec![TypeDeclId::ordinary(module_identity(3, "ordinary"), "List")],
            sealed_domains: vec![domain.clone()],
            domain_constructors: vec![nil_constructor(), cons_constructor()],
            type_functions: vec![TypeComputationHeadId::new(
                module_identity(4, "type_functions"),
                "NormalizeList",
            )],
            associated_projections: vec![AssociatedFamilyProjection {
                head: head.clone(),
                interface_args: vec![
                    CanonicalTypeExpr::Var("Xs".to_string()),
                    CanonicalTypeExpr::Var("Ys".to_string()),
                ],
                kind: Kind::Type,
                rigidity: ProjectionRigidity::Neutral,
                mode: AssociatedFamilyProjectionMode::NeutralBlockedOrUnavailable,
            }],
            associated_families: vec![dependency_ref(digest)],
            type_function_summaries: vec![type_function_dependency_ref("sha256:type-fn")],
            closure_metadata: AssociatedFamilyClosureMetadata {
                public_closure_checked: true,
                public_ordinary_type_count: 1,
                public_sealed_domain_count: 1,
                public_domain_constructor_count: 2,
                public_type_function_count: 1,
                public_associated_family_count: 1,
                public_projection_count: 1,
                helper_family_count: 1,
            },
        },
        source_anchor: anchor("sealed type family Out"),
        revalidation_metadata: AssociatedFamilyRevalidationMetadata {
            spec_version: SummaryVersion::SPEC063_ASSOCIATED_FAMILY_V4,
            kind_and_domain_checked: true,
            coverage_and_overlap_checked: true,
            coherence_checked: true,
            recursion_checked: true,
            decreases: vec![decreases],
        },
    }
}

fn append_nil_scheme(
    head: AssociatedFamilyHeadId,
    result_name: &str,
    decreases: Option<ValidatedDecreasesSummary>,
) -> AssociatedFamilyScheme {
    let domain = type_list_domain();

    AssociatedFamilyScheme {
        head: head.clone(),
        params: vec![AssociatedFamilySchemeParam {
            name: "Ys".to_string(),
            ty: CanonicalTypeExpr::Primitive("TypeList".to_string()),
            kind: Kind::Type,
            domain_constraint: Some(domain.clone()),
            source_anchor: anchor("param Ys"),
        }],
        result_domain: CanonicalTypeExpr::Primitive("TypeList".to_string()),
        result_kind: Kind::Type,
        equations: vec![AssociatedFamilyEquation {
            head,
            ordinal: 0,
            interface_arg_patterns: vec![
                AssociatedFamilyPattern::DomainConstructor {
                    constructor: Box::new(nil_constructor()),
                    domain: Box::new(domain.clone()),
                    fields: Vec::new(),
                    constraint: AssociatedFamilyResultConstraint::Domain(domain.clone()),
                    source_anchor: anchor("Nil"),
                },
                AssociatedFamilyPattern::Var {
                    name: "Ys".to_string(),
                    constraint: AssociatedFamilyResultConstraint::Domain(domain.clone()),
                    source_anchor: anchor("Ys"),
                },
            ],
            result: AssociatedFamilyResultExpr::Var {
                name: result_name.to_string(),
                kind: Kind::Type,
                constraint: AssociatedFamilyResultConstraint::Domain(domain),
                source_anchor: anchor("result Ys"),
            },
            decreases,
            source_anchor: anchor("impl Append<Nil, Ys>"),
            case_head_anchor: anchor("Append<Nil, Ys>"),
        }],
        source_anchor: anchor("scheme Append<Nil, Ys>"),
    }
}

#[test]
fn associated_family_head_identity_is_interface_member_pair_not_debug_string() {
    let head = append_head();
    let same = append_head();
    let other_interface = InterfaceIdentityId::new(module_identity(8, "iterator"), "Iterator");
    let other_member = AssociatedMemberIdentityId::associated_type(
        other_interface.clone(),
        "Item",
        vec!["Iterator".to_string(), "Item".to_string()],
    );
    let different = AssociatedFamilyHeadId {
        interface: other_interface,
        member: other_member,
    };

    assert_eq!(head, same);
    assert_eq!(hash_of(&head), hash_of(&same));
    assert_ne!(head, different);
    assert_eq!(head.member.interface, head.interface);

    let json = serde_json::to_string(&head).expect("head serializes as typed identity");
    assert!(json.contains("Append"));
    assert!(json.contains("Out"));
    let decoded: AssociatedFamilyHeadId = serde_json::from_str(&json).expect("head deserializes");
    assert_eq!(decoded, head);
}

#[test]
fn associated_family_projection_helpers_classify_ordinary_reducible_rigid_and_neutral() {
    let head = append_head();
    let args = vec![
        CanonicalTypeExpr::Var("Xs".to_string()),
        CanonicalTypeExpr::Var("Ys".to_string()),
    ];

    let ordinary = AssociatedFamilyProjection {
        head: head.clone(),
        interface_args: args.clone(),
        kind: Kind::Type,
        rigidity: ProjectionRigidity::Neutral,
        mode: AssociatedFamilyProjectionMode::OrdinaryAssociatedProjection,
    };
    let reducible = AssociatedFamilyProjection {
        head: head.clone(),
        interface_args: args.clone(),
        kind: Kind::Type,
        rigidity: ProjectionRigidity::Neutral,
        mode: AssociatedFamilyProjectionMode::ReducibleSealedFamilyHead,
    };
    let rigid = AssociatedFamilyProjection {
        head: head.clone(),
        interface_args: args.clone(),
        kind: Kind::Type,
        rigidity: ProjectionRigidity::Rigid,
        mode: AssociatedFamilyProjectionMode::RigidWhereBoundProjection,
    };
    let neutral = AssociatedFamilyProjection {
        head,
        interface_args: args,
        kind: Kind::Type,
        rigidity: ProjectionRigidity::Neutral,
        mode: AssociatedFamilyProjectionMode::NeutralBlockedOrUnavailable,
    };

    assert_eq!(
        ordinary.classification(),
        AssociatedFamilyProjectionKind::OrdinaryAssociatedProjection
    );
    assert_eq!(
        reducible.classification(),
        AssociatedFamilyProjectionKind::ReducibleSealedFamilyHead
    );
    assert_eq!(
        rigid.classification(),
        AssociatedFamilyProjectionKind::RigidWhereBoundProjection
    );
    assert_eq!(
        neutral.classification(),
        AssociatedFamilyProjectionKind::NeutralBlockedOrUnavailable
    );

    assert!(ordinary.is_ordinary_associated_projection());
    assert!(reducible.is_reducible_family_head());
    assert!(rigid.is_rigid_where_bound_projection());
    assert!(neutral.is_neutral_blocked_or_unavailable());
}

#[test]
fn checked_associated_family_scheme_preserves_domain_patterns_and_recursive_projection_rhs() {
    let head = append_head();
    let domain = type_list_domain();

    let recursive_projection = AssociatedFamilyResultExpr::AssociatedFamilyProjection {
        head: head.clone(),
        interface_args: vec![
            AssociatedFamilyResultExpr::Var {
                name: "T".to_string(),
                kind: Kind::Type,
                constraint: AssociatedFamilyResultConstraint::Domain(domain.clone()),
                source_anchor: anchor("T"),
            },
            AssociatedFamilyResultExpr::Var {
                name: "Ys".to_string(),
                kind: Kind::Type,
                constraint: AssociatedFamilyResultConstraint::Domain(domain.clone()),
                source_anchor: anchor("Ys"),
            },
        ],
        kind: Kind::Type,
        constraint: AssociatedFamilyResultConstraint::Domain(domain.clone()),
        rigidity: ProjectionRigidity::Neutral,
        source_anchor: anchor("<Append<T, Ys>>::Out"),
    };

    let cons_result = AssociatedFamilyResultExpr::DomainConstructorApp {
        constructor: cons_constructor(),
        domain: domain.clone(),
        args: vec![
            AssociatedFamilyResultExpr::Var {
                name: "H".to_string(),
                kind: Kind::Type,
                constraint: AssociatedFamilyResultConstraint::Kind(Kind::Type),
                source_anchor: anchor("H"),
            },
            recursive_projection.clone(),
        ],
        kind: Kind::Type,
        constraint: AssociatedFamilyResultConstraint::Domain(domain.clone()),
        source_anchor: anchor("Cons<H, <Append<T, Ys>>::Out>"),
    };

    let scheme = AssociatedFamilyScheme {
        head: head.clone(),
        params: vec![
            AssociatedFamilySchemeParam {
                name: "H".to_string(),
                ty: CanonicalTypeExpr::Primitive("Type".to_string()),
                kind: Kind::Type,
                domain_constraint: None,
                source_anchor: anchor("param H"),
            },
            AssociatedFamilySchemeParam {
                name: "T".to_string(),
                ty: CanonicalTypeExpr::Primitive("TypeList".to_string()),
                kind: Kind::Type,
                domain_constraint: Some(domain.clone()),
                source_anchor: anchor("param T"),
            },
            AssociatedFamilySchemeParam {
                name: "Ys".to_string(),
                ty: CanonicalTypeExpr::Primitive("TypeList".to_string()),
                kind: Kind::Type,
                domain_constraint: Some(domain.clone()),
                source_anchor: anchor("param Ys"),
            },
        ],
        result_domain: CanonicalTypeExpr::Primitive("TypeList".to_string()),
        result_kind: Kind::Type,
        equations: vec![AssociatedFamilyEquation {
            head: head.clone(),
            ordinal: 1,
            interface_arg_patterns: vec![
                AssociatedFamilyPattern::DomainConstructor {
                    constructor: Box::new(cons_constructor()),
                    domain: Box::new(domain.clone()),
                    fields: vec![
                        AssociatedFamilyPattern::Var {
                            name: "H".to_string(),
                            constraint: AssociatedFamilyResultConstraint::Kind(Kind::Type),
                            source_anchor: anchor("H"),
                        },
                        AssociatedFamilyPattern::Var {
                            name: "T".to_string(),
                            constraint: AssociatedFamilyResultConstraint::Domain(domain.clone()),
                            source_anchor: anchor("T"),
                        },
                    ],
                    constraint: AssociatedFamilyResultConstraint::Domain(domain.clone()),
                    source_anchor: anchor("Cons<H, T>"),
                },
                AssociatedFamilyPattern::Var {
                    name: "Ys".to_string(),
                    constraint: AssociatedFamilyResultConstraint::Domain(domain.clone()),
                    source_anchor: anchor("Ys"),
                },
            ],
            result: cons_result,
            decreases: Some(ValidatedDecreasesSummary {
                parameter: "Xs".to_string(),
                parameter_index: 0,
                domain: domain.clone(),
                structural_recursion_checked: true,
                source_anchor: anchor("decreases Xs"),
            }),
            source_anchor: anchor("impl Append<Cons<H, T>, Ys>"),
            case_head_anchor: anchor("Append<Cons<H, T>, Ys>"),
        }],
        source_anchor: anchor("recursive Append scheme"),
    };

    assert_eq!(scheme.head, head);
    assert_eq!(scheme.equations[0].ordinal, 1);
    assert_eq!(
        scheme.equations[0].result,
        scheme.equations[0].result.clone()
    );
    assert_eq!(
        hash_of(&scheme.equations[0].result),
        hash_of(&scheme.equations[0].result.clone())
    );

    let json = serde_json::to_string_pretty(&scheme).expect("scheme serializes");
    assert!(json.contains("DomainConstructorApp"));
    assert!(json.contains("AssociatedFamilyProjection"));
    assert!(json.contains("Append"));
    let decoded: AssociatedFamilyScheme = serde_json::from_str(&json).expect("scheme deserializes");
    assert_eq!(decoded, scheme);
    assert_eq!(hash_of(&decoded), hash_of(&scheme));
}

#[test]
fn v4_associated_family_summary_roundtrips_and_version_contract_accepts_v4() {
    let family = public_append_family_summary("Ys", "sha256:family-a");
    let same = public_append_family_summary("Ys", "sha256:family-a");

    assert_eq!(family, same);
    assert_eq!(hash_of(&family), hash_of(&same));
    assert_eq!(
        family.export_mode,
        AssociatedFamilyExportMode::TransparentEquations
    );
    assert_eq!(
        family.revalidation_metadata.spec_version,
        SummaryVersion::SPEC063_ASSOCIATED_FAMILY_V4
    );
    assert!(
        family
            .dependency_closure
            .closure_metadata
            .public_closure_checked
    );
    assert!(family.dependency_closure.associated_families[0].normalizer_available);
    assert!(!family.dependency_closure.associated_families[0].source_visible);
    assert_eq!(family.dependency_closure.associated_projections.len(), 1);
    assert!(
        family.dependency_closure.associated_projections[0].is_neutral_blocked_or_unavailable()
    );

    let json = serde_json::to_string_pretty(&family).expect("family summary serializes");
    assert!(json.contains("TransparentEquations"));
    assert!(json.contains("normalizer_available"));
    assert!(json.contains("sha256:family-a"));
    let decoded: AssociatedFamilySummary =
        serde_json::from_str(&json).expect("family summary deserializes");
    assert_eq!(decoded, family);
    assert_eq!(hash_of(&decoded), hash_of(&family));

    let module = ModuleSemanticSummary::new(module_identity(10, "v4"))
        .with_version(SummaryVersion::SPEC063_ASSOCIATED_FAMILY_V4)
        .with_exported_associated_family(family);

    assert_eq!(module.version, SummaryVersion::SPEC063_ASSOCIATED_FAMILY_V4);
    assert_eq!(module.exported_associated_families.len(), 1);
    module
        .validate_summary_version_contract()
        .expect("V4 carries public associated-family summaries");
}

#[test]
fn associated_family_summaries_participate_in_semantic_cache_keys() {
    let module = module_identity(11, "cache");
    let base = ModuleSemanticSummary::new(module.clone())
        .with_version(SummaryVersion::SPEC063_ASSOCIATED_FAMILY_V4)
        .with_exported_associated_family(public_append_family_summary("Ys", "sha256:family-a"));
    let changed_result = ModuleSemanticSummary::new(module.clone())
        .with_version(SummaryVersion::SPEC063_ASSOCIATED_FAMILY_V4)
        .with_exported_associated_family(public_append_family_summary(
            "DifferentYs",
            "sha256:family-a",
        ));
    let changed_dependency = ModuleSemanticSummary::new(module.clone())
        .with_version(SummaryVersion::SPEC063_ASSOCIATED_FAMILY_V4)
        .with_exported_associated_family(public_append_family_summary("Ys", "sha256:family-b"));

    let mut changed_metadata_family = public_append_family_summary("Ys", "sha256:family-a");
    changed_metadata_family
        .dependency_closure
        .closure_metadata
        .helper_family_count = 2;
    let changed_metadata = ModuleSemanticSummary::new(module)
        .with_version(SummaryVersion::SPEC063_ASSOCIATED_FAMILY_V4)
        .with_exported_associated_family(changed_metadata_family);

    assert_ne!(
        base.semantic_cache_key(),
        changed_result.semantic_cache_key()
    );
    assert_ne!(
        base.semantic_cache_key(),
        changed_dependency.semantic_cache_key()
    );
    assert_ne!(
        base.semantic_cache_key(),
        changed_metadata.semantic_cache_key()
    );
}

#[test]
fn v1_v2_and_v3_summaries_with_associated_family_facts_are_malformed() {
    for version in [
        SummaryVersion::SPEC057_ORDINARY_TYPE_V1,
        SummaryVersion::SPEC059_SEALED_DOMAIN_V2,
        SummaryVersion::SPEC062_TYPE_COMPUTATION_V3,
    ] {
        let summary = ModuleSemanticSummary::new(module_identity(12, "pre-v4-with-family"))
            .with_version(version)
            .with_exported_associated_family(public_append_family_summary("Ys", "sha256:bad"));

        assert_eq!(
            summary.validate_summary_version_contract(),
            Err(ModuleSemanticSummaryValidationError::AssociatedFamiliesRequireV4 { version })
        );
    }
}
