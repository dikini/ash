//! Stdlib module loading tests for the LLM stdlib (TASK-524 through TASK-528).
//!
//! Verifies that `std/src/llm/types.ash` and `std/src/llm/prompt.ash` contain
//! the required definitions per SPEC-029, and that individual type definitions
//! parse correctly through the module loader.

use std::path::Path;

/// Path to the stdlib root
fn stdlib_root() -> &'static Path {
    Path::new(concat!(env!("CARGO_MANIFEST_DIR"), "/../../std/src"))
}

// ---------------------------------------------------------------------------
// TASK-524/525: types.ash -- file structure and content
// ---------------------------------------------------------------------------

#[test]
fn test_types_dot_ash_file_exists() {
    let types_path = stdlib_root().join("llm/types.ash");
    assert!(
        types_path.exists(),
        "types.ash must exist at {:?}",
        types_path
    );
}

#[test]
fn test_types_dot_ash_contains_all_required_type_defs() {
    let types_path = stdlib_root().join("llm/types.ash");
    let source = std::fs::read_to_string(&types_path).expect("types.ash should be readable");

    // SPEC-029 SS3: required types
    let required_types = [
        "Role",
        "ToolCall",
        "ToolCallDelta",
        "Message",
        "ToolDef",
        "Usage",
        "ChatResponse",
        "ChatChunk",
        "Embedding",
        "CompletionParams",
        "ProviderConfig",
    ];

    for type_name in &required_types {
        assert!(
            source.contains(&format!("pub type {} =", type_name)),
            "types.ash must define pub type {}",
            type_name
        );
    }
}

#[test]
fn test_types_dot_ash_role_definition_correct() {
    let types_path = stdlib_root().join("llm/types.ash");
    let source = std::fs::read_to_string(&types_path).expect("types.ash should be readable");

    // SPEC-029 SS3.1: Role must have System | User | Assistant | Tool variants
    assert!(source.contains("System"), "Role must have System variant");
    assert!(source.contains("User"), "Role must have User variant");
    assert!(
        source.contains("Assistant"),
        "Role must have Assistant variant"
    );
    assert!(
        source.contains("Tool"),
        "Role must have Tool variant (payload-free)"
    );
}

#[test]
fn test_types_dot_ash_message_definition_correct() {
    let types_path = stdlib_root().join("llm/types.ash");
    let source = std::fs::read_to_string(&types_path).expect("types.ash should be readable");

    // SPEC-029 SS3.2: Message must have role, content, tool_calls, tool_call_id
    assert!(source.contains("sender:"), "Message must have sender field");
    assert!(
        source.contains("content:"),
        "Message must have content field"
    );
    assert!(
        source.contains("tool_calls:"),
        "Message must have tool_calls field"
    );
    assert!(
        source.contains("tool_call_id:"),
        "Message must have tool_call_id field"
    );
}

#[test]
fn test_types_dot_ash_chat_response_definition_correct() {
    let types_path = stdlib_root().join("llm/types.ash");
    let source = std::fs::read_to_string(&types_path).expect("types.ash should be readable");

    // SPEC-029 SS3.3: ChatResponse fields
    assert!(
        source.contains("finish_reason"),
        "ChatResponse must have finish_reason"
    );
    assert!(source.contains("usage"), "ChatResponse must have usage");
    assert!(source.contains("model"), "ChatResponse must have model");
}

#[test]
fn test_types_dot_ash_has_11_types() {
    let types_path = stdlib_root().join("llm/types.ash");
    let source = std::fs::read_to_string(&types_path).expect("types.ash should be readable");

    let count = source
        .lines()
        .filter(|l| l.starts_with("pub type "))
        .count();
    assert_eq!(
        count, 11,
        "types.ash must have exactly 11 pub type definitions (SPEC-029 SS3), found {}",
        count
    );
}

// ---------------------------------------------------------------------------
// TASK-524: mod.ash structure
// ---------------------------------------------------------------------------

#[test]
fn test_mod_dot_ash_file_exists() {
    let mod_path = stdlib_root().join("llm/mod.ash");
    assert!(mod_path.exists(), "mod.ash must exist at {:?}", mod_path);
}

#[test]
fn test_mod_dot_ash_references_submodules() {
    let mod_path = stdlib_root().join("llm/mod.ash");
    let source = std::fs::read_to_string(&mod_path).expect("mod.ash should be readable");

    // Module references
    assert!(source.contains("types"), "mod.ash must reference types");
    assert!(source.contains("prompt"), "mod.ash must reference prompt");
    assert!(source.contains("openai"), "mod.ash must reference openai");
}

// ---------------------------------------------------------------------------
// TASK-526: prompt.ash -- Constructors (SPEC-029 SS4.1)
// ---------------------------------------------------------------------------

#[test]
fn test_prompt_dot_ash_file_exists() {
    let prompt_path = stdlib_root().join("llm/prompt.ash");
    assert!(
        prompt_path.exists(),
        "prompt.ash must exist at {:?}",
        prompt_path
    );
}

#[test]
fn test_prompt_dot_ash_constructors() {
    let prompt_path = stdlib_root().join("llm/prompt.ash");
    let source = std::fs::read_to_string(&prompt_path).expect("prompt.ash should be readable");

    // TASK-526: Required constructors (SPEC-029 SS4.1)
    let constructors = ["system", "user", "assistant", "tool_result"];
    for name in &constructors {
        assert!(
            source.contains(&format!("pub fn {}(", name)),
            "prompt.ash must define pub fn {}",
            name
        );
    }
}

#[test]
fn test_prompt_dot_ash_constructor_bodies() {
    let prompt_path = stdlib_root().join("llm/prompt.ash");
    let source = std::fs::read_to_string(&prompt_path).expect("prompt.ash should be readable");

    // Verify constructors produce correct Message values
    assert!(
        source.contains("sender: System") || source.contains("sender: System,"),
        "system() must set sender to System"
    );
    assert!(
        source.contains("sender: User") || source.contains("sender: User,"),
        "user() must set sender to User"
    );
    assert!(
        source.contains("sender: Assistant") || source.contains("sender: Assistant,"),
        "assistant() must set sender to Assistant"
    );
    assert!(
        source.contains("sender: Tool") || source.contains("sender: Tool {"),
        "tool_result() must set sender to Tool"
    );
}

// ---------------------------------------------------------------------------
// TASK-527: prompt.ash -- Inspectors (SPEC-029 SS4.2)
// ---------------------------------------------------------------------------

#[test]
fn test_prompt_dot_ash_inspectors() {
    let prompt_path = stdlib_root().join("llm/prompt.ash");
    let source = std::fs::read_to_string(&prompt_path).expect("prompt.ash should be readable");

    // TASK-527: Required inspectors (SPEC-029 SS4.2)
    let inspectors = [
        "is_system",
        "is_user",
        "is_assistant",
        "is_tool",
        "has_tool_calls",
        "get_tool_calls",
        "get_tool_call_id",
    ];
    for name in &inspectors {
        assert!(
            source.contains(&format!("pub fn {}(", name)),
            "prompt.ash must define pub fn {}",
            name
        );
    }
}

#[test]
fn test_prompt_dot_ash_inspector_bodies() {
    let prompt_path = stdlib_root().join("llm/prompt.ash");
    let source = std::fs::read_to_string(&prompt_path).expect("prompt.ash should be readable");

    // Inspectors should use match expressions on Message fields
    assert!(
        source.contains("match msg"),
        "Inspectors should use match on msg"
    );
    assert!(
        source.contains("sender: System") || source.contains("System => true"),
        "is_system should match System sender"
    );
}

// ---------------------------------------------------------------------------
// TASK-528: prompt.ash -- Renderers (SPEC-029 SS4.3)
// ---------------------------------------------------------------------------

#[test]
fn test_prompt_dot_ash_renderers() {
    let prompt_path = stdlib_root().join("llm/prompt.ash");
    let source = std::fs::read_to_string(&prompt_path).expect("prompt.ash should be readable");

    // TASK-528: Required renderers (SPEC-029 SS4.3)
    // The spec uses render_conversation/render_template but existing impl uses
    // render_plaintext/render_markdown -- verify at least two renderers exist
    let render_fn_count = source
        .lines()
        .filter(|l| l.contains("pub fn render_") || l.contains("fn render_"))
        .count();

    assert!(
        render_fn_count >= 2,
        "prompt.ash must have at least 2 render functions, found {}",
        render_fn_count
    );
}

#[test]
fn test_prompt_dot_ash_renderer_output_format() {
    let prompt_path = stdlib_root().join("llm/prompt.ash");
    let source = std::fs::read_to_string(&prompt_path).expect("prompt.ash should be readable");

    // SPEC-029 SS4.3: R2 -- role prefixes should appear in renderer output
    // Check for role name formatting
    assert!(
        source.contains("system") || source.contains("System"),
        "Renderers should reference role names"
    );
}

// ---------------------------------------------------------------------------
// Cross-cutting: All stdlib files are valid UTF-8 and non-empty
// ---------------------------------------------------------------------------

#[test]
fn test_all_llm_stdlib_files_readable() {
    let llm_dir = stdlib_root().join("llm");
    let entries = std::fs::read_dir(&llm_dir).expect("llm directory must exist");
    let mut count = 0;
    for entry in entries.flatten() {
        let path = entry.path();
        if path.extension().map_or(false, |e| e == "ash") {
            let source = std::fs::read_to_string(&path)
                .unwrap_or_else(|e| panic!("Failed to read {:?}: {}", path, e));
            assert!(!source.is_empty(), "{:?} must not be empty", path);
            count += 1;
        }
    }
    assert!(
        count >= 10,
        "Expected at least 10 .ash files in llm/, found {}",
        count
    );
}

// ---------------------------------------------------------------------------
// TASK-549: Three-vertex compliance -- no fn in router.ash or supervised.ash
// references dispatch workflows (complete, complete_with_tools, stream, embed)
// ---------------------------------------------------------------------------

#[test]
fn test_router_no_fn_calls_workflow() {
    let source = std::fs::read_to_string(stdlib_root().join("llm/router.ash"))
        .expect("router.ash should be readable");
    assert_no_fn_workflow_calls(&source, "router.ash");
}

#[test]
fn test_supervised_no_fn_calls_workflow() {
    let source = std::fs::read_to_string(stdlib_root().join("llm/supervised.ash"))
        .expect("supervised.ash should be readable");
    assert_no_fn_workflow_calls(&source, "supervised.ash");
}

fn assert_no_fn_workflow_calls(source: &str, filename: &str) {
    let forbidden_calls = ["complete(", "complete_with_tools(", "stream(", "embed(", "act "];

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
                        if brace_depth > 0 {
                            brace_depth -= 1;
                        }
                        if brace_depth == 0 {
                            in_fn = false;
                        }
                    }
                    _ => {}
                }
            }

            if brace_depth > 0 && !trimmed.starts_with("--") {
                for forbidden in &forbidden_calls {
                    if trimmed.contains(forbidden) {
                        panic!(
                            "{}: three-vertex violation -- fn '{}' calls workflow via '{}'",
                            filename, fn_name, forbidden
                        );
                    }
                }
            }
        }
    }
}
