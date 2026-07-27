//! End-to-end validation of LLM stdlib module files (TASK-543).
//!
//! Uses structural engine APIs (`check_module_file`, `ModuleFile` type metadata lowering, and
//! `count_pub_fn_snippets`) instead of string-matching. Validates SPEC-030 §3.5, §4.4, §5.4.
//!
//! Key finding: prompt.ash has 27 `pub fn` declarations. Earlier parser slices
//! raised coverage gradually; after keyword-field handling, `role` -> `sender`,
//! and list-literal cleanup, all 27 prompt helpers now parse with zero snippet
//! diagnostics. This test keeps that stdlib helper surface from regressing.

use ash_core::module_graph::ModuleId;
use ash_core::semantic_summary::{ModuleIdentity, ModuleSourceOrigin};
use ash_engine::Engine;
use ash_engine::module_loader::count_pub_fn_snippets;
use ash_parser::lower::lower_module_type_metadata;
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

/// Creates an isolated stdlib-shaped fixture for import-resolution tests.
///
/// The resolver searches the consumer's directory before the canonical stdlib
/// root, so a sibling `llm/` tree preserves the `use llm::...` contract without
/// adding transient files to the tracked `std/src` corpus.
fn llm_fixture_consumer(name: &str, source: &str) -> (tempfile::TempDir, PathBuf) {
    let fixture = tempfile::tempdir().expect("LLM fixture directory creates");
    let fixture_llm = fixture.path().join("llm");
    std::fs::create_dir(&fixture_llm).expect("LLM fixture module directory creates");

    let source_llm = stdlib_root().join("llm");
    for entry in std::fs::read_dir(&source_llm).expect("canonical LLM stdlib directory reads") {
        let entry = entry.expect("canonical LLM stdlib directory entry reads");
        let source = entry.path();
        if source
            .extension()
            .is_some_and(|extension| extension == "ash")
        {
            let destination = fixture_llm.join(entry.file_name());
            std::fs::copy(&source, &destination).unwrap_or_else(|error| {
                panic!(
                    "copy LLM stdlib fixture '{}' to '{}': {error}",
                    source.display(),
                    destination.display()
                )
            });
        }
    }

    let consumer = fixture.path().join(name);
    std::fs::write(&consumer, source).expect("write isolated LLM consumer");
    (fixture, consumer)
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
    let module =
        ash_parser::parse_surface_file(&source).unwrap_or_else(|errors| panic!("{errors:?}"));
    let type_defs = lower_module_type_metadata(
        &module,
        ModuleIdentity::new(
            None,
            ModuleId(543),
            vec!["std".to_string(), "llm".to_string(), "types".to_string()],
            ModuleSourceOrigin::File("std/src/llm/types.ash".to_string()),
        ),
    )
    .type_defs
    .into_iter()
    .filter(|type_def| matches!(type_def.visibility, ash_core::ast::Visibility::Public))
    .collect::<Vec<_>>();

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
// Requirement 3: full pub fn export coverage from prompt.ash
//
// All 27 prompt helpers now parse with no diagnostics. Keep this regression
// anchored here because LLM stdlib helper syntax has historically drifted.
// ---------------------------------------------------------------------------

#[test]
fn test_prompt_ash_pub_fn_full_parse_coverage() {
    let source = read_stdlib_file("llm/prompt.ash");
    let (count, diagnostics) = count_pub_fn_snippets(&source);

    // TASK-546: constructor field names now allow keywords.
    // TASK-548: added 4 new functions (append_response, append_tool_result,
    // is_final, render_template).
    // TASK-549: renamed Message field `role` -> `sender` to avoid keyword
    // collision, unblocking match patterns and parameter names.
    // Removed unnecessary list literal wrappers from append/append_tool_result/
    // append_response. Now 27 of 27 pub fns parse.
    assert_eq!(
        count, 27,
        "expected exactly 27 parseable pub fns from prompt.ash (regression?), got {count}",
    );
    assert!(
        diagnostics.is_empty(),
        "expected 0 diagnostics (all 27 pub fns parse), got {diagnostics:?}",
    );
}

/// All 27 pub fns in prompt.ash now parse after role->sender rename and
/// removal of list literal wrappers. Previously #[ignore] target test.
#[test]
fn test_prompt_ash_all_27_pub_fns_parse() {
    let source = read_stdlib_file("llm/prompt.ash");
    let (count, diagnostics) = count_pub_fn_snippets(&source);

    assert_eq!(
        count, 27,
        "all 27 pub fns should parse once parser supports record constructors, got {count}",
    );
    assert!(
        diagnostics.is_empty(),
        "no diagnostics expected: {diagnostics:?}",
    );
}

// ---------------------------------------------------------------------------
// Requirement 2: import path resolution
// ---------------------------------------------------------------------------

#[test]
fn test_llm_types_import_resolves() {
    let (_fixture, consumer) = llm_fixture_consumer(
        "consumer.ash",
        "use llm::types::Role;\nuse llm::types::Message;\nfn main() { {} }\n",
    );

    let engine = make_engine();
    let result = engine.parse_file(&consumer);

    assert!(
        result.is_ok(),
        "use llm::types::Role should resolve: {result:?}",
    );
}

// ---------------------------------------------------------------------------
// Requirement 2b: re-export import path (use llm::Role via mod.ash pub use)
//
// TASK-547: `use llm::Role` now resolves from within std/src/ because
// collect_module_exports gracefully skips application parse failures in
// child modules (dispatch.ash etc.), so re-exports from mod.ash survive.
// ---------------------------------------------------------------------------

#[test]
fn test_llm_reexport_role_resolves() {
    let (_fixture, consumer) =
        llm_fixture_consumer("reexport_role.ash", "use llm::Role;\nfn main() { {} }\n");

    let engine = make_engine();
    let result = engine.parse_file(&consumer);

    assert!(
        result.is_ok(),
        "use llm::Role should resolve via mod.ash re-export: {result:?}",
    );
}

#[test]
fn test_llm_reexport_message_resolves() {
    let (_fixture, consumer) = llm_fixture_consumer(
        "reexport_message.ash",
        "use llm::Message;\nfn main() { {} }\n",
    );

    let engine = make_engine();
    let result = engine.parse_file(&consumer);

    assert!(
        result.is_ok(),
        "use llm::Message should resolve via mod.ash re-export: {result:?}",
    );
}

// ---------------------------------------------------------------------------
// TASK-547: bad import gives a clear error (not opaque "ContextError")
// ---------------------------------------------------------------------------

#[test]
fn test_bad_import_gives_clear_error() {
    let (_fixture, consumer) = llm_fixture_consumer(
        "bad_import.ash",
        "use nonexistent::Foo;\nfn main() { {} }\n",
    );

    let engine = make_engine();
    let result = engine.parse_file(&consumer);

    assert!(result.is_err(), "use nonexistent::Foo should fail");
    let err_msg = format!("{:?}", result.unwrap_err());
    assert!(
        err_msg.contains("not found"),
        "error should mention 'not found', got: {err_msg}",
    );
}

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
// dispatch.ash: target helper functions
// ---------------------------------------------------------------------------

#[test]
fn test_dispatch_ash_pub_fn_count_matches_current_helpers() {
    let source = read_stdlib_file("llm/dispatch.ash");
    let (count, diagnostics) = count_pub_fn_snippets(&source);

    assert_eq!(
        count, 2,
        "dispatch.ash should have exactly 2 target pub fn helpers, got {count}",
    );
    assert!(
        diagnostics.is_empty(),
        "dispatch.ash should have zero pub fn diagnostics: {diagnostics:?}",
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
        if path.extension().is_some_and(|e| e == "ash") {
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

            // prompt.ash: all 27 pub fns now parse cleanly. supervised.ash keeps warnings for
            // parseable ModuleFile helpers whose bodies use richer expression syntax than the
            // narrow helper counter.
            let tolerated_supervised_warning = file_name == "supervised.ash"
                && result.warnings.len() == 2
                && result
                    .warnings
                    .iter()
                    .any(|warning| warning.contains("parse_supervisor_response"))
                && result
                    .warnings
                    .iter()
                    .any(|warning| warning.contains("format_tool_calls_for_review"));
            let tolerated_router_warnings = file_name == "router.ash"
                && !result.warnings.is_empty()
                && result.warnings.iter().all(|warning| {
                    warning.contains("parse_route") || warning.contains("select_model")
                });
            assert!(
                result.warnings.is_empty()
                    || tolerated_supervised_warning
                    || tolerated_router_warnings,
                "{}: unexpected warnings: {:?}",
                file_name,
                result.warnings,
            );

            checked += 1;
        }
    }
    assert!(
        checked >= 9,
        "should have checked all current .ash files in llm/, found {checked}",
    );
}

#[test]
fn test_supervised_helpers_preserve_rejection_feedback_and_tool_call_details() {
    let source = read_stdlib_file("llm/supervised.ash");

    assert!(
        source.contains("Reject { feedback: text }"),
        "supervised rejection should preserve model-provided feedback"
    );
    assert!(
        source.contains("No tool calls awaiting approval"),
        "tool-call review should handle the empty list explicitly"
    );
    assert!(
        source.contains("First call id:")
            && source.contains("First call name:")
            && source.contains("First call arguments:"),
        "tool-call review should include meaningful first-call details"
    );
}
