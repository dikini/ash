use super::*;
use ash_core::ast::{TypeBody, TypeDef, TypeExpr, VariantDef, Visibility};
use ash_core::semantic_summary::ConstructorId;
use ash_core::type_ir::PromotedConstructorApp;

// ============================================================
// TypeInfo Tests
// ============================================================

#[test]
fn test_type_info_name() {
    let enum_def = TypeInfo::Enum {
        name: "Option".to_string(),
        params: vec![],
        variants: vec![],
    };
    assert_eq!(enum_def.name(), "Option");

    let struct_def = TypeInfo::Struct {
        name: "Point".to_string(),
        params: vec![],
        fields: vec![],
    };
    assert_eq!(struct_def.name(), "Point");
}

#[test]
fn test_type_info_lookup_variant() {
    let enum_def = TypeInfo::Enum {
        name: "Option".to_string(),
        params: vec![],
        variants: vec![
            VariantInfo {
                name: "Some".to_string(),
                fields: vec![("value".to_string(), Type::Int)],
                payload_shape: VariantPayloadShape::Record,
            },
            VariantInfo {
                name: "None".to_string(),
                fields: vec![],
                payload_shape: VariantPayloadShape::Unit,
            },
        ],
    };

    let (idx, variant) = enum_def.lookup_variant("Some").unwrap();
    assert_eq!(idx, 0);
    assert_eq!(variant.name, "Some");

    let (idx, variant) = enum_def.lookup_variant("None").unwrap();
    assert_eq!(idx, 1);
    assert_eq!(variant.name, "None");

    assert!(enum_def.lookup_variant("Unknown").is_none());
}

#[test]
fn test_struct_info_lookup_variant_returns_none() {
    let struct_def = TypeInfo::Struct {
        name: "Point".to_string(),
        params: vec![],
        fields: vec![("x".to_string(), Type::Int)],
    };
    assert!(struct_def.lookup_variant("x").is_none());
}

// ============================================================
// TypeEnv Tests
// ============================================================

#[test]
fn test_type_env_new() {
    let env = TypeEnv::new();
    assert!(!env.has_type("Option"));
    assert!(!env.has_constructor("Some"));
}

#[test]
fn test_type_env_with_builtin_types() {
    let env = TypeEnv::with_builtin_types();

    // Check Option type exists
    assert!(env.has_type("Option"));
    assert!(env.has_constructor("Some"));
    assert!(env.has_constructor("None"));

    // Check Result type exists
    assert!(env.has_type("Result"));
    assert!(env.has_constructor("Ok"));
    assert!(env.has_constructor("Err"));

    // Neither the retired tower wrapper nor its runtime implementation detail is source-visible.
    assert!(!env.has_type("ActEnv"));
    assert!(!env.has_type("Act"));
    assert!(!env.has_type("Proc"));
}

#[test]
fn test_lookup_constructor() {
    let env = TypeEnv::with_builtin_types();

    let (type_name, variant_idx) = env.lookup_constructor("Some").unwrap();
    assert_eq!(type_name, "Option");
    assert_eq!(variant_idx, 0);

    let (type_name, variant_idx) = env.lookup_constructor("None").unwrap();
    assert_eq!(type_name, "Option");
    assert_eq!(variant_idx, 1);

    let (type_name, variant_idx) = env.lookup_constructor("Ok").unwrap();
    assert_eq!(type_name, "Result");
    assert_eq!(variant_idx, 0);

    let (type_name, variant_idx) = env.lookup_constructor("Err").unwrap();
    assert_eq!(type_name, "Result");
    assert_eq!(variant_idx, 1);

    assert!(env.lookup_constructor("Unknown").is_none());
}

#[test]
fn test_lookup_type() {
    let env = TypeEnv::with_builtin_types();

    let type_def = env.lookup_type("Option").unwrap();
    assert_eq!(type_def.name, "Option");
    assert_eq!(type_def.params.len(), 1);

    let type_def = env.lookup_type("Result").unwrap();
    assert_eq!(type_def.name, "Result");
    assert_eq!(type_def.params.len(), 2);

    assert!(env.lookup_type("Unknown").is_none());
}

#[test]
fn test_get_variant() {
    let env = TypeEnv::with_builtin_types();

    let (type_info, variant_idx, variant) = env.get_variant("Some").unwrap();
    assert_eq!(type_info.name(), "Option");
    assert_eq!(variant_idx, 0);
    assert_eq!(variant.name, "Some");
    assert_eq!(variant.fields.len(), 1);
    assert_eq!(variant.fields[0].0, "value");

    let (_, _, variant) = env.get_variant("None").unwrap();
    assert_eq!(variant.name, "None");
    assert!(variant.fields.is_empty());

    assert!(env.get_variant("Unknown").is_none());
}

#[test]
fn test_register_custom_type() {
    let mut env = TypeEnv::new();

    let status_type = TypeDef {
        name: "Status".to_string(),
        params: vec![],
        body: TypeBody::Enum(vec![
            VariantDef {
                name: "Pending".to_string(),
                fields: vec![],
                payload: VariantPayload::Unit,
            },
            VariantDef {
                name: "Complete".to_string(),
                fields: vec![("result".to_string(), TypeExpr::Named("Int".to_string()))],
                payload: VariantPayload::Record(vec![(
                    "result".to_string(),
                    TypeExpr::Named("Int".to_string()),
                )]),
            },
        ]),
        visibility: Visibility::Public,
        builtin: false,
    };

    env.register_type(&status_type).unwrap();

    assert!(env.has_type("Status"));
    assert!(env.has_constructor("Pending"));
    assert!(env.has_constructor("Complete"));

    let (type_name, idx) = env.lookup_constructor("Pending").unwrap();
    assert_eq!(type_name, "Status");
    assert_eq!(idx, 0);

    let (type_name, idx) = env.lookup_constructor("Complete").unwrap();
    assert_eq!(type_name, "Status");
    assert_eq!(idx, 1);
}

#[test]
fn test_register_type_identity_keeps_constructors_hidden() {
    let mut env = TypeEnv::new();

    let hidden_type = TypeDef {
        name: "Hidden".to_string(),
        params: vec!["A".to_string()],
        body: TypeBody::Enum(vec![VariantDef {
            name: "Hidden".to_string(),
            fields: vec![("value".to_string(), TypeExpr::Named("A".to_string()))],
            payload: VariantPayload::Record(vec![(
                "value".to_string(),
                TypeExpr::Named("A".to_string()),
            )]),
        }]),
        visibility: Visibility::Private,
        builtin: false,
    };

    env.register_type_identity(&hidden_type).unwrap();

    let type_def = env
        .lookup_type("Hidden")
        .expect("type identity should register");
    assert_eq!(type_def.params.len(), 1);
    assert!(
        env.lookup_constructor("Hidden").is_none(),
        "identity-only registration should not expose constructors"
    );
}

#[test]
fn test_expose_type_representation_registers_constructors_after_identity() {
    let mut env = TypeEnv::new();

    let hidden_type = TypeDef {
        name: "Hidden".to_string(),
        params: vec![],
        body: TypeBody::Enum(vec![VariantDef {
            name: "Reveal".to_string(),
            fields: vec![("value".to_string(), TypeExpr::Named("Int".to_string()))],
            payload: VariantPayload::Record(vec![(
                "value".to_string(),
                TypeExpr::Named("Int".to_string()),
            )]),
        }]),
        visibility: Visibility::Private,
        builtin: false,
    };

    env.register_type_identity(&hidden_type).unwrap();
    assert!(env.lookup_constructor("Reveal").is_none());

    env.expose_type_representation("Hidden").unwrap();

    let (type_name, variant_idx) = env
        .lookup_constructor("Reveal")
        .expect("constructor should become visible after representation exposure");
    assert_eq!(type_name, "Hidden");
    assert_eq!(variant_idx, 0);
}

#[test]
fn test_option_type_structure() {
    let env = TypeEnv::with_builtin_types();

    // Check AST type definition
    let type_def = env.lookup_type("Option").unwrap();
    assert_eq!(type_def.name, "Option");
    assert_eq!(type_def.params.len(), 1);

    // Check internal type info
    let type_info = env.lookup_type_info("Option").unwrap();
    match type_info {
        TypeInfo::Enum {
            name,
            params,
            variants,
        } => {
            assert_eq!(name, "Option");
            assert_eq!(params.len(), 1);
            assert_eq!(variants.len(), 2);

            // Some variant
            assert_eq!(variants[0].name, "Some");
            assert_eq!(variants[0].fields.len(), 1);
            assert_eq!(variants[0].fields[0].0, "value");
            // Should be a type variable
            assert!(matches!(variants[0].fields[0].1, Type::Var(_)));

            // None variant
            assert_eq!(variants[1].name, "None");
            assert!(variants[1].fields.is_empty());
        }
        _ => panic!("Option should be an enum"),
    }
}

#[test]
fn test_result_type_structure() {
    let env = TypeEnv::with_builtin_types();

    // Check AST type definition
    let ast_type_def = env.lookup_type("Result").unwrap();
    assert_eq!(ast_type_def.name, "Result");
    assert_eq!(ast_type_def.params.len(), 2);

    // Check internal type info
    let type_info = env.lookup_type_info("Result").unwrap();
    match type_info {
        TypeInfo::Enum {
            name,
            params,
            variants,
        } => {
            assert_eq!(name, "Result");
            assert_eq!(params.len(), 2);
            assert_eq!(variants.len(), 2);

            // Ok variant
            assert_eq!(variants[0].name, "Ok");
            assert_eq!(variants[0].fields.len(), 1);
            assert_eq!(variants[0].fields[0].0, "value");

            // Err variant
            assert_eq!(variants[1].name, "Err");
            assert_eq!(variants[1].fields.len(), 1);
            assert_eq!(variants[1].fields[0].0, "error");
        }
        _ => panic!("Result should be an enum"),
    }
}

#[test]
fn type_expr_constructor_converts_properly() {
    use crate::kind::Kind;

    let env = TypeEnv::with_builtin_types();

    // Option<Int> should become Constructor { name: "Option", args: [Int] }
    let type_expr = TypeExpr::Constructor {
        name: "Option".to_string(),
        args: vec![TypeExpr::Named("Int".to_string())],
    };

    let ty = type_expr_to_type(&type_expr, &HashMap::new(), &env).unwrap();

    match ty {
        Type::Constructor { name, args, kind } => {
            assert_eq!(name.display(), "Option");
            assert_eq!(args.len(), 1);
            assert_eq!(args[0], Type::Int);
            assert_eq!(kind, Kind::Type);
        }
        _ => panic!("Expected Type::Constructor, got {:?}", ty),
    }
}

#[test]
fn task689d_act_env_type_expr_is_not_source_denotable() {
    let env = TypeEnv::with_builtin_types();
    let type_expr = TypeExpr::Constructor {
        name: "Fn".to_string(),
        args: vec![
            TypeExpr::Named("ActEnv".to_string()),
            TypeExpr::Tuple(vec![
                TypeExpr::Named("ActEnv".to_string()),
                TypeExpr::Named("Int".to_string()),
            ]),
        ],
    };

    let err = type_expr_to_type(&type_expr, &HashMap::new(), &env)
        .expect_err("ActEnv is runtime-owned and not source-denotable");
    assert!(err.to_string().contains("ActEnv"), "{err}");
}

fn task896_module_identity() -> ModuleIdentity {
    ModuleIdentity::new(
        Some(CrateId(896)),
        ModuleId(118),
        vec!["typeenv".into(), "task896".into()],
        ash_core::semantic_summary::ModuleSourceOrigin::Synthetic {
            reason: "task-896-type-function-promoted-closure".into(),
        },
    )
}

fn task896_source_anchor(label: &str) -> SourceAnchor {
    SourceAnchor::new(
        SourceOrigin::Synthetic {
            reason: "task-896-type-function-promoted-closure".into(),
        },
        None,
        label,
    )
}

fn task896_promoted_nat_summary_for_typeenv(
    visibility: Visibility,
) -> (
    ModuleSemanticSummary,
    PromotedDataKindId,
    PromotedConstructorId,
) {
    let module = task896_module_identity();
    let source_type = TypeDeclId::ordinary(module.clone(), "Nat");
    let source_constructor =
        ConstructorId::variant(source_type.clone(), "Z", ConstructorPayloadKind::Unit);
    let kind = PromotedDataKindId::new(module.clone(), source_type.clone(), "NatKind");
    let constructor = PromotedConstructorId::new(kind.clone(), source_constructor.clone(), "Z");
    let summary = ModuleSemanticSummary::new(module.clone())
        .with_version(SummaryVersion::SPEC065_PROMOTED_DATA_KIND_V6)
        .with_exported_type(TypeDeclSummary::new(
            source_type.clone(),
            "Nat",
            Visibility::Public,
            RepresentationExposure::Exposed,
            TypeRepresentationSummary::Exposed(TypeBody::Enum(vec![VariantDef {
                name: "Z".into(),
                fields: vec![],
                payload: VariantPayload::Unit,
            }])),
            task896_source_anchor("Nat"),
        ))
        .with_exported_constructor(ConstructorSummary::new(
            source_constructor.clone(),
            source_type.clone(),
            "Z",
            ConstructorPayloadKind::Unit,
            Visibility::Public,
            task896_source_anchor("Z"),
        ))
        .with_exported_promoted_data_kind(
            PromotedDataKindSummary::new(
                kind.clone(),
                "NatKind",
                visibility,
                source_type,
                task896_source_anchor("NatKind"),
            )
            .with_constructor(PromotedConstructorSummary::new(
                constructor.clone(),
                "Z",
                source_constructor,
                vec![],
                visibility,
                task896_source_anchor("promoted Z"),
            )),
        );
    (summary, kind, constructor)
}

fn task896_type_function_def_returning_promoted_z(
    module: &ModuleIdentity,
    kind: &PromotedDataKindId,
    constructor: &PromotedConstructorId,
) -> TypeFunctionDef {
    let head = TypeComputationHeadId::new(module.clone(), "ZeroNat");
    TypeFunctionDef {
        visibility: Visibility::Public,
        head: head.clone(),
        name: "ZeroNat".into(),
        params: vec![],
        return_type: CanonicalTypeExpr::Primitive("Type".into()),
        return_kind: Kind::Type,
        result_constraint: TypeFunctionResultConstraint::Kind(Kind::Type),
        decreases: None,
        source_anchors: TypeFunctionSourceAnchors {
            definition: task896_source_anchor("type fn ZeroNat"),
            decreases: None,
        },
        equations: vec![TypeFunctionEquation {
            head,
            ordinal: 0,
            patterns: vec![],
            result: TypeFunctionResultExpr::PromotedDataConstructorApp {
                constructor: Box::new(constructor.clone()),
                data_kind: Box::new(kind.clone()),
                args: vec![],
                kind: Kind::Type,
                constraint: TypeFunctionResultConstraint::Kind(Kind::Type),
                source_anchor: task896_source_anchor("ZeroNat rhs"),
            },
            source_anchor: task896_source_anchor("case ZeroNat = Z"),
            case_head_anchor: task896_source_anchor("ZeroNat case head"),
        }],
    }
}

#[test]
fn task896_public_type_function_summary_records_promoted_data_kind_dependency() {
    let (summary, kind, constructor) = task896_promoted_nat_summary_for_typeenv(Visibility::Public);
    let mut env = TypeEnv::new();
    env.register_module_semantic_summary(&summary)
        .expect("public promoted kind imports");
    let def = task896_type_function_def_returning_promoted_z(&summary.module, &kind, &constructor);

    let exported = env
        .lower_public_type_function_summary(&def)
        .expect("public promoted constructor dependency is export-closed");

    assert!(exported.dependency_summary_refs.iter().any(|dependency| {
        dependency.summary_ref.module == summary.module
            && dependency.summary_ref.version == SummaryVersion::SPEC065_PROMOTED_DATA_KIND_V6
    }));
}

#[test]
fn task896_public_type_function_export_rejects_private_promoted_data_kind_dependency() {
    let (summary, kind, constructor) = task896_promoted_nat_summary_for_typeenv(Visibility::Public);
    let mut env = TypeEnv::new();
    env.register_module_semantic_summary(&summary)
        .expect("public promoted kind imports before privacy mutation");
    env.promoted_data_kind_summaries
        .get_mut(&kind)
        .expect("registered kind")
        .visibility = Visibility::Private;
    let def = task896_type_function_def_returning_promoted_z(&summary.module, &kind, &constructor);

    let err = env
        .lower_public_type_function_summary(&def)
        .expect_err("public type function must not leak private promoted data kind");
    let msg = err.to_string();
    assert!(
        msg.contains("private") && msg.contains("promoted data kind") && msg.contains("NatKind"),
        "unexpected diagnostic: {msg}"
    );
}

#[test]
fn task896_public_type_function_export_rejects_private_promoted_constructor_dependency() {
    let (summary, kind, constructor) = task896_promoted_nat_summary_for_typeenv(Visibility::Public);
    let mut env = TypeEnv::new();
    env.register_module_semantic_summary(&summary)
        .expect("public promoted kind imports before constructor privacy mutation");
    env.promoted_constructor_summaries
        .get_mut(&constructor)
        .expect("registered promoted constructor")
        .visibility = Visibility::Private;
    let def = task896_type_function_def_returning_promoted_z(&summary.module, &kind, &constructor);

    let err = env
        .lower_public_type_function_summary(&def)
        .expect_err("public type function must not leak private promoted constructor");
    let msg = err.to_string();
    assert!(
        msg.contains("private") && msg.contains("promoted data constructor") && msg.contains("Z"),
        "unexpected diagnostic: {msg}"
    );
}

#[test]
fn task896_associated_family_result_conversion_rejects_promoted_constructor_without_panic() {
    let (summary, kind, constructor) = task896_promoted_nat_summary_for_typeenv(Visibility::Public);
    let promoted =
        CanonicalTypeExpr::PromotedDataConstructorApp(Box::new(PromotedConstructorApp {
            constructor,
            data_kind: kind,
            args: vec![],
            kind: Kind::Type,
        }));

    let err = associated_family_result_from_canonical(promoted, Span::new(118, 119, 1, 1))
        .expect_err("promoted constructors are not associated-family result carriers");
    let msg = err.to_string();
    assert!(
        msg.contains("promoted data constructor")
            && msg.contains("associated-family result")
            && msg.contains("Z"),
        "unexpected diagnostic for {:?}: {msg}",
        summary.module
    );
}

#[test]
fn unfold_option_int() {
    let env = TypeEnv::with_builtin_types();

    // Unfold Option<Int>
    let unfolded = env
        .unfold_constructor(&QualifiedName::root("Option"), &[Type::Int])
        .unwrap();

    // Should get: Some { value: Int } | None
    match unfolded {
        UnfoldedBody::Enum(variants) => {
            assert_eq!(variants.len(), 2);

            // Check Some variant
            let some = &variants[0];
            assert_eq!(some.name, "Some");
            assert_eq!(some.fields.len(), 1);
            assert_eq!(some.fields[0].0, "value");
            assert_eq!(some.fields[0].1, Type::Int);

            // Check None variant
            let none = &variants[1];
            assert_eq!(none.name, "None");
            assert!(none.fields.is_empty());
        }
        _ => panic!("Expected enum body, got {:?}", unfolded),
    }
}

#[test]
fn unfold_result_int_string() {
    let env = TypeEnv::with_builtin_types();

    // Unfold Result<Int, String>
    let unfolded = env
        .unfold_constructor(&QualifiedName::root("Result"), &[Type::Int, Type::String])
        .unwrap();

    // Should get: Ok { value: Int } | Err { error: String }
    match unfolded {
        UnfoldedBody::Enum(variants) => {
            assert_eq!(variants.len(), 2);

            // Check Ok variant
            let ok = &variants[0];
            assert_eq!(ok.name, "Ok");
            assert_eq!(ok.fields.len(), 1);
            assert_eq!(ok.fields[0].0, "value");
            assert_eq!(ok.fields[0].1, Type::Int);

            // Check Err variant
            let err = &variants[1];
            assert_eq!(err.name, "Err");
            assert_eq!(err.fields.len(), 1);
            assert_eq!(err.fields[0].0, "error");
            assert_eq!(err.fields[0].1, Type::String);
        }
        _ => panic!("Expected enum body, got {:?}", unfolded),
    }
}

#[test]
fn unfold_constructor_wrong_arity() {
    let env = TypeEnv::with_builtin_types();

    // Option expects 1 type argument, but we provide 2
    let result = env.unfold_constructor(&QualifiedName::root("Option"), &[Type::Int, Type::String]);

    assert!(matches!(
        result,
        Err(TypeError::ConstructorArityMismatch {
            name,
            expected_arity: 1,
            found_arity: 2,
            ..
        }) if name == "Option"
    ));
}
