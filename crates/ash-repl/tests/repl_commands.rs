//! Integration tests for the canonical REPL command surface.

use ash_repl::{
    CONTINUATION_PROMPT, NORMAL_PROMPT, STARTUP_BANNER, canonical_command_names, help_text,
};

#[test]
fn canonical_repl_contract_exposes_spec_commands() {
    assert_eq!(NORMAL_PROMPT, "ash> ");
    assert_eq!(CONTINUATION_PROMPT, "... ");
    assert_eq!(
        STARTUP_BANNER,
        "Ash REPL - Type :help for help, :quit to exit"
    );

    assert_eq!(
        canonical_command_names(),
        &[":help", ":quit", ":type", ":ast", ":clear"]
    );
}

#[test]
fn canonical_help_text_lists_supported_commands_and_aliases() {
    let help = help_text();

    assert!(help.contains(":help, :h"));
    assert!(help.contains(":quit, :q"));
    assert!(help.contains(":type, :t"));
    assert!(help.contains(":ast"));
    assert!(help.contains(":clear"));
    assert!(!help.contains(":bindings"));
}
