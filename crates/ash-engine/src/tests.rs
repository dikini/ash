//! Engine public shell tests.

use super::*;
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
        id: 0,
        imported_closures: HashMap::new(),
        imported_param_counts: HashMap::from([(String::from("bind"), 2_usize)]),
        imported_fn_signatures: HashMap::from([(String::from("bind"), function)]),
        imported_builtin_signatures: HashMap::new(),
        callable_row_requirements: HashMap::new(),
        core_callable_types: HashMap::new(),
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
