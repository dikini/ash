//! TASK-850: summary versioning/cache invalidation at the core summary boundary.

use ash_core::ast::{Span, Visibility};
use ash_core::kind::Kind;
use ash_core::module_graph::{CrateId, ModuleId};
use ash_core::semantic_summary::{
    AssociatedMemberIdentityId, AssociatedMemberIdentitySummary, DomainConstructorId,
    DomainConstructorSummary, InterfaceIdentityId, InterfaceIdentitySummary, ModuleIdentity,
    ModuleSemanticSummary, ModuleSemanticSummaryValidationError, ModuleSourceOrigin,
    ModuleSummaryRef, ReExportSummary, RepresentationExposure, ReservedSemanticIdentitySlots,
    SealedDomainId, SealedDomainSummary, SourceAnchor, SourceOrigin, SummaryVersion, TypeDeclId,
    TypeDeclSummary, TypeFunctionClosureMetadata, TypeFunctionDependencySummaryRef,
    TypeFunctionExportMode, TypeFunctionRevalidationMetadata, TypeFunctionSummary,
    TypeRepresentationSummary,
};
use ash_core::type_ir::{
    CanonicalTypeExpr, TypeComputationHeadId, TypeFunctionEquation, TypeFunctionPattern,
    TypeFunctionPatternConstraint, TypeFunctionResultConstraint, TypeFunctionResultExpr,
    TypeFunctionSourceAnchors,
};

fn module_identity(module_id: usize, name: &str) -> ModuleIdentity {
    ModuleIdentity::new(
        Some(CrateId(850)),
        ModuleId(module_id),
        vec!["task850".to_string(), name.to_string()],
        ModuleSourceOrigin::Synthetic {
            reason: format!("task-850 {name}"),
        },
    )
}

fn anchor(label: &str) -> SourceAnchor {
    SourceAnchor::new(
        SourceOrigin::Synthetic {
            reason: "task-850 summary cache test".to_string(),
        },
        Some(Span { start: 1, end: 2 }),
        label,
    )
}

fn ordinary_type(module: &ModuleIdentity) -> TypeDeclSummary {
    ordinary_type_with_params(module, Vec::new())
}

fn ordinary_type_with_params(module: &ModuleIdentity, params: Vec<String>) -> TypeDeclSummary {
    let id = TypeDeclId::ordinary(module.clone(), "Token");
    TypeDeclSummary::new(
        id,
        "Token",
        Visibility::Public,
        RepresentationExposure::Opaque,
        TypeRepresentationSummary::opaque(false),
        anchor("type Token"),
    )
    .with_params(params)
}

fn sealed_domain(
    module: &ModuleIdentity,
    name: &str,
    constructor_name: &str,
) -> SealedDomainSummary {
    let domain_id = SealedDomainId::new(module.clone(), name);
    let constructor_id = DomainConstructorId::new(domain_id.clone(), constructor_name);
    SealedDomainSummary::new(domain_id, name, Visibility::Public, anchor("sealed domain"))
        .with_constructor(DomainConstructorSummary::new(
            constructor_id,
            constructor_name,
            Vec::new(),
            anchor("domain constructor"),
        ))
}

fn dependency_ref(digest: &str) -> TypeFunctionDependencySummaryRef {
    TypeFunctionDependencySummaryRef {
        summary_ref: ModuleSummaryRef {
            module: module_identity(900, "dependency"),
            version: SummaryVersion::SPEC059_SEALED_DOMAIN_V2,
        },
        digest: Some(digest.to_string()),
        compiler_algorithm_version: Some("task-850-algorithm".to_string()),
    }
}

fn type_function(
    module: &ModuleIdentity,
    name: &str,
    result_name: &str,
    digest: &str,
) -> TypeFunctionSummary {
    let head = TypeComputationHeadId::new(module.clone(), name);
    TypeFunctionSummary {
        exported_name: name.to_string(),
        head: head.clone(),
        visibility: Visibility::Public,
        params: vec![],
        return_type: CanonicalTypeExpr::Primitive("Type".to_string()),
        return_kind: Kind::Type,
        result_constraint: TypeFunctionResultConstraint::Kind(Kind::Type),
        export_mode: TypeFunctionExportMode::TransparentEquations,
        source_anchors: TypeFunctionSourceAnchors {
            definition: anchor("type fn definition"),
            decreases: None,
        },
        equations: vec![TypeFunctionEquation {
            head,
            ordinal: 0,
            patterns: vec![TypeFunctionPattern::Wildcard {
                constraint: TypeFunctionPatternConstraint::Kind(Kind::Type),
                source_anchor: anchor("wildcard"),
            }],
            result: TypeFunctionResultExpr::Primitive {
                name: result_name.to_string(),
                kind: Kind::Type,
                constraint: TypeFunctionResultConstraint::Kind(Kind::Type),
                source_anchor: anchor("result"),
            },
            source_anchor: anchor("case"),
            case_head_anchor: anchor("case head"),
        }],
        dependency_summary_refs: vec![dependency_ref(digest)],
        closure_metadata: TypeFunctionClosureMetadata {
            public_closure_checked: true,
            public_ordinary_type_count: 1,
            public_sealed_domain_count: 1,
            public_type_function_count: 1,
            public_projection_count: 0,
        },
        revalidation_metadata: TypeFunctionRevalidationMetadata {
            spec_version: SummaryVersion::SPEC062_TYPE_COMPUTATION_V3,
            structural_recursion_checked: true,
            kind_and_domain_checked: true,
            coverage_and_overlap_checked: true,
            decreases_param: None,
        },
    }
}

#[test]
fn cache_key_changes_when_summary_version_changes_even_for_identical_ordinary_types() {
    let module = module_identity(1, "versioned");
    let v1 = ModuleSemanticSummary::new(module.clone()).with_exported_type(ordinary_type(&module));
    let v2 = ModuleSemanticSummary::new(module.clone())
        .with_version(SummaryVersion::SPEC059_SEALED_DOMAIN_V2)
        .with_exported_type(ordinary_type(&module));

    assert_ne!(v1.semantic_cache_key(), v2.semantic_cache_key());
}

#[test]
fn cache_key_changes_for_type_function_equations_dependencies_and_sealed_domains() {
    let module = module_identity(2, "facts");
    let base = ModuleSemanticSummary::new(module.clone())
        .with_version(SummaryVersion::SPEC062_TYPE_COMPUTATION_V3)
        .with_exported_type(ordinary_type(&module))
        .with_exported_sealed_domain(sealed_domain(&module, "Domain", "A"))
        .with_exported_type_function(type_function(&module, "Normalize", "A", "sha256:a"));

    let changed_equation = ModuleSemanticSummary::new(module.clone())
        .with_version(SummaryVersion::SPEC062_TYPE_COMPUTATION_V3)
        .with_exported_type(ordinary_type(&module))
        .with_exported_sealed_domain(sealed_domain(&module, "Domain", "A"))
        .with_exported_type_function(type_function(&module, "Normalize", "B", "sha256:a"));
    let changed_dependency = ModuleSemanticSummary::new(module.clone())
        .with_version(SummaryVersion::SPEC062_TYPE_COMPUTATION_V3)
        .with_exported_type(ordinary_type(&module))
        .with_exported_sealed_domain(sealed_domain(&module, "Domain", "A"))
        .with_exported_type_function(type_function(&module, "Normalize", "A", "sha256:b"));
    let changed_domain = ModuleSemanticSummary::new(module.clone())
        .with_version(SummaryVersion::SPEC062_TYPE_COMPUTATION_V3)
        .with_exported_type(ordinary_type(&module))
        .with_exported_sealed_domain(sealed_domain(&module, "Domain", "Other"))
        .with_exported_type_function(type_function(&module, "Normalize", "A", "sha256:a"));

    assert_ne!(
        base.semantic_cache_key(),
        changed_equation.semantic_cache_key()
    );
    assert_ne!(
        base.semantic_cache_key(),
        changed_dependency.semantic_cache_key()
    );
    assert_ne!(
        base.semantic_cache_key(),
        changed_domain.semantic_cache_key()
    );
}

#[test]
fn cache_key_changes_for_ordinary_type_params_import_refs_and_closure_metadata() {
    let module = module_identity(4, "metadata");
    let base = ModuleSemanticSummary::new(module.clone())
        .with_version(SummaryVersion::SPEC062_TYPE_COMPUTATION_V3)
        .with_exported_type(ordinary_type(&module))
        .with_exported_type_function(type_function(&module, "Normalize", "A", "sha256:a"));

    let changed_type_params = ModuleSemanticSummary::new(module.clone())
        .with_version(SummaryVersion::SPEC062_TYPE_COMPUTATION_V3)
        .with_exported_type(ordinary_type_with_params(&module, vec!["T".to_string()]))
        .with_exported_type_function(type_function(&module, "Normalize", "A", "sha256:a"));

    let changed_import_ref = ModuleSemanticSummary::new(module.clone())
        .with_version(SummaryVersion::SPEC062_TYPE_COMPUTATION_V3)
        .with_exported_type(ordinary_type(&module))
        .with_imported_summary_ref(ModuleSummaryRef {
            module: module_identity(901, "other-dependency"),
            version: SummaryVersion::SPEC059_SEALED_DOMAIN_V2,
        })
        .with_exported_type_function(type_function(&module, "Normalize", "A", "sha256:a"));

    let mut changed_metadata_fn = type_function(&module, "Normalize", "A", "sha256:a");
    changed_metadata_fn
        .closure_metadata
        .public_type_function_count = 2;
    let changed_closure_metadata = ModuleSemanticSummary::new(module.clone())
        .with_version(SummaryVersion::SPEC062_TYPE_COMPUTATION_V3)
        .with_exported_type(ordinary_type(&module))
        .with_exported_type_function(changed_metadata_fn);

    assert_ne!(
        base.semantic_cache_key(),
        changed_type_params.semantic_cache_key(),
        "ordinary type parameter/arity changes must invalidate summary cache keys"
    );
    assert_ne!(
        base.semantic_cache_key(),
        changed_import_ref.semantic_cache_key(),
        "imported summary references must invalidate summary cache keys"
    );
    assert_ne!(
        base.semantic_cache_key(),
        changed_closure_metadata.semantic_cache_key(),
        "type-function closure metadata must invalidate summary cache keys"
    );
}

#[test]
fn cache_key_changes_for_all_current_summary_surfaces() {
    let module = module_identity(5, "all-surfaces");
    let base = ModuleSemanticSummary::new(module.clone())
        .with_version(SummaryVersion::SPEC062_TYPE_COMPUTATION_V3)
        .with_exported_type(ordinary_type(&module));
    let origin = TypeDeclId::ordinary(module.clone(), "Token");

    let with_re_export = base
        .clone()
        .with_re_export(ReExportSummary::new(vec!["Alias".to_string()], origin));

    let interface = InterfaceIdentityId::new(module.clone(), "Iterable");
    let with_interface = ModuleSemanticSummary {
        interface_identities: vec![InterfaceIdentitySummary::new(
            interface.clone(),
            "Iterable",
            vec!["Iterable".to_string()],
            anchor("interface"),
        )],
        ..base.clone()
    };

    let member = AssociatedMemberIdentityId::associated_type(
        interface,
        "Item",
        vec!["Iterable".to_string(), "Item".to_string()],
    );
    let with_associated_member = ModuleSemanticSummary {
        associated_member_identities: vec![AssociatedMemberIdentitySummary::new(
            member,
            "Item",
            anchor("associated member"),
        )],
        ..base.clone()
    };

    let with_reserved_slots = ModuleSemanticSummary {
        reserved_identity_slots: ReservedSemanticIdentitySlots {
            future_type_functions: vec!["FutureNormalize".to_string()],
            ..ReservedSemanticIdentitySlots::default()
        },
        ..base.clone()
    };

    let with_diagnostic_anchor = ModuleSemanticSummary {
        diagnostic_anchors: vec![anchor("diagnostic")],
        ..base.clone()
    };

    for changed in [
        with_re_export,
        with_interface,
        with_associated_member,
        with_reserved_slots,
        with_diagnostic_anchor,
    ] {
        assert_ne!(
            base.semantic_cache_key(),
            changed.semantic_cache_key(),
            "every current semantic-summary surface must invalidate the structural cache key"
        );
    }
}

#[test]
fn v1_and_v2_with_computation_facts_are_not_computation_aware_summaries() {
    for version in [
        SummaryVersion::SPEC057_ORDINARY_TYPE_V1,
        SummaryVersion::SPEC059_SEALED_DOMAIN_V2,
    ] {
        let module = module_identity(3, "pre-v3-with-facts");
        let summary = ModuleSemanticSummary::new(module.clone())
            .with_version(version)
            .with_exported_type(ordinary_type(&module))
            .with_exported_type_function(type_function(&module, "Bad", "A", "sha256:a"));

        assert_eq!(
            summary.validate_summary_version_contract(),
            Err(ModuleSemanticSummaryValidationError::TypeFunctionsRequireV3 { version })
        );
    }
}
