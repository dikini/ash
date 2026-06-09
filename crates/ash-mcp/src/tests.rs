use super::*;
use std::io::Write;

fn ash_source() -> &'static str {
    "fn helper() -> Int { 1 }\ncapability sensor: epistemic()\nworkflow main { observe sensor done }"
}

fn write_temp_ash(content: &str) -> tempfile::NamedTempFile {
    let mut f = tempfile::Builder::new()
        .suffix(".ash")
        .tempfile()
        .expect("temp file");
    f.write_all(content.as_bytes()).expect("write");
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
