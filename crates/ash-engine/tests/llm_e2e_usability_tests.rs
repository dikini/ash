//! TASK-550: End-to-end usability validation for the LLM stdlib.
//!
//! Verifies that the full chain works:
//! 1. An .ash file using `use llm::*` types parses through the engine
//! 2. SPEC-029 sections are substantively covered by the stdlib files
//! 3. All PLAN-027 success criteria are met

use ash_engine::Engine;
use ash_engine::module_loader::count_pub_fn_snippets;
use std::path::{Path, PathBuf};

fn stdlib_root() -> &'static Path {
    Path::new(concat!(env!("CARGO_MANIFEST_DIR"), "/../../std/src"))
}

fn make_engine() -> Engine {
    Engine::new().build().expect("engine should build")
}

fn stdlib_path(relative: &str) -> PathBuf {
    stdlib_root().join(relative)
}

fn read_stdlib_file(relative: &str) -> String {
    let path = stdlib_root().join(relative);
    std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("failed to read {relative}: {e}"))
}

// ---------------------------------------------------------------------------
// Criterion 1: All 27 prompt.ash pub fns parse
// ---------------------------------------------------------------------------

#[test]
fn test_all_prompt_pub_fns_parse() {
    let source = read_stdlib_file("llm/prompt.ash");
    let (count, diagnostics) = count_pub_fn_snippets(&source);
    assert_eq!(
        count,
        27,
        "expected 27 parseable pub fns, got {count} ({})",
        diagnostics.len()
    );
    assert!(
        diagnostics.is_empty(),
        "expected 0 diagnostics, got {diagnostics:?}"
    );
}

// ---------------------------------------------------------------------------
// Criterion 2: ash check on all llm/ files reports 0 errors
// ---------------------------------------------------------------------------

#[test]
fn test_all_llm_files_check_clean() {
    let engine = make_engine();
    let llm_dir = stdlib_path("llm");
    let mut checked = 0;

    for entry in std::fs::read_dir(&llm_dir).expect("llm dir should exist") {
        let entry = entry.expect("dir entry");
        let path = entry.path();
        if path.extension().is_none_or(|e| e != "ash") {
            continue;
        }
        let result = engine
            .check_module_file(&path)
            .unwrap_or_else(|e| panic!("{}: check failed: {e}", path.display()));

        assert!(
            result.errors.is_empty(),
            "{}: unexpected errors: {:?}",
            path.display(),
            result.errors,
        );
        assert!(
            result.warnings.is_empty(),
            "{}: unexpected warnings: {:?}",
            path.display(),
            result.warnings,
        );
        checked += 1;
    }

    assert!(
        checked >= 6,
        "expected at least 6 llm/ files, checked {checked}"
    );
}

// ---------------------------------------------------------------------------
// Criterion 3: use llm::Role resolves from application code
// ---------------------------------------------------------------------------

#[test]
fn test_use_llm_role_resolves() {
    let consumer = stdlib_path("_e2e_role_test.ash");
    std::fs::write(&consumer, "use llm::Role;\nworkflow main { done }").expect("write");
    let engine = make_engine();
    let result = engine.parse_file(&consumer);
    let _ = std::fs::remove_file(&consumer);
    assert!(result.is_ok(), "use llm::Role should resolve: {result:?}");
}

#[test]
fn test_use_llm_message_resolves() {
    let consumer = stdlib_path("_e2e_message_test.ash");
    std::fs::write(&consumer, "use llm::Message;\nworkflow main { done }").expect("write");
    let engine = make_engine();
    let result = engine.parse_file(&consumer);
    let _ = std::fs::remove_file(&consumer);
    assert!(
        result.is_ok(),
        "use llm::Message should resolve: {result:?}"
    );
}

// ---------------------------------------------------------------------------
// Criterion 4: New SPEC-029 functions present
// ---------------------------------------------------------------------------

#[test]
fn test_new_functions_present_in_prompt_ash() {
    let source = read_stdlib_file("llm/prompt.ash");
    for name in &[
        "append_response",
        "append_tool_result",
        "is_final",
        "render_template",
    ] {
        assert!(
            source.contains(&format!("pub fn {name}(")),
            "prompt.ash should declare pub fn {name}",
        );
    }
}

// ---------------------------------------------------------------------------
// Criterion 5: Three-vertex compliance
// ---------------------------------------------------------------------------

#[test]
fn test_three_vertex_compliance() {
    for filename in &["router.ash", "supervised.ash"] {
        let source = read_stdlib_file(&format!("llm/{filename}"));
        let forbidden = [
            "complete(",
            "complete_with_tools(",
            "stream(",
            "embed(",
            "act ",
        ];

        let mut in_fn = false;
        let mut brace_depth = 0usize;
        let mut fn_name = String::new();

        for line in source.lines() {
            let trimmed = line.trim();
            if trimmed.starts_with("fn ") {
                in_fn = true;
                fn_name = trimmed.split('(').next().unwrap_or("").to_string();
                brace_depth = 0;
            } else if trimmed.starts_with("workflow ") {
                in_fn = false;
            }
            if in_fn {
                for ch in trimmed.chars() {
                    match ch {
                        '{' => brace_depth += 1,
                        '}' => {
                            brace_depth = brace_depth.saturating_sub(1);
                            if brace_depth == 0 {
                                in_fn = false;
                            }
                        }
                        _ => {}
                    }
                }
                if brace_depth > 0 && !trimmed.starts_with("--") {
                    for f in &forbidden {
                        assert!(
                            !trimmed.contains(f),
                            "{filename}: three-vertex violation -- fn '{fn_name}' calls '{f}'",
                        );
                    }
                }
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Criterion 6: End-to-end workflow parsing with llm types
// ---------------------------------------------------------------------------

#[test]
fn test_e2e_workflow_with_llm_types() {
    // Write a consumer .ash file that imports several llm types and uses them
    // in a workflow. This exercises the full path:
    //   load_ordinary_file -> collect_module_exports -> resolve use targets -> parse
    let consumer = stdlib_path("_e2e_workflow_test.ash");
    std::fs::write(
        &consumer,
        r#"use llm::Role;
use llm::Message;
use llm::ChatResponse;
use llm::ToolCall;

workflow chat_demo {
    let sys = Message {
        sender: System,
        content: "You are a helpful assistant.",
        tool_calls: None,
        tool_call_id: None
    };
    let usr = Message {
        sender: User,
        content: "Hello!",
        tool_calls: None,
        tool_call_id: None
    };
    done
}
"#,
    )
    .expect("write consumer");

    let engine = make_engine();
    let result = engine.parse_file(&consumer);
    let _ = std::fs::remove_file(&consumer);

    assert!(
        result.is_ok(),
        "e2e workflow with llm types should parse: {result:?}",
    );
}

// ---------------------------------------------------------------------------
// SPEC-029 section coverage audit
// ---------------------------------------------------------------------------

#[test]
fn test_spec_029_section_coverage() {
    // §2 Module Structure: namespace layout verified by file existence
    for file in &[
        "llm/mod.ash",
        "llm/types.ash",
        "llm/prompt.ash",
        "llm/dispatch.ash",
    ] {
        assert!(
            stdlib_path(file).exists(),
            "SPEC-029 §2: {file} should exist",
        );
    }

    // §3 Types: all 11 types declared
    let types_source = read_stdlib_file("llm/types.ash");
    for type_name in &[
        "Role",
        "Message",
        "ChatResponse",
        "ToolCall",
        "ToolDef",
        "Usage",
        "ChatChunk",
        "ToolCallDelta",
        "Embedding",
        "ProviderConfig",
        "CompletionParams",
    ] {
        assert!(
            types_source.contains(&format!("pub type {type_name}")),
            "SPEC-029 §3: types.ash should declare {type_name}",
        );
    }

    // §4 Pure Functions: constructors and inspectors present
    let prompt_source = read_stdlib_file("llm/prompt.ash");
    // §4.1 Constructors
    for name in &["system", "user", "assistant", "tool_result"] {
        assert!(
            prompt_source.contains(&format!("pub fn {name}(")),
            "SPEC-029 §4.1: prompt.ash should have constructor {name}",
        );
    }
    // §4.2 Inspectors + new functions
    for name in &[
        "has_tool_calls",
        "is_final",
        "get_tool_calls",
        "append_response",
        "append_tool_result",
    ] {
        assert!(
            prompt_source.contains(&format!("pub fn {name}(")),
            "SPEC-029 §4.2: prompt.ash should have inspector {name}",
        );
    }
    // §4.3 Renderers
    for name in &["render_plaintext", "render_markdown", "render_template"] {
        assert!(
            prompt_source.contains(&format!("pub fn {name}(")),
            "SPEC-029 §4.3: prompt.ash should have renderer {name}",
        );
    }

    // §5 Dispatch module exists
    assert!(
        stdlib_path("llm/dispatch.ash").exists(),
        "SPEC-029 §5: dispatch.ash should exist",
    );

    // §6 Orchestration helpers
    for file in &["llm/router.ash", "llm/supervised.ash"] {
        assert!(
            stdlib_path(file).exists(),
            "SPEC-029 §8: {file} should exist",
        );
    }

    // §8 Agent Workflows: router and supervised have workflow declarations
    for file in &["router.ash", "supervised.ash"] {
        let source = read_stdlib_file(&format!("llm/{file}"));
        assert!(
            source.contains("workflow "),
            "SPEC-029 §8: {file} should declare a workflow",
        );
    }
}
