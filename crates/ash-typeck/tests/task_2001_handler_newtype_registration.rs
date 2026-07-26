//! TASK-2001: module-registration contracts for handler and newtype syntax.
//!
//! Parsing the declaration carriers is insufficient: module registration must
//! retain the handler distinction and a newtype's nominal identity.

use ash_core::ast::{TypeBody, TypeExpr, Visibility};
use ash_core::module_graph::{CrateId, ModuleId};
use ash_core::semantic_summary::{
    ConstructorId, ConstructorPayloadKind, ConstructorSummary, ModuleIdentity,
    ModuleSemanticSummary, ModuleSourceOrigin, RepresentationExposure, SourceAnchor, SourceOrigin,
    TypeDeclId, TypeDeclSummary, TypeDeclarationKind, TypeRepresentationSummary,
};
use ash_parser::parse_surface_file;
use ash_parser::surface::{ComputationRowItem, Definition, Program, ProgramEntry, Type};
use ash_typeck::types::Type as CheckedType;
use ash_typeck::{CallableDeclarationKind, TypeEnv, type_check_program};

fn registered_env(source: &str) -> TypeEnv {
    let module = parse_surface_file(source)
        .unwrap_or_else(|errors| panic!("TASK-2001 source should parse: {errors:?}"));
    let mut env = TypeEnv::with_builtin_types();
    env.register_surface_module_declarations(&module)
        .expect("parsed module declarations should register");
    env
}

#[test]
fn parsed_handler_registers_as_handler_callable_not_ordinary_function() {
    let env = registered_env(
        r#"
        handler canonical_handler(comp: Unit) -> Unit { comp }
        fn ordinary(comp: Unit) -> Unit { comp }
        "#,
    );

    assert_eq!(
        env.callable_declaration_kind("canonical_handler"),
        Some(CallableDeclarationKind::Handler)
    );
    env.require_handler_callable("canonical_handler")
        .expect("a parsed handler must satisfy a handler-only admission boundary");
    assert_eq!(
        env.callable_declaration_kind("ordinary"),
        Some(CallableDeclarationKind::Function)
    );
    assert!(
        env.require_handler_callable("ordinary").is_err(),
        "an ordinary function must not satisfy a handler-only admission boundary"
    );
}

#[test]
fn derived_handler_registers_as_handler_callable_in_value_namespace() {
    let env = registered_env(
        r#"
        interface Clock<T> { sleep(Int) -> Null }
        type TestClock = SystemClock(Int);
        impl Clock<TestClock> {
            sleep(milliseconds) = null
            derive handler clock;
        }
        "#,
    );

    assert_eq!(
        env.callable_declaration_kind("clock"),
        Some(CallableDeclarationKind::Handler),
        "a derived handler must use the same value-namespace marker as a parsed handler"
    );
    assert!(
        env.lookup_variable("clock").is_none(),
        "derive handler registration must not fabricate a callable type binding or signature"
    );
    env.require_handler_callable("clock")
        .expect("a derived handler must satisfy the normal handler-only admission query");
}

#[test]
fn canonical_handler_row_binder_is_preserved_as_a_distinct_row_kind() {
    let module = parse_surface_file(
        r#"
        handler h<A, r: Row>(comp: Unit -> {ClockFs::sleep | r} A) -> {r} A {
            on comp {
                ClockFs::sleep(value, resume) => value,
                done(value) => value
            }
        }
        "#,
    )
    .expect("SPEC-095b canonical handler signatures must parse before row-kind checking");

    let Definition::Handler(handler) = &module.definitions[0] else {
        panic!("canonical row-polymorphic declaration must remain a handler");
    };
    assert_eq!(handler.type_params.len(), 2);
    assert_eq!(handler.type_params[1].name.as_ref(), "r");
    assert_eq!(
        handler.type_params[1]
            .kind
            .as_ref()
            .expect("r: Row must not collapse to an ordinary type parameter")
            .kind
            .to_string(),
        "Row"
    );
    assert!(matches!(
        &handler.params[0].ty,
        Type::Fn(_, Some(row), result)
            if matches!(result.as_ref(), Type::Name(name) if name.as_ref() == "A")
                && matches!(
                    row.items.as_slice(),
                    [
                        ComputationRowItem::Operation { path, .. },
                        ComputationRowItem::Tail { variable, .. },
                    ]
                        if path.iter().map(AsRef::as_ref).eq(["ClockFs", "sleep"])
                            && variable.as_ref() == "r"
                )
    ));
    assert!(matches!(
        &handler.return_type,
        Type::Fn(params, Some(row), result)
            if params.is_empty()
                && matches!(result.as_ref(), Type::Name(name) if name.as_ref() == "A")
                && matches!(
                    row.items.as_slice(),
                    [ComputationRowItem::WholeRow { variable, .. }]
                        if variable.as_ref() == "r"
                )
    ));
}

#[test]
fn row_kinded_handler_signature_registers_when_r_appears_only_in_rows() {
    let module = parse_surface_file(
        r#"
        interface Clock<T> { sleep(Int) -> Null }
        type TestClock = SystemClock(Int);
        impl Clock<TestClock> { sleep(milliseconds) = null }
        handler h<A, r: Row>(comp: Unit -> {TestClock::sleep | r} A) -> {r} A {
            on comp {
                TestClock::sleep(value, resume) => value,
                done(value) => value
            }
        }
        "#,
    )
    .expect("a canonical Row binder must parse before callable-row registration");

    // This is the public declaration/type registration boundary. The current
    // narrow handler-body checker cannot yet type an answer of `{r} A`: that
    // surface return is represented as a zero-parameter computation carrier,
    // while the handler body checker still compares `done` directly to the
    // carrier rather than its answer. Registration must nevertheless admit the
    // signature and retain the handler marker.
    let mut env = TypeEnv::with_builtin_types();
    env.register_surface_module_declarations(&module)
        .expect("a Row binder used only by computation rows must register");
    assert_eq!(
        env.callable_declaration_kind("h"),
        Some(CallableDeclarationKind::Handler)
    );
}

#[test]
fn row_kinded_binder_used_as_a_proper_type_is_rejected_deterministically() {
    let module = parse_surface_file("handler bad<r: Row>(x: r) -> Unit { x }")
        .expect("the parser must preserve the Row binder so type checking can reject its misuse");

    let mut env = TypeEnv::with_builtin_types();
    let error = env
        .register_surface_module_declarations(&module)
        .expect_err("a Row binder cannot be used as a proper parameter type");
    assert!(
        error
            .to_string()
            .contains("row-kinded parameter 'r' cannot be used as a proper type"),
        "row-kind misuse diagnostics must be deterministic: {error}"
    );
}

#[test]
fn row_kinded_binder_cannot_publish_function_or_builtin_callable_markers() {
    for (name, source) in [
        ("bad_fn", "fn bad_fn<r: Row>(x: r) -> Unit { x }"),
        (
            "bad_builtin",
            "builtin fn bad_builtin<r: Row>(x: r) -> Unit;",
        ),
    ] {
        let module = parse_surface_file(source)
            .unwrap_or_else(|errors| panic!("{name} fixture should parse: {errors:?}"));
        let mut env = TypeEnv::with_builtin_types();
        let error = env
            .register_surface_module_declarations(&module)
            .expect_err("a Row binder cannot be used as a proper callable parameter type");
        assert!(
            error
                .to_string()
                .contains("row-kinded parameter 'r' cannot be used as a proper type"),
            "{name} must report the stable row-kind diagnostic: {error}"
        );
        assert_eq!(
            env.callable_declaration_kind(name),
            None,
            "failed registration of {name} must roll back its callable declaration marker"
        );
    }
}

#[test]
fn derive_handler_materializes_only_impl_operations_as_a_source_handler_fact() {
    let module = parse_surface_file(
        r#"
        interface Clock<T> { sleep(Int) -> Null }
        type TestClock = SystemClock(Int);
        impl Clock<TestClock> {
            sleep(milliseconds) = null
            handler explicit_clock(comp: () -> {TestClock::sleep} Null) -> Null {
                on comp {
                    TestClock::sleep(milliseconds, resume) => null,
                    done(value) => value
                }
            }
            derive handler clock;
        }
        fn main() -> Null { null }
        "#,
    )
    .expect("the concrete derive-handler fixture must parse");
    let entry = module
        .definitions
        .iter()
        .find_map(|definition| match definition {
            Definition::Function(function) if function.name.as_ref() == "main" => {
                Some(ProgramEntry {
                    function: function.name.clone(),
                    span: function.span,
                })
            }
            _ => None,
        })
        .expect("fixture must contain main");
    let program = Program {
        definitions: module.definitions,
        entry,
    };

    let checked = type_check_program(&program)
        .expect("derivation is source-only and must not require runtime execution");
    let derived = checked
        .checked_handlers
        .get("clock")
        .expect("derive handler clock must materialize a handler-marked value-namespace fact");
    assert_eq!(derived.callable_kind, CallableDeclarationKind::Handler);
    assert_eq!(derived.clauses.len(), 1);
    assert_eq!(derived.clauses[0].operation.impl_type, "TestClock");
    assert_eq!(derived.clauses[0].operation.interface, "Clock");
    assert_eq!(derived.clauses[0].operation.operation, "sleep");
    assert!(
        !checked.checked_handlers.contains_key("explicit_clock"),
        "derive must select impl operations, never a co-located explicit handler"
    );
}

#[test]
fn derive_handler_materializes_the_exact_union_of_two_impl_operations() {
    let module = parse_surface_file(
        r#"
        interface Clock<T> {
            sleep(Int) -> Null,
            wake(Int) -> Null
        }
        type TestClock = SystemClock(Int);
        impl Clock<TestClock> {
            sleep(milliseconds) = null
            wake(milliseconds) = null
            handler explicit_clock(comp: () -> {TestClock::sleep} Null) -> Null {
                on comp {
                    TestClock::sleep(milliseconds, resume) => null,
                    done(value) => value
                }
            }
            derive handler clock;
        }
        fn main() -> Null { null }
        "#,
    )
    .expect("the two-operation derive fixture must parse");
    let entry = module
        .definitions
        .iter()
        .find_map(|definition| match definition {
            Definition::Function(function) if function.name.as_ref() == "main" => {
                Some(ProgramEntry {
                    function: function.name.clone(),
                    span: function.span,
                })
            }
            _ => None,
        })
        .expect("fixture must contain main");
    let program = Program {
        definitions: module.definitions,
        entry,
    };

    let checked = type_check_program(&program)
        .expect("derivation is a source-only fact and must not require runtime execution");
    let derived = checked
        .checked_handlers
        .get("clock")
        .expect("derive handler clock must materialize a checked source fact");
    assert_eq!(derived.callable_kind, CallableDeclarationKind::Handler);
    assert_eq!(derived.clauses.len(), 2);
    assert_eq!(
        derived
            .clauses
            .iter()
            .map(|clause| clause.operation.operation.as_str())
            .collect::<Vec<_>>(),
        vec!["sleep", "wake"],
        "derive must select exactly the declared impl operations in declaration order"
    );
    assert_eq!(
        derived
            .input_row
            .items
            .iter()
            .map(|item| item.canonical_key())
            .collect::<Vec<_>>(),
        vec![
            "operation:TestClock::Clock::sleep".to_string(),
            "operation:TestClock::Clock::wake".to_string(),
        ]
    );
    assert_eq!(derived.input_row.tail.as_deref(), Some("r"));
    assert!(derived.residual_row.items.is_empty());
    assert_eq!(derived.residual_row.tail.as_deref(), Some("r"));
    assert_eq!(derived.output_row, derived.residual_row);
    assert!(
        !checked.checked_handlers.contains_key("explicit_clock"),
        "a derived handler must not select or materialize a co-located explicit handler"
    );
}

#[test]
fn derived_handler_is_polymorphic_over_its_answer_and_residual_row() {
    let module = parse_surface_file(
        r#"
        interface Clock<T> {
            sleep(Int) -> Int,
            wake(String) -> Bool
        }
        type TestClock = SystemClock(Int);
        impl Clock<TestClock> {
            sleep(milliseconds) = milliseconds
            wake(label) = true
            derive handler clock;
        }
        fn main() -> Null { null }
        "#,
    )
    .expect("the polymorphic derive-handler fixture must parse");
    let entry = module
        .definitions
        .iter()
        .find_map(|definition| match definition {
            Definition::Function(function) if function.name.as_ref() == "main" => {
                Some(ProgramEntry {
                    function: function.name.clone(),
                    span: function.span,
                })
            }
            _ => None,
        })
        .expect("fixture must contain main");
    let program = Program {
        definitions: module.definitions,
        entry,
    };

    let checked = type_check_program(&program)
        .expect("derive polymorphism is a source-only type fact, not runtime behavior");
    let derived = checked
        .checked_handlers
        .get("clock")
        .expect("derive handler clock must materialize a checked source fact");

    // `derive handler` is the total identity fold: its answer is the input
    // computation's independently quantified answer, never one operation's
    // result type.  The existing checked-declaration fact is the narrow test
    // seam: no Core handler or runtime frame is required to observe this.
    assert!(matches!(derived.answer_type, CheckedType::Var(_)));
    assert_eq!(derived.input_result_type, derived.answer_type);
    assert_ne!(derived.answer_type, CheckedType::Int);
    assert_ne!(derived.answer_type, CheckedType::Bool);

    // The same derived handler must quantify an open residual row.  Peeling
    // every declared operation preserves that row both for its result and for
    // every generated continuation.
    assert!(derived.residual_row.items.is_empty());
    assert_eq!(derived.residual_row.tail.as_deref(), Some("r"));
    assert_eq!(derived.output_row, derived.residual_row);
    assert!(derived.clauses.iter().all(|clause| {
        clause.continuation_row == derived.residual_row
            && clause.continuation_multiplicity == ash_typeck::ContinuationMultiplicity::Affine
    }));

    assert_eq!(
        derived
            .clauses
            .iter()
            .map(|clause| (
                clause.operation.impl_type.as_str(),
                clause.operation.interface.as_str(),
                clause.operation.operation.as_str(),
            ))
            .collect::<Vec<_>>(),
        vec![
            ("TestClock", "Clock", "sleep"),
            ("TestClock", "Clock", "wake"),
        ],
        "the polymorphic fact must still preserve every exact impl operation identity"
    );
}

#[test]
fn derived_handler_clauses_keep_desugaring_origins_at_the_derive_site() {
    let module = parse_surface_file(
        r#"
        interface Clock<T> {
            sleep(Int) -> Null,
            wake(Int) -> Null
        }
        type TestClock = SystemClock(Int);
        impl Clock<TestClock> {
            sleep(milliseconds) = null
            wake(milliseconds) = null
            derive handler clock;
        }
        fn main() -> Null { null }
        "#,
    )
    .expect("the provenance fixture must parse");
    let derive_span = module
        .definitions
        .iter()
        .find_map(|definition| match definition {
            Definition::Impl(implementation) => implementation
                .derived_handlers
                .iter()
                .find(|derived| derived.name.as_ref() == "clock")
                .map(|derived| derived.span),
            _ => None,
        })
        .expect("fixture must retain the derive handler source span");
    let entry = module
        .definitions
        .iter()
        .find_map(|definition| match definition {
            Definition::Function(function) if function.name.as_ref() == "main" => {
                Some(ProgramEntry {
                    function: function.name.clone(),
                    span: function.span,
                })
            }
            _ => None,
        })
        .expect("fixture must contain main");
    let program = Program {
        definitions: module.definitions,
        entry,
    };

    let checked = type_check_program(&program)
        .expect("derivation provenance is a source-only fact, not runtime behavior");
    let derived = checked
        .checked_handlers
        .get("clock")
        .expect("derive handler clock must materialize a checked source fact");
    assert!(
        derived.clauses.iter().all(|clause| matches!(
            &clause.origin,
            ash_parser::surface::SurfaceOrigin::Desugaring { source_span, rule }
                if *source_span == derive_span && rule.as_ref() == "derive handler"
        )),
        "every synthesized clause must retain desugaring provenance at its derive handler site"
    );
}

#[test]
fn parsed_newtype_registers_a_nominal_identity_and_constructor_not_an_alias() {
    let env = registered_env("newtype OrderId = OrderId(Int);");

    let order_id = env
        .nominal_newtype("OrderId")
        .expect("parsed newtype must retain its nominal registration");
    assert_eq!(order_id.type_name(), "OrderId");
    assert_eq!(order_id.constructor(), "OrderId");
    assert_eq!(order_id.representation_name(), "Int");
    assert_ne!(
        order_id.identity(),
        env.nominal_type_identity("Int")
            .expect("built-in representation type has an identity"),
        "newtype identity must not collapse to its representation"
    );
    assert!(
        !env.is_transparent_alias("OrderId"),
        "a newtype must not be registered as a transparent alias"
    );
    assert_eq!(
        env.lookup_constructor("OrderId"),
        Some(("OrderId".to_string(), 0))
    );
}

#[test]
fn ordinary_alias_summary_cannot_forge_nominality_from_constructor_shape() {
    let module = ModuleIdentity::new(
        Some(CrateId(2001)),
        ModuleId(2001),
        vec!["task_2001".to_string(), "forged_alias".to_string()],
        ModuleSourceOrigin::Synthetic {
            reason: "TASK-2001 regression".to_string(),
        },
    );
    let type_id = TypeDeclId::ordinary(module.clone(), "ForgedAlias");
    let anchor = SourceAnchor::new(
        SourceOrigin::Synthetic {
            reason: "TASK-2001 regression".to_string(),
        },
        None,
        "forged ordinary alias",
    );
    let ordinary_alias = TypeDeclSummary::new(
        type_id.clone(),
        "ForgedAlias",
        Visibility::Public,
        RepresentationExposure::Exposed,
        TypeRepresentationSummary::exposed(TypeBody::Alias(TypeExpr::Named("Int".into()))),
        anchor.clone(),
    )
    .with_declaration_kind(TypeDeclarationKind::Ordinary);
    let forged_constructor = ConstructorSummary::new(
        ConstructorId::variant(
            type_id.clone(),
            "ForgedAlias",
            ConstructorPayloadKind::Tuple,
        ),
        type_id,
        "ForgedAlias",
        ConstructorPayloadKind::Tuple,
        Visibility::Public,
        anchor,
    );
    let summary = ModuleSemanticSummary::new(module)
        .with_exported_type(ordinary_alias)
        .with_exported_constructor(forged_constructor);

    let mut env = TypeEnv::with_builtin_types();
    let error = env.register_module_semantic_summary(&summary).expect_err(
        "only an explicit nominal-newtype declaration kind may admit alias-backed constructors",
    );
    assert!(
        error
            .to_string()
            .contains("references a parent without an exposed enum body"),
        "an ordinary alias-shaped summary must retain ordinary-constructor validation: {error}"
    );
    assert!(
        env.nominal_newtype("ForgedAlias").is_none(),
        "failed ordinary-alias import must not install nominal metadata"
    );
}

#[test]
fn imported_nominal_newtype_retains_the_provider_type_declaration_identity() {
    let provider = ModuleIdentity::new(
        Some(CrateId(2001)),
        ModuleId(2002),
        vec!["provider".to_string(), "orders".to_string()],
        ModuleSourceOrigin::Synthetic {
            reason: "TASK-2001 provider identity regression".to_string(),
        },
    );
    let provider_id = TypeDeclId::ordinary(provider.clone(), "OrderId");
    let anchor = SourceAnchor::new(
        SourceOrigin::Synthetic {
            reason: "TASK-2001 provider identity regression".to_string(),
        },
        None,
        "provider OrderId",
    );
    let newtype = TypeDeclSummary::new(
        provider_id.clone(),
        "OrderId",
        Visibility::Public,
        RepresentationExposure::Exposed,
        TypeRepresentationSummary::exposed(TypeBody::Alias(TypeExpr::Named("Int".into()))),
        anchor.clone(),
    )
    .with_declaration_kind(TypeDeclarationKind::NominalNewtype);
    let constructor = ConstructorSummary::new(
        ConstructorId::variant(
            provider_id.clone(),
            "OrderId",
            ConstructorPayloadKind::Tuple,
        ),
        provider_id.clone(),
        "OrderId",
        ConstructorPayloadKind::Tuple,
        Visibility::Public,
        anchor,
    );
    let summary = ModuleSemanticSummary::new(provider)
        .with_exported_type(newtype)
        .with_exported_constructor(constructor);

    let mut env = TypeEnv::with_builtin_types();
    env.register_module_semantic_summary(&summary)
        .expect("a marked provider newtype summary should register");
    assert_eq!(
        env.nominal_newtype("OrderId")
            .expect("imported nominal metadata")
            .identity(),
        provider_id,
        "import registration must preserve the provider TypeDeclId rather than synthesize a local fallback"
    );
}
