//! TASK-895: TypeEnv registration/kinding for promoted data constructors.
//!
//! These tests stay at the TypeEnv semantic-summary boundary. They deliberately
//! do not assert parser lowering, type-function acceptance, proposition
//! acceptance, normalizer reduction, or runtime ADT behavior.

use ash_core::ast::{TypeBody, TypeExpr, VariantDef, VariantPayload, Visibility};
use ash_core::kind::Kind;
use ash_core::module_graph::{CrateId, ModuleId};
use ash_core::semantic_summary::{
    ConstructorId, ConstructorPayloadKind, ModuleIdentity, ModuleSemanticSummary,
    ModuleSourceOrigin, PromotedConstructorFieldSummary, PromotedConstructorId,
    PromotedConstructorSummary, PromotedDataKindId, PromotedDataKindSummary,
    RepresentationExposure, SourceAnchor, SourceOrigin, SummaryVersion, TypeDeclId,
    TypeDeclSummary, TypeRepresentationSummary,
};
use ash_typeck::TypeEnv;

fn module(id: usize) -> ModuleIdentity {
    ModuleIdentity::new(
        Some(CrateId(895)),
        ModuleId(id),
        vec!["task895".to_string(), format!("m{id}")],
        ModuleSourceOrigin::Synthetic {
            reason: format!("task-895-{id}"),
        },
    )
}

fn anchor(label: &str) -> SourceAnchor {
    SourceAnchor::new(
        SourceOrigin::Synthetic {
            reason: "task-895-promoted-registration".into(),
        },
        None,
        label,
    )
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

fn promoted_kind_id(
    module: &ModuleIdentity,
    source_type_name: &str,
    kind_name: &str,
) -> PromotedDataKindId {
    PromotedDataKindId::new(
        module.clone(),
        TypeDeclId::ordinary(module.clone(), source_type_name),
        kind_name,
    )
}

trait OrdinaryTypeIdExt {
    fn ordinary_type(&self, name: &str) -> TypeDeclId;
}

impl OrdinaryTypeIdExt for ModuleIdentity {
    fn ordinary_type(&self, name: &str) -> TypeDeclId {
        TypeDeclId::ordinary(self.clone(), name)
    }
}

fn promoted_ctor_summary(
    kind: &PromotedDataKindId,
    source_type_name: &str,
    ctor_name: &str,
    payload_kind: ConstructorPayloadKind,
    fields: Vec<PromotedConstructorFieldSummary>,
) -> PromotedConstructorSummary {
    let source_ctor = ConstructorId::variant(
        kind.source_type
            .module
            .clone()
            .ordinary_type(source_type_name),
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

fn promoted_kind_summary(
    module: &ModuleIdentity,
    source_type_name: &str,
    kind_name: &str,
    constructors: Vec<PromotedConstructorSummary>,
) -> PromotedDataKindSummary {
    constructors.into_iter().fold(
        PromotedDataKindSummary::new(
            promoted_kind_id(module, source_type_name, kind_name),
            kind_name,
            Visibility::Public,
            TypeDeclId::ordinary(module.clone(), source_type_name),
            anchor(kind_name),
        ),
        PromotedDataKindSummary::with_constructor,
    )
}

fn v6_summary(module: &ModuleIdentity) -> ModuleSemanticSummary {
    ModuleSemanticSummary::new(module.clone())
        .with_version(SummaryVersion::SPEC065_PROMOTED_DATA_KIND_V6)
}

#[test]
fn registers_zero_argument_promoted_constructor_with_kind_and_domain_metadata() {
    let module = module(1);
    let nat = source_type(&module, "Nat", vec![unit_variant("Z")]);
    let kind_id = promoted_kind_id(&module, "Nat", "NatKind");
    let ctor = promoted_ctor_summary(&kind_id, "Nat", "Z", ConstructorPayloadKind::Unit, vec![]);
    let ctor_id = ctor.id.clone();
    let summary = v6_summary(&module)
        .with_exported_type(nat)
        .with_exported_promoted_data_kind(promoted_kind_summary(
            &module,
            "Nat",
            "NatKind",
            vec![ctor],
        ));

    let mut env = TypeEnv::new();
    env.register_module_semantic_summary(&summary)
        .expect("promoted zero-argument constructor registers");

    let kind = env
        .lookup_promoted_data_kind("NatKind")
        .expect("promoted data kind is source-visible");
    assert_eq!(kind.id, kind_id);
    assert_eq!(kind.constructors.len(), 1);

    let registered_ctor = env
        .lookup_promoted_constructor_by_id(&ctor_id)
        .expect("promoted constructor identity is registered");
    assert_eq!(registered_ctor.id, ctor_id);

    let kinding = env
        .promoted_constructor_kind(&ctor_id)
        .expect("promoted constructor kinding is available");
    assert_eq!(kinding.kind, Kind::Type);
    assert_eq!(kinding.result_data_kind, kind_id);
    assert!(kinding.field_data_kind_constraints.is_empty());

    assert_eq!(
        env.lookup_constructor("Z"),
        None,
        "promotion must not register a runtime constructor"
    );
    assert!(
        env.lookup_sealed_domain("NatKind").is_none(),
        "promoted data kinds must not register as sealed domains"
    );
}

#[test]
fn registers_promoted_constructor_with_promoted_field_domain_metadata() {
    let module = module(2);
    let elem = source_type(&module, "Elem", vec![unit_variant("E")]);
    let boxed = source_type(
        &module,
        "Boxed",
        vec![tuple_variant("Box", vec![TypeExpr::Named("Elem".into())])],
    );

    let elem_kind = promoted_kind_id(&module, "Elem", "ElemKind");
    let boxed_kind = promoted_kind_id(&module, "Boxed", "BoxedKind");
    let field = PromotedConstructorFieldSummary::new(
        "value",
        Kind::Type,
        Some(elem_kind.clone()),
        anchor("value"),
    );
    let boxed_ctor = promoted_ctor_summary(
        &boxed_kind,
        "Boxed",
        "Box",
        ConstructorPayloadKind::Tuple,
        vec![field],
    );
    let boxed_ctor_id = boxed_ctor.id.clone();
    let summary = v6_summary(&module)
        .with_exported_type(elem)
        .with_exported_type(boxed)
        .with_exported_promoted_data_kind(promoted_kind_summary(
            &module,
            "Elem",
            "ElemKind",
            vec![promoted_ctor_summary(
                &elem_kind,
                "Elem",
                "E",
                ConstructorPayloadKind::Unit,
                vec![],
            )],
        ))
        .with_exported_promoted_data_kind(promoted_kind_summary(
            &module,
            "Boxed",
            "BoxedKind",
            vec![boxed_ctor],
        ));

    let mut env = TypeEnv::new();
    env.register_module_semantic_summary(&summary)
        .expect("field constrained to a promoted data kind registers");

    let kinding = env
        .promoted_constructor_kind(&boxed_ctor_id)
        .expect("fielded promoted constructor kinding is available");
    assert_eq!(kinding.kind, Kind::arrow(Kind::Type, Kind::Type));
    assert_eq!(kinding.result_data_kind, boxed_kind);
    assert_eq!(kinding.field_data_kind_constraints, vec![Some(elem_kind)]);
}

#[test]
fn rejects_payload_field_whose_source_type_does_not_match_promoted_kind_constraint() {
    let module = module(6);
    let elem = source_type(&module, "Elem", vec![unit_variant("E")]);
    let other = source_type(&module, "Other", vec![unit_variant("O")]);
    let boxed = source_type(
        &module,
        "Boxed",
        vec![tuple_variant("Box", vec![TypeExpr::Named("Other".into())])],
    );

    let elem_kind = promoted_kind_id(&module, "Elem", "ElemKind");
    let other_kind = promoted_kind_id(&module, "Other", "OtherKind");
    let boxed_kind = promoted_kind_id(&module, "Boxed", "BoxedKind");
    let bad_field = PromotedConstructorFieldSummary::new(
        "value",
        Kind::Type,
        Some(elem_kind.clone()),
        anchor("value"),
    );
    let boxed_ctor = promoted_ctor_summary(
        &boxed_kind,
        "Boxed",
        "Box",
        ConstructorPayloadKind::Tuple,
        vec![bad_field],
    );
    let summary = v6_summary(&module)
        .with_exported_type(elem)
        .with_exported_type(other)
        .with_exported_type(boxed)
        .with_exported_promoted_data_kind(promoted_kind_summary(
            &module,
            "Elem",
            "ElemKind",
            vec![promoted_ctor_summary(
                &elem_kind,
                "Elem",
                "E",
                ConstructorPayloadKind::Unit,
                vec![],
            )],
        ))
        .with_exported_promoted_data_kind(promoted_kind_summary(
            &module,
            "Other",
            "OtherKind",
            vec![promoted_ctor_summary(
                &other_kind,
                "Other",
                "O",
                ConstructorPayloadKind::Unit,
                vec![],
            )],
        ))
        .with_exported_promoted_data_kind(promoted_kind_summary(
            &module,
            "Boxed",
            "BoxedKind",
            vec![boxed_ctor],
        ));

    let mut env = TypeEnv::new();
    let err = env
        .register_module_semantic_summary(&summary)
        .expect_err("field constraint must match the actual source field type");
    let actual = err.to_string();
    assert!(
        actual.contains("field 'value' in promoted constructor 'Box' expects source field type for promoted data kind 'ElemKind'"),
        "unexpected diagnostic: {actual}"
    );
}

#[test]
fn rejects_record_payload_field_name_mismatch() {
    let module = module(7);
    let elem = source_type(&module, "Elem", vec![unit_variant("E")]);
    let record = source_type(
        &module,
        "RecordBox",
        vec![VariantDef {
            name: "Box".into(),
            fields: vec![("actual".into(), TypeExpr::Named("Elem".into()))],
            payload: VariantPayload::Record(vec![(
                "actual".into(),
                TypeExpr::Named("Elem".into()),
            )]),
        }],
    );

    let elem_kind = promoted_kind_id(&module, "Elem", "ElemKind");
    let record_kind = promoted_kind_id(&module, "RecordBox", "RecordKind");
    let bad_field = PromotedConstructorFieldSummary::new(
        "wrong",
        Kind::Type,
        Some(elem_kind.clone()),
        anchor("wrong"),
    );
    let record_ctor = promoted_ctor_summary(
        &record_kind,
        "RecordBox",
        "Box",
        ConstructorPayloadKind::Record,
        vec![bad_field],
    );
    let summary = v6_summary(&module)
        .with_exported_type(elem)
        .with_exported_type(record)
        .with_exported_promoted_data_kind(promoted_kind_summary(
            &module,
            "Elem",
            "ElemKind",
            vec![promoted_ctor_summary(
                &elem_kind,
                "Elem",
                "E",
                ConstructorPayloadKind::Unit,
                vec![],
            )],
        ))
        .with_exported_promoted_data_kind(promoted_kind_summary(
            &module,
            "RecordBox",
            "RecordKind",
            vec![record_ctor],
        ));

    let mut env = TypeEnv::new();
    let err = env
        .register_module_semantic_summary(&summary)
        .expect_err("record promoted field names must match source record fields");
    let actual = err.to_string();
    assert!(
        actual.contains(
            "field 'wrong' in promoted constructor 'Box' does not match source field 'actual'"
        ),
        "unexpected diagnostic: {actual}"
    );
}

#[test]
fn rejects_promoted_data_kind_that_omits_source_constructors() {
    let module = module(8);
    let nat = source_type(
        &module,
        "Nat",
        vec![
            unit_variant("Z"),
            tuple_variant("S", vec![TypeExpr::Named("Nat".into())]),
        ],
    );
    let nat_kind = promoted_kind_id(&module, "Nat", "NatKind");
    let summary = v6_summary(&module)
        .with_exported_type(nat)
        .with_exported_promoted_data_kind(promoted_kind_summary(
            &module,
            "Nat",
            "NatKind",
            vec![promoted_ctor_summary(
                &nat_kind,
                "Nat",
                "Z",
                ConstructorPayloadKind::Unit,
                vec![],
            )],
        ));

    let mut env = TypeEnv::new();
    let err = env
        .register_module_semantic_summary(&summary)
        .expect_err("promoted data kind must include every source constructor");
    let actual = err.to_string();
    assert!(
        actual.contains(
            "promoted data-kind 'NatKind' has 1 constructor(s) but source ADT 'Nat' has 2"
        ),
        "unexpected diagnostic: {actual}"
    );
}

#[test]
fn rejects_promoted_data_kind_constructors_out_of_source_order() {
    let module = module(9);
    let nat = source_type(
        &module,
        "Nat",
        vec![
            unit_variant("Z"),
            tuple_variant("S", vec![TypeExpr::Named("Nat".into())]),
        ],
    );
    let nat_kind = promoted_kind_id(&module, "Nat", "NatKind");
    let s_field = PromotedConstructorFieldSummary::new(
        "pred",
        Kind::Type,
        Some(nat_kind.clone()),
        anchor("pred"),
    );
    let s_ctor = promoted_ctor_summary(
        &nat_kind,
        "Nat",
        "S",
        ConstructorPayloadKind::Tuple,
        vec![s_field],
    );
    let z_ctor = promoted_ctor_summary(&nat_kind, "Nat", "Z", ConstructorPayloadKind::Unit, vec![]);
    let summary = v6_summary(&module)
        .with_exported_type(nat)
        .with_exported_promoted_data_kind(promoted_kind_summary(
            &module,
            "Nat",
            "NatKind",
            vec![s_ctor, z_ctor],
        ));

    let mut env = TypeEnv::new();
    let err = env
        .register_module_semantic_summary(&summary)
        .expect_err("promoted constructor order must match source order");
    let actual = err.to_string();
    assert!(
        actual
            .contains("promoted constructor 'S' at index 0 does not match source constructor 'Z'"),
        "unexpected diagnostic: {actual}"
    );
}

#[test]
fn rejects_payload_field_without_promoted_data_kind_constraint() {
    let module = module(3);
    let nat = source_type(
        &module,
        "Nat",
        vec![tuple_variant("S", vec![TypeExpr::Named("Int".into())])],
    );
    let kind_id = promoted_kind_id(&module, "Nat", "NatKind");
    let unsupported_field =
        PromotedConstructorFieldSummary::new("pred", Kind::Type, None, anchor("pred"));
    let summary = v6_summary(&module)
        .with_exported_type(nat)
        .with_exported_promoted_data_kind(promoted_kind_summary(
            &module,
            "Nat",
            "NatKind",
            vec![promoted_ctor_summary(
                &kind_id,
                "Nat",
                "S",
                ConstructorPayloadKind::Tuple,
                vec![unsupported_field],
            )],
        ));

    let mut env = TypeEnv::new();
    let err = env
        .register_module_semantic_summary(&summary)
        .expect_err("non-promoted payload fields are rejected for the MVP");
    let actual = err.to_string();
    assert!(
        actual.contains(
            "field 'pred' in promoted constructor 'S' lacks promoted data-kind constraint"
        ),
        "unexpected diagnostic: {actual}"
    );
}

#[test]
fn rejects_promoted_constructor_whose_source_constructor_does_not_belong_to_source_adt() {
    let module = module(4);
    let nat = source_type(&module, "Nat", vec![unit_variant("Z")]);
    let other = source_type(&module, "Other", vec![unit_variant("OtherZ")]);
    let kind_id = promoted_kind_id(&module, "Nat", "NatKind");
    let bad_source_ctor = ConstructorId::variant(
        TypeDeclId::ordinary(module.clone(), "Other"),
        "OtherZ",
        ConstructorPayloadKind::Unit,
    );
    let bad_ctor = PromotedConstructorSummary::new(
        PromotedConstructorId::new(kind_id.clone(), bad_source_ctor.clone(), "Z"),
        "Z",
        bad_source_ctor,
        vec![],
        Visibility::Public,
        anchor("Z"),
    );
    let summary = v6_summary(&module)
        .with_exported_type(nat)
        .with_exported_type(other)
        .with_exported_promoted_data_kind(promoted_kind_summary(
            &module,
            "Nat",
            "NatKind",
            vec![bad_ctor],
        ));

    let mut env = TypeEnv::new();
    let err = env
        .register_module_semantic_summary(&summary)
        .expect_err("source constructor parent must be the promoted source ADT");
    let actual = err.to_string();
    assert!(
        actual.contains(
            "source constructor for promoted constructor 'Z' does not belong to source ADT 'Nat'"
        ),
        "unexpected diagnostic: {actual}"
    );
}

#[test]
fn ordinary_adt_summaries_do_not_trigger_automatic_datakinds_promotion() {
    let module = module(5);
    let nat = source_type(&module, "Nat", vec![unit_variant("Z")]);
    let summary = v6_summary(&module).with_exported_type(nat);

    let mut env = TypeEnv::new();
    env.register_module_semantic_summary(&summary)
        .expect("ordinary ADT summary registers");

    assert!(env.lookup_promoted_data_kind("Nat").is_none());
    assert!(env.lookup_promoted_data_kind("NatKind").is_none());
}
