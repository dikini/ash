use ash_core::kind::Kind;
use ash_core::module_graph::{CrateId, ModuleId};
use ash_core::semantic_summary::{
    ModuleIdentity, ModuleSourceOrigin, SourceAnchor, SourceOrigin, TypeDeclId,
};
use ash_core::type_ir::{
    CanonicalTypeExpr, PartialTypeArg, PartialTypeConstructorApp, TypeConstructorExpr,
    TypeConstructorHeadId, TypeHoleAmbiguity, TypeHoleId, TypeHoleMetadata,
};

fn module_identity() -> ModuleIdentity {
    ModuleIdentity::new(
        Some(CrateId(899)),
        ModuleId(119),
        vec!["task_899".to_string(), "types".to_string()],
        ModuleSourceOrigin::Synthetic {
            reason: "TASK-899 carrier test".to_string(),
        },
    )
}

fn source_anchor(label: &str) -> SourceAnchor {
    SourceAnchor::new(
        SourceOrigin::Synthetic {
            reason: "TASK-899 focused test".to_string(),
        },
        None,
        label,
    )
}

fn nominal_type_decl(name: &str) -> TypeDeclId {
    TypeDeclId::ordinary(module_identity(), name)
}

#[test]
fn type_hole_id_preserves_stable_numeric_identity_and_metadata() {
    let hole = TypeHoleId::new(42);
    let metadata = TypeHoleMetadata::new(
        hole,
        source_anchor("do target hole"),
        Some(Kind::Type),
        TypeHoleAmbiguity::ExpectedValueSlot,
    );

    assert_eq!(hole.as_u64(), 42);
    assert_eq!(metadata.id, hole);
    assert_eq!(metadata.expected_kind, Some(Kind::Type));
    assert_eq!(metadata.ambiguity, TypeHoleAmbiguity::ExpectedValueSlot);
    assert_eq!(metadata.source_anchor.label, "do target hole");
}

#[test]
fn partial_constructor_application_preserves_hole_arguments_without_fake_nominal_saturation() {
    let result_head = TypeConstructorHeadId::nominal(nominal_type_decl("Result"), "Result");
    let value_hole = TypeHoleId::new(7);
    let error_arg = CanonicalTypeExpr::Var("E".to_string());
    let hole_metadata = TypeHoleMetadata::new(
        value_hole,
        source_anchor("_"),
        Some(Kind::Type),
        TypeHoleAmbiguity::ExpectedValueSlot,
    );
    let partial = PartialTypeConstructorApp::new_with_hole_metadata(
        result_head.clone(),
        vec![
            PartialTypeArg::Hole(value_hole),
            PartialTypeArg::Applied(error_arg.clone()),
        ],
        Kind::n_ary(1),
        vec![hole_metadata.clone()],
        Some(source_anchor("Result<_, E>")),
    );

    assert_eq!(partial.head, result_head);
    assert_eq!(partial.result_kind, Kind::n_ary(1));
    assert_eq!(
        partial.source_anchor.as_ref().unwrap().label,
        "Result<_, E>"
    );
    assert!(matches!(partial.args[0], PartialTypeArg::Hole(id) if id == value_hole));
    assert!(matches!(&partial.args[1], PartialTypeArg::Applied(expr) if expr == &error_arg));
    assert_eq!(partial.metadata_for_hole(value_hole), Some(&hole_metadata));
    assert_eq!(
        partial
            .metadata_for_hole(value_hole)
            .unwrap()
            .source_anchor
            .label,
        "_"
    );

    let constructor_expr = TypeConstructorExpr::PartialApplication(partial.clone());
    match constructor_expr {
        TypeConstructorExpr::PartialApplication(app) => assert_eq!(app.args, partial.args),
        other => panic!("expected partial application carrier, got {other:?}"),
    }

    let saturated_nominal = CanonicalTypeExpr::NominalApp {
        origin: nominal_type_decl("Result"),
        visible_name: "Result".to_string(),
        args: vec![CanonicalTypeExpr::Var("_".to_string()), error_arg],
        kind: Kind::Type,
    };

    assert_ne!(
        format!("{partial:?}"),
        format!("{saturated_nominal:?}"),
        "partial applications must not be encoded as saturated nominal applications with fake args",
    );
}

#[test]
fn constructor_expr_distinguishes_proper_types_constructor_heads_and_partial_apps() {
    let list_decl = nominal_type_decl("List");
    let proper = TypeConstructorExpr::ProperType(CanonicalTypeExpr::NominalApp {
        origin: list_decl.clone(),
        visible_name: "List".to_string(),
        args: vec![CanonicalTypeExpr::Primitive("Int".to_string())],
        kind: Kind::Type,
    });
    let head =
        TypeConstructorExpr::ConstructorHead(TypeConstructorHeadId::nominal(list_decl, "List"));
    let partial = TypeConstructorExpr::PartialApplication(PartialTypeConstructorApp::new(
        TypeConstructorHeadId::nominal(nominal_type_decl("Result"), "Result"),
        vec![
            PartialTypeArg::Hole(TypeHoleId::new(1)),
            PartialTypeArg::Applied(CanonicalTypeExpr::Primitive("String".to_string())),
        ],
        Kind::n_ary(1),
        None,
    ));

    assert_ne!(proper, head);
    assert_ne!(head, partial);
    assert_ne!(proper, partial);
}
