use super::*;
use std::io::Write;

fn ash_source() -> &'static str {
    "fn helper() -> Int { 1 }\ncapability sensor: epistemic()\nworkflow main { observe sensor done }"
}

fn write_temp_ash(content: &str) -> tempfile::NamedTempFile {
    let mut f = tempfile::Builder::new()
        .suffix(".ash")
        .tempfile()
        .expect("create temp .ash file");
    f.write_all(content.as_bytes())
        .expect("write temp .ash content");
    f
}

fn extract_text(result: &CallToolResult) -> String {
    result
        .content
        .iter()
        .filter_map(|c| c.as_text().map(|t| t.text.clone()))
        .collect()
}

fn extract_json(result: &CallToolResult) -> Vec<serde_json::Value> {
    result
        .content
        .iter()
        .filter_map(|c| c.as_text().and_then(|t| serde_json::from_str(&t.text).ok()))
        .collect()
}

fn server() -> AshMcpServer {
    AshMcpServer::new()
}

// -- ash_get_diagnostics --

#[test]
fn test_diagnostics_clean_workflow() {
    let f = write_temp_ash(ash_source());
    let path = f.path().to_str().expect("path");
    let s = server();
    let result = s.ash_get_diagnostics(Parameters(FileParams {
        file: path.to_string(),
    }));
    assert!(
        result.is_error.is_none() || result.is_error == Some(false),
        "should not be an error response"
    );
    // The content should contain a summary mentioning "No issues" or diagnostics
    let text = extract_text(&result);
    assert!(
        text.contains("No issues"),
        "expected clean summary, got: {text}"
    );
}

#[test]
fn test_diagnostics_bad_file() {
    let s = server();
    let result = s.ash_get_diagnostics(Parameters(FileParams {
        file: "/nonexistent/path.ash".to_string(),
    }));
    assert_eq!(
        result.is_error,
        Some(true),
        "should be error for missing file"
    );
}

// -- ash_document_symbols --

#[test]
fn test_symbols_returns_entries() {
    let f = write_temp_ash(ash_source());
    let path = f.path().to_str().expect("path");
    let s = server();
    let result = s.ash_document_symbols(Parameters(FileParams {
        file: path.to_string(),
    }));
    let text = extract_text(&result);
    // Should have at least one symbol
    assert!(
        text.contains("symbol(s)"),
        "expected symbol summary, got: {text}"
    );
}

// -- ash_hover --

#[test]
fn test_hover_on_keyword() {
    let f = write_temp_ash(ash_source());
    let path = f.path().to_str().expect("path");
    let s = server();
    let result = s.ash_hover(Parameters(PositionParams {
        file: path.to_string(),
        line: 3,
        column: 11, // "observe" keyword area
    }));
    let text = extract_text(&result);
    // Hover should return something (info or "No hover")
    assert!(
        text.contains("Hover info") || text.contains("No hover info"),
        "expected hover response, got: {text}"
    );
}

// -- ash_goto_definition --

#[test]
fn test_goto_definition_on_workflow() {
    let f = write_temp_ash(ash_source());
    let path = f.path().to_str().expect("path");
    let s = server();
    // "main" starts at column 10 on line 3
    let result = s.ash_goto_definition(Parameters(PositionParams {
        file: path.to_string(),
        line: 3,
        column: 10,
    }));
    let text = extract_text(&result);
    assert!(
        text.contains("Definition") || text.contains("No definition"),
        "expected goto response, got: {text}"
    );
}

// -- ash_complete --

#[test]
fn test_completion_returns_items() {
    let f = write_temp_ash(ash_source());
    let path = f.path().to_str().expect("path");
    let s = server();
    let result = s.ash_complete(Parameters(PositionParams {
        file: path.to_string(),
        line: 1,
        column: 1,
    }));
    let text = extract_text(&result);
    assert!(
        text.contains("completion(s)"),
        "expected completion summary, got: {text}"
    );
    // Should have at least some completions (keywords)
    assert!(
        !text.contains("0 completion(s)"),
        "expected non-zero completions"
    );
}

// -- ash_find_references --

#[test]
fn test_find_references_finds_definition_and_call() {
    let source = "fn helper() -> Int { 1 }\nworkflow main { let x = helper() done }\n";
    let f = write_temp_ash(source);
    let path = f.path().to_str().expect("path");
    let s = server();
    // "helper" on line 2 (1-indexed): "workflow main { let x = helper() done }"
    // Count columns: "workflow main { let x = " = 24 chars, so helper starts at col 25
    let result = s.ash_find_references(Parameters(PositionParams {
        file: path.to_string(),
        line: 2,
        column: 25,
    }));
    let text = extract_text(&result);
    assert!(
        text.contains("2 reference(s)"),
        "expected 2 references (def + call), got: {text}"
    );
}

#[test]
fn test_find_references_empty_honest() {
    let f = write_temp_ash("workflow main { done }\n");
    let path = f.path().to_str().expect("path");
    let s = server();
    // Position on whitespace — no identifier
    let result = s.ash_find_references(Parameters(PositionParams {
        file: path.to_string(),
        line: 1,
        column: 15, // space between "main" and "{"
    }));
    let text = extract_text(&result);
    assert!(
        text.contains("No references found"),
        "expected empty honest summary, got: {text}"
    );
}

// -- ash_workspace_symbols --

#[test]
fn test_workspace_symbols_finds_match() {
    let dir = tempfile::tempdir().expect("temp dir");
    std::fs::write(dir.path().join("lib.ash"), "fn helper() -> Int { 1 }\n").unwrap();

    let s = server();
    let result = s.ash_workspace_symbols(Parameters(WorkspaceSymbolParams {
        root: dir.path().to_str().expect("path").to_string(),
        query: "helper".to_string(),
    }));
    let text = extract_text(&result);
    assert!(
        text.contains("1 symbol(s) matching 'helper'"),
        "expected workspace symbol summary, got: {text}"
    );
}

#[test]
fn test_workspace_symbols_empty_honest() {
    let dir = tempfile::tempdir().expect("temp dir");
    std::fs::write(dir.path().join("lib.ash"), "fn helper() -> Int { 1 }\n").unwrap();

    let s = server();
    let result = s.ash_workspace_symbols(Parameters(WorkspaceSymbolParams {
        root: dir.path().to_str().expect("path").to_string(),
        query: "missing".to_string(),
    }));
    let text = extract_text(&result);
    assert!(
        text.contains("No symbols matching 'missing'"),
        "expected empty summary, got: {text}"
    );
}

// -- VFS auto-open (SPEC-038 §8.5) --

#[test]
fn test_vfs_auto_opens_file() {
    let f = write_temp_ash(ash_source());
    let path = f.path().to_str().expect("path");
    let s = server();
    // File not yet in VFS
    assert!(
        s.vfs.get(&AshMcpServer::file_uri(path).unwrap()).is_none(),
        "should not be open yet"
    );
    // Calling a tool should auto-open it
    let _ = s.ash_document_symbols(Parameters(FileParams {
        file: path.to_string(),
    }));
    assert!(
        s.vfs.get(&AshMcpServer::file_uri(path).unwrap()).is_some(),
        "should be auto-opened after tool call"
    );
}

#[test]
fn test_vfs_caches_across_calls() {
    let f = write_temp_ash(ash_source());
    let path = f.path().to_str().expect("path");
    let s = server();
    let _ = s.ash_get_diagnostics(Parameters(FileParams {
        file: path.to_string(),
    }));
    let _ = s.ash_document_symbols(Parameters(FileParams {
        file: path.to_string(),
    }));
    // Second call should use cached version (same VFS entry version)
    let entry = s.vfs.get(&AshMcpServer::file_uri(path).unwrap()).unwrap();
    assert_eq!(entry.version, 0, "should be version 0 from initial open");
}

// -- ash_mcp_health --

#[test]
fn test_health_tool_returns_status() {
    let s = server();
    let result = s.ash_mcp_health();
    assert!(
        result.is_error.is_none() || result.is_error == Some(false),
        "health tool should not return an error"
    );
}

#[test]
fn test_health_tool_contains_version() {
    let s = server();
    let result = s.ash_mcp_health();
    let text = extract_text(&result);
    let expected_version = env!("CARGO_PKG_VERSION");
    assert!(
        text.contains(expected_version),
        "health tool should include workspace version {expected_version}, got: {text}"
    );
}

#[test]
fn test_health_tool_contains_tool_names() {
    let s = server();
    let result = s.ash_mcp_health();
    let text = extract_text(&result);
    for tool in [
        "ash_get_diagnostics",
        "ash_hover",
        "ash_goto_definition",
        "ash_complete",
        "ash_find_rust_implementation",
        "ash_find_ash_usage",
        "ash_mcp_health",
    ] {
        assert!(text.contains(tool), "health missing tool {tool}: {text}");
    }
}

#[test]
fn test_find_rust_implementation_effect_positive() {
    let s = server();
    let result = s.find_rust_implementation_tool(
        "Effect".to_string(),
        "std/src/types.ash".to_string(),
        1,
        1,
    );

    assert!(result.is_error.is_none() || result.is_error == Some(false));
    let payloads = extract_json(&result);
    let payload = payloads.first().expect("json payload");
    assert_eq!(payload["found"], true);
    assert_eq!(payload["rust_symbol"], "ash_core::effect::Effect");
    assert!(
        payload["file"]
            .as_str()
            .is_some_and(|file| file.ends_with("crates/ash-core/src/effect.rs")),
        "unexpected file payload: {payload}"
    );
    assert!(payload["start_line"].as_u64().unwrap_or_default() > 0);
    assert!(payload["start_column"].as_u64().unwrap_or_default() > 0);
}

#[test]
fn test_find_ash_usage_effect_positive() {
    let s = server();
    let result = s.find_ash_usage_tool("ash_core::effect::Effect".to_string());

    assert!(result.is_error.is_none() || result.is_error == Some(false));
    let payloads = extract_json(&result);
    let payload = payloads.first().expect("json payload");
    assert_eq!(payload["rust_symbol"], "ash_core::effect::Effect");
    assert!(
        payload["usages"]
            .as_array()
            .is_some_and(|usages| !usages.is_empty()),
        "expected at least one Ash usage: {payload}"
    );
}

#[test]
fn test_find_rust_implementation_qualified_variant_normalizes_to_type() {
    let s = server();
    let result = s.find_rust_implementation_tool(
        "Effect::Epistemic".to_string(),
        "std/src/types.ash".to_string(),
        1,
        1,
    );

    let payloads = extract_json(&result);
    let payload = payloads.first().expect("json payload");
    assert_eq!(payload["found"], true, "qualified lookup failed: {payload}");
    assert_eq!(payload["rust_symbol"], "ash_core::effect::Effect");
}

#[test]
fn test_find_rust_implementation_namespace_qualified_symbol_uses_terminal_fallback() {
    let s = server();
    let result = s.find_rust_implementation_tool(
        "std::types::Effect".to_string(),
        "std/src/types.ash".to_string(),
        1,
        1,
    );

    let payloads = extract_json(&result);
    let payload = payloads.first().expect("json payload");
    assert_eq!(
        payload["found"], true,
        "namespace-qualified lookup failed: {payload}"
    );
    assert_eq!(payload["rust_symbol"], "ash_core::effect::Effect");
}

#[test]
fn test_workspace_root_for_unrelated_absolute_file_does_not_fall_back_to_server_cwd() {
    let temp = tempfile::tempdir().expect("temp dir");
    let scratch = temp.path().join("scratch.ash");
    std::fs::write(
        &scratch,
        "type Effect = String
",
    )
    .expect("scratch file");

    let root = AshMcpServer::workspace_root_for_file(scratch.to_str().expect("utf8 path"));
    assert_eq!(root, temp.path());
    assert!(
        !root.join(".ash/cross_lang_config.yaml").exists(),
        "scratch root should not inherit server cwd config"
    );
}

#[test]
fn test_find_ash_mapping_prefers_exact_and_parent_matches_before_segment_fallback() {
    fn mapping(ash_symbol: &str, rust_symbol: &str) -> cross_lang::SymbolMapping {
        cross_lang::SymbolMapping {
            ash_symbol: ash_symbol.to_string(),
            ash_kind: "type".to_string(),
            rust_symbol: rust_symbol.to_string(),
            rust_kind: "enum".to_string(),
            confidence: cross_lang::ConfidenceLevel::High,
            source: cross_lang::MappingSource::Manual,
        }
    }

    let mappings = vec![
        mapping("Effect", "ash_core::effect::Effect"),
        mapping("std::types::Effect", "ash_std::types::Effect"),
    ];

    let exact =
        AshMcpServer::find_ash_mapping(&mappings, "std::types::Effect").expect("exact mapping");
    assert_eq!(exact.rust_symbol, "ash_std::types::Effect");

    let parent = AshMcpServer::find_ash_mapping(&mappings, "std::types::Effect::Epistemic")
        .expect("qualified parent mapping");
    assert_eq!(parent.rust_symbol, "ash_std::types::Effect");

    assert!(
        AshMcpServer::find_ash_mapping(&mappings, "Effect::namespace::Thing").is_none(),
        "fallback should not match arbitrary namespace/container segments"
    );
}

#[test]
fn test_mask_ash_non_code_regions_handles_escaped_quotes_in_strings() {
    let mut in_block_comment = false;
    let masked = AshMcpServer::mask_ash_non_code(
        r#"let message = "quoted \" Effect remains a string"; type Effect = String"#,
        &mut in_block_comment,
    );

    assert_eq!(masked.matches("Effect").count(), 1, "masked line: {masked}");
}

#[test]
fn test_cross_lang_config_root_expands_from_crate_subdirectory() {
    let crate_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let root = AshMcpServer::cross_lang_config_root_for_root(crate_dir);
    assert!(
        root.join(".ash/cross_lang_config.yaml").exists(),
        "expected workspace config root, got {}",
        root.display()
    );
}

#[test]
fn test_associated_item_detection_distinguishes_nested_modules() {
    let associated_parts = ["ash_core", "effect", "Effect", "join"];
    assert!(AshMcpServer::should_search_associated_item(
        std::path::Path::new("crates/ash-core/src/effect.rs"),
        &associated_parts,
    ));

    let nested_parts = ["my_crate", "outer", "inner", "Widget"];
    assert!(!AshMcpServer::should_search_associated_item(
        std::path::Path::new("crates/my-crate/src/outer/inner.rs"),
        &nested_parts,
    ));
}

#[test]
fn test_find_rust_symbol_location_resolves_associated_method_file() {
    let workspace = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(std::path::Path::parent)
        .expect("workspace root");
    let location =
        AshMcpServer::find_rust_symbol_location(workspace, "ash_core::effect::Effect::join")
            .expect("lookup should not error")
            .expect("join method should be found");

    assert!(
        location.file.ends_with("crates/ash-core/src/effect.rs"),
        "unexpected file: {}",
        location.file.display()
    );
}

#[test]
fn test_find_rust_implementation_namespace_qualified_variant_normalizes_to_type() {
    let s = server();
    let result = s.find_rust_implementation_tool(
        "std::types::Effect::Epistemic".to_string(),
        "std/src/types.ash".to_string(),
        1,
        1,
    );

    let payloads = extract_json(&result);
    let payload = payloads.first().expect("json payload");
    assert_eq!(
        payload["found"], true,
        "namespace-qualified variant lookup failed: {payload}"
    );
    assert_eq!(payload["rust_symbol"], "ash_core::effect::Effect");
}

#[test]
fn test_find_ash_usage_reports_all_token_matches_without_substrings() {
    let mut usages = Vec::new();
    let workspace = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(std::path::Path::parent)
        .expect("workspace root");
    AshMcpServer::scan_ash_usages(
        &workspace.join("crates/ash-mcp/tests/fixtures/effect_usage.ash"),
        "Effect",
        &mut usages,
    );

    assert!(
        usages
            .iter()
            .any(|usage| usage.line == 14 && usage.column == 21),
        "expected Effect return type usage: {usages:?}"
    );
    assert!(
        usages
            .iter()
            .any(|usage| usage.line == 15 && usage.column == 22),
        "expected second Effect same fixture usage: {usages:?}"
    );
    assert!(
        usages.iter().all(|usage| usage.ash_symbol == "Effect"),
        "unexpected usage symbols: {usages:?}"
    );
    assert!(
        usages.iter().all(|usage| usage.line != 4
            && usage.line != 5
            && usage.line != 6
            && usage.line != 9
            && usage.line != 10),
        "comments and strings must not be reported as usages: {usages:?}"
    );
}

#[test]
fn test_find_ash_usage_honors_configured_extensions() {
    let temp = tempfile::tempdir().expect("tempdir");
    let source = temp.path().join("module.ashx");
    std::fs::write(&source, "type Effect = String\n").expect("write fixture");

    let usages =
        AshMcpServer::find_ash_usages_for_symbol(temp.path(), "Effect", &[".ashx".to_string()]);

    assert_eq!(usages.len(), 1, "expected .ashx usage: {usages:?}");
    assert!(usages[0].file.ends_with("module.ashx"));
    assert_eq!(usages[0].line, 1);
    assert_eq!(usages[0].column, 6);
}

#[test]
fn test_health_tool_contains_ok() {
    let s = server();
    let result = s.ash_mcp_health();
    let text = extract_text(&result);
    assert!(
        text.to_lowercase().contains("ok") || text.to_lowercase().contains("status"),
        "health tool should report status, got: {text}"
    );
}
