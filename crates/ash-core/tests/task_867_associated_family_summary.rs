use ash_core::kind::Kind;
use ash_core::module_graph::{CrateId, ModuleId};
use ash_core::semantic_summary::{
    AssociatedFamilyClosureMetadata, AssociatedFamilyDependencyClosure, AssociatedFamilyExportMode,
    AssociatedFamilyRevalidationMetadata, AssociatedFamilySummary, AssociatedMemberIdentityId,
    AssociatedMemberIdentitySummary, InterfaceIdentityId, InterfaceIdentitySummary, ModuleIdentity,
    ModuleSemanticSummary, ModuleSemanticSummaryValidationError, ModuleSourceOrigin, SourceAnchor,
    SourceOrigin, SummaryVersion,
};
use ash_core::type_ir::{
    AssociatedFamilyEquation, AssociatedFamilyHeadId, AssociatedFamilyPattern,
    AssociatedFamilyResultConstraint, AssociatedFamilyResultExpr, AssociatedFamilyScheme,
    AssociatedFamilySchemeParam, CanonicalTypeExpr,
};

fn module(id: usize) -> ModuleIdentity {
    ModuleIdentity::new(
        Some(CrateId(867)),
        ModuleId(id),
        vec!["task867".into(), format!("m{id}")],
        ModuleSourceOrigin::Synthetic {
            reason: format!("TASK-867 core test module {id}"),
        },
    )
}

fn anchor(label: &str) -> SourceAnchor {
    SourceAnchor::new(
        SourceOrigin::Synthetic {
            reason: "task-867-core-test".into(),
        },
        None,
        label,
    )
}

fn family_summary(
    module: &ModuleIdentity,
    visible_name: &str,
    rhs_name: &str,
) -> AssociatedFamilySummary {
    let interface_identity = InterfaceIdentityId::new(module.clone(), "Iterator");
    let member_identity = AssociatedMemberIdentityId::associated_type(
        interface_identity.clone(),
        visible_name,
        vec!["Iterator".into(), visible_name.into()],
    );
    let head = AssociatedFamilyHeadId {
        interface: interface_identity.clone(),
        member: member_identity.clone(),
    };
    let constraint = AssociatedFamilyResultConstraint::Kind(Kind::Type);
    let scheme = AssociatedFamilyScheme {
        head: head.clone(),
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
            head: head.clone(),
            ordinal: 0,
            interface_arg_patterns: vec![AssociatedFamilyPattern::Var {
                name: "T".into(),
                constraint: constraint.clone(),
                source_anchor: anchor("pattern T"),
            }],
            result: AssociatedFamilyResultExpr::Primitive {
                name: rhs_name.into(),
                kind: Kind::Type,
                constraint: constraint.clone(),
                source_anchor: anchor("rhs"),
            },
            decreases: None,
            source_anchor: anchor("equation"),
            case_head_anchor: anchor("case"),
        }],
        source_anchor: anchor("scheme"),
    };

    AssociatedFamilySummary {
        head,
        interface_identity,
        member_identity,
        visible_name: visible_name.into(),
        result_domain: CanonicalTypeExpr::Primitive("Type".into()),
        result_kind: Kind::Type,
        export_mode: AssociatedFamilyExportMode::TransparentEquations,
        schemes: vec![scheme],
        dependency_closure: AssociatedFamilyDependencyClosure {
            ordinary_types: vec![],
            sealed_domains: vec![],
            domain_constructors: vec![],
            type_functions: vec![],
            associated_projections: vec![],
            associated_families: vec![],
            type_function_summaries: vec![],
            closure_metadata: AssociatedFamilyClosureMetadata {
                public_closure_checked: true,
                public_ordinary_type_count: 0,
                public_sealed_domain_count: 0,
                public_domain_constructor_count: 0,
                public_type_function_count: 0,
                public_associated_family_count: 1,
                public_projection_count: 0,
                helper_family_count: 0,
            },
        },
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

fn summary_with_family(
    version: SummaryVersion,
    family: AssociatedFamilySummary,
) -> ModuleSemanticSummary {
    let module = family.interface_identity.module.clone();
    let interface_summary = InterfaceIdentitySummary::new(
        family.interface_identity.clone(),
        "Iterator",
        vec!["Iterator".into()],
        anchor("interface"),
    );
    let member_summary = AssociatedMemberIdentitySummary::new(
        family.member_identity.clone(),
        family.visible_name.clone(),
        anchor("member"),
    );
    ModuleSemanticSummary::new(module)
        .with_version(version)
        .with_interface_identity(interface_summary)
        .with_associated_member_identity(member_summary)
        .with_exported_associated_family(family)
}

#[test]
fn task_867_v1_v2_v3_reject_non_empty_associated_family_facts() {
    for version in [
        SummaryVersion::SPEC057_ORDINARY_TYPE_V1,
        SummaryVersion::SPEC059_SEALED_DOMAIN_V2,
        SummaryVersion::SPEC062_TYPE_COMPUTATION_V3,
    ] {
        let family = family_summary(&module(version.0 as usize), "Item", "String");
        let summary = summary_with_family(version, family);

        let err = summary
            .validate_summary_version_contract()
            .expect_err("pre-V4 summaries must reject associated-family payloads");

        assert_eq!(
            err,
            ModuleSemanticSummaryValidationError::AssociatedFamiliesRequireV4 { version }
        );
    }
}

#[test]
fn task_867_v4_accepts_non_empty_associated_family_facts() {
    let family = family_summary(&module(4), "Item", "String");
    let summary = summary_with_family(SummaryVersion::SPEC063_ASSOCIATED_FAMILY_V4, family);

    summary
        .validate_summary_version_contract()
        .expect("V4 is the first schema allowed to carry associated-family summaries");
}

#[test]
fn task_867_semantic_cache_key_changes_for_associated_family_payloads() {
    let module = module(40);
    let empty_v4 = ModuleSemanticSummary::new(module.clone())
        .with_version(SummaryVersion::SPEC063_ASSOCIATED_FAMILY_V4);
    let with_item = summary_with_family(
        SummaryVersion::SPEC063_ASSOCIATED_FAMILY_V4,
        family_summary(&module, "Item", "String"),
    );
    let with_other_rhs = summary_with_family(
        SummaryVersion::SPEC063_ASSOCIATED_FAMILY_V4,
        family_summary(&module, "Item", "Int"),
    );

    assert_ne!(
        empty_v4.semantic_cache_key(),
        with_item.semantic_cache_key(),
        "associated-family facts must participate in semantic cache invalidation"
    );
    assert_ne!(
        with_item.semantic_cache_key(),
        with_other_rhs.semantic_cache_key(),
        "scheme/RHS changes must change the summary cache key"
    );
}
