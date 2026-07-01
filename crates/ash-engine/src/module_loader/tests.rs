//! Module loader tests.

use super::*;
use ash_core::ast::{TypeBody, VariantDef, VariantPayload, Visibility};
use ash_core::kind::Kind;
use ash_core::module_graph::CrateId;
use ash_core::semantic_summary::{
    ConstructorId, ConstructorPayloadKind, ConstructorSummary, PromotedConstructorFieldSummary,
    PromotedConstructorSummary, PromotedDataKindSummary, PropositionFactRole,
    PropositionFactSummary, RepresentationExposure, SourceAnchor as SummarySourceAnchor,
    SourceOrigin as SummarySourceOrigin, TypeFunctionClosureMetadata, TypeFunctionExportMode,
    TypeFunctionRevalidationMetadata,
};
use ash_core::type_ir::{
    PromotedConstructorApp, TypeEqualityProposition, TypeFunctionEquation,
    TypeFunctionSourceAnchors, TypeProposition, TypePropositionTerm,
};

fn task896_module(id: usize) -> ModuleIdentity {
    ModuleIdentity::new(
        Some(CrateId(8961)),
        ModuleId(id),
        vec!["task896_loader".to_string(), format!("m{id}")],
        ModuleSourceOrigin::Synthetic {
            reason: format!("task-896-loader-{id}"),
        },
    )
}

fn task896_anchor(label: &str) -> SummarySourceAnchor {
    SummarySourceAnchor::new(
        SummarySourceOrigin::Synthetic {
            reason: "task-896-loader-selected-summary".into(),
        },
        None,
        label,
    )
}

fn task896_source_type(module: &ModuleIdentity) -> TypeDeclSummary {
    TypeDeclSummary::new(
        TypeDeclId::ordinary(module.clone(), "Nat"),
        "Nat",
        Visibility::Public,
        RepresentationExposure::Exposed,
        TypeRepresentationSummary::Exposed(TypeBody::Enum(vec![VariantDef {
            name: "Z".into(),
            fields: vec![],
            payload: VariantPayload::Unit,
        }])),
        task896_anchor("Nat"),
    )
}

fn task896_source_constructor(module: &ModuleIdentity) -> ConstructorSummary {
    let nat = TypeDeclId::ordinary(module.clone(), "Nat");
    ConstructorSummary::new(
        ConstructorId::variant(nat.clone(), "Z", ConstructorPayloadKind::Unit),
        nat,
        "Z",
        ConstructorPayloadKind::Unit,
        Visibility::Public,
        task896_anchor("Z"),
    )
}

fn task896_promoted_ids(module: &ModuleIdentity) -> (PromotedDataKindId, PromotedConstructorId) {
    let kind = PromotedDataKindId::new(
        module.clone(),
        TypeDeclId::ordinary(module.clone(), "Nat"),
        "NatKind",
    );
    let source_ctor = ConstructorId::variant(
        TypeDeclId::ordinary(module.clone(), "Nat"),
        "Z",
        ConstructorPayloadKind::Unit,
    );
    let ctor = PromotedConstructorId::new(kind.clone(), source_ctor, "Z");
    (kind, ctor)
}

fn task896_promoted_kind_summary(
    module: &ModuleIdentity,
    kind: &PromotedDataKindId,
    ctor: &PromotedConstructorId,
) -> PromotedDataKindSummary {
    let source_ctor = ConstructorId::variant(
        TypeDeclId::ordinary(module.clone(), "Nat"),
        "Z",
        ConstructorPayloadKind::Unit,
    );
    PromotedDataKindSummary::new(
        kind.clone(),
        "NatKind",
        Visibility::Public,
        TypeDeclId::ordinary(module.clone(), "Nat"),
        task896_anchor("NatKind"),
    )
    .with_constructor(PromotedConstructorSummary::new(
        ctor.clone(),
        "Z",
        source_ctor,
        vec![],
        Visibility::Public,
        task896_anchor("promoted Z"),
    ))
}

fn task896_promoted_app(
    kind: &PromotedDataKindId,
    ctor: &PromotedConstructorId,
) -> PromotedConstructorApp {
    PromotedConstructorApp {
        constructor: ctor.clone(),
        data_kind: kind.clone(),
        args: vec![],
        kind: Kind::Type,
    }
}

fn task896_promoted_type_function(
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
            definition: task896_anchor("type fn PromotedZero"),
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
                source_anchor: task896_anchor("PromotedZero rhs"),
            },
            source_anchor: task896_anchor("case PromotedZero = Z"),
            case_head_anchor: task896_anchor("PromotedZero case head"),
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

fn task896_promoted_summary() -> ModuleSemanticSummary {
    let module = task896_module(1);
    let (kind, ctor) = task896_promoted_ids(&module);
    ModuleSemanticSummary::new(module.clone())
        .with_version(SummaryVersion::SPEC065_PROMOTED_DATA_KIND_V6)
        .with_exported_type(task896_source_type(&module))
        .with_exported_constructor(task896_source_constructor(&module))
        .with_exported_promoted_data_kind(task896_promoted_kind_summary(&module, &kind, &ctor))
        .with_exported_type_function(task896_promoted_type_function(&module, &kind, &ctor))
}

fn task896_promoted_summary_named(
    module_id: usize,
    source_type_name: &str,
    source_constructor_name: &str,
    data_kind_name: &str,
) -> ModuleSemanticSummary {
    let module = task896_module(module_id);
    let source_type = TypeDeclId::ordinary(module.clone(), source_type_name);
    let source_constructor = ConstructorId::variant(
        source_type.clone(),
        source_constructor_name,
        ConstructorPayloadKind::Unit,
    );
    let kind = PromotedDataKindId::new(module.clone(), source_type.clone(), data_kind_name);
    let ctor = PromotedConstructorId::new(
        kind.clone(),
        source_constructor.clone(),
        source_constructor_name,
    );
    ModuleSemanticSummary::new(module.clone())
        .with_version(SummaryVersion::SPEC065_PROMOTED_DATA_KIND_V6)
        .with_exported_type(TypeDeclSummary::new(
            source_type.clone(),
            source_type_name,
            Visibility::Public,
            RepresentationExposure::Exposed,
            TypeRepresentationSummary::Exposed(TypeBody::Enum(vec![VariantDef {
                name: source_constructor_name.into(),
                fields: vec![],
                payload: VariantPayload::Unit,
            }])),
            task896_anchor(source_type_name),
        ))
        .with_exported_constructor(ConstructorSummary::new(
            source_constructor.clone(),
            source_type.clone(),
            source_constructor_name,
            ConstructorPayloadKind::Unit,
            Visibility::Public,
            task896_anchor(source_constructor_name),
        ))
        .with_exported_promoted_data_kind(
            PromotedDataKindSummary::new(
                kind.clone(),
                data_kind_name,
                Visibility::Public,
                source_type,
                task896_anchor(data_kind_name),
            )
            .with_constructor(PromotedConstructorSummary::new(
                ctor.clone(),
                source_constructor_name,
                source_constructor,
                vec![],
                Visibility::Public,
                task896_anchor(source_constructor_name),
            )),
        )
        .with_exported_type_function(task896_promoted_type_function(&module, &kind, &ctor))
}

#[allow(clippy::too_many_lines)]
fn task896_promoted_summary_with_field_constraint() -> ModuleSemanticSummary {
    let module = task896_module(21);
    let elem_type = TypeDeclId::ordinary(module.clone(), "Elem");
    let elem_constructor =
        ConstructorId::variant(elem_type.clone(), "E", ConstructorPayloadKind::Unit);
    let elem_kind = PromotedDataKindId::new(module.clone(), elem_type.clone(), "ElemKind");
    let elem_promoted_constructor =
        PromotedConstructorId::new(elem_kind.clone(), elem_constructor.clone(), "E");

    let maybe_type = TypeDeclId::ordinary(module.clone(), "MaybeElem");
    let none_constructor =
        ConstructorId::variant(maybe_type.clone(), "None", ConstructorPayloadKind::Unit);
    let some_constructor =
        ConstructorId::variant(maybe_type.clone(), "Some", ConstructorPayloadKind::Tuple);
    let maybe_kind = PromotedDataKindId::new(module.clone(), maybe_type.clone(), "MaybeElemKind");
    let none_promoted_constructor =
        PromotedConstructorId::new(maybe_kind.clone(), none_constructor.clone(), "None");
    let some_promoted_constructor =
        PromotedConstructorId::new(maybe_kind.clone(), some_constructor.clone(), "Some");

    ModuleSemanticSummary::new(module.clone())
        .with_version(SummaryVersion::SPEC065_PROMOTED_DATA_KIND_V6)
        .with_exported_type(TypeDeclSummary::new(
            elem_type.clone(),
            "Elem",
            Visibility::Public,
            RepresentationExposure::Exposed,
            TypeRepresentationSummary::Exposed(TypeBody::Enum(vec![VariantDef {
                name: "E".into(),
                fields: vec![],
                payload: VariantPayload::Unit,
            }])),
            task896_anchor("Elem"),
        ))
        .with_exported_constructor(ConstructorSummary::new(
            elem_constructor.clone(),
            elem_type.clone(),
            "E",
            ConstructorPayloadKind::Unit,
            Visibility::Public,
            task896_anchor("E"),
        ))
        .with_exported_type(TypeDeclSummary::new(
            maybe_type.clone(),
            "MaybeElem",
            Visibility::Public,
            RepresentationExposure::Exposed,
            TypeRepresentationSummary::Exposed(TypeBody::Enum(vec![
                VariantDef {
                    name: "None".into(),
                    fields: vec![],
                    payload: VariantPayload::Unit,
                },
                VariantDef {
                    name: "Some".into(),
                    fields: vec![("0".into(), CoreTypeExpr::Named("Elem".into()))],
                    payload: VariantPayload::Tuple(vec![CoreTypeExpr::Named("Elem".into())]),
                },
            ])),
            task896_anchor("MaybeElem"),
        ))
        .with_exported_constructor(ConstructorSummary::new(
            none_constructor.clone(),
            maybe_type.clone(),
            "None",
            ConstructorPayloadKind::Unit,
            Visibility::Public,
            task896_anchor("None"),
        ))
        .with_exported_constructor(ConstructorSummary::new(
            some_constructor.clone(),
            maybe_type.clone(),
            "Some",
            ConstructorPayloadKind::Tuple,
            Visibility::Public,
            task896_anchor("Some"),
        ))
        .with_exported_promoted_data_kind(
            PromotedDataKindSummary::new(
                elem_kind.clone(),
                "ElemKind",
                Visibility::Public,
                elem_type,
                task896_anchor("ElemKind"),
            )
            .with_constructor(PromotedConstructorSummary::new(
                elem_promoted_constructor,
                "E",
                elem_constructor,
                vec![],
                Visibility::Public,
                task896_anchor("promoted E"),
            )),
        )
        .with_exported_promoted_data_kind(
            PromotedDataKindSummary::new(
                maybe_kind.clone(),
                "MaybeElemKind",
                Visibility::Public,
                maybe_type,
                task896_anchor("MaybeElemKind"),
            )
            .with_constructor(PromotedConstructorSummary::new(
                none_promoted_constructor.clone(),
                "None",
                none_constructor,
                vec![],
                Visibility::Public,
                task896_anchor("promoted None"),
            ))
            .with_constructor(PromotedConstructorSummary::new(
                some_promoted_constructor,
                "Some",
                some_constructor,
                vec![PromotedConstructorFieldSummary::new(
                    "0",
                    Kind::Type,
                    Some(elem_kind),
                    task896_anchor("promoted Some field"),
                )],
                Visibility::Public,
                task896_anchor("promoted Some"),
            )),
        )
        .with_exported_type_function(task896_promoted_type_function(
            &module,
            &maybe_kind,
            &none_promoted_constructor,
        ))
}

fn task896_promoted_proposition_summary_named(
    module_id: usize,
    source_type_name: &str,
    source_constructor_name: &str,
    data_kind_name: &str,
) -> ModuleSemanticSummary {
    let mut summary = task896_promoted_summary_named(
        module_id,
        source_type_name,
        source_constructor_name,
        data_kind_name,
    );
    let kind = summary.exported_promoted_data_kinds[0].id.clone();
    let ctor = summary.exported_promoted_data_kinds[0].constructors[0]
        .id
        .clone();
    let app =
        CanonicalTypeExpr::PromotedDataConstructorApp(Box::new(task896_promoted_app(&kind, &ctor)));
    summary
        .exported_proposition_facts
        .push(PropositionFactSummary {
            proposition: TypeProposition::Equality(TypeEqualityProposition {
                lhs: TypePropositionTerm::Canonical(app.clone()),
                rhs: TypePropositionTerm::Canonical(app),
            }),
            role: PropositionFactRole::Requirement,
            source_anchor: task896_anchor("promoted proposition fact"),
            predicate_dependencies: vec![],
            dependency_summary_refs: vec![],
            outcome: None,
        });
    summary
}

#[test]
fn task896_selected_type_function_summary_retains_promoted_dependencies() {
    let source = task896_promoted_summary();
    let selected =
        selected_type_function_semantic_summary(&source, "PromotedZero", "ImportedPromotedZero")
            .expect("selected type-function summary");

    assert_eq!(
        selected.version,
        SummaryVersion::SPEC065_PROMOTED_DATA_KIND_V6
    );
    assert_eq!(selected.exported_promoted_data_kinds.len(), 1);
    assert_eq!(selected.exported_types.len(), 1);
    assert_eq!(selected.exported_constructors.len(), 1);
    assert_eq!(selected.exported_type_functions.len(), 1);
    assert_eq!(
        selected.exported_type_functions[0].exported_name,
        "ImportedPromotedZero"
    );

    let mut env = ash_typeck::TypeEnv::new();
    env.register_module_semantic_summary(&selected)
        .expect("selected promoted type-function summary remains revalidatable");
}

#[test]
fn task896_selected_type_function_summary_retains_promoted_field_constraint_dependencies() {
    let source = task896_promoted_summary_with_field_constraint();
    let selected =
        selected_type_function_semantic_summary(&source, "PromotedZero", "ImportedPromotedZero")
            .expect("selected type-function summary");

    assert_eq!(selected.exported_promoted_data_kinds.len(), 2);
    assert!(
        selected
            .exported_promoted_data_kinds
            .iter()
            .all(|data_kind| is_dependency_metadata_name(&data_kind.exported_name))
    );
    assert_eq!(selected.exported_types.len(), 2);
    assert_eq!(selected.exported_constructors.len(), 3);

    let mut env = ash_typeck::TypeEnv::new();
    env.register_module_semantic_summary(&selected)
        .expect("selected type-function summary retains transitive promoted field constraints");
    assert!(env.lookup_promoted_data_kind("MaybeElemKind").is_none());
    assert!(env.lookup_promoted_data_kind("ElemKind").is_none());
}

#[test]
fn task896_selected_type_function_hidden_promoted_data_kind_dependencies_do_not_alias_collide() {
    let left_source = task896_promoted_summary_named(11, "LeftNat", "LeftZ", "NatKind");
    let right_source = task896_promoted_summary_named(12, "RightNat", "RightZ", "NatKind");
    let left = selected_type_function_semantic_summary(
        &left_source,
        "PromotedZero",
        "ImportedLeftPromotedZero",
    )
    .expect("left selected type-function summary");
    let right = selected_type_function_semantic_summary(
        &right_source,
        "PromotedZero",
        "ImportedRightPromotedZero",
    )
    .expect("right selected type-function summary");
    let left_kind = left.exported_promoted_data_kinds[0].id.clone();
    let right_kind = right.exported_promoted_data_kinds[0].id.clone();

    let mut env = ash_typeck::TypeEnv::new();
    env.register_module_semantic_summaries(&[left, right])
        .expect(
            "hidden promoted data-kind dependencies with the same source name register by identity",
        );

    assert!(
        env.lookup_promoted_data_kind("NatKind").is_none(),
        "hidden promoted data-kind dependency metadata must not create a source-visible alias"
    );
    assert!(env.lookup_promoted_data_kind_by_id(&left_kind).is_some());
    assert!(env.lookup_promoted_data_kind_by_id(&right_kind).is_some());
}

#[test]
fn task896_selected_proposition_summary_retains_promoted_dependencies() {
    let source = {
        let module = task896_module(2);
        let (kind, ctor) = task896_promoted_ids(&module);
        let app = CanonicalTypeExpr::PromotedDataConstructorApp(Box::new(task896_promoted_app(
            &kind, &ctor,
        )));
        let proposition = TypeProposition::Equality(TypeEqualityProposition {
            lhs: TypePropositionTerm::Canonical(app.clone()),
            rhs: TypePropositionTerm::Canonical(app),
        });
        ModuleSemanticSummary::new(module.clone())
            .with_version(SummaryVersion::SPEC065_PROMOTED_DATA_KIND_V6)
            .with_exported_type(task896_source_type(&module))
            .with_exported_constructor(task896_source_constructor(&module))
            .with_exported_promoted_data_kind(task896_promoted_kind_summary(&module, &kind, &ctor))
            .with_exported_proposition_fact(PropositionFactSummary {
                proposition,
                role: PropositionFactRole::Requirement,
                source_anchor: task896_anchor("Z == Z"),
                predicate_dependencies: vec![],
                dependency_summary_refs: vec![],
                outcome: None,
            })
    };
    let selected =
        selected_proposition_semantic_summary(Some(&source)).expect("selected proposition summary");

    assert_eq!(
        selected.version,
        SummaryVersion::SPEC065_PROMOTED_DATA_KIND_V6
    );
    assert_eq!(selected.exported_promoted_data_kinds.len(), 1);
    assert_eq!(selected.exported_proposition_facts.len(), 1);

    let mut env = ash_typeck::TypeEnv::new();
    env.register_module_semantic_summary(&selected)
        .expect("selected promoted proposition summary remains revalidatable");
    assert!(
        env.lookup_promoted_data_kind("NatKind").is_none(),
        "selected proposition promoted dependencies must remain hidden metadata"
    );
    assert!(
        env.lookup_promoted_data_kind_by_id(&selected.exported_promoted_data_kinds[0].id)
            .is_some()
    );
}

#[test]
fn task896_selected_proposition_hidden_promoted_data_kind_dependencies_do_not_alias_collide() {
    let left_source = task896_promoted_proposition_summary_named(31, "LeftNat", "LeftZ", "NatKind");
    let right_source =
        task896_promoted_proposition_summary_named(32, "RightNat", "RightZ", "NatKind");
    let left = selected_proposition_semantic_summary(Some(&left_source))
        .expect("left selected proposition summary");
    let right = selected_proposition_semantic_summary(Some(&right_source))
        .expect("right selected proposition summary");
    let left_kind = left.exported_promoted_data_kinds[0].id.clone();
    let right_kind = right.exported_promoted_data_kinds[0].id.clone();

    let mut env = ash_typeck::TypeEnv::new();
    env.register_module_semantic_summaries(&[left, right])
        .expect("hidden proposition promoted data-kind dependencies with the same source name register by identity");

    assert!(
        env.lookup_promoted_data_kind("NatKind").is_none(),
        "selected proposition promoted dependencies must not create a source-visible alias"
    );
    assert!(env.lookup_promoted_data_kind_by_id(&left_kind).is_some());
    assert!(env.lookup_promoted_data_kind_by_id(&right_kind).is_some());
}

#[test]
fn task896_merge_imported_summary_payloads_retains_hidden_promoted_data_kind_dependencies() {
    let source = task896_promoted_summary();
    let selected =
        selected_type_function_semantic_summary(&source, "PromotedZero", "ImportedPromotedZero")
            .expect("selected type-function summary");
    let kind = selected.exported_promoted_data_kinds[0].id.clone();
    let mut existing = selected.clone();
    existing.exported_promoted_data_kinds.clear();
    existing.exported_type_functions.clear();

    merge_imported_summary_payloads(&mut existing, selected);

    assert_eq!(existing.exported_promoted_data_kinds.len(), 1);
    assert!(is_dependency_metadata_name(
        &existing.exported_promoted_data_kinds[0].exported_name
    ));
    assert_eq!(existing.exported_promoted_data_kinds[0].id, kind);
    let mut env = ash_typeck::TypeEnv::new();
    env.register_module_semantic_summary(&existing)
        .expect("merged selected import retains revalidatable hidden promoted dependencies");
    assert!(env.lookup_promoted_data_kind("NatKind").is_none());
    assert!(env.lookup_promoted_data_kind_by_id(&kind).is_some());
}

#[test]
fn task896_selected_summary_identity_facts_reject_conflicting_hidden_promoted_data_kind_payloads() {
    let source = task896_promoted_summary();
    let left =
        selected_type_function_semantic_summary(&source, "PromotedZero", "ImportedPromotedZero")
            .expect("selected type-function summary");
    let mut right = left.clone();
    right.exported_promoted_data_kinds[0].exported_name = "$ash_dependency$conflict".into();

    assert!(!selected_summary_identity_facts_are_compatible(
        &left, &right
    ));
}

#[test]
fn task896_merge_selected_summary_export_retains_hidden_promoted_data_kind_dependencies_and_v6() {
    let source = task896_promoted_summary();
    let selected =
        selected_type_function_semantic_summary(&source, "PromotedZero", "ImportedPromotedZero")
            .expect("selected type-function summary");
    let kind = selected.exported_promoted_data_kinds[0].id.clone();
    let mut exports = ModuleExports::default();

    merge_selected_summary_export(&mut exports, &source, selected)
        .expect("selected summary merges into re-export summary");

    let summary = exports
        .semantic_summary
        .expect("selected merge creates semantic summary");
    assert_eq!(
        summary.version,
        SummaryVersion::SPEC065_PROMOTED_DATA_KIND_V6
    );
    assert_eq!(summary.exported_promoted_data_kinds.len(), 1);
    assert!(is_dependency_metadata_name(
        &summary.exported_promoted_data_kinds[0].exported_name
    ));
    summary
        .validate_summary_version_contract()
        .expect("merged summary version matches promoted data-kind payload");

    let mut env = ash_typeck::TypeEnv::new();
    env.register_module_semantic_summary(&summary)
        .expect("re-export merge keeps hidden promoted dependency metadata revalidatable");
    assert!(env.lookup_promoted_data_kind("NatKind").is_none());
    assert!(env.lookup_promoted_data_kind_by_id(&kind).is_some());
}

#[test]
fn import_continuation_is_limited_to_nested_use_trees() {
    assert!(import_needs_more_lines("use child::{\n    Role"));
    assert!(!import_needs_more_lines("use child"));
    assert!(
        !import_needs_more_lines("use child {"),
        "unsupported root-brace syntax must not consume following source lines"
    );
}

/// Test 1: `pub mod child;` loads the child module's exports and stores
/// them in `child_modules` under the child name.
///
/// For a file module `parent.ash`, `pub mod child;` looks for
/// `parent/child.ash` or `parent/child/mod.ash` (Rust-like module resolution).
#[test]
fn test_pub_mod_types_loads_child_exports() {
    let temp = tempfile::tempdir().expect("tempdir");
    let dir = temp.path();

    // Create parent/ subdirectory for child module
    std::fs::create_dir(dir.join("parent")).expect("create parent dir");

    // child.ash: defines a public type
    std::fs::write(
        dir.join("parent").join("child.ash"),
        "pub type Role = System | User | Assistant;",
    )
    .expect("write child");

    // parent.ash: declares pub mod child;
    std::fs::write(dir.join("parent.ash"), "pub mod child;").expect("write parent");

    let mut cache = HashMap::new();
    let exports = collect_module_exports(&dir.join("parent.ash"), &mut cache, &mut HashSet::new())
        .expect("collecting parent exports should succeed");

    let child = exports
        .child_modules
        .get("child")
        .expect("child_modules should contain 'child'");
    assert!(
        child.type_defs.contains_key("Role"),
        "child module should export Role"
    );
}

/// Test 2: `pub use child::{Role}` re-exports still work alongside
/// `pub mod child;` -- the parent's `type_defs` contains Role.
///
/// Note: `pub use child::{Role}` resolves `child` via the crate root, not via
/// the file module's subdirectory. For `pub mod child;` we use `parent/child.ash`.
#[test]
fn test_pub_use_resolves_via_child_module() {
    let temp = tempfile::tempdir().expect("tempdir");
    let dir = temp.path();

    // Create parent/ subdirectory for child module (pub mod resolution)
    std::fs::create_dir(dir.join("parent")).expect("create parent dir");

    // child.ash in parent/ for pub mod child;
    std::fs::write(
        dir.join("parent").join("child.ash"),
        "pub type Role = System | User;",
    )
    .expect("write child");

    // Also create child.ash in root for pub use resolution
    std::fs::write(dir.join("child.ash"), "pub type Role = System | User;")
        .expect("write child for use");

    // parent.ash: both pub mod child; and pub use child::{Role};
    std::fs::write(
        dir.join("parent.ash"),
        "pub mod child;\npub use child::{Role};",
    )
    .expect("write parent");

    let mut cache = HashMap::new();
    let exports = collect_module_exports(&dir.join("parent.ash"), &mut cache, &mut HashSet::new())
        .expect("collecting parent exports should succeed");

    // Role is re-exported via pub use
    assert!(
        exports.type_defs.contains_key("Role"),
        "parent should re-export Role via pub use"
    );
    // Also present in child_modules
    assert!(
        exports.child_modules.contains_key("child"),
        "child_modules should contain 'child'"
    );
}

/// Test 3: Child exports are NOT flattened into the parent -- only
/// explicitly `pub use`d items appear at the parent's top level.
#[test]
fn test_child_exports_not_flattened() {
    let temp = tempfile::tempdir().expect("tempdir");
    let dir = temp.path();

    // Create parent/ subdirectory for child module
    std::fs::create_dir(dir.join("parent")).expect("create parent dir");

    // child.ash: defines two public types
    std::fs::write(
        dir.join("parent").join("child.ash"),
        "pub type Alpha = A | B;\npub type Beta = C | D;",
    )
    .expect("write child");

    // Also create child.ash in root for pub use resolution
    std::fs::write(
        dir.join("child.ash"),
        "pub type Alpha = A | B;\npub type Beta = C | D;",
    )
    .expect("write child for use");

    // parent.ash: declares pub mod child; but only re-exports Alpha
    std::fs::write(
        dir.join("parent.ash"),
        "pub mod child;\npub use child::{Alpha};",
    )
    .expect("write parent");

    let mut cache = HashMap::new();
    let exports = collect_module_exports(&dir.join("parent.ash"), &mut cache, &mut HashSet::new())
        .expect("collecting parent exports should succeed");

    // Alpha should be re-exported
    assert!(
        exports.type_defs.contains_key("Alpha"),
        "parent should re-export Alpha"
    );
    // Beta should NOT appear in parent's type_defs (not re-exported)
    assert!(
        !exports.type_defs.contains_key("Beta"),
        "Beta should not be flattened into parent -- only pub use items appear"
    );
    // Both Alpha and Beta should exist in the child module
    let child = exports
        .child_modules
        .get("child")
        .expect("child_modules should contain 'child'");
    assert!(
        child.type_defs.contains_key("Alpha"),
        "child should have Alpha"
    );
    assert!(
        child.type_defs.contains_key("Beta"),
        "child should have Beta"
    );
}

/// Test 4: A file with `pub mod nonexistent;` should produce an error
/// because the child module file does not exist.
#[test]
fn test_nonexistent_pub_mod_errors() {
    let temp = tempfile::tempdir().expect("tempdir");
    let dir = temp.path();
    std::fs::write(dir.join("parent.ash"), "pub mod nonexistent;").expect("write parent");

    let mut cache = HashMap::new();
    let result = collect_module_exports(&dir.join("parent.ash"), &mut cache, &mut HashSet::new());

    let err =
        result.expect_err("collecting exports from file with nonexistent pub mod should fail");
    let msg = format!("{err}");
    assert!(
        msg.contains("pub mod 'nonexistent'") || msg.contains("module not found"),
        "error message should reference the missing module: {msg}",
    );
}

/// Test 5: `builtin fn` declarations are extracted as callables.
#[test]
fn test_builtin_fn_extraction() {
    let temp = tempfile::tempdir().expect("tempdir");
    let dir = temp.path();

    std::fs::write(
        dir.join("module.ash"),
        "\
pub builtin fn add(x: Int, y: Int) -> Int;
builtin fn private_helper(a: String) -> String;
pub type Role = System | User;",
    )
    .expect("write module");

    let mut cache = HashMap::new();
    let exports = collect_module_exports(&dir.join("module.ash"), &mut cache, &mut HashSet::new())
        .expect("collecting exports should succeed");

    // Only pub builtin fn is exported; module-private builtin fn is not.
    assert!(
        exports.callables.contains_key("add"),
        "module should export callable 'add'"
    );
    assert!(
        !exports.callables.contains_key("private_helper"),
        "module-private builtin fn should NOT be exported"
    );

    // Verify parameter names
    let add = exports.callables.get("add").expect("add callable");
    assert_eq!(add.params, vec!["x", "y"]);
    assert_eq!(add.exported_name, "add");

    // Verify type def is also collected (not disrupted by builtin fn extraction)
    assert!(
        exports.type_defs.contains_key("Role"),
        "module should still export type Role"
    );
}

/// Test 6: Mixed `pub fn` and `builtin fn` declarations coexist.
#[test]
fn test_mixed_fn_and_builtin_fn() {
    let temp = tempfile::tempdir().expect("tempdir");
    let dir = temp.path();

    std::fs::write(
        dir.join("module.ash"),
        "\
pub fn double(x: Int) -> Int { x * 2 }
pub builtin fn triple(x: Int) -> Int;
pub type Flag = On | Off;",
    )
    .expect("write module");

    let mut cache = HashMap::new();
    let exports = collect_module_exports(&dir.join("module.ash"), &mut cache, &mut HashSet::new())
        .expect("collecting exports should succeed");

    assert!(
        exports.callables.contains_key("double"),
        "module should export callable 'double' (pub fn)"
    );
    assert!(
        exports.callables.contains_key("triple"),
        "module should export callable 'triple' (pub builtin fn)"
    );
    assert!(
        exports.type_defs.contains_key("Flag"),
        "module should export type Flag"
    );

    // Verify builtin fn has Builtin kind
    let triple = exports.callables.get("triple").expect("triple callable");
    assert!(
        matches!(triple.kind, CallableKind::Builtin { .. }),
        "builtin fn kind should be CallableKind::Builtin"
    );
}

/// Test 7: `builtin fn` with type parameters is extracted correctly.
#[test]
fn test_builtin_fn_with_type_params() {
    let temp = tempfile::tempdir().expect("tempdir");
    let dir = temp.path();

    std::fs::write(
        dir.join("module.ash"),
        "pub builtin fn identity<T>(value: T) -> T;",
    )
    .expect("write module");

    let mut cache = HashMap::new();
    let exports = collect_module_exports(&dir.join("module.ash"), &mut cache, &mut HashSet::new())
        .expect("collecting exports should succeed");

    assert!(
        exports.callables.contains_key("identity"),
        "module should export callable 'identity'"
    );
    let identity = exports
        .callables
        .get("identity")
        .expect("identity callable");
    assert_eq!(identity.params, vec!["value"]);
}

#[test]
fn builtin_fn_callable_kind_carries_module_name() {
    let temp = tempfile::tempdir().expect("tempdir");
    let dir = temp.path();

    std::fs::write(
        dir.join("string.ash"),
        "pub builtin fn concat(a: String, b: String) -> String;\n",
    )
    .expect("write");
    std::fs::write(
        dir.join("caller.ash"),
        "use string::{concat}\nworkflow main { ret 0 }\n",
    )
    .expect("write");

    let result = super::load_ordinary_file(&dir.join("caller.ash")).expect("load");
    let callable = result
        .imported_callables
        .get("concat")
        .expect("concat callable");
    match &callable.kind {
        super::CallableKind::Builtin { module } => {
            assert_eq!(
                module.as_str(),
                "string",
                "module name must be populated from the import path by load_ordinary_file"
            );
        }
        other @ super::CallableKind::User { .. } => {
            panic!("expected Builtin {{ module }}, got: {other:?}")
        }
    }
}

#[test]
fn builtin_fn_glob_import_carries_module_name() {
    let temp = tempfile::tempdir().expect("tempdir");
    let dir = temp.path();

    std::fs::write(
        dir.join("math.ash"),
        "pub builtin fn add(x: Int, y: Int) -> Int;\npub builtin fn sub(x: Int, y: Int) -> Int;\n",
    )
    .expect("write");
    std::fs::write(
        dir.join("caller.ash"),
        "use math::*\nworkflow main { ret 0 }\n",
    )
    .expect("write");

    let result = super::load_ordinary_file(&dir.join("caller.ash")).expect("load");

    for name in &["add", "sub"] {
        let callable = result
            .imported_callables
            .get(*name)
            .unwrap_or_else(|| panic!("'{name}' should be in imported_callables"));
        match &callable.kind {
            super::CallableKind::Builtin { module } => {
                assert_eq!(
                    module.as_str(),
                    "math",
                    "glob import must stamp module name on Builtin callable '{name}'"
                );
            }
            other @ super::CallableKind::User { .. } => {
                panic!("expected Builtin {{ module }} for '{name}', got: {other:?}")
            }
        }
    }
}

#[test]
fn builtin_fn_higher_order_signature_imports_cleanly() {
    let temp = tempfile::tempdir().expect("tempdir");
    let dir = temp.path();

    std::fs::write(
        dir.join("act.ash"),
        "pub builtin fn bind<A, B>(ma: Act<A>, f: Fn(A) -> Act<B>) -> Act<B>;\n",
    )
    .expect("write module");

    let mut cache = HashMap::new();
    let exports = collect_module_exports(&dir.join("act.ash"), &mut cache, &mut HashSet::new())
        .expect(
            "higher-order builtin fn signatures should parse for current std::act placeholders",
        );

    assert!(
        exports.callables.contains_key("bind"),
        "expected higher-order builtin fn to be exported"
    );
}

#[test]
fn type_identity_collector_includes_builtin_type_forms() {
    let defs = with_legacy_type_snippet_compat(|| {
        collect_type_identity_defs_from_source_compat(
            "builtin type ActEnv;\npub builtin type PublicOpaque;\ntype Local = Int;\npub type Exported = String;",
        )
    })
    .expect("collect type identities");

    let names = defs.iter().map(|def| def.name.as_str()).collect::<Vec<_>>();
    assert_eq!(names, vec!["ActEnv", "PublicOpaque", "Local", "Exported"]);
    assert!(
        defs.iter()
            .find(|def| def.name == "ActEnv")
            .unwrap()
            .builtin
    );
    assert!(
        defs.iter()
            .find(|def| def.name == "PublicOpaque")
            .unwrap()
            .builtin
    );
}

#[test]
fn module_exports_include_opaque_private_type_identities_without_representation() {
    let temp = tempfile::tempdir().expect("tempdir");
    let module = temp.path().join("types.ash");
    std::fs::write(
        &module,
        "builtin type PrivateOpaque;\ntype PrivateAlias = Int;\npub builtin type PublicOpaque;\npub type PublicAlias = String;",
    )
    .expect("write module");

    let exports = collect_module_exports(&module, &mut HashMap::new(), &mut HashSet::new())
        .expect("collect exports");

    assert!(exports.type_defs.contains_key("PublicOpaque"));
    assert!(exports.type_defs.contains_key("PublicAlias"));
    let private_opaque = exports
        .type_defs
        .get("PrivateOpaque")
        .expect("private builtin identity should export opaquely");
    assert!(private_opaque.builtin);
    assert!(matches!(private_opaque.body, CoreTypeBody::Struct(ref fields) if fields.is_empty()));
    assert!(
        !exports.type_defs.contains_key("PrivateAlias"),
        "private ordinary aliases must not be exported/importable downstream"
    );
    assert!(!exports.constructor_defs.contains_key("PrivateOpaque"));
    assert!(!exports.constructor_defs.contains_key("PrivateAlias"));
}

#[test]
fn private_type_identity_can_import_without_representation_or_constructor() {
    let temp = tempfile::tempdir().expect("tempdir");
    let dir = temp.path();
    std::fs::write(
        dir.join("inner.ash"),
        "type Secret = Int;\npub type Public = Int;",
    )
    .expect("write inner");
    std::fs::write(dir.join("outer.ash"), "pub use inner::{Public};").expect("write outer");
    std::fs::write(
        dir.join("caller.ash"),
        "use outer::{Public}\nworkflow main { ret 0 }\n",
    )
    .expect("write caller");

    let loaded = load_ordinary_file(&dir.join("caller.ash"))
        .expect("public type remains importable through public re-export");
    assert!(
        loaded
            .imported_type_defs
            .iter()
            .any(|def| def.name == "Public")
    );
    assert!(
        !loaded
            .imported_type_defs
            .iter()
            .any(|def| def.name == "Secret")
    );

    let reexport_secret_module = dir.join("reexport_secret.ash");
    std::fs::write(&reexport_secret_module, "pub use inner::{Secret};")
        .expect("write re-export secret module");
    let err = collect_module_exports(
        &reexport_secret_module,
        &mut HashMap::new(),
        &mut HashSet::new(),
    )
    .expect_err("private ordinary re-export must fail");
    let msg = err.to_string();
    assert!(
        msg.contains("Secret") && msg.contains("pub use"),
        "private ordinary re-export diagnostic should mention Secret and pub use: {msg}"
    );

    let secret_caller = dir.join("secret_caller.ash");
    std::fs::write(
        &secret_caller,
        "use inner::{Secret}\nworkflow main { ret 0 }\n",
    )
    .expect("write secret caller");
    let err = load_ordinary_file(&secret_caller)
        .expect_err("private ordinary Secret identity should not import");
    let msg = err.to_string();
    assert!(
        msg.contains("Secret") && msg.contains("not found"),
        "private ordinary import diagnostic should mention Secret not found: {msg}"
    );
}

fn task_860_test_module_identity(
    module_id: usize,
    name: &str,
) -> ash_core::semantic_summary::ModuleIdentity {
    ash_core::semantic_summary::ModuleIdentity::new(
        Some(ash_core::module_graph::CrateId(860)),
        ash_core::module_graph::ModuleId(module_id),
        vec!["task860".to_string(), name.to_string()],
        ash_core::semantic_summary::ModuleSourceOrigin::Synthetic {
            reason: format!("task-860 {name}"),
        },
    )
}

fn task_860_anchor(label: &str) -> ash_core::semantic_summary::SourceAnchor {
    ash_core::semantic_summary::SourceAnchor::new(
        ash_core::semantic_summary::SourceOrigin::Synthetic {
            reason: "task-860 engine associated family summary merge".to_string(),
        },
        None,
        label,
    )
}

fn task_860_family_summary(result: &str) -> ash_core::semantic_summary::AssociatedFamilySummary {
    let module = task_860_test_module_identity(1, "families");
    let interface = ash_core::semantic_summary::InterfaceIdentityId::new(module, "Append");
    let member = ash_core::semantic_summary::AssociatedMemberIdentityId::associated_type(
        interface.clone(),
        "Out",
        vec!["Append".to_string(), "Out".to_string()],
    );
    let head = ash_core::type_ir::AssociatedFamilyHeadId {
        interface: interface.clone(),
        member: member.clone(),
    };
    let domain = ash_core::semantic_summary::SealedDomainId::new(
        task_860_test_module_identity(2, "domain"),
        "TypeList",
    );
    let projection = ash_core::type_ir::AssociatedFamilyProjection {
        head: head.clone(),
        interface_args: vec![ash_core::type_ir::CanonicalTypeExpr::Var("Xs".to_string())],
        kind: ash_core::kind::Kind::Type,
        rigidity: ash_core::type_ir::ProjectionRigidity::Neutral,
        mode: ash_core::type_ir::AssociatedFamilyProjectionMode::NeutralBlockedOrUnavailable,
    };
    ash_core::semantic_summary::AssociatedFamilySummary {
        head: head.clone(),
        interface_identity: interface,
        member_identity: member,
        visible_name: "Append::Out".to_string(),
        result_domain: ash_core::type_ir::CanonicalTypeExpr::Primitive("TypeList".to_string()),
        result_kind: ash_core::kind::Kind::Type,
        export_mode: ash_core::semantic_summary::AssociatedFamilyExportMode::TransparentEquations,
        schemes: vec![ash_core::type_ir::AssociatedFamilyScheme {
            head: head.clone(),
            params: Vec::new(),
            result_domain: ash_core::type_ir::CanonicalTypeExpr::Primitive("TypeList".to_string()),
            result_kind: ash_core::kind::Kind::Type,
            equations: vec![ash_core::type_ir::AssociatedFamilyEquation {
                head,
                ordinal: 0,
                interface_arg_patterns: Vec::new(),
                result: ash_core::type_ir::AssociatedFamilyResultExpr::Var {
                    name: result.to_string(),
                    kind: ash_core::kind::Kind::Type,
                    constraint: ash_core::type_ir::AssociatedFamilyResultConstraint::Domain(
                        domain.clone(),
                    ),
                    source_anchor: task_860_anchor("family result"),
                },
                decreases: None,
                source_anchor: task_860_anchor("family equation"),
                case_head_anchor: task_860_anchor("family case head"),
            }],
            source_anchor: task_860_anchor("family scheme"),
        }],
        dependency_closure: ash_core::semantic_summary::AssociatedFamilyDependencyClosure {
            ordinary_types: Vec::new(),
            sealed_domains: vec![domain],
            domain_constructors: Vec::new(),
            type_functions: Vec::new(),
            associated_projections: vec![projection],
            associated_families: Vec::new(),
            type_function_summaries: Vec::new(),
            closure_metadata: ash_core::semantic_summary::AssociatedFamilyClosureMetadata {
                public_closure_checked: true,
                public_ordinary_type_count: 0,
                public_sealed_domain_count: 1,
                public_domain_constructor_count: 0,
                public_type_function_count: 0,
                public_associated_family_count: 1,
                public_projection_count: 1,
                helper_family_count: 0,
            },
        },
        source_anchor: task_860_anchor("family summary"),
        revalidation_metadata: ash_core::semantic_summary::AssociatedFamilyRevalidationMetadata {
            spec_version: ash_core::semantic_summary::SummaryVersion::SPEC063_ASSOCIATED_FAMILY_V4,
            kind_and_domain_checked: true,
            coverage_and_overlap_checked: true,
            coherence_checked: true,
            recursion_checked: false,
            decreases: Vec::new(),
        },
    }
}

#[test]
fn task_860_imported_summary_merge_preserves_associated_family_payloads() {
    let module = task_860_test_module_identity(3, "summary");
    let family = task_860_family_summary("Ys");
    let same_family = task_860_family_summary("Ys");
    let different_family_payload = task_860_family_summary("DifferentYs");

    let left = ash_core::semantic_summary::ModuleSemanticSummary::new(module.clone())
        .with_version(ash_core::semantic_summary::SummaryVersion::SPEC063_ASSOCIATED_FAMILY_V4)
        .with_exported_associated_family(family.clone());
    let same = ash_core::semantic_summary::ModuleSemanticSummary::new(module.clone())
        .with_version(ash_core::semantic_summary::SummaryVersion::SPEC063_ASSOCIATED_FAMILY_V4)
        .with_exported_associated_family(same_family);
    let different = ash_core::semantic_summary::ModuleSemanticSummary::new(module)
        .with_version(ash_core::semantic_summary::SummaryVersion::SPEC063_ASSOCIATED_FAMILY_V4)
        .with_exported_associated_family(different_family_payload);

    assert!(imported_summary_type_set_matches(&left, &same));
    assert!(!imported_summary_type_set_matches(&left, &different));

    let mut summaries = vec![left];
    let mut keys = summaries
        .iter()
        .map(imported_summary_key)
        .collect::<HashSet<_>>();
    merge_or_push_imported_semantic_summary(&mut summaries, &mut keys, same);
    assert_eq!(
        summaries.len(),
        1,
        "identical family facts should deduplicate"
    );
    merge_or_push_imported_semantic_summary(&mut summaries, &mut keys, different);
    assert_eq!(
        summaries.len(),
        2,
        "different associated-family payloads must not be dropped as compatible"
    );
    assert_eq!(summaries[0].exported_associated_families, vec![family]);
}

#[test]
fn task_1771_rejects_imported_macro_summary_template_signature_mismatch() {
    let module = ash_parser::parse_surface_file("pub macro id_int(x: Int) -> Int => x;")
        .expect("module parses");
    let mut summary = ash_parser::surface::collect_public_macro_summaries(&module, "provider")
        .expect("summary collects")
        .pop()
        .expect("public macro summary exists");
    summary
        .typed_signature
        .as_mut()
        .expect("typed summary exists")
        .return_type = Some(Type::Name("String".into()));
    let table =
        ash_parser::surface::build_local_macro_table(&module).expect("local macro table builds");
    let template = table.resolve("id_int").expect("template exists").clone();
    let mut exports = ModuleExports::default();

    let err = insert_macro_summary_export(&mut exports, summary, template)
        .expect_err("summary/template signature mismatch rejects");
    let message = err.to_string();
    assert!(
        message.contains("typed signature does not match template"),
        "unexpected error: {message}"
    );
}
