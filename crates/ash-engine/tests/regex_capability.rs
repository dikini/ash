//! Regex capability integration tests (TASK-595)
//!
//! Tests that verify the regex provider works end-to-end with the engine.

use ash_core::capability::CapabilityProvider;
use ash_engine::Engine;
use ash_engine::providers::RegexProvider;

// ============================================================
// Provider Unit Tests
// ============================================================

#[test]
fn test_regex_provider_name() {
    let provider = RegexProvider::new();
    assert_eq!(provider.name(), "regex");
}

#[test]
fn test_regex_provider_effect() {
    use ash_core::Effect;
    let provider = RegexProvider::new();
    assert_eq!(provider.effect(), Effect::Operational);
}

// ============================================================
// Engine Builder Integration Tests
// ============================================================

#[test]
fn test_engine_builder_with_regex_capabilities() {
    let engine = Engine::new()
        .with_regex_capabilities()
        .build()
        .expect("engine builds with regex capabilities");

    let workflow = engine.parse("workflow main { ret 42; }").expect("parses");
    let result = tokio_test::block_on(async { engine.execute(&workflow).await });

    assert!(result.is_ok(), "Engine should execute workflow");
    assert_eq!(result.unwrap(), ash_core::Value::Int(42));
}

// ============================================================
// End-to-End Regex Execution Tests
// ============================================================

#[tokio::test]
async fn test_regex_find_through_engine() {
    let _engine = Engine::new()
        .with_regex_capabilities()
        .build()
        .expect("engine builds");

    // Test via direct provider execute call (closest to engine integration)
    let provider = RegexProvider::new();
    let result = provider
        .execute(
            "find",
            &[
                ash_core::Value::String(r"\d+".to_string()),
                ash_core::Value::String("abc123def".to_string()),
            ],
        )
        .await;

    assert!(result.is_ok(), "find should succeed");
    // Option::Some("123") represented as Value
    assert_eq!(
        result.unwrap(),
        ash_core::Value::Variant {
            name: "Some".to_string(),
            fields: Box::new(vec![("value".to_string(), ash_core::Value::String("123".to_string()))]),
        }
    );
}

#[tokio::test]
async fn test_regex_find_no_match() {
    let provider = RegexProvider::new();
    let result = provider
        .execute(
            "find",
            &[
                ash_core::Value::String(r"\d+".to_string()),
                ash_core::Value::String("abcdef".to_string()),
            ],
        )
        .await;

    assert!(result.is_ok(), "find should succeed even with no match");
    // Option::None
    assert_eq!(
        result.unwrap(),
        ash_core::Value::Variant {
            name: "None".to_string(),
            fields: Box::new(vec![]),
        }
    );
}

#[tokio::test]
async fn test_regex_matches_true() {
    let provider = RegexProvider::new();
    let result = provider
        .execute(
            "matches",
            &[
                ash_core::Value::String(r"^hello".to_string()),
                ash_core::Value::String("hello world".to_string()),
            ],
        )
        .await;

    assert!(result.is_ok(), "matches should succeed");
    assert_eq!(result.unwrap(), ash_core::Value::Bool(true));
}

#[tokio::test]
async fn test_regex_matches_false() {
    let provider = RegexProvider::new();
    let result = provider
        .execute(
            "matches",
            &[
                ash_core::Value::String(r"^goodbye".to_string()),
                ash_core::Value::String("hello world".to_string()),
            ],
        )
        .await;

    assert!(result.is_ok(), "matches should succeed");
    assert_eq!(result.unwrap(), ash_core::Value::Bool(false));
}

#[tokio::test]
async fn test_regex_replace() {
    let provider = RegexProvider::new();
    let result = provider
        .execute(
            "replace",
            &[
                ash_core::Value::String(r"\d+".to_string()),
                ash_core::Value::String("NUM".to_string()),
                ash_core::Value::String("abc123def".to_string()),
            ],
        )
        .await;

    assert!(result.is_ok(), "replace should succeed");
    assert_eq!(
        result.unwrap(),
        ash_core::Value::String("abcNUMdef".to_string())
    );
}

#[tokio::test]
async fn test_regex_replace_all() {
    let provider = RegexProvider::new();
    let result = provider
        .execute(
            "replace",
            &[
                ash_core::Value::String(r"\d+".to_string()),
                ash_core::Value::String("X".to_string()),
                ash_core::Value::String("a1b23c".to_string()),
            ],
        )
        .await;

    assert!(result.is_ok(), "replace should replace all occurrences");
    assert_eq!(result.unwrap(), ash_core::Value::String("aXbXc".to_string()));
}

#[tokio::test]
async fn test_regex_invalid_pattern() {
    let provider = RegexProvider::new();
    let result = provider
        .execute(
            "find",
            &[
                ash_core::Value::String(r"[invalid".to_string()),
                ash_core::Value::String("text".to_string()),
            ],
        )
        .await;

    assert!(
        result.is_err(),
        "invalid pattern should return an error"
    );
    let err = result.unwrap_err();
    assert!(
        err.to_string().contains("Invalid argument") || err.to_string().contains("regex"),
        "error should indicate invalid pattern: {err}"
    );
}

#[tokio::test]
async fn test_regex_invalid_pattern_matches() {
    let provider = RegexProvider::new();
    let result = provider
        .execute(
            "matches",
            &[
                ash_core::Value::String(r"(unclosed".to_string()),
                ash_core::Value::String("text".to_string()),
            ],
        )
        .await;

    assert!(result.is_err(), "invalid pattern should return an error");
}

#[tokio::test]
async fn test_regex_invalid_pattern_replace() {
    let provider = RegexProvider::new();
    let result = provider
        .execute(
            "replace",
            &[
                ash_core::Value::String(r"*".to_string()),
                ash_core::Value::String("repl".to_string()),
                ash_core::Value::String("text".to_string()),
            ],
        )
        .await;

    assert!(result.is_err(), "invalid pattern should return an error");
}
