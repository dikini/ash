use super::*;
use futures::StreamExt;
use serde_json::json;
use tower::{Service, ServiceExt};
use tower_lsp_server::LspService;
use tower_lsp_server::jsonrpc::Request;
use tower_lsp_server::ls_types::{
    DocumentSymbolResponse, HoverContents, MarkupKind, PublishDiagnosticsParams,
};

fn initialize_request(id: i64) -> Request {
    Request::build("initialize")
        .params(json!({"capabilities": {}}))
        .id(id)
        .finish()
}

async fn initialized_service() -> (
    tower_lsp_server::LspService<AshLanguageServer>,
    tower_lsp_server::ClientSocket,
) {
    let (mut service, socket) = LspService::new(AshLanguageServer::new);
    let response = service
        .ready()
        .await
        .expect("service ready")
        .call(initialize_request(1))
        .await
        .expect("initialize response transport ok")
        .expect("initialize response exists");
    assert!(response.is_ok(), "initialize should succeed: {response:?}");
    (service, socket)
}

fn open_request(uri: &str, version: i32, text: &str) -> Request {
    Request::build("textDocument/didOpen")
        .params(json!({
            "textDocument": {
                "uri": uri,
                "languageId": "ash",
                "version": version,
                "text": text
            }
        }))
        .finish()
}

fn change_request(uri: &str, version: i32, text: &str) -> Request {
    Request::build("textDocument/didChange")
        .params(json!({
            "textDocument": { "uri": uri, "version": version },
            "contentChanges": [{ "text": text }]
        }))
        .finish()
}

fn close_request(uri: &str) -> Request {
    Request::build("textDocument/didClose")
        .params(json!({
            "textDocument": { "uri": uri }
        }))
        .finish()
}

fn hover_request(id: i64, uri: &str, line: u32, character: u32) -> Request {
    Request::build("textDocument/hover")
        .params(json!({
            "textDocument": { "uri": uri },
            "position": { "line": line, "character": character }
        }))
        .id(id)
        .finish()
}

fn document_symbol_request(id: i64, uri: &str) -> Request {
    Request::build("textDocument/documentSymbol")
        .params(json!({
            "textDocument": { "uri": uri }
        }))
        .id(id)
        .finish()
}

#[tokio::test(flavor = "current_thread")]
async fn did_open_publishes_diagnostics_notification() {
    let (mut service, mut socket) = initialized_service().await;
    let uri = "file:///test.ash";

    let response = service
        .ready()
        .await
        .expect("service ready")
        .call(open_request(uri, 1, "workflow main { orient 1 done }"))
        .await
        .expect("didOpen transport ok");
    assert_eq!(response, None, "didOpen is a notification");

    let outbound = socket
        .next()
        .await
        .expect("publishDiagnostics notification");
    assert_eq!(outbound.method(), "textDocument/publishDiagnostics");
    let params_value = outbound.params().cloned().expect("diagnostic params");
    let params: PublishDiagnosticsParams =
        serde_json::from_value(params_value).expect("decode diagnostics");
    assert_eq!(params.uri.to_string(), uri);
    assert!(
        !params.diagnostics.is_empty(),
        "orient-only workflow should emit L001"
    );
}

#[tokio::test(flavor = "current_thread")]
async fn hover_request_returns_markdown() {
    let (mut service, _socket) = initialized_service().await;
    let uri = "file:///hover.ash";

    let _ = service
        .ready()
        .await
        .expect("service ready")
        .call(open_request(uri, 1, "workflow main { done }"))
        .await
        .expect("didOpen transport ok");

    let response = service
        .ready()
        .await
        .expect("service ready")
        .call(hover_request(2, uri, 0, 1))
        .await
        .expect("hover transport ok")
        .expect("hover response exists");

    assert!(
        response.is_ok(),
        "hover response should be ok: {response:?}"
    );
    let result = response.result().cloned().expect("hover result");
    let hover: Hover = serde_json::from_value(result).expect("decode hover");
    match hover.contents {
        HoverContents::Markup(markup) => {
            assert_eq!(markup.kind, MarkupKind::Markdown);
            assert!(markup.value.contains("workflow <name>"));
        }
        other => panic!("expected markdown hover, got {other:?}"),
    }
}

#[tokio::test(flavor = "current_thread")]
async fn document_symbol_request_returns_symbols() {
    let (mut service, _socket) = initialized_service().await;
    let uri = "file:///symbols.ash";
    let source = "fn helper() -> Int { 1 }\nworkflow main { done }";

    let _ = service
        .ready()
        .await
        .expect("service ready")
        .call(open_request(uri, 1, source))
        .await
        .expect("didOpen transport ok");

    let response = service
        .ready()
        .await
        .expect("service ready")
        .call(document_symbol_request(3, uri))
        .await
        .expect("documentSymbol transport ok")
        .expect("documentSymbol response exists");

    assert!(
        response.is_ok(),
        "documentSymbol response should be ok: {response:?}"
    );
    let result = response.result().cloned().expect("documentSymbol result");
    let symbols: DocumentSymbolResponse = serde_json::from_value(result).expect("decode symbols");
    match symbols {
        DocumentSymbolResponse::Nested(items) => {
            assert!(!items.is_empty(), "expected at least one symbol");
            assert!(items.iter().any(|s| s.name == "helper"));
            assert!(items.iter().any(|s| s.name == "main"));
        }
        DocumentSymbolResponse::Flat(items) => {
            assert!(!items.is_empty(), "expected at least one symbol");
            assert!(items.iter().any(|s| s.name == "helper"));
            assert!(items.iter().any(|s| s.name == "main"));
        }
    }
}

#[tokio::test(flavor = "current_thread")]
async fn did_close_clears_diagnostics() {
    let (mut service, mut socket) = initialized_service().await;
    let uri = "file:///close.ash";

    let _ = service
        .ready()
        .await
        .expect("service ready")
        .call(open_request(uri, 1, "workflow main { orient 1 done }"))
        .await
        .expect("didOpen transport ok");
    let _first = socket.next().await.expect("initial diagnostics");

    let response = service
        .ready()
        .await
        .expect("service ready")
        .call(close_request(uri))
        .await
        .expect("didClose transport ok");
    assert_eq!(response, None, "didClose is a notification");

    let outbound = socket.next().await.expect("clear diagnostics notification");
    assert_eq!(outbound.method(), "textDocument/publishDiagnostics");
    let params_value = outbound.params().cloned().expect("diagnostic params");
    let params: PublishDiagnosticsParams =
        serde_json::from_value(params_value).expect("decode diagnostics");
    assert_eq!(params.uri.to_string(), uri);
    assert!(
        params.diagnostics.is_empty(),
        "didClose should clear diagnostics"
    );
}

#[tokio::test(flavor = "current_thread")]
async fn did_change_republishes_updated_diagnostics() {
    let (mut service, mut socket) = initialized_service().await;
    let uri = "file:///change.ash";

    let _ = service
        .ready()
        .await
        .expect("service ready")
        .call(open_request(
            uri,
            1,
            "workflow main { observe sensor done }",
        ))
        .await
        .expect("didOpen transport ok");
    let first = socket.next().await.expect("initial diagnostics");
    let first_params: PublishDiagnosticsParams =
        serde_json::from_value(first.params().cloned().expect("initial params"))
            .expect("decode initial diagnostics");

    let _ = service
        .ready()
        .await
        .expect("service ready")
        .call(change_request(uri, 2, "workflow main { orient 1 done }"))
        .await
        .expect("didChange transport ok");
    let second = socket.next().await.expect("updated diagnostics");
    let second_params: PublishDiagnosticsParams =
        serde_json::from_value(second.params().cloned().expect("updated params"))
            .expect("decode updated diagnostics");

    assert!(
        first_params.diagnostics.is_empty(),
        "baseline workflow should have no diagnostics"
    );
    assert!(
        !second_params.diagnostics.is_empty(),
        "updated workflow should emit diagnostics"
    );
}

fn goto_definition_request(id: i64, uri: &str, line: u32, character: u32) -> Request {
    Request::build("textDocument/definition")
        .params(json!({
            "textDocument": { "uri": uri },
            "position": { "line": line, "character": character }
        }))
        .id(id)
        .finish()
}

fn completion_request(id: i64, uri: &str, line: u32, character: u32) -> Request {
    Request::build("textDocument/completion")
        .params(json!({
            "textDocument": { "uri": uri },
            "position": { "line": line, "character": character }
        }))
        .id(id)
        .finish()
}

#[tokio::test(flavor = "current_thread")]
async fn goto_definition_returns_location() {
    let (mut service, _socket) = initialized_service().await;
    let uri = "file:///goto.ash";
    let source = "fn helper() -> Int { 1 }\nworkflow main { done }";

    let _ = service
        .ready()
        .await
        .expect("service ready")
        .call(open_request(uri, 1, source))
        .await
        .expect("didOpen transport ok");

    // Cursor on "main" in workflow declaration (line 1, col 9)
    let response = service
        .ready()
        .await
        .expect("service ready")
        .call(goto_definition_request(2, uri, 1, 9))
        .await
        .expect("goto_definition transport ok")
        .expect("goto_definition response exists");

    assert!(
        response.is_ok(),
        "goto_definition response should be ok: {response:?}"
    );
    let result = response.result().cloned().expect("goto_definition result");
    // Should return a Location pointing to the workflow definition
    assert!(result.is_object(), "result should be an object");
    let result_str = serde_json::to_string(&result).expect("serialize");
    assert!(result_str.contains("range"), "should contain range field");
}

#[tokio::test(flavor = "current_thread")]
async fn goto_definition_returns_none_for_unknown() {
    let (mut service, _socket) = initialized_service().await;
    let uri = "file:///goto-unknown.ash";

    let _ = service
        .ready()
        .await
        .expect("service ready")
        .call(open_request(uri, 1, "workflow main { done }"))
        .await
        .expect("didOpen transport ok");

    // Cursor on whitespace
    let response = service
        .ready()
        .await
        .expect("service ready")
        .call(goto_definition_request(2, uri, 0, 8))
        .await
        .expect("goto_definition transport ok")
        .expect("goto_definition response exists");

    assert!(response.is_ok(), "response should be ok");
    let result = response.result().cloned().expect("result");
    assert!(result.is_null(), "should return null for whitespace cursor");
}

#[tokio::test(flavor = "current_thread")]
async fn completion_returns_items() {
    let (mut service, _socket) = initialized_service().await;
    let uri = "file:///completion.ash";
    let source = "fn helper() -> Int { 1 }\nworkflow main { done }";

    let _ = service
        .ready()
        .await
        .expect("service ready")
        .call(open_request(uri, 1, source))
        .await
        .expect("didOpen transport ok");

    let response = service
        .ready()
        .await
        .expect("service ready")
        .call(completion_request(2, uri, 0, 0))
        .await
        .expect("completion transport ok")
        .expect("completion response exists");

    assert!(
        response.is_ok(),
        "completion response should be ok: {response:?}"
    );
    let result = response.result().cloned().expect("completion result");
    // Result should be an array of completion items
    let result_str = serde_json::to_string(&result).expect("serialize");
    assert!(
        result_str.contains("fn"),
        "completions should include 'fn' keyword"
    );
    assert!(
        result_str.contains("helper"),
        "completions should include 'helper' function"
    );
}
