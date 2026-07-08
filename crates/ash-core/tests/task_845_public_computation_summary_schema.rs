use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

use ash_core::ast::{Span, Visibility};
use ash_core::kind::Kind;
use ash_core::module_graph::{CrateId, ModuleId};
use ash_core::semantic_summary::{
    ModuleIdentity, ModuleSemanticSummary, ModuleSemanticSummaryValidationError,
    ModuleSourceOrigin, ModuleSummaryRef, SourceAnchor, SourceOrigin, SummaryVersion,
    TypeFunctionClosureMetadata, TypeFunctionDependencySummaryRef, TypeFunctionExportMode,
    TypeFunctionParamSummary, TypeFunctionRevalidationMetadata, TypeFunctionSummary,
};
use ash_core::type_ir::{
    CanonicalTypeExpr, TypeComputationHeadId, TypeFunctionEquation, TypeFunctionPattern,
    TypeFunctionPatternConstraint, TypeFunctionResultConstraint, TypeFunctionResultExpr,
    TypeFunctionSourceAnchors,
};

fn hash_of<T: Hash>(value: &T) -> u64 {
    let mut hasher = DefaultHasher::new();
    value.hash(&mut hasher);
    hasher.finish()
}

fn module_identity(module_id: usize, name: &str) -> ModuleIdentity {
    ModuleIdentity::new(
        Some(CrateId(845)),
        ModuleId(module_id),
        vec!["phase114".to_string(), name.to_string()],
        ModuleSourceOrigin::Synthetic {
            reason: format!("task-845 {name}"),
        },
    )
}

fn anchor(label: &str, start: usize, end: usize) -> SourceAnchor {
    SourceAnchor::new(
        SourceOrigin::Synthetic {
            reason: "task-845 public computation summary schema test".to_string(),
        },
        Some(Span { start, end }),
        label,
    )
}

fn dependency_ref() -> TypeFunctionDependencySummaryRef {
    TypeFunctionDependencySummaryRef {
        summary_ref: ModuleSummaryRef {
            module: module_identity(100, "deps"),
            version: SummaryVersion::SPEC059_SEALED_DOMAIN_V2,
        },
        digest: Some("sha256:sealed-domain-digest".to_string()),
        compiler_algorithm_version: Some("spec-062-mvp".to_string()),
    }
}

fn public_append_summary() -> TypeFunctionSummary {
    let module = module_identity(1, "type_fns");
    let head = TypeComputationHeadId::new(module, "Append");

    TypeFunctionSummary {
        exported_name: "Append".to_string(),
        head: head.clone(),
        visibility: Visibility::Public,
        params: vec![
            TypeFunctionParamSummary {
                name: "xs".to_string(),
                ty: CanonicalTypeExpr::Primitive("TypeList".to_string()),
                kind: Kind::Type,
                domain_constraint: None,
                source_anchor: anchor("param xs", 10, 12),
            },
            TypeFunctionParamSummary {
                name: "ys".to_string(),
                ty: CanonicalTypeExpr::Primitive("TypeList".to_string()),
                kind: Kind::Type,
                domain_constraint: None,
                source_anchor: anchor("param ys", 14, 16),
            },
        ],
        return_type: CanonicalTypeExpr::Primitive("TypeList".to_string()),
        return_kind: Kind::Type,
        result_constraint: TypeFunctionResultConstraint::Kind(Kind::Type),
        export_mode: TypeFunctionExportMode::TransparentEquations,
        source_anchors: TypeFunctionSourceAnchors {
            definition: anchor("pub type fn Append", 0, 80),
            decreases: Some(anchor("decreases xs", 30, 42)),
        },
        equations: vec![TypeFunctionEquation {
            head,
            ordinal: 0,
            patterns: vec![
                TypeFunctionPattern::Var {
                    name: "xs".to_string(),
                    constraint: TypeFunctionPatternConstraint::Kind(Kind::Type),
                    source_anchor: anchor("pattern xs", 50, 52),
                },
                TypeFunctionPattern::Var {
                    name: "ys".to_string(),
                    constraint: TypeFunctionPatternConstraint::Kind(Kind::Type),
                    source_anchor: anchor("pattern ys", 54, 56),
                },
            ],
            result: TypeFunctionResultExpr::Var {
                name: "ys".to_string(),
                kind: Kind::Type,
                constraint: TypeFunctionResultConstraint::Kind(Kind::Type),
                source_anchor: anchor("result ys", 60, 62),
            },
            source_anchor: anchor("case Append<xs, ys>", 45, 65),
            case_head_anchor: anchor("case head Append", 45, 51),
        }],
        dependency_summary_refs: vec![dependency_ref()],
        closure_metadata: TypeFunctionClosureMetadata {
            public_closure_checked: true,
            public_ordinary_type_count: 0,
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
fn public_type_function_summary_is_equal_hashable_and_serde_roundtrips() {
    let summary = public_append_summary();
    let same = public_append_summary();

    assert_eq!(summary, same);
    assert_eq!(hash_of(&summary), hash_of(&same));
    assert_eq!(
        summary.export_mode,
        TypeFunctionExportMode::TransparentEquations
    );
    assert_eq!(summary.visibility, Visibility::Public);
    assert_eq!(
        summary
            .equations
            .iter()
            .map(|eq| eq.ordinal)
            .collect::<Vec<_>>(),
        [0]
    );

    let json = serde_json::to_string_pretty(&summary).expect("summary serializes");
    assert!(json.contains("TransparentEquations"));
    assert!(json.contains("sha256:sealed-domain-digest"));
    assert!(json.contains("public_closure_checked"));

    let decoded: TypeFunctionSummary = serde_json::from_str(&json).expect("summary deserializes");
    assert_eq!(decoded, summary);
    assert_eq!(hash_of(&decoded), hash_of(&summary));
}

#[test]
fn module_summary_defaults_exported_type_functions_for_older_payloads() {
    let summary = ModuleSemanticSummary::new(module_identity(2, "older-payload"));
    let mut value = serde_json::to_value(summary).expect("module summary serializes");
    value
        .as_object_mut()
        .expect("object")
        .remove("exported_type_functions");

    let decoded: ModuleSemanticSummary =
        serde_json::from_value(value).expect("older payload decodes");
    assert!(decoded.exported_type_functions.is_empty());
    decoded
        .validate_summary_version_contract()
        .expect("V1 with default-empty computation field remains valid");

    ModuleSemanticSummary::new(module_identity(22, "v2_empty"))
        .with_version(SummaryVersion::SPEC059_SEALED_DOMAIN_V2)
        .validate_summary_version_contract()
        .expect("V2 with empty computation field remains valid");
}

#[test]
fn v3_module_summary_may_carry_public_type_function_summaries() {
    let summary = ModuleSemanticSummary::new(module_identity(3, "v3"))
        .with_version(SummaryVersion::SPEC062_TYPE_COMPUTATION_V3)
        .with_exported_type_function(public_append_summary());

    assert_eq!(summary.version, SummaryVersion::SPEC062_TYPE_COMPUTATION_V3);
    assert_eq!(summary.exported_type_functions.len(), 1);
    summary
        .validate_summary_version_contract()
        .expect("V3 may carry public computation summaries");
}

#[test]
fn v1_and_v2_with_non_empty_type_functions_are_malformed() {
    for version in [
        SummaryVersion::SPEC057_ORDINARY_TYPE_V1,
        SummaryVersion::SPEC059_SEALED_DOMAIN_V2,
    ] {
        let summary = ModuleSemanticSummary::new(module_identity(4, "malformed"))
            .with_version(version)
            .with_exported_type_function(public_append_summary());

        assert_eq!(
            summary.validate_summary_version_contract(),
            Err(ModuleSemanticSummaryValidationError::TypeFunctionsRequireV3 { version })
        );
    }
}

#[test]
fn unknown_future_summary_versions_are_rejected_before_registration() {
    let summary =
        ModuleSemanticSummary::new(module_identity(5, "future")).with_version(SummaryVersion(99));

    assert_eq!(
        summary.validate_summary_version_contract(),
        Err(
            ModuleSemanticSummaryValidationError::UnsupportedSummaryVersion {
                version: SummaryVersion(99),
            }
        )
    );
}

#[test]
fn dependency_refs_preserve_summary_version_digest_and_algorithm_metadata() {
    let summary = public_append_summary();
    let dep = summary
        .dependency_summary_refs
        .first()
        .expect("test summary includes dependency ref");

    assert_eq!(
        dep.summary_ref.version,
        SummaryVersion::SPEC059_SEALED_DOMAIN_V2
    );
    assert_eq!(dep.digest.as_deref(), Some("sha256:sealed-domain-digest"));
    assert_eq!(
        dep.compiler_algorithm_version.as_deref(),
        Some("spec-062-mvp")
    );

    let changed_digest = TypeFunctionDependencySummaryRef {
        digest: Some("sha256:different".to_string()),
        ..dep.clone()
    };
    assert_ne!(hash_of(dep), hash_of(&changed_digest));
}
