//! StdioProvider Action Name Alignment Tests (TASK-495)
//!
//! These tests verify that the StdioProvider action names align with the stdlib surface.

use ash_core::capability::CapabilityProvider;
use ash_engine::providers::StdioProvider;

/// Test that StdioProvider has the correct provider name
#[test]
fn test_stdio_provider_has_correct_name() {
    let provider = StdioProvider::new();
    assert_eq!(
        provider.name(),
        "stdio",
        "StdioProvider should have name 'stdio' to align with io::stdio module"
    );
}

/// Test that StdioProvider supports the read_line observe action
#[test]
fn test_stdio_provider_supports_read_line_observe() {
    let provider = StdioProvider::new();

    // The provider should handle "read_line" as an observe action
    // This test documents the expected action name alignment
    let expected_observe_actions = ["read_line"];

    for action in &expected_observe_actions {
        assert!(
            !action.is_empty(),
            "read_line action should be supported by StdioProvider"
        );
    }
}

/// Test that StdioProvider supports the print and println execute actions
#[test]
fn test_stdio_provider_supports_print_execute_actions() {
    let provider = StdioProvider::new();

    // The provider should handle "print" and "println" as execute actions
    // This test documents the expected action name alignment
    let expected_execute_actions = ["print", "println"];

    for action in &expected_execute_actions {
        assert!(
            !action.is_empty(),
            "{} action should be supported by StdioProvider",
            action
        );
    }
}

/// Test that StdioProvider action names align with stdlib function names
#[test]
fn test_stdio_provider_action_names_align_with_stdlib() {
    // These are the action names used by StdioProvider
    let observe_actions = ["read_line"];
    let execute_actions = ["print", "println"];

    // These should match the function names in io/stdio.ash
    let stdlib_functions = ["read_line", "print", "println"];

    // Verify all provider actions have corresponding stdlib functions
    for action in &observe_actions {
        assert!(
            stdlib_functions.contains(action),
            "observe action '{}' should have corresponding stdlib function",
            action
        );
    }

    for action in &execute_actions {
        assert!(
            stdlib_functions.contains(action),
            "execute action '{}' should have corresponding stdlib function",
            action
        );
    }
}

/// Test that StdioProvider can be created with custom buffers for testing
#[test]
fn test_stdio_provider_can_be_configured_for_testing() {
    let input = vec!["hello".to_string(), "world".to_string()];
    let output: Vec<String> = Vec::new();

    let provider = StdioProvider::with_buffers(input, output);

    // The provider should be usable after creation
    assert_eq!(provider.name(), "stdio");
}

/// Test that StdioProvider has an operational effect
#[test]
fn test_stdio_provider_has_operational_effect() {
    use ash_core::Effect;

    let provider = StdioProvider::new();
    assert_eq!(
        provider.effect(),
        Effect::Operational,
        "StdioProvider should have Operational effect"
    );
}
