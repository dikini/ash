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

// -- ash_find_references (placeholder) --

#[test]
fn test_find_references_placeholder() {
    let f = write_temp_ash(ash_source());
    let path = f.path().to_str().expect("path");
    let s = server();
    let result = s.ash_find_references(Parameters(PositionParams {
        file: path.to_string(),
        line: 1,
        column: 1,
    }));
    let text = extract_text(&result);
    assert!(
        text.contains("not yet implemented"),
        "expected placeholder message, got: {text}"
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
    // Should mention at least one known tool name
    assert!(
        text.contains("ash_get_diagnostics") || text.contains("ash_hover"),
        "health tool should list available tool names, got: {text}"
    );
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
