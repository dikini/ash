//! TASK-896: promoted constructor apps integrate with type functions and propositions.
//!
//! These tests stay at the core/summary/TypeEnv boundary. They intentionally do
//! not introduce parser lowering for promoted constructor source syntax.

use ash_core::ast::{TypeBody, TypeExpr, VariantDef, VariantPayload, Visibility};
use ash_core::kind::Kind;
use ash_core::module_graph::{CrateId, ModuleId};
use ash_core::semantic_summary::{
    AssociatedFamilyClosureMetadata, AssociatedFamilyDependencyClosure, AssociatedFamilyExportMode,
    AssociatedFamilyRevalidationMetadata, AssociatedFamilySummary, AssociatedMemberIdentityId,
    AssociatedMemberIdentitySummary, ConstructorId, ConstructorPayloadKind, InterfaceIdentityId,
    InterfaceIdentitySummary, ModuleIdentity, ModuleSemanticSummary, ModuleSourceOrigin,
    PromotedConstructorFieldSummary, PromotedConstructorId, PromotedConstructorSummary,
    PromotedDataKindId, PromotedDataKindSummary, PropositionFactRole as SummaryPropositionFactRole,
    PropositionFactSummary, RepresentationExposure, SourceAnchor, SourceOrigin, SummaryVersion,
    TypeDeclId, TypeDeclSummary, TypeFunctionClosureMetadata, TypeFunctionExportMode,
    TypeFunctionRevalidationMetadata, TypeFunctionSummary, TypeRepresentationSummary,
};
use ash_core::type_ir::{
    AssociatedFamilyEquation, AssociatedFamilyHeadId, AssociatedFamilyPattern,
    AssociatedFamilyResultConstraint, AssociatedFamilyResultExpr, AssociatedFamilyScheme,
    AssociatedFamilySchemeParam, CanonicalTypeExpr, NormalTypeExpr, PromotedConstructorApp,
    PropositionEvidenceRule, PropositionOutcome, TypeComputationHeadId, TypeDisequalityProposition,
    TypeEqualityProposition, TypeFunctionEquation, TypeFunctionResultConstraint,
    TypeFunctionResultExpr, TypeFunctionSourceAnchors, TypeProposition, TypePropositionTerm,
};
use ash_typeck::TypeEnv;
use ash_typeck::normalizer::Normalizer;

fn module(id: usize) -> ModuleIdentity {
    ModuleIdentity::new(
        Some(CrateId(896)),
        ModuleId(id),
        vec!["task896".to_string(), format!("m{id}")],
        ModuleSourceOrigin::Synthetic {
            reason: format!("task-896-{id}"),
        },
    )
}

fn anchor(label: &str) -> SourceAnchor {
    SourceAnchor::new(
        SourceOrigin::Synthetic {
            reason: "task-896-promoted-integration".into(),
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

fn tuple_variant(name: &str, fields: Vec<TypeExpr>) -> VariantDef {
    VariantDef {
        name: name.into(),
        fields: fields
            .iter()
            .enumerate()
            .map(|(index, ty)| (format!("_{index}"), ty.clone()))
            .collect(),
        payload: VariantPayload::Tuple(fields),
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
    promoted_ctor_summary_with_fields(
        kind,
        source_type,
        ctor_name,
        ConstructorPayloadKind::Unit,
        vec![],
    )
}

fn promoted_ctor_summary_with_fields(
    kind: &PromotedDataKindId,
    source_type: &str,
    ctor_name: &str,
    payload_kind: ConstructorPayloadKind,
    fields: Vec<PromotedConstructorFieldSummary>,
) -> PromotedConstructorSummary {
    let source_ctor = ConstructorId::variant(
        TypeDeclId::ordinary(kind.source_type.module.clone(), source_type),
        ctor_name,
        payload_kind,
    );
    PromotedConstructorSummary::new(
        PromotedConstructorId::new(kind.clone(), source_ctor.clone(), ctor_name),
        ctor_name,
        source_ctor,
        fields,
        Visibility::Public,
        anchor(ctor_name),
    )
}

fn promoted_nat_summary(
    module: &ModuleIdentity,
) -> (
    ModuleSemanticSummary,
    PromotedDataKindId,
    PromotedConstructorId,
) {
    let nat_type = source_type(module, "Nat", vec![unit_variant("Z")]);
    let nat_kind = promoted_kind_id(module, "Nat", "NatKind");
    let z_ctor = promoted_ctor_summary(&nat_kind, "Nat", "Z");
    let z_ctor_id = z_ctor.id.clone();
    let summary = ModuleSemanticSummary::new(module.clone())
        .with_version(SummaryVersion::SPEC065_PROMOTED_DATA_KIND_V6)
        .with_exported_type(nat_type)
        .with_exported_promoted_data_kind(
            PromotedDataKindSummary::new(
                nat_kind.clone(),
                "NatKind",
                Visibility::Public,
                TypeDeclId::ordinary(module.clone(), "Nat"),
                anchor("NatKind"),
            )
            .with_constructor(z_ctor),
        );
    (summary, nat_kind, z_ctor_id)
}

fn recursive_promoted_nat_summary(
    module: &ModuleIdentity,
) -> (
    ModuleSemanticSummary,
    PromotedDataKindId,
    PromotedConstructorId,
    PromotedConstructorId,
) {
    let nat_type = source_type(
        module,
        "Nat",
        vec![
            unit_variant("Z"),
            tuple_variant("S", vec![TypeExpr::Named("Nat".into())]),
        ],
    );
    let nat_kind = promoted_kind_id(module, "Nat", "NatKind");
    let z_ctor = promoted_ctor_summary(&nat_kind, "Nat", "Z");
    let z_ctor_id = z_ctor.id.clone();
    let s_ctor = promoted_ctor_summary_with_fields(
        &nat_kind,
        "Nat",
        "S",
        ConstructorPayloadKind::Tuple,
        vec![PromotedConstructorFieldSummary::new(
            "pred",
            Kind::Type,
            Some(nat_kind.clone()),
            anchor("pred"),
        )],
    );
    let s_ctor_id = s_ctor.id.clone();
    let summary = ModuleSemanticSummary::new(module.clone())
        .with_version(SummaryVersion::SPEC065_PROMOTED_DATA_KIND_V6)
        .with_exported_type(nat_type)
        .with_exported_promoted_data_kind(
            PromotedDataKindSummary::new(
                nat_kind.clone(),
                "NatKind",
                Visibility::Public,
                TypeDeclId::ordinary(module.clone(), "Nat"),
                anchor("NatKind"),
            )
            .with_constructor(z_ctor)
            .with_constructor(s_ctor),
        );
    (summary, nat_kind, z_ctor_id, s_ctor_id)
}

fn promoted_app(kind: &PromotedDataKindId, ctor: &PromotedConstructorId) -> PromotedConstructorApp {
    promoted_app_with_args(kind, ctor, vec![])
}

fn promoted_app_with_args(
    kind: &PromotedDataKindId,
    ctor: &PromotedConstructorId,
    args: Vec<CanonicalTypeExpr>,
) -> PromotedConstructorApp {
    PromotedConstructorApp {
        constructor: ctor.clone(),
        data_kind: kind.clone(),
        args,
        kind: Kind::Type,
    }
}

fn type_function_summary_returning_promoted_z(
    module: &ModuleIdentity,
    kind: &PromotedDataKindId,
    ctor: &PromotedConstructorId,
) -> TypeFunctionSummary {
    let head = TypeComputationHeadId::new(module.clone(), "ZeroNat");
    TypeFunctionSummary {
        exported_name: "ZeroNat".into(),
        head: head.clone(),
        visibility: Visibility::Public,
        params: vec![],
        return_type: CanonicalTypeExpr::Primitive("Type".into()),
        return_kind: Kind::Type,
        result_constraint: TypeFunctionResultConstraint::Kind(Kind::Type),
        export_mode: TypeFunctionExportMode::TransparentEquations,
        source_anchors: TypeFunctionSourceAnchors {
            definition: anchor("type fn ZeroNat"),
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
                source_anchor: anchor("ZeroNat rhs"),
            },
            source_anchor: anchor("case ZeroNat = Z"),
            case_head_anchor: anchor("ZeroNat case head"),
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

fn associated_family_head(
    module: &ModuleIdentity,
    interface: &str,
    member: &str,
) -> AssociatedFamilyHeadId {
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

fn identity_associated_family_summary(
    module: &ModuleIdentity,
    interface: &str,
    member: &str,
) -> AssociatedFamilySummary {
    let family_head = associated_family_head(module, interface, member);
    let constraint = AssociatedFamilyResultConstraint::Kind(Kind::Type);
    AssociatedFamilySummary {
        head: family_head.clone(),
        interface_identity: family_head.interface.clone(),
        member_identity: family_head.member.clone(),
        visible_name: member.into(),
        result_domain: CanonicalTypeExpr::Primitive("Type".into()),
        result_kind: Kind::Type,
        export_mode: AssociatedFamilyExportMode::TransparentEquations,
        schemes: vec![AssociatedFamilyScheme {
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
                result: AssociatedFamilyResultExpr::Var {
                    name: "T".into(),
                    kind: Kind::Type,
                    constraint: AssociatedFamilyResultConstraint::Kind(Kind::Type),
                    source_anchor: anchor("result T"),
                },
                decreases: None,
                source_anchor: anchor("equation"),
                case_head_anchor: anchor("case"),
            }],
            source_anchor: anchor("scheme"),
        }],
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

#[test]
fn zero_arg_promoted_constructor_in_imported_type_function_rhs_normalizes() {
    let module = module(1);
    let (summary, nat_kind, z_ctor) = promoted_nat_summary(&module);
    let summary = summary.with_exported_type_function(type_function_summary_returning_promoted_z(
        &module, &nat_kind, &z_ctor,
    ));

    let mut env = TypeEnv::new();
    env.register_module_semantic_summary(&summary)
        .expect("V6 summary with promoted kind and promoted RHS type fn imports");

    let head = summary.exported_type_functions[0].head.clone();
    let normal = Normalizer::new(&env)
        .normalize_known_computation_app(&head, vec![], &Kind::Type)
        .expect("promoted constructor RHS normalizes");

    assert_eq!(
        normal,
        NormalTypeExpr::PromotedDataConstructorApp {
            constructor: Box::new(z_ctor),
            data_kind: Box::new(nat_kind),
            args: vec![],
            kind: Kind::Type,
        }
    );
}

#[test]
fn promoted_constructor_terms_in_proposition_operands_solve_after_registration() {
    let module = module(2);
    let (summary, nat_kind, z_ctor) = promoted_nat_summary(&module);
    let app =
        CanonicalTypeExpr::PromotedDataConstructorApp(Box::new(promoted_app(&nat_kind, &z_ctor)));
    let proposition = TypeProposition::Equality(TypeEqualityProposition {
        lhs: TypePropositionTerm::Canonical(app.clone()),
        rhs: TypePropositionTerm::Canonical(app),
    });

    let mut env = TypeEnv::new();
    env.register_module_semantic_summary(&summary)
        .expect("promoted data kind registers before proposition solving");

    let outcome = env
        .solve_proposition(&proposition, Some(anchor("Z == Z")))
        .expect("registered promoted constructor proposition solves");

    match outcome {
        PropositionOutcome::Satisfied(evidence) => {
            assert_eq!(evidence.rule, PropositionEvidenceRule::DefinitionalEquality);
            let terms = evidence
                .normalized_terms
                .expect("equality evidence records promoted normal terms");
            assert!(matches!(
                terms.lhs,
                NormalTypeExpr::PromotedDataConstructorApp { .. }
            ));
            assert_eq!(terms.lhs, terms.rhs);
        }
        other => panic!("expected promoted constructor equality to satisfy, got {other:?}"),
    }
}

#[test]
fn direct_proposition_solving_rejects_unregistered_promoted_constructor_equality_operand() {
    let module = module(7);
    let kind = promoted_kind_id(&module, "Nat", "NatKind");
    let ctor = promoted_ctor_summary(&kind, "Nat", "Z").id;
    let app = CanonicalTypeExpr::PromotedDataConstructorApp(Box::new(promoted_app(&kind, &ctor)));
    let proposition = TypeProposition::Equality(TypeEqualityProposition {
        lhs: TypePropositionTerm::Canonical(app.clone()),
        rhs: TypePropositionTerm::Canonical(app),
    });

    let env = TypeEnv::new();
    let err = env
        .solve_proposition(&proposition, Some(anchor("unregistered Z == Z")))
        .expect_err("direct proposition solving must validate promoted data-kind registration");
    let msg = err.to_string();
    assert!(
        msg.contains("promoted data kind") && msg.contains("NatKind"),
        "unexpected diagnostic: {msg}"
    );
}

#[test]
fn direct_proposition_solving_rejects_unregistered_promoted_constructor_disequality_operand() {
    let module = module(8);
    let kind = promoted_kind_id(&module, "Nat", "NatKind");
    let ctor = promoted_ctor_summary(&kind, "Nat", "Z").id;
    let app = CanonicalTypeExpr::PromotedDataConstructorApp(Box::new(promoted_app(&kind, &ctor)));
    let proposition = TypeProposition::Disequality(TypeDisequalityProposition {
        lhs: TypePropositionTerm::Canonical(app.clone()),
        rhs: TypePropositionTerm::Canonical(app),
    });

    let env = TypeEnv::new();
    let err = env
        .solve_proposition(&proposition, Some(anchor("unregistered Z != Z")))
        .expect_err("direct disequality solving must validate promoted data-kind registration");
    let msg = err.to_string();
    assert!(
        msg.contains("promoted data kind") && msg.contains("NatKind"),
        "unexpected diagnostic: {msg}"
    );
}

#[test]
fn associated_family_selection_blocks_promoted_constructor_capture_instead_of_panicking() {
    let module = module(9);
    let (summary, nat_kind, z_ctor) = promoted_nat_summary(&module);
    let family = identity_associated_family_summary(&module, "PromotedCarrier", "Out");
    let family_head = family.head.clone();
    let summary = summary
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

    let mut env = TypeEnv::new();
    env.register_module_semantic_summary(&summary)
        .expect("associated family and promoted constructor metadata import together");
    let promoted_arg =
        CanonicalTypeExpr::PromotedDataConstructorApp(Box::new(promoted_app(&nat_kind, &z_ctor)));

    let err = env
        .reduce_associated_family_projection_once(&family_head, &[promoted_arg])
        .expect_err("promoted constructor apps are not capturable associated-family arguments");
    let msg = err.to_string();
    assert!(
        msg.contains("associated-family selection") && msg.contains("blocked"),
        "unexpected diagnostic: {msg}"
    );
}

#[test]
fn proposition_summary_rejects_promoted_constructor_before_kind_registration() {
    let module = module(3);
    let kind = promoted_kind_id(&module, "Nat", "NatKind");
    let ctor = promoted_ctor_summary(&kind, "Nat", "Z").id;
    let app = CanonicalTypeExpr::PromotedDataConstructorApp(Box::new(promoted_app(&kind, &ctor)));
    let proposition = TypeProposition::Equality(TypeEqualityProposition {
        lhs: TypePropositionTerm::Canonical(app.clone()),
        rhs: TypePropositionTerm::Canonical(app),
    });
    let summary = ModuleSemanticSummary::new(module)
        .with_version(SummaryVersion::SPEC065_PROMOTED_DATA_KIND_V6)
        .with_exported_proposition_fact(PropositionFactSummary {
            proposition,
            role: SummaryPropositionFactRole::Requirement,
            source_anchor: anchor("where Z == Z"),
            predicate_dependencies: vec![],
            dependency_summary_refs: vec![],
            outcome: None,
        });

    let mut env = TypeEnv::new();
    let err = env
        .register_module_semantic_summary(&summary)
        .expect_err("promoted constructor propositions require registered promoted data kinds");
    let msg = err.to_string();
    assert!(
        msg.contains("promoted data kind") && msg.contains("NatKind"),
        "unexpected diagnostic: {msg}"
    );
}

#[test]
fn proposition_summary_with_public_promoted_constructor_operand_registers_after_kind_metadata() {
    let module = module(4);
    let (summary, nat_kind, z_ctor) = promoted_nat_summary(&module);
    let app =
        CanonicalTypeExpr::PromotedDataConstructorApp(Box::new(promoted_app(&nat_kind, &z_ctor)));
    let proposition = TypeProposition::Equality(TypeEqualityProposition {
        lhs: TypePropositionTerm::Canonical(app.clone()),
        rhs: TypePropositionTerm::Canonical(app),
    });
    let summary = summary.with_exported_proposition_fact(PropositionFactSummary {
        proposition,
        role: SummaryPropositionFactRole::Requirement,
        source_anchor: anchor("public where Z == Z"),
        predicate_dependencies: vec![],
        dependency_summary_refs: vec![],
        outcome: None,
    });

    let mut env = TypeEnv::new();
    env.register_module_semantic_summary(&summary)
        .expect("public proposition fact may use a registered public promoted constructor operand");
}

#[test]
fn constrained_promoted_constructor_canonical_arg_rejects_non_carrier_value() {
    let module = module(5);
    let (summary, nat_kind, _z_ctor, s_ctor) = recursive_promoted_nat_summary(&module);
    let bad_successor =
        CanonicalTypeExpr::PromotedDataConstructorApp(Box::new(promoted_app_with_args(
            &nat_kind,
            &s_ctor,
            vec![CanonicalTypeExpr::Primitive("Type".into())],
        )));
    let head = TypeComputationHeadId::new(module.clone(), "BadCanonicalSucc");
    let summary = summary.with_exported_type_function(TypeFunctionSummary {
        exported_name: "BadCanonicalSucc".into(),
        head: head.clone(),
        visibility: Visibility::Public,
        params: vec![],
        return_type: bad_successor,
        return_kind: Kind::Type,
        result_constraint: TypeFunctionResultConstraint::Kind(Kind::Type),
        export_mode: TypeFunctionExportMode::TransparentEquations,
        source_anchors: TypeFunctionSourceAnchors {
            definition: anchor("type fn BadCanonicalSucc"),
            decreases: None,
        },
        equations: vec![],
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
    });

    let mut env = TypeEnv::new();
    let err = env
        .register_module_semantic_summary(&summary)
        .expect_err("non-promoted canonical argument must not satisfy promoted data-kind field");
    let msg = err.to_string();
    assert!(
        msg.contains("promoted data-kind constrained field") && msg.contains("NatKind"),
        "unexpected diagnostic: {msg}"
    );
}

#[test]
fn constrained_promoted_constructor_result_arg_rejects_non_carrier_value() {
    let module = module(6);
    let (summary, nat_kind, _z_ctor, s_ctor) = recursive_promoted_nat_summary(&module);
    let head = TypeComputationHeadId::new(module.clone(), "BadResultSucc");
    let summary = summary.with_exported_type_function(TypeFunctionSummary {
        exported_name: "BadResultSucc".into(),
        head: head.clone(),
        visibility: Visibility::Public,
        params: vec![],
        return_type: CanonicalTypeExpr::Primitive("Type".into()),
        return_kind: Kind::Type,
        result_constraint: TypeFunctionResultConstraint::Kind(Kind::Type),
        export_mode: TypeFunctionExportMode::TransparentEquations,
        source_anchors: TypeFunctionSourceAnchors {
            definition: anchor("type fn BadResultSucc"),
            decreases: None,
        },
        equations: vec![TypeFunctionEquation {
            head,
            ordinal: 0,
            patterns: vec![],
            result: TypeFunctionResultExpr::PromotedDataConstructorApp {
                constructor: Box::new(s_ctor),
                data_kind: Box::new(nat_kind),
                args: vec![TypeFunctionResultExpr::Primitive {
                    name: "Type".into(),
                    kind: Kind::Type,
                    constraint: TypeFunctionResultConstraint::Kind(Kind::Type),
                    source_anchor: anchor("primitive Type arg"),
                }],
                kind: Kind::Type,
                constraint: TypeFunctionResultConstraint::Kind(Kind::Type),
                source_anchor: anchor("BadResultSucc rhs"),
            },
            source_anchor: anchor("case BadResultSucc = S Type"),
            case_head_anchor: anchor("BadResultSucc case head"),
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
    });

    let mut env = TypeEnv::new();
    let err = env
        .register_module_semantic_summary(&summary)
        .expect_err("non-promoted result argument must not satisfy promoted data-kind field");
    let msg = err.to_string();
    assert!(
        msg.contains("promoted data-kind constrained field") && msg.contains("NatKind"),
        "unexpected diagnostic: {msg}"
    );
}
