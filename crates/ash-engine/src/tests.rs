//! Engine public shell tests.

use super::*;
use ash_core::semantic_summary::TypeDeclId;
use proptest::prelude::*;

// ============================================================
// Engine Creation Tests
// ============================================================

#[test]
fn test_engine_new_build_succeeds() {
    // Basic test: Engine::new().build() should succeed
    let result = Engine::new().build();
    assert!(
        result.is_ok(),
        "Engine::new().build() should succeed but got: {result:?}"
    );
}

#[test]
fn test_engine_default_succeeds() {
    // Engine::default() should succeed using the builder
    let _engine: Engine = Engine::default();
}

#[test]
fn test_engine_builder_returns_valid_engine() {
    let engine = Engine::new().build().expect("engine builds");
    // The engine should be usable (not panic, not be null, etc.)
    // We verify this by checking it can execute basic operations
    let _ = &engine;
}

#[test]
fn task_2069_canonical_module_parse_failure_does_not_select_legacy_route() {
    let temporary = tempfile::tempdir().expect("temporary source directory exists");
    let path = temporary.path().join("broken_module_root.ash");
    let source = "pub mod api { fn serve() -> Int { 1 } fn main() -> Int { 1 }";
    std::fs::write(&path, source).expect("module-shaped source is written");

    let engine = Engine::new().build().expect("engine builds");
    let error = engine
        .canonical_module_closure_from_source(&path, source, "main")
        .expect_err("a malformed module root must fail closed instead of returning legacy None");

    assert!(
        error
            .to_string()
            .contains("canonical module source parse failed"),
        "module parse failure should identify the canonical route boundary: {error}"
    );
}

#[test]
fn task_2069_parseable_callable_root_lowering_failure_does_not_select_legacy_route() {
    let temporary = tempfile::tempdir().expect("temporary source directory exists");
    let path = temporary.path().join("unsupported.ash");
    let source = "fn main() -> Int { match 1 { 1 => 1, _ => 0 } }";
    std::fs::write(&path, source).expect("parseable unsupported source is written");

    let engine = Engine::new().build().expect("engine builds");
    let error = engine
        .canonical_module_closure_from_source(&path, source, "main")
        .expect_err("a parseable but unsupported callable root must fail closed");

    assert!(
        error.to_string().contains("canonical") || error.to_string().contains("unsupported"),
        "the failure must identify the canonical route boundary: {error}"
    );
}

#[test]
fn task_2069_parseable_callable_import_failure_does_not_select_legacy_route() {
    let temporary = tempfile::tempdir().expect("temporary source directory exists");
    let path = temporary.path().join("missing_import.ash");
    let source = "use crate::missing::serve; fn main() -> Int { 42 }";
    std::fs::write(&path, source).expect("parseable invalid-import source is written");

    let engine = Engine::new().build().expect("engine builds");
    let error = engine
        .canonical_module_closure_from_source(&path, source, "main")
        .expect_err("a parseable callable import failure must fail closed");

    assert!(
        error.to_string().contains("canonical") || error.to_string().contains("import"),
        "the failure must identify the canonical import boundary: {error}"
    );
}

#[test]
fn task_2069_legacy_runtime_prelude_remains_ordinary_compatibility_route() {
    let temporary = tempfile::tempdir().expect("temporary source directory exists");
    let path = temporary.path().join("runtime_entry.ash");
    let source = "use result::Result; use runtime::RuntimeError; fn main() -> Result<(), RuntimeError> { Ok { value: {} } }";
    std::fs::write(&path, source).expect("legacy runtime-prelude source is written");

    let engine = Engine::new().build().expect("engine builds");
    assert_eq!(
        engine
            .canonical_module_closure_from_source(&path, source, "main")
            .expect("legacy runtime-prelude classification should not be an error"),
        None,
        "the ordinary compatibility loader must retain ownership of the legacy runtime prelude"
    );
}

#[test]
fn task_2069_ordinary_root_without_entry_remains_compatibility_diagnostic_route() {
    let temporary = tempfile::tempdir().expect("temporary source directory exists");
    let path = temporary.path().join("missing_entry.ash");
    let source = "fn helper() -> Int { 1 }";
    std::fs::write(&path, source).expect("ordinary source without entry is written");

    let engine = Engine::new().build().expect("engine builds");
    assert_eq!(
        engine
            .canonical_module_closure_from_source(&path, source, "main")
            .expect("missing-entry compatibility classification should not be an error"),
        None,
        "the ordinary loader must retain its established missing-entry diagnostic"
    );
}

#[test]
fn task_2069_canonical_source_route_lowers_and_executes_file_and_inline_children() {
    let temporary = tempfile::tempdir().expect("temporary source directory exists");
    let file_path = temporary.path().join("main.ash");
    let file_source = "pub mod api; use crate::api::serve as remote; fn main() -> Int { remote() }";
    std::fs::write(&file_path, file_source).expect("file-backed module root is written");
    std::fs::write(
        temporary.path().join("api.ash"),
        "pub fn serve() -> Int { 2 }",
    )
    .expect("file-backed structural child is written");

    let engine = Engine::new().build().expect("engine builds");
    for (path, source) in [
        (file_path, file_source.to_owned()),
        (
            temporary.path().join("inline.ash"),
            "pub mod api { pub fn serve() -> Int { 2 } } use crate::api::serve as remote; fn main() -> Int { remote() }".to_owned(),
        ),
    ] {
        std::fs::write(&path, &source).expect("source-form module root is written");
        let closure = engine
            .canonical_module_closure_from_source(&path, &source, "main")
            .expect("canonical source route lowers without a legacy fallback")
            .expect("structural module source produces a checked closure");
        assert_eq!(closure.modules().len(), 2);

        let admitted = engine
            .admit_linked_module_closure(closure)
            .expect("canonical source closure admits through Engine");
        let (request, _cancellation) = engine
            .new_admitted_program_request(&admitted, None)
            .expect("Engine issues a request for the checked route");
        let terminal = futures::executor::block_on(engine.execute_admitted_program(&request))
            .expect("checked CPS route executes the canonical source closure");
        assert_eq!(terminal, CanonicalTerminalEnvelopeV1::returned(Value::Int(2)));
    }
}

#[test]
fn task_2069_canonical_source_route_lowers_and_executes_an_ordinary_root() {
    let temporary = tempfile::tempdir().expect("temporary source directory exists");
    let path = temporary.path().join("ordinary.ash");
    let source = "fn main() -> Int { 2 }";
    std::fs::write(&path, source).expect("ordinary source is written");

    let engine = Engine::new().build().expect("engine builds");
    let closure = engine
        .canonical_module_closure_from_source(&path, source, "main")
        .expect("ordinary source should use the canonical route")
        .expect("ordinary root produces a checked closure");
    let admitted = engine
        .admit_linked_module_closure(closure)
        .expect("ordinary canonical closure admits through Engine");
    let (request, _cancellation) = engine
        .new_admitted_program_request(&admitted, None)
        .expect("Engine issues a request for the canonical route");
    let terminal = futures::executor::block_on(engine.execute_admitted_program(&request))
        .expect("checked CPS route executes the ordinary canonical closure");

    assert_eq!(
        terminal,
        CanonicalTerminalEnvelopeV1::returned(Value::Int(2))
    );
}

#[test]
fn task_2069_canonical_source_route_lowers_and_executes_modulo() {
    let temporary = tempfile::tempdir().expect("temporary source directory exists");
    let path = temporary.path().join("ordinary.ash");
    let source = "fn main() -> Int { 7 % 3 }";
    std::fs::write(&path, source).expect("ordinary source is written");

    let engine = Engine::new().build().expect("engine builds");
    let closure = engine
        .canonical_module_closure_from_source(&path, source, "main")
        .expect("ordinary modulo source should use the canonical route")
        .expect("ordinary modulo root produces a checked closure");
    let admitted = engine
        .admit_linked_module_closure(closure)
        .expect("ordinary modulo closure admits through Engine");
    let (request, _cancellation) = engine
        .new_admitted_program_request(&admitted, None)
        .expect("Engine issues a request for the canonical modulo route");
    let terminal = futures::executor::block_on(engine.execute_admitted_program(&request))
        .expect("checked CPS route executes modulo through the canonical closure");

    assert_eq!(
        terminal,
        CanonicalTerminalEnvelopeV1::returned(Value::Int(1))
    );
}

#[test]
fn task_2069_canonical_source_route_lowers_and_executes_record_field_call() {
    let temporary = tempfile::tempdir().expect("temporary source directory exists");
    let path = temporary.path().join("ordinary.ash");
    let source =
        "fn helper() -> Int { 41 } fn main() -> Int { let person = { age: helper() }; person.age }";
    std::fs::write(&path, source).expect("ordinary source is written");

    let engine = Engine::new().build().expect("engine builds");
    let closure = engine
        .canonical_module_closure_from_source(&path, source, "main")
        .expect("record field call source should use the canonical route")
        .expect("record field call root produces a checked closure");
    let admitted = engine
        .admit_linked_module_closure(closure)
        .expect("record field call closure admits through Engine");
    let (request, _cancellation) = engine
        .new_admitted_program_request(&admitted, None)
        .expect("Engine issues a request for the record field call route");
    let terminal = futures::executor::block_on(engine.execute_admitted_program(&request))
        .expect("checked CPS route executes the record field call");

    assert_eq!(
        terminal,
        CanonicalTerminalEnvelopeV1::returned(Value::Int(41))
    );
}

#[test]
fn task_2069_canonical_source_route_lowers_and_executes_nested_record_field_call() {
    let temporary = tempfile::tempdir().expect("temporary source directory exists");
    let path = temporary.path().join("ordinary.ash");
    let source = "fn helper() -> Int { 41 } fn main() -> Int { let person = { inner: { age: helper() } }; person.inner.age }";
    std::fs::write(&path, source).expect("ordinary source is written");

    let engine = Engine::new().build().expect("engine builds");
    let closure = engine
        .canonical_module_closure_from_source(&path, source, "main")
        .expect("nested record field call source should use the canonical route")
        .expect("nested record field call root produces a checked closure");
    let admitted = engine
        .admit_linked_module_closure(closure)
        .expect("nested record field call closure admits through Engine");
    let (request, _cancellation) = engine
        .new_admitted_program_request(&admitted, None)
        .expect("Engine issues a request for the nested record field call route");
    let terminal = futures::executor::block_on(engine.execute_admitted_program(&request))
        .expect("checked CPS route executes the nested record field call");

    assert_eq!(
        terminal,
        CanonicalTerminalEnvelopeV1::returned(Value::Int(41))
    );
}

#[test]
fn task_2069_canonical_source_route_lowers_and_executes_record_field_expression_call() {
    let temporary = tempfile::tempdir().expect("temporary source directory exists");
    let path = temporary.path().join("ordinary.ash");
    let source = "fn helper() -> Int { 40 } fn main() -> Int { let person = { age: helper() + 1 }; person.age }";
    std::fs::write(&path, source).expect("ordinary source is written");

    let engine = Engine::new().build().expect("engine builds");
    let closure = engine
        .canonical_module_closure_from_source(&path, source, "main")
        .expect("record field expression source should use the canonical route")
        .expect("record field expression root produces a checked closure");
    let admitted = engine
        .admit_linked_module_closure(closure)
        .expect("record field expression closure admits through Engine");
    let (request, _cancellation) = engine
        .new_admitted_program_request(&admitted, None)
        .expect("Engine issues a request for the record field expression route");
    let terminal = futures::executor::block_on(engine.execute_admitted_program(&request))
        .expect("checked CPS route executes the record field expression");

    assert_eq!(
        terminal,
        CanonicalTerminalEnvelopeV1::returned(Value::Int(41))
    );
}

#[test]
fn task_2069_canonical_source_route_is_declaration_order_independent() {
    let temporary = tempfile::tempdir().expect("temporary source directory exists");
    let path = temporary.path().join("order.ash");
    let sources = [
        "crate app; pub mod api { pub fn helper() -> Int { 40 } } use crate::api::helper as remote; fn main() -> Int { remote() + 1 }",
        "crate app; fn main() -> Int { remote() + 1 } use crate::api::helper as remote; pub mod api { pub fn helper() -> Int { 40 } }",
    ];
    let engine = Engine::new().build().expect("engine builds");

    let terminals = sources.map(|source| {
        std::fs::write(&path, source).expect("order-variant source is written");
        let closure = engine
            .canonical_module_closure_from_source(&path, source, "main")
            .expect("order-variant source uses the canonical route")
            .expect("order-variant source produces a checked closure");
        let admitted = engine
            .admit_linked_module_closure(closure)
            .expect("order-variant closure admits through Engine");
        let (request, _cancellation) = engine
            .new_admitted_program_request(&admitted, None)
            .expect("Engine issues a request for the order-variant route");
        futures::executor::block_on(engine.execute_admitted_program(&request))
            .expect("checked CPS route executes both order variants")
    });

    assert_eq!(terminals[0], terminals[1]);
    assert_eq!(
        terminals[0],
        CanonicalTerminalEnvelopeV1::returned(Value::Int(41))
    );
}

#[test]
fn task_2069_canonical_source_route_does_not_select_arbitrary_non_root_callable() {
    let temporary = tempfile::tempdir().expect("temporary source directory exists");
    let file_path = temporary.path().join("main.ash");
    let file_source = "pub mod api; use crate::api::serve as remote; fn main() -> Int { remote() }";
    std::fs::write(&file_path, file_source).expect("file-backed module root is written");
    std::fs::write(
        temporary.path().join("api.ash"),
        "pub fn serve() -> Int { 2 } pub fn alternate() -> Int { 3 }",
    )
    .expect("file-backed structural child is written");

    let engine = Engine::new().build().expect("engine builds");
    let closure = engine
        .canonical_module_closure_from_source(&file_path, file_source, "main")
        .expect("canonical source route lowers without a legacy fallback")
        .expect("structural module source produces a checked closure");

    let non_root = closure
        .modules()
        .iter()
        .find(|module| module.interface().artifact().key() != closure.root())
        .expect("closure contains the structural child");
    assert_eq!(
        non_root.entry_name(),
        Some(""),
        "non-root modules must use a neutral carrier; callable imports use canonical local entries"
    );
}

#[test]
fn task_2069_canonical_source_route_uses_supplied_root_source_authority() {
    let temporary = tempfile::tempdir().expect("temporary source directory exists");
    let file_path = temporary.path().join("main.ash");
    let on_disk_source = "pub mod api { pub fn serve() -> Int { 1 } } fn main() -> Int { 1 }";
    let supplied_source = "pub mod api { pub fn serve() -> Int { 2 } } fn main() -> Int { 2 }";
    std::fs::write(&file_path, on_disk_source).expect("file-backed source is written");

    let engine = Engine::new().build().expect("engine builds");
    let closure = engine
        .canonical_module_closure_from_source(&file_path, supplied_source, "main")
        .expect("canonical source route accepts the supplied root source")
        .expect("structural module source produces a checked closure");
    let admitted = engine
        .admit_linked_module_closure(closure)
        .expect("canonical source closure admits through Engine");
    let (request, _cancellation) = engine
        .new_admitted_program_request(&admitted, None)
        .expect("Engine issues a request for the checked route");
    let terminal = futures::executor::block_on(engine.execute_admitted_program(&request))
        .expect("checked CPS route executes the supplied source closure");

    assert_eq!(
        terminal,
        CanonicalTerminalEnvelopeV1::returned(Value::Int(2))
    );
}

#[test]
fn task_2001_file_declaration_resolution_retains_local_newtype_module_identity() {
    let temporary = tempfile::tempdir().expect("temporary source directory exists");
    let path = temporary.path().join("local_newtype_identity.ash");
    std::fs::write(
        &path,
        r"
        newtype LocalId = LocalId(Int);
        fn main() -> LocalId { LocalId(7) }
        ",
    )
    .expect("file-backed entry source is written");

    let engine = Engine::new().build().expect("engine builds");
    let mut entry = engine
        .parse_file(&path)
        .expect("ordinary file-backed entry parses");
    engine
        .check(&mut entry)
        .expect("ordinary file-backed entry checks before declaration resolution inspection");

    let program = engine
        .get_surface_program(entry.id)
        .expect("file-backed entry retains its parsed program");
    let module = engine
        .get_surface_program_module_identity(entry.id)
        .expect("file-backed entry retains its canonical module identity");
    let resolution_env = declaration_resolution_env(
        &ash_typeck::TypeEnv::with_builtin_types(),
        &program,
        module.clone(),
    )
    .expect("declaration resolution accepts the checked local newtype");

    assert_eq!(
        resolution_env.nominal_type_identity("LocalId"),
        Some(TypeDeclId::ordinary(module, "LocalId")),
        "the post-check declaration resolver must retain the file module's canonical local newtype identity rather than regenerate TypeEnv's synthetic fallback",
    );
}

#[test]
fn task_2001_inline_declaration_resolution_retains_standalone_newtype_module_identity() {
    let engine = Engine::new().build().expect("engine builds");
    let mut entry = engine
        .parse(
            r"
            newtype InlineId = InlineId(Int);
            fn main() -> InlineId { InlineId(7) }
            ",
        )
        .expect("standalone inline entry parses");
    engine
        .check(&mut entry)
        .expect("standalone inline entry checks before declaration resolution inspection");

    let program = engine
        .get_surface_program(entry.id)
        .expect("standalone inline entry retains its parsed program");
    let module = ash_typeck::standalone_program_module_identity();
    let resolution_env = declaration_resolution_env(
        &ash_typeck::TypeEnv::with_builtin_types(),
        &program,
        module.clone(),
    )
    .expect("declaration resolution accepts the checked inline local newtype");

    assert_eq!(
        resolution_env.nominal_type_identity("InlineId"),
        Some(TypeDeclId::ordinary(module, "InlineId")),
        "the post-check inline declaration resolver must reuse the standalone program identity rather than regenerate TypeEnv's synthetic fallback",
    );
}

#[tokio::test]
async fn production_source_run_admits_supported_literal_through_checked_cps_inspection_bridge() {
    let engine = Engine::new().build().expect("engine builds");

    let source_value = engine
        .run("fn main() -> Int { 42 }")
        .await
        .expect("supported literal source route admits through checked Core/CPS");
    assert_eq!(source_value, Value::Int(42));
    assert_eq!(engine.checked_cps_inspection_count(), 1);

    let mut entry = engine
        .parse("fn main() -> Int { 7 }")
        .expect("application source parses");
    engine.check(&mut entry).expect("application source checks");
    let admitted = engine
        .admit_application(ApplicationAdmissionRequest {
            entry_name: "main".to_string(),
            body: entry.core.clone(),
            application_id: None,
            run_id: None,
            required_capabilities: Vec::new(),
            requires: Vec::new(),
            ensures: Vec::new(),
        })
        .await;
    assert!(matches!(
        admitted,
        ApplicationAdmissionOutcome::Rejected { failure, .. }
            if failure.kind == ApplicationFailureKind::AdmissionFailure
    ));
    assert_eq!(engine.checked_cps_inspection_count(), 1);

    engine
        .lower_entry_to_checked_cps(&entry)
        .expect("the explicit inspection bridge accepts the literal entry");
    assert_eq!(engine.checked_cps_inspection_count(), 2);
}

#[tokio::test]
async fn zero_input_bootstrap_materializes_the_checked_cps_inspection_bridge() {
    let success_engine = Engine::new().build().expect("engine builds");
    let success_result = success_engine
        .bootstrap_entry_source_result(
            r"
            use result::Result
            use runtime::RuntimeError

            fn main() -> Result<(), RuntimeError> { Ok { value: {} } }
            ",
        )
        .await
        .expect("canonical Ok entry executes through checked Core/CPS admission");
    assert_eq!(success_result.exit_code, 0);
    let Value::Variant { name, fields } = success_result.terminal_value else {
        panic!("canonical entry must return the Ok variant");
    };
    assert_eq!(name, "Ok");
    assert_eq!(fields.len(), 1);
    assert_eq!(fields[0].0, "value");
    assert!(matches!(&fields[0].1, Value::Record(record) if record.is_empty()));
    assert_eq!(success_engine.checked_cps_inspection_count(), 1);

    let trap_engine = Engine::new().build().expect("engine builds");
    let trap_result = trap_engine
        .bootstrap_entry_source_result(
            r#"
            use result::Result
            use runtime::RuntimeError

            fn main() -> Result<(), RuntimeError> {
                Err { error: RuntimeError(42, "boom") }
            }
            "#,
        )
        .await
        .expect("supported runtime-error constructor entry executes through checked Core/CPS");
    assert_eq!(trap_result.exit_code, 42);
    assert_eq!(trap_engine.checked_cps_inspection_count(), 1);
}

// ============================================================
// EngineBuilder Configuration Tests
// ============================================================

#[test]
fn test_builder_stdio_capabilities_chaining() {
    // with_stdio_capabilities should return Self for chaining
    let builder = Engine::new();
    let builder = builder.with_stdio_capabilities();
    let result = builder.build();
    assert!(
        result.is_ok(),
        "Builder with stdio capabilities should build successfully"
    );
}

#[test]
fn test_builder_fs_capabilities_chaining() {
    // with_fs_capabilities should return Self for chaining
    let builder = Engine::new();
    let builder = builder.with_fs_capabilities();
    let result = builder.build();
    assert!(
        result.is_ok(),
        "Builder with fs capabilities should build successfully"
    );
}

#[test]
fn test_builder_chaining_multiple_capabilities() {
    // Multiple capability methods should chain together
    let result = Engine::new()
        .with_stdio_capabilities()
        .with_fs_capabilities()
        .build();
    assert!(
        result.is_ok(),
        "Builder with multiple capabilities should build successfully"
    );
}

#[test]
fn test_builder_chaining_order_independent() {
    // Order of capability configuration should not matter
    let engine1 = Engine::new()
        .with_stdio_capabilities()
        .with_fs_capabilities()
        .build();

    let engine2 = Engine::new()
        .with_fs_capabilities()
        .with_stdio_capabilities()
        .build();

    // Both should succeed (we can't compare engines directly without PartialEq,
    // but we can verify both builds succeed)
    assert!(engine1.is_ok(), "First build order should succeed");
    assert!(engine2.is_ok(), "Second build order should succeed");
}

#[test]
fn test_builder_reusable_pattern() {
    // The builder pattern should be usable for multiple engines
    let base_builder = Engine::new();

    let engine1 = base_builder.build();
    // Note: After build(), the builder is consumed. This test documents
    // the expected usage pattern where a new builder is created each time.
    assert!(engine1.is_ok());

    // Creating a new engine from a new builder
    let engine2 = Engine::new().build();
    assert!(engine2.is_ok());
}

// ============================================================
// Send + Sync Thread Safety Tests
// ============================================================

#[test]
fn test_engine_is_send() {
    // Compile-time check: Engine must be Send
    fn assert_send<T: Send>() {}
    assert_send::<Engine>();
}

#[test]
fn test_engine_is_sync() {
    // Compile-time check: Engine must be Sync
    fn assert_sync<T: Sync>() {}
    assert_sync::<Engine>();
}

#[test]
fn test_engine_builder_is_send() {
    // Compile-time check: EngineBuilder should be Send for flexibility
    fn assert_send<T: Send>() {}
    assert_send::<EngineBuilder>();
}

#[test]
fn test_engine_error_is_send_sync() {
    // Compile-time check: EngineError must be Send + Sync for error handling
    fn assert_send<T: Send>() {}
    fn assert_sync<T: Sync>() {}
    assert_send::<EngineError>();
    assert_sync::<EngineError>();
}

#[tokio::test]
async fn test_engine_can_be_shared_across_tasks() {
    // Runtime check: Engine can be shared across async tasks
    use std::sync::Arc;

    let engine = Arc::new(Engine::new().build().expect("engine builds"));

    let engine_clone = Arc::clone(&engine);
    let task = tokio::spawn(async move {
        // Access the engine in a spawned task
        let _ = &*engine_clone;
        true
    });

    let result = task.await.expect("task completed");
    assert!(result, "Engine should be accessible across tasks");
}

// ============================================================
// Error Type Tests
// ============================================================

#[test]
fn test_engine_error_parse_variant() {
    // Verify Parse variant exists and can be created
    let err = EngineError::Parse("syntax error".to_string());
    assert!(
        matches!(err, EngineError::Parse(_)),
        "Error should be Parse variant"
    );
}

#[test]
fn test_engine_error_type_variant() {
    // Verify Type variant exists and can be created
    let err = EngineError::Type("type mismatch".to_string());
    assert!(
        matches!(err, EngineError::Type(_)),
        "Error should be Type variant"
    );
}

#[test]
fn test_engine_error_execution_variant() {
    // Verify Execution variant exists and can be created
    let err = EngineError::Execution("runtime error".to_string());
    assert!(
        matches!(err, EngineError::Execution(_)),
        "Error should be Execution variant"
    );
}

#[test]
fn test_engine_error_io_variant() {
    // Verify Io variant exists with std::io::Error
    let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "file not found");
    let err = EngineError::Io(io_err);
    assert!(
        matches!(err, EngineError::Io(_)),
        "Error should be Io variant"
    );
}

#[test]
fn test_engine_error_capability_not_found_variant() {
    // Verify CapabilityNotFound variant exists
    let err = EngineError::CapabilityNotFound("fs:read".to_string());
    assert!(
        matches!(err, EngineError::CapabilityNotFound(_)),
        "Error should be CapabilityNotFound variant"
    );
}

#[test]
fn test_engine_error_display_format() {
    // Verify error messages are informative
    let parse_err = EngineError::Parse("unexpected token".to_string());
    let display = format!("{parse_err}");
    assert!(
        display.contains("unexpected token"),
        "Parse error should display message: {display}"
    );
}

#[test]
fn test_engine_error_from_io_error() {
    // Verify automatic conversion from std::io::Error
    let io_err = std::io::Error::new(std::io::ErrorKind::PermissionDenied, "access denied");
    let engine_err: EngineError = io_err.into();
    assert!(
        matches!(engine_err, EngineError::Io(_)),
        "Should convert from io::Error"
    );
}

// ============================================================
// Property-Based Tests
// ============================================================

proptest! {
    /// Property: Engine creation always succeeds (when not configured with invalid options)
    #[test]
    fn prop_engine_creation_succeeds(_dummy in any::<u8>()) {
        let result = Engine::new().build();
        prop_assert!(result.is_ok(), "Engine creation should always succeed");
    }

    /// Property: Error messages preserve their content
    #[test]
    fn prop_error_message_preservation(message in "[a-zA-Z0-9_ ]{1,100}") {
        let err = EngineError::Parse(message.clone());
        let display = format!("{err}");
        prop_assert!(
            display.contains(&message),
            "Error display should contain original message"
        );
    }

    /// Property: CapabilityNotFound preserves capability name
    #[test]
    fn prop_capability_name_preservation(name in "[a-z_][a-z0-9_:]{1,50}") {
        let err = EngineError::CapabilityNotFound(name.clone());
        if let EngineError::CapabilityNotFound(found_name) = err {
            prop_assert_eq!(found_name, name, "Capability name should be preserved");
        } else {
            prop_assert!(false, "Error should be CapabilityNotFound variant");
        }
    }
}

// ============================================================
// TODO/Future Tests (marked as ignore until implemented)
// ============================================================

#[test]
fn test_engine_parse_valid_source() {
    let engine = Engine::new().build().unwrap();
    let result = engine.parse("fn main() { {} }");
    assert!(result.is_ok());
}

#[test]
fn test_engine_parse_invalid_source_returns_parse_error() {
    let engine = Engine::new().build().unwrap();
    let result = engine.parse("invalid syntax!!!");
    assert!(matches!(result, Err(EngineError::Parse(_))));
}

#[test]
fn test_engine_check_valid_entry() {
    let engine = Engine::new().build().unwrap();
    let mut entry = engine.parse("fn main() { 42 }").unwrap();
    let result = engine.check(&mut entry);
    assert!(result.is_ok());
}

#[test]
fn test_engine_infer_expression_type_reports_canonical_names() {
    let engine = Engine::new().build().unwrap();

    assert_eq!(engine.infer_expression_type("42").unwrap(), "Int");
    assert_eq!(engine.infer_expression_type("\"hello\"").unwrap(), "String");
    assert_eq!(
        engine.infer_expression_type("[1, 2, 3]").unwrap(),
        "List<Int>"
    );
    assert_eq!(engine.infer_expression_type("1 + 2").unwrap(), "Int");
    assert_eq!(engine.infer_expression_type("!true").unwrap(), "Bool");
}

// ============================================================
// EngineBuilder HTTP Capabilities Tests
// ============================================================

#[test]
fn test_builder_http_capabilities_registers_provider() {
    let config = HttpConfig::new();
    let engine = Engine::new()
        .with_http_capabilities(config)
        .build()
        .expect("engine builds with HTTP capabilities");
    assert!(
        engine.has_provider("http"),
        "Builder with HTTP capabilities should register the HTTP provider"
    );
}

#[test]
fn test_builder_http_capabilities_with_custom_config_registers_provider() {
    let config = HttpConfig {
        timeout_seconds: 60,
        max_redirects: 5,
        verify_ssl: false,
    };
    let engine = Engine::new()
        .with_http_capabilities(config)
        .build()
        .expect("engine builds with custom HTTP config");
    assert!(
        engine.has_provider("http"),
        "Builder with custom HTTP config should register the HTTP provider"
    );
}

#[test]
fn test_builder_http_default_config() {
    // Test HttpConfig::new() provides sensible defaults
    let config = HttpConfig::new();
    assert_eq!(config.timeout_seconds, 30);
    assert_eq!(config.max_redirects, 10);
    assert!(config.verify_ssl);
}

// ============================================================
// EngineBuilder Custom Provider Tests
// ============================================================

#[test]
fn test_builder_custom_provider_chaining() {
    // with_custom_provider should return Self for chaining
    use providers::StdioProvider;
    use std::sync::Arc;

    let provider = StdioProvider::new();
    let builder = Engine::new();
    let builder = builder.with_custom_provider("custom_stdio", Arc::new(provider));
    let result = builder.build();
    assert!(
        result.is_ok(),
        "Builder with custom provider should build successfully"
    );
}

#[test]
fn test_builder_custom_provider_overrides_builtin() {
    // Custom providers with the same name as built-ins should override them
    use providers::StdioProvider;

    let custom_stdio = StdioProvider::new();
    let result = Engine::new()
        .with_stdio_capabilities() // Enable built-in stdio
        .with_custom_provider("stdio", std::sync::Arc::new(custom_stdio)) // Override with custom
        .build();
    assert!(
        result.is_ok(),
        "Builder should allow overriding built-in providers"
    );
}

#[test]
fn test_builder_multiple_custom_providers() {
    // Multiple custom providers should all be registered
    use providers::{FsProvider, StdioProvider};

    let result = Engine::new()
        .with_custom_provider("my_stdio", std::sync::Arc::new(StdioProvider::new()))
        .with_custom_provider("my_fs", std::sync::Arc::new(FsProvider::new()))
        .build();
    assert!(
        result.is_ok(),
        "Builder with multiple custom providers should build successfully"
    );
}

// ============================================================
// EngineBuilder Combined Configuration Tests
// ============================================================

#[test]
fn test_builder_stdio_fs_custom_together() {
    // Test stdio, fs, and custom providers together (HTTP returns error until implemented)
    use providers::StdioProvider;

    let result = Engine::new()
        .with_stdio_capabilities()
        .with_fs_capabilities()
        .with_custom_provider("custom", std::sync::Arc::new(StdioProvider::new()))
        .build();

    assert!(
        result.is_ok(),
        "Builder with stdio, fs, and custom providers should build successfully"
    );
}

#[test]
fn test_builder_http_with_other_capabilities_registers_provider() {
    let engine = Engine::new()
        .with_stdio_capabilities()
        .with_fs_capabilities()
        .with_http_capabilities(HttpConfig::new())
        .build()
        .expect("engine builds with stdio, fs, and HTTP capabilities");

    assert!(
        engine.has_provider("http"),
        "Builder with HTTP should register the HTTP provider"
    );
}

#[test]
fn test_builder_complex_chaining_order_without_http() {
    // Different ordering should all work (without HTTP which returns error)
    use providers::StdioProvider;

    let engine1 = Engine::new()
        .with_stdio_capabilities()
        .with_fs_capabilities()
        .build();

    let engine2 = Engine::new()
        .with_custom_provider("custom", std::sync::Arc::new(StdioProvider::new()))
        .with_stdio_capabilities()
        .build();

    assert!(engine1.is_ok(), "First order should succeed");
    assert!(engine2.is_ok(), "Second order should succeed");
}

#[test]
fn test_http_config_clone() {
    // HttpConfig should be cloneable
    let config = HttpConfig {
        timeout_seconds: 45,
        max_redirects: 3,
        verify_ssl: false,
    };
    let config_clone = config.clone();
    assert_eq!(config.timeout_seconds, config_clone.timeout_seconds);
    assert_eq!(config.max_redirects, config_clone.max_redirects);
    assert_eq!(config.verify_ssl, config_clone.verify_ssl);
}

#[test]
fn test_http_config_default() {
    // HttpConfig should implement Default
    let config = HttpConfig::default();
    assert_eq!(config.timeout_seconds, 0); // Default for u64
    assert_eq!(config.max_redirects, 0); // Default for u32
    assert!(!config.verify_ssl); // Default for bool
}

// ============================================================
// LLM Provider Builder Tests
// ============================================================

#[test]
fn test_builder_with_llm_capabilities_succeeds() {
    use crate::providers::llm::LlmConfig;
    use std::collections::HashMap;

    let mut configs = HashMap::new();
    configs.insert("openai".to_string(), LlmConfig::openai("sk-test123"));

    let result = Engine::new().with_llm_capabilities(configs).build();
    assert!(result.is_ok(), "Engine with LLM capabilities should build");
}

#[test]
fn test_builder_with_llm_capabilities_multi_provider() {
    use crate::providers::llm::LlmConfig;
    use std::collections::HashMap;

    let mut configs = HashMap::new();
    configs.insert("openai".to_string(), LlmConfig::openai("sk-test123"));
    configs.insert("ollama".to_string(), LlmConfig::ollama());

    let result = Engine::new().with_llm_capabilities(configs).build();
    assert!(
        result.is_ok(),
        "Engine with multiple LLM providers should build"
    );
}

#[test]
fn test_builder_with_llm_capabilities_invalid_config() {
    use crate::providers::llm::LlmConfig;
    use std::collections::HashMap;

    let mut configs = HashMap::new();
    configs.insert(
        "invalid".to_string(),
        LlmConfig {
            api_base: "not-a-url".to_string(),
            api_key: "key".to_string(),
            ..LlmConfig::default()
        },
    );

    // Should NOT fail - just prints warning and skips registration
    let result = Engine::new().with_llm_capabilities(configs).build();
    assert!(
        result.is_ok(),
        "Engine should build even with invalid LLM config (skips registration)"
    );
}

#[test]
fn test_bind_imported_callable_types_uses_imported_pub_fn_signature() {
    use ash_parser::input::new_input;
    use ash_parser::parse_module::parse_fn_definition;
    use ash_parser::surface::Definition;
    use ash_typeck::Type;
    use ash_typeck::type_env::TypeEnv;
    use winnow::Parser;

    let mut input = new_input("pub fn bind(ma: Int, f: (Int) -> Int) -> Int { ma }");
    let parsed = parse_fn_definition
        .parse_next(&mut input)
        .expect("function definition should parse");
    let Definition::Function(function) = parsed else {
        panic!("expected ordinary function definition");
    };

    let mut workflow = Entry {
        core: ash_core::Expr::Literal(ash_core::Value::Null),
        core_lowering: EntryCoreLowering::Available,
        lowering_sidecars: EntryLoweringSidecars {
            entry_body_origin: SourceAnchor::new(
                SourceOrigin::Synthetic {
                    reason: "unit test entry".to_string(),
                },
                None,
                "unit test entry",
            ),
            expansion_origins: Vec::new(),
            identifier_hygiene: Vec::new(),
            callable_contracts: BTreeMap::new(),
        },
        id: 0,
        owner_token: std::sync::Arc::new(()),
        imported_closures: HashMap::new(),
        imported_param_counts: HashMap::from([(String::from("bind"), 2_usize)]),
        imported_fn_signatures: HashMap::from([(String::from("bind"), function)]),
        imported_builtin_signatures: HashMap::new(),
        callable_row_requirements: HashMap::new(),
        core_callable_types: HashMap::new(),
        declared_concrete_operation: None,
    };

    let mut env = TypeEnv::with_builtin_types();
    bind_imported_callable_types(&mut env, &workflow)
        .expect("imported pub fn signature should bind cleanly");

    let Some(bound_ty) = env.lookup_call_target(None, "bind") else {
        panic!("expected imported pub fn binding for bind");
    };

    match bound_ty {
        Type::Fn(params, ret) => {
            assert_eq!(params.len(), 2, "bind should preserve arity");
            assert!(matches!(params[0], Type::Int), "first param should be Int");
            assert!(
                matches!(&params[1], Type::Fn(inner, inner_ret)
                    if inner.len() == 1
                        && matches!(inner[0], Type::Int)
                        && matches!(inner_ret.as_ref(), Type::Int)),
                "second param should preserve (Int) -> Int, got {:?}",
                params[1]
            );
            assert!(
                matches!(ret.as_ref(), Type::Int),
                "return type should be Int"
            );
        }
        other => panic!("expected Type::Fn for imported ordinary fn, got {other:?}"),
    }

    // keep mutable binding used so clippy doesn't complain if future fields change
    workflow.imported_param_counts.clear();
}
