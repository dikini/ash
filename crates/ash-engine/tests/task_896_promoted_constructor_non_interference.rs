//! TASK-896: promoted constructor semantic summaries do not interfere with engine ADT/domain behavior.
//!
//! This test stays at the engine-visible semantic summary import boundary instead
//! of adding public promoted-constructor source syntax.

use ash_core::ast::{TypeBody, VariantDef, VariantPayload, Visibility};
use ash_core::kind::Kind;
use ash_core::module_graph::{CrateId, ModuleId};
use ash_core::semantic_summary::{
    ConstructorId, ConstructorPayloadKind, ConstructorSummary, ModuleIdentity,
    ModuleSemanticSummary, ModuleSourceOrigin, PromotedConstructorId, PromotedConstructorSummary,
    PromotedDataKindId, PromotedDataKindSummary, RepresentationExposure, SourceAnchor,
    SourceOrigin, SummaryVersion, TypeDeclId, TypeDeclSummary, TypeFunctionClosureMetadata,
    TypeFunctionExportMode, TypeFunctionRevalidationMetadata, TypeFunctionSummary,
    TypeRepresentationSummary,
};
use ash_core::type_ir::{
    CanonicalTypeExpr, TypeComputationHeadId, TypeFunctionEquation, TypeFunctionResultConstraint,
    TypeFunctionResultExpr, TypeFunctionSourceAnchors,
};
use ash_typeck::TypeEnv;

fn module(id: usize) -> ModuleIdentity {
    ModuleIdentity::new(
        Some(CrateId(8960)),
        ModuleId(id),
        vec!["task896_engine".to_string(), format!("m{id}")],
        ModuleSourceOrigin::Synthetic {
            reason: format!("task-896-engine-{id}"),
        },
    )
}

fn anchor(label: &str) -> SourceAnchor {
    SourceAnchor::new(
        SourceOrigin::Synthetic {
            reason: "task-896-engine-non-interference".into(),
        },
        None,
        label,
    )
}

fn unit_variant(name: &str) -> VariantDef {
    VariantDef {
        name: name.into(),
        fields: vec![],
        payload: VariantPayload::Unit,
    }
}

fn source_type(module: &ModuleIdentity, name: &str, variants: Vec<VariantDef>) -> TypeDeclSummary {
    TypeDeclSummary::new(
        TypeDeclId::ordinary(module.clone(), name),
        name,
        Visibility::Public,
        RepresentationExposure::Exposed,
        TypeRepresentationSummary::Exposed(TypeBody::Enum(variants)),
        anchor(name),
    )
}

fn promoted_kind_id(
    module: &ModuleIdentity,
    source_type: &str,
    kind_name: &str,
) -> PromotedDataKindId {
    PromotedDataKindId::new(
        module.clone(),
        TypeDeclId::ordinary(module.clone(), source_type),
        kind_name,
    )
}

fn promoted_ctor_summary(
    kind: &PromotedDataKindId,
    source_type: &str,
    ctor_name: &str,
) -> PromotedConstructorSummary {
    let source_ctor = ConstructorId::variant(
        TypeDeclId::ordinary(kind.source_type.module.clone(), source_type),
        ctor_name,
        ConstructorPayloadKind::Unit,
    );
    PromotedConstructorSummary::new(
        PromotedConstructorId::new(kind.clone(), source_ctor.clone(), ctor_name),
        ctor_name,
        source_ctor,
        vec![],
        Visibility::Public,
        anchor(ctor_name),
    )
}

fn promoted_type_function(
    module: &ModuleIdentity,
    kind: &PromotedDataKindId,
    ctor: &PromotedConstructorId,
) -> TypeFunctionSummary {
    let head = TypeComputationHeadId::new(module.clone(), "PromotedZero");
    TypeFunctionSummary {
        exported_name: "PromotedZero".into(),
        head: head.clone(),
        visibility: Visibility::Public,
        params: vec![],
        return_type: CanonicalTypeExpr::Primitive("Type".into()),
        return_kind: Kind::Type,
        result_constraint: TypeFunctionResultConstraint::Kind(Kind::Type),
        export_mode: TypeFunctionExportMode::TransparentEquations,
        source_anchors: TypeFunctionSourceAnchors {
            definition: anchor("type fn PromotedZero"),
            decreases: None,
        },
        equations: vec![TypeFunctionEquation {
            head,
            ordinal: 0,
            patterns: vec![],
            result: TypeFunctionResultExpr::PromotedDataConstructorApp {
                constructor: Box::new(ctor.clone()),
                data_kind: Box::new(kind.clone()),
                args: vec![],
                kind: Kind::Type,
                constraint: TypeFunctionResultConstraint::Kind(Kind::Type),
                source_anchor: anchor("PromotedZero rhs"),
            },
            source_anchor: anchor("case PromotedZero = Z"),
            case_head_anchor: anchor("PromotedZero case head"),
        }],
        dependency_summary_refs: vec![],
        closure_metadata: TypeFunctionClosureMetadata {
            public_closure_checked: true,
            public_ordinary_type_count: 1,
            public_sealed_domain_count: 0,
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
fn promoted_constructor_summary_import_does_not_register_runtime_constructor_or_sealed_domain() {
    let promoted_module = module(1);
    let ordinary_module = module(2);

    let nat_type = source_type(&promoted_module, "Nat", vec![unit_variant("Z")]);
    let nat_kind = promoted_kind_id(&promoted_module, "Nat", "NatKind");
    let z_promoted = promoted_ctor_summary(&nat_kind, "Nat", "Z");
    let z_promoted_id = z_promoted.id.clone();
    let promoted_summary = ModuleSemanticSummary::new(promoted_module.clone())
        .with_version(SummaryVersion::SPEC065_PROMOTED_DATA_KIND_V6)
        .with_exported_type(nat_type)
        .with_exported_promoted_data_kind(
            PromotedDataKindSummary::new(
                nat_kind.clone(),
                "NatKind",
                Visibility::Public,
                TypeDeclId::ordinary(promoted_module.clone(), "Nat"),
                anchor("NatKind"),
            )
            .with_constructor(z_promoted),
        )
        .with_exported_type_function(promoted_type_function(
            &promoted_module,
            &nat_kind,
            &z_promoted_id,
        ));

    let status_id = TypeDeclId::ordinary(ordinary_module.clone(), "Status");
    let ordinary_summary = ModuleSemanticSummary::new(ordinary_module.clone())
        .with_version(SummaryVersion::SPEC065_PROMOTED_DATA_KIND_V6)
        .with_exported_type(source_type(
            &ordinary_module,
            "Status",
            vec![unit_variant("Ready")],
        ))
        .with_exported_constructor(ConstructorSummary::new(
            ConstructorId::variant(status_id.clone(), "Ready", ConstructorPayloadKind::Unit),
            status_id,
            "Ready",
            ConstructorPayloadKind::Unit,
            Visibility::Public,
            anchor("Ready"),
        ));

    let mut env = TypeEnv::new();
    env.register_module_semantic_summaries(&[promoted_summary, ordinary_summary])
        .expect("engine-visible V6 summaries import transactionally");

    assert!(
        env.lookup_promoted_constructor_by_id(&z_promoted_id)
            .is_some(),
        "promoted constructor should be available only through promoted metadata"
    );
    assert_eq!(
        env.lookup_constructor("Z"),
        None,
        "promoted constructor import must not create an ordinary runtime constructor"
    );
    assert!(
        env.lookup_sealed_domain("NatKind").is_none(),
        "promoted data kind import must not create a sealed-domain marker"
    );
    assert_eq!(
        env.lookup_constructor("Ready"),
        Some(("Status".to_string(), 0)),
        "ordinary ADT constructor behavior remains unchanged beside promoted metadata"
    );
}
