//! End-to-end validation of LLM stdlib module files (TASK-543).
//!
//! Uses structural engine APIs (check_module_file, collect_public_type_defs_from_source,
//! count_pub_fn_snippets) instead of string-matching. Validates SPEC-030 §3.5, §4.4, §5.4.
//!
//! Key finding: prompt.ash has 23 `pub fn` declarations. After TASK-546 fix
//! (keywords allowed as constructor field names), 12 parse through
//! `parse_fn_definition`. The remaining 11 still use features unsupported by
//! the parser. These are silently dropped during module loading -- this test
//! documents the known gap.

use ash_engine::Engine;
use ash_engine::module_loader::{collect_public_type_defs_from_source, count_pub_fn_snippets};
use std::path::{Path, PathBuf};

fn stdlib_root() -> &'static Path {
    Path::new(concat!(env!("CARGO_MANIFEST_DIR"), "/../../std/src"))
}

fn make_engine() -> Engine {
    Engine::new().build().expect("engine should build")
}

fn read_stdlib_file(relative: &str) -> String {
    let path = stdlib_root().join(relative);
    std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("failed to read {relative}: {e}"))
}

fn stdlib_path(relative: &str) -> PathBuf {
    stdlib_root().join(relative)
}

// ---------------------------------------------------------------------------
// Requirement 1: ash check std/src/llm/types.ash succeeds
// ---------------------------------------------------------------------------

#[test]
fn test_types_ash_check_module_file() {
    let engine = make_engine();
    let result = engine
        .check_module_file(&stdlib_path("llm/types.ash"))
        .expect("check_module_file on types.ash should succeed");

    assert_eq!(
        result.type_count, 11,
        "types.ash should have 11 pub type definitions, got {}",
        result.type_count,
    );
    assert_eq!(
        result.fn_count, 0,
        "types.ash should have 0 pub fn definitions, got {}",
        result.fn_count,
    );
    // Float is now a builtin type (TASK-545) -- Embedding and CompletionParams resolve cleanly
    assert_eq!(
        result.errors.len(),
        0,
        "types.ash should have 0 errors, got {:?}",
        result.errors,
    );
    assert!(
        result.warnings.is_empty(),
        "types.ash should have zero warnings: {:?}",
        result.warnings,
    );
}

// ---------------------------------------------------------------------------
// Requirement 1 (structural): parsed type names match SPEC-029 §3
// ---------------------------------------------------------------------------

#[test]
fn test_types_ash_structural_type_names() {
    let source = read_stdlib_file("llm/types.ash");
    let type_defs =
        collect_public_type_defs_from_source(&source).expect("parsing types.ash should succeed");

    let expected_names = [
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

    let actual_names: Vec<&str> = type_defs.iter().map(|td| td.name.as_str()).collect();

    for name in &expected_names {
        assert!(
            actual_names.contains(name),
            "types.ash should define '{name}', found: {actual_names:?}",
        );
    }
    assert_eq!(
        actual_names.len(),
        expected_names.len(),
        "types.ash should have exactly {} types, found {}: {actual_names:?}",
        expected_names.len(),
        actual_names.len(),
    );
}

// ---------------------------------------------------------------------------
// Requirement 3: pub fn export coverage from prompt.ash
//
// NOTE: parse_fn_definition only handles a subset of Ash function syntax.
// 16 of 23 pub fns in prompt.ash use record constructors or match expressions
// that the parser cannot handle. This test documents the gap.
// ---------------------------------------------------------------------------

#[test]
fn test_prompt_ash_pub_fn_partial_parse_coverage() {
    let source = read_stdlib_file("llm/prompt.ash");
    let (count, diagnostics) = count_pub_fn_snippets(&source);

    // TASK-546 fix: constructor field names now allow keywords (e.g., `role`).
    // Previously 7 of 23 parsed; now 12 parse because `role:` in Message
    // constructors no longer fails the identifier() keyword check.
    assert_eq!(
        count, 12,
        "expected exactly 12 parseable pub fns from prompt.ash (regression?), got {}",
        count,
    );
    assert_eq!(
        diagnostics.len(),
        23 - count,
        "expected {} diagnostics for unparseable pub fns, got {}",
        23 - count,
        diagnostics.len(),
    );

    // Verify diagnostics include function names (SPEC-030 §5.3)
    let diag_names: Vec<&str> = diagnostics
        .iter()
        .filter_map(|d| d.name.as_deref())
        .collect();
    assert!(
        !diag_names.is_empty(),
        "diagnostics should include function names",
    );
}

/// Target-state test: when parse_fn_definition supports record constructors and
/// match expressions, all 23 pub fns in prompt.ash should parse cleanly.
/// Remove #[ignore] once the parser is extended.
#[test]
#[ignore = "waiting for parser support for remaining 11 of 23 pub fns (match expressions, etc)"]
fn test_prompt_ash_all_23_pub_fns_parse() {
    let source = read_stdlib_file("llm/prompt.ash");
    let (count, diagnostics) = count_pub_fn_snippets(&source);

    assert_eq!(
        count, 23,
        "all 23 pub fns should parse once parser supports record constructors, got {}",
        count,
    );
    assert!(
        diagnostics.is_empty(),
        "no diagnostics expected: {:?}",
        diagnostics,
    );
}

// ---------------------------------------------------------------------------
// Requirement 2: import path resolution
// ---------------------------------------------------------------------------

#[test]
fn test_llm_types_import_resolves() {
    // Place consumer inside std/src/ so `use llm::types::Role` resolves
    let consumer = stdlib_path("_e2e_consumer_test.ash");
    std::fs::write(
        &consumer,
        "use llm::types::Role;\nuse llm::types::Message;\nworkflow main { done }\n",
    )
    .expect("write consumer");

    let engine = make_engine();
    let result = engine.parse_file(&consumer);
    let _ = std::fs::remove_file(&consumer);

    assert!(
        result.is_ok(),
        "use llm::types::Role should resolve: {:?}",
        result,
    );
}

// ---------------------------------------------------------------------------
// Requirement 2b: re-export import path (use llm::Role via mod.ash pub use)
//
// NOTE: `use llm::Role` from outside std/src/ does NOT currently resolve
// because the import resolver requires multi-segment paths for directory-style
// modules. This is a known limitation -- re-exports work within the module
// hierarchy (`use types::Role` from llm/_test.ash) but not across the
// directory boundary. Tracked as a future improvement.
// ---------------------------------------------------------------------------

// ---------------------------------------------------------------------------
// Requirement 4: mod.ash pub mod loading
// ---------------------------------------------------------------------------

#[test]
fn test_mod_ash_pub_mod_loads_children() {
    let engine = make_engine();
    let result = engine
        .check_module_file(&stdlib_path("llm/mod.ash"))
        .expect("check_module_file on mod.ash should succeed");

    // mod.ash itself defines no pub types -- it re-exports from children
    assert_eq!(
        result.type_count, 0,
        "mod.ash should have 0 direct pub type definitions, got {}",
        result.type_count,
    );
    assert_eq!(
        result.fn_count, 0,
        "mod.ash should have 0 direct pub fn definitions, got {}",
        result.fn_count,
    );
}

// ---------------------------------------------------------------------------
// dispatch.ash: workflows, not pub fns
// ---------------------------------------------------------------------------

#[test]
fn test_dispatch_ash_has_no_pub_fns() {
    let source = read_stdlib_file("llm/dispatch.ash");
    let (count, diagnostics) = count_pub_fn_snippets(&source);

    // dispatch.ash uses `workflow` declarations, not `pub fn`
    assert_eq!(
        count, 0,
        "dispatch.ash should have 0 pub fn definitions (uses workflow), got {}",
        count,
    );
    assert!(
        diagnostics.is_empty(),
        "dispatch.ash should have zero pub fn diagnostics: {:?}",
        diagnostics,
    );
}

// ---------------------------------------------------------------------------
// Cross-cutting: all LLM stdlib files check without fatal errors
// ---------------------------------------------------------------------------

#[test]
fn test_all_llm_stdlib_files_check_without_fatal_errors() {
    let engine = make_engine();
    let llm_dir = stdlib_root().join("llm");
    let entries = std::fs::read_dir(&llm_dir).expect("llm directory must exist");

    let mut checked = 0;
    for entry in entries.flatten() {
        let path = entry.path();
        if path.extension().map_or(false, |e| e == "ash") {
            let file_name = path.file_name().unwrap().to_string_lossy();

            // Skip temp test files
            if file_name.starts_with('_') {
                continue;
            }

            let result = engine
                .check_module_file(&path)
                .unwrap_or_else(|e| panic!("check_module_file failed for {file_name}: {e}"));

            // types.ash: Float is now builtin (TASK-545), expect 0 errors
            if file_name == "types.ash" {
                assert_eq!(
                    result.errors.len(),
                    0,
                    "types.ash: expected 0 errors, got {:?}",
                    result.errors,
                );
            } else {
                assert!(
                    result.errors.is_empty(),
                    "{}: unexpected errors: {:?}",
                    file_name,
                    result.errors,
                );
            }

            // prompt.ash has known pub fn parse limitations -- warnings expected
            if file_name == "prompt.ash" {
                assert!(
                    !result.warnings.is_empty(),
                    "prompt.ash: expected pub fn parse warnings, got none",
                );
            } else {
                assert!(
                    result.warnings.is_empty(),
                    "{}: unexpected warnings: {:?}",
                    file_name,
                    result.warnings,
                );
            }

            checked += 1;
        }
    }
    assert!(
        checked >= 10,
        "should have checked at least 10 .ash files in llm/, found {checked}",
    );
}
