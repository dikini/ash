use ash_core::ast::{TypeBody, TypeDef, TypeExpr, VariantDef, VariantPayload, Visibility};
use ash_core::module_graph::{CrateId, ModuleId};
use ash_core::semantic_summary::{ModuleIdentity, ModuleSourceOrigin};
use ash_parser::surface::{
    AssociatedTypeBinding, AssociatedTypeDecl, AssociatedTypeKind, ImplDef, InterfaceDef,
    InterfaceTypeParam, Type as SurfaceType, Visibility as SurfaceVisibility,
};
use ash_parser::token::Span;
use ash_typeck::{
    Kind, PatternCanonicalConstructor, PatternCanonicalType, PatternCanonicalization,
    PatternCanonicalizationBlockedReason, QualifiedName, Type, TypeEnv, TypeVar,
};

fn result_type() -> TypeDef {
    TypeDef {
        name: "Result".into(),
        params: vec!["T".into(), "E".into()],
        body: TypeBody::Enum(vec![
            VariantDef {
                name: "Ok".into(),
                fields: vec![("value".into(), TypeExpr::Named("T".into()))],
                payload: VariantPayload::Record(vec![(
                    "value".into(),
                    TypeExpr::Named("T".into()),
                )]),
            },
            VariantDef {
                name: "Err".into(),
                fields: vec![("error".into(), TypeExpr::Named("E".into()))],
                payload: VariantPayload::Record(vec![(
                    "error".into(),
                    TypeExpr::Named("E".into()),
                )]),
            },
        ]),
        visibility: Visibility::Public,
        builtin: false,
    }
}

fn int_result_alias_type() -> TypeDef {
    TypeDef {
        name: "IntResult".into(),
        params: vec!["E".into()],
        body: TypeBody::Alias(TypeExpr::Constructor {
            name: "Result".into(),
            args: vec![TypeExpr::Named("Int".into()), TypeExpr::Named("E".into())],
        }),
        visibility: Visibility::Public,
        builtin: false,
    }
}

fn env_with_result_and_alias() -> TypeEnv {
    let mut env = TypeEnv::new();
    env.register_type(&result_type()).expect("register Result");
    env.register_type(&int_result_alias_type())
        .expect("register transparent IntResult alias");
    env
}

fn task_module() -> ModuleIdentity {
    ModuleIdentity::new(
        Some(CrateId(913)),
        ModuleId(913),
        vec!["task_913".into(), "pattern_canonicalization".into()],
        ModuleSourceOrigin::Synthetic {
            reason: "TASK-913 selected projection fixture".into(),
        },
    )
}

fn span() -> Span {
    Span::default()
}

fn interface_param(name: &str) -> InterfaceTypeParam {
    InterfaceTypeParam {
        name: name.into(),
        domain: None,
        kind: None,
        span: span(),
    }
}

fn surface_name(name: &str) -> SurfaceType {
    SurfaceType::Name(name.into())
}

fn iterator_interface_def() -> InterfaceDef {
    InterfaceDef {
        visibility: SurfaceVisibility::Inherited,
        name: "Iterator".into(),
        type_params: vec![interface_param("Element")],
        associated_types: vec![AssociatedTypeDecl {
            name: "Item".into(),
            kind: AssociatedTypeKind::SealedFamily {
                result_domain: SurfaceType::Name("Type".into()),
                decreases: None,
                span: span(),
            },
            span: span(),
        }],
        methods: vec![],
        span: span(),
    }
}

fn iterator_list_impl(param_name: &str) -> ImplDef {
    ImplDef {
        visibility: SurfaceVisibility::Inherited,
        interface: "Iterator".into(),
        type_params: vec![interface_param(param_name)],
        type_args: vec![surface_name(param_name)],
        where_bounds: vec![],
        associated_type_bindings: vec![AssociatedTypeBinding {
            name: "Item".into(),
            ty: surface_name(param_name),
            span: span(),
        }],
        methods: vec![],
        span: span(),
    }
}

fn env_with_result_alias_and_iterator_family() -> TypeEnv {
    let mut env = TypeEnv::with_builtin_types();
    env.set_current_module_identity(task_module());
    env.register_type(&int_result_alias_type())
        .expect("register transparent IntResult alias");
    env.register_interface(&iterator_interface_def())
        .expect("register Iterator family");
    env.register_impl(&iterator_list_impl("A"))
        .expect("register Iterator<A>::Item family impl");
    env
}

fn result_ty(ok: Type, err: Type) -> Type {
    Type::Constructor {
        name: QualifiedName::root("Result"),
        args: vec![ok, err],
        kind: Kind::Type,
    }
}

fn int_result_ty(err: Type) -> Type {
    Type::Constructor {
        name: QualifiedName::root("IntResult"),
        args: vec![err],
        kind: Kind::Type,
    }
}

fn list_constructor_ty(item: Type) -> Type {
    Type::Constructor {
        name: QualifiedName::root("List"),
        args: vec![item],
        kind: Kind::Type,
    }
}

fn expect_matchable(result: PatternCanonicalization) -> PatternCanonicalType {
    match result {
        PatternCanonicalization::Matchable(canonical) => canonical,
        PatternCanonicalization::Blocked {
            source_type,
            reason,
        } => panic!("expected matchable ADT for {source_type:?}, blocked by {reason:?}"),
    }
}

fn expect_blocked(result: PatternCanonicalization) -> (Type, PatternCanonicalizationBlockedReason) {
    match result {
        PatternCanonicalization::Blocked {
            source_type,
            reason,
        } => (source_type, reason),
        PatternCanonicalization::Matchable(canonical) => {
            panic!("expected blocked pattern canonicalization, got {canonical:?}")
        }
    }
}

fn assert_result_constructor_universe(
    constructors: &[PatternCanonicalConstructor],
    ok_type: Type,
    err_type: Type,
) {
    assert_eq!(constructors.len(), 2);
    assert_eq!(constructors[0].name, "Ok");
    assert_eq!(constructors[0].variant_index, 0);
    assert_eq!(constructors[0].fields, vec![("value".to_string(), ok_type)]);
    assert_eq!(constructors[1].name, "Err");
    assert_eq!(constructors[1].variant_index, 1);
    assert_eq!(
        constructors[1].fields,
        vec![("error".to_string(), err_type)]
    );
}

#[test]
fn direct_adt_canonicalizes_and_exposes_constructor_universe() {
    let env = env_with_result_and_alias();
    let source = result_ty(Type::Int, Type::String);

    let canonical = expect_matchable(env.canonicalize_type_for_pattern(&source));

    assert_eq!(canonical.source_type, source);
    assert_eq!(canonical.canonical_type, result_ty(Type::Int, Type::String));
    assert_eq!(canonical.canonical_name, QualifiedName::root("Result"));
    assert_eq!(canonical.canonical_type_args, vec![Type::Int, Type::String]);
    assert_result_constructor_universe(&canonical.constructors, Type::Int, Type::String);
}

#[test]
fn transparent_alias_to_adt_canonicalizes_to_underlying_constructor_universe() {
    let env = env_with_result_and_alias();
    let source = int_result_ty(Type::Bool);

    let canonical = expect_matchable(env.canonicalize_type_for_pattern(&source));

    assert_eq!(canonical.source_type, source);
    assert_eq!(canonical.canonical_type, result_ty(Type::Int, Type::Bool));
    assert_eq!(canonical.canonical_name, QualifiedName::root("Result"));
    assert_eq!(canonical.canonical_type_args, vec![Type::Int, Type::Bool]);
    assert_result_constructor_universe(&canonical.constructors, Type::Int, Type::Bool);
}

#[test]
fn selected_associated_projection_to_adt_canonicalizes_to_constructor_universe() {
    let env = env_with_result_alias_and_iterator_family();
    let projected_result = result_ty(Type::Int, Type::String);
    let source = Type::Associated {
        interface: "Iterator".into(),
        base: Box::new(list_constructor_ty(projected_result.clone())),
        name: "Item".into(),
    };

    let canonical = expect_matchable(env.canonicalize_type_for_pattern(&source));

    assert_eq!(canonical.source_type, source);
    assert_eq!(canonical.canonical_type, projected_result);
    assert_eq!(canonical.canonical_name, QualifiedName::root("Result"));
    assert_eq!(canonical.canonical_type_args, vec![Type::Int, Type::String]);
    assert_result_constructor_universe(&canonical.constructors, Type::Int, Type::String);
}

#[test]
fn unresolved_associated_projection_returns_typed_blocked_result() {
    let env = env_with_result_and_alias();
    let source = Type::Associated {
        interface: "Iterable".into(),
        base: Box::new(Type::Var(TypeVar(913))),
        name: "Item".into(),
    };

    match env.canonicalize_type_for_pattern(&source) {
        PatternCanonicalization::Blocked {
            source_type,
            reason:
                PatternCanonicalizationBlockedReason::RigidAssociatedProjection { interface, member },
        } => {
            assert_eq!(source_type, source);
            assert_eq!(interface, "Iterable");
            assert_eq!(member, "Item");
        }
        other => panic!("expected rigid associated projection to block, got {other:?}"),
    }
}

#[test]
fn primitive_type_returns_typed_non_matchable_result() {
    let env = env_with_result_and_alias();

    let (source_type, reason) = expect_blocked(env.canonicalize_type_for_pattern(&Type::Int));

    assert_eq!(source_type, Type::Int);
    assert_eq!(reason, PatternCanonicalizationBlockedReason::NonAdt);
}

#[test]
fn top_level_type_variable_returns_typed_blocked_result() {
    let env = env_with_result_and_alias();
    let source = Type::Var(TypeVar(9130));

    let (source_type, reason) = expect_blocked(env.canonicalize_type_for_pattern(&source));

    assert_eq!(source_type, source);
    assert_eq!(reason, PatternCanonicalizationBlockedReason::TypeVariable);
}

#[test]
fn nested_type_variable_inside_adt_args_returns_non_concrete_blocked_result() {
    let env = env_with_result_and_alias();
    let source = result_ty(Type::Var(TypeVar(9131)), Type::String);

    let (source_type, reason) = expect_blocked(env.canonicalize_type_for_pattern(&source));

    assert_eq!(source_type, source);
    assert_eq!(
        reason,
        PatternCanonicalizationBlockedReason::NonConcreteTypeArgument
    );
}

#[test]
fn nested_type_variable_inside_transparent_alias_args_returns_non_concrete_blocked_result() {
    let env = env_with_result_and_alias();
    let source = int_result_ty(Type::Var(TypeVar(9132)));

    let (source_type, reason) = expect_blocked(env.canonicalize_type_for_pattern(&source));

    assert_eq!(source_type, source);
    assert_eq!(
        reason,
        PatternCanonicalizationBlockedReason::NonConcreteTypeArgument
    );
}

#[test]
fn unknown_nominal_constructor_returns_typed_blocked_result() {
    let env = env_with_result_and_alias();
    let source = Type::Constructor {
        name: QualifiedName::root("Missing"),
        args: vec![],
        kind: Kind::Type,
    };

    let (source_type, reason) = expect_blocked(env.canonicalize_type_for_pattern(&source));

    assert_eq!(source_type, source);
    assert_eq!(
        reason,
        PatternCanonicalizationBlockedReason::UnknownType {
            name: QualifiedName::root("Missing")
        }
    );
}

#[test]
fn constructor_variable_application_returns_typed_blocked_result() {
    let env = env_with_result_and_alias();
    let source = Type::ConstructorVariableApp {
        constructor: "M".into(),
        args: vec![Type::Int],
        kind: Kind::Type,
    };

    let (source_type, reason) = expect_blocked(env.canonicalize_type_for_pattern(&source));

    assert_eq!(source_type, source);
    assert_eq!(
        reason,
        PatternCanonicalizationBlockedReason::ConstructorVariableApplication {
            constructor: "M".into()
        }
    );
}
