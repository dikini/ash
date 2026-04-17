//! Ash LSP server binary.
//!
//! Production-facing LSP transport wrapper around `ash-lsp-core`.

#![allow(
    clippy::significant_drop_tightening,
    clippy::option_if_let_else,
    clippy::manual_let_else,
    missing_docs
)]

use ash_lint::LintConfig;
use ash_lsp_core::analysis::AnalysisCache;
use ash_lsp_core::completion::completions as core_completions;
use ash_lsp_core::goto::goto_definition as core_goto_definition;
use ash_lsp_core::hover::hover_at as core_hover_at;
use ash_lsp_core::symbols::document_symbols as core_document_symbols;
use ash_lsp_core::vfs::Vfs;
use clap::Parser;
use lsp_types::{
    Diagnostic as CoreDiagnostic, DocumentSymbol as CoreDocumentSymbol,
    TextDocumentContentChangeEvent as CoreTextDocumentContentChangeEvent, Uri as CoreUri,
};
use std::net::SocketAddr;
use tokio::io::{stdin, stdout};
use tokio::net::TcpListener;
use tower_lsp_server::jsonrpc::Result;
use tower_lsp_server::ls_types::{
    CompletionOptions, CompletionParams, CompletionResponse, Diagnostic,
    DidChangeTextDocumentParams, DidCloseTextDocumentParams, DidOpenTextDocumentParams,
    DocumentSymbol, DocumentSymbolParams, DocumentSymbolResponse, GotoDefinitionParams,
    GotoDefinitionResponse, Hover, HoverParams, HoverProviderCapability, InitializeParams,
    InitializeResult, InitializedParams, MessageType, OneOf, ServerCapabilities, ServerInfo,
    TextDocumentContentChangeEvent, TextDocumentSyncCapability, TextDocumentSyncKind, Uri,
};
use tower_lsp_server::{Client, LanguageServer, LspService, Server};
use tracing::{debug, error, info};
use tracing_subscriber::EnvFilter;

#[derive(Debug, Parser)]
#[command(name = "ash-lsp")]
#[command(about = "Ash Language Server Protocol server")]
struct Args {
    /// Listen on localhost TCP instead of stdio.
    #[arg(long)]
    port: Option<u16>,
}

#[derive(Debug)]
struct AshLanguageServer {
    client: Client,
    vfs: Vfs,
    cache: AnalysisCache,
    config: LintConfig,
}

impl AshLanguageServer {
    fn new(client: Client) -> Self {
        Self {
            client,
            vfs: Vfs::new(),
            cache: AnalysisCache::new(),
            config: LintConfig::default(),
        }
    }

    fn tower_uri_to_core(uri: &Uri) -> std::result::Result<CoreUri, serde_json::Error> {
        serde_json::from_value(serde_json::to_value(uri)?)
    }

    fn tower_changes_to_core(
        changes: Vec<TextDocumentContentChangeEvent>,
    ) -> std::result::Result<Vec<CoreTextDocumentContentChangeEvent>, serde_json::Error> {
        changes
            .into_iter()
            .map(|change| serde_json::from_value(serde_json::to_value(change)?))
            .collect()
    }

    fn core_diagnostics_to_tower(
        diagnostics: Vec<CoreDiagnostic>,
    ) -> std::result::Result<Vec<Diagnostic>, serde_json::Error> {
        diagnostics
            .into_iter()
            .map(|diag| serde_json::from_value(serde_json::to_value(diag)?))
            .collect()
    }

    fn core_document_symbols_to_tower(
        symbols: Vec<CoreDocumentSymbol>,
    ) -> std::result::Result<Vec<DocumentSymbol>, serde_json::Error> {
        symbols
            .into_iter()
            .map(|symbol| serde_json::from_value(serde_json::to_value(symbol)?))
            .collect()
    }

    async fn publish_current_diagnostics(&self, uri: Uri, version: Option<i32>) {
        debug!(uri = uri.as_str(), ?version, "publishing diagnostics");
        let core_uri = match Self::tower_uri_to_core(&uri) {
            Ok(uri) => uri,
            Err(err) => {
                error!(error = %err, "failed to convert tower URI to core URI");
                return;
            }
        };

        let diagnostics = self.cache.analyze(&core_uri, &self.vfs, &self.config);
        let diagnostics = match Self::core_diagnostics_to_tower(diagnostics) {
            Ok(diags) => diags,
            Err(err) => {
                error!(error = %err, "failed to convert core diagnostics to tower diagnostics");
                return;
            }
        };

        self.client
            .publish_diagnostics(uri, diagnostics, version)
            .await;
    }

    async fn clear_diagnostics(&self, uri: Uri) {
        debug!(uri = uri.as_str(), "clearing diagnostics");
        self.client.publish_diagnostics(uri, Vec::new(), None).await;
    }
}

impl LanguageServer for AshLanguageServer {
    async fn initialize(&self, _: InitializeParams) -> Result<InitializeResult> {
        info!("initialize request received");
        Ok(InitializeResult {
            server_info: Some(ServerInfo {
                name: "ash-lsp".to_string(),
                version: Some(env!("CARGO_PKG_VERSION").to_string()),
            }),
            capabilities: ServerCapabilities {
                text_document_sync: Some(TextDocumentSyncCapability::Kind(
                    TextDocumentSyncKind::INCREMENTAL,
                )),
                hover_provider: Some(HoverProviderCapability::Simple(true)),
                completion_provider: Some(CompletionOptions {
                    resolve_provider: Some(false),
                    trigger_characters: Some(vec![".".to_string(), ":".to_string()]),
                    ..CompletionOptions::default()
                }),
                document_symbol_provider: Some(OneOf::Left(true)),
                definition_provider: Some(OneOf::Left(true)),
                ..ServerCapabilities::default()
            },
            offset_encoding: None,
        })
    }

    async fn initialized(&self, _: InitializedParams) {
        info!("ash-lsp initialized");
        self.client
            .log_message(MessageType::INFO, "ash-lsp initialized")
            .await;
    }

    async fn shutdown(&self) -> Result<()> {
        info!("shutdown request received");
        Ok(())
    }

    async fn did_open(&self, params: DidOpenTextDocumentParams) {
        let text_document = params.text_document;
        let uri = text_document.uri;
        let version = text_document.version;
        let text = text_document.text;

        info!(uri = uri.as_str(), version, "did_open");
        let core_uri = match Self::tower_uri_to_core(&uri) {
            Ok(uri) => uri,
            Err(err) => {
                error!(error = %err, "failed to convert did_open URI");
                return;
            }
        };

        self.vfs.open(core_uri, version, text);
        self.publish_current_diagnostics(uri, Some(version)).await;
    }

    async fn did_change(&self, params: DidChangeTextDocumentParams) {
        let text_document = params.text_document;
        let uri = text_document.uri;
        let version = text_document.version;
        let changes = params.content_changes;

        info!(
            uri = uri.as_str(),
            version,
            num_changes = changes.len(),
            "did_change"
        );
        let core_uri = match Self::tower_uri_to_core(&uri) {
            Ok(uri) => uri,
            Err(err) => {
                error!(error = %err, "failed to convert did_change URI");
                return;
            }
        };
        let core_changes = match Self::tower_changes_to_core(changes) {
            Ok(changes) => changes,
            Err(err) => {
                error!(error = %err, "failed to convert did_change content changes");
                return;
            }
        };

        self.vfs.change(&core_uri, version, core_changes);
        self.cache.invalidate(&core_uri);
        self.publish_current_diagnostics(uri, Some(version)).await;
    }

    async fn did_close(&self, params: DidCloseTextDocumentParams) {
        let uri = params.text_document.uri;
        info!(uri = uri.as_str(), "did_close");
        let core_uri = match Self::tower_uri_to_core(&uri) {
            Ok(uri) => uri,
            Err(err) => {
                error!(error = %err, "failed to convert did_close URI");
                return;
            }
        };

        self.vfs.close(&core_uri);
        self.cache.invalidate(&core_uri);
        self.clear_diagnostics(uri).await;
    }

    async fn hover(&self, params: HoverParams) -> Result<Option<Hover>> {
        let uri = params.text_document_position_params.text_document.uri;
        let position = params.text_document_position_params.position;
        debug!(
            uri = uri.as_str(),
            line = position.line,
            character = position.character,
            "hover request"
        );

        let core_uri = match Self::tower_uri_to_core(&uri) {
            Ok(uri) => uri,
            Err(err) => {
                error!(error = %err, "failed to convert hover URI");
                return Ok(None);
            }
        };

        let Some(entry) = self.vfs.get(&core_uri) else {
            return Ok(None);
        };

        if self.cache.get(&core_uri).is_none() {
            let _ = self.cache.analyze(&core_uri, &self.vfs, &self.config);
        }

        let Some(analysis) = self.cache.get(&core_uri) else {
            return Ok(None);
        };
        let Some(module) = analysis.module else {
            return Ok(None);
        };

        let Some(core_hover) = core_hover_at(
            module.as_ref(),
            &entry.content,
            position.line,
            position.character,
        ) else {
            return Ok(None);
        };

        let hover_value = match serde_json::to_value(core_hover) {
            Ok(value) => value,
            Err(err) => {
                error!(error = %err, "failed to serialize core hover");
                return Ok(None);
            }
        };

        let hover = match serde_json::from_value(hover_value) {
            Ok(hover) => hover,
            Err(err) => {
                error!(error = %err, "failed to convert core hover to tower hover");
                return Ok(None);
            }
        };

        Ok(Some(hover))
    }

    async fn goto_definition(
        &self,
        params: GotoDefinitionParams,
    ) -> Result<Option<GotoDefinitionResponse>> {
        let uri = params.text_document_position_params.text_document.uri;
        let position = params.text_document_position_params.position;
        debug!(
            uri = uri.as_str(),
            line = position.line,
            character = position.character,
            "goto_definition request"
        );

        let core_uri = match Self::tower_uri_to_core(&uri) {
            Ok(uri) => uri,
            Err(err) => {
                error!(error = %err, "failed to convert goto_definition URI");
                return Ok(None);
            }
        };

        let Some(entry) = self.vfs.get(&core_uri) else {
            return Ok(None);
        };

        if self.cache.get(&core_uri).is_none() {
            let _ = self.cache.analyze(&core_uri, &self.vfs, &self.config);
        }

        let Some(analysis) = self.cache.get(&core_uri) else {
            return Ok(None);
        };
        let Some(module) = analysis.module else {
            return Ok(None);
        };

        let core_response = core_goto_definition(
            module.as_ref(),
            &entry.content,
            &core_uri,
            position.line,
            position.character,
        );

        let Some(response) = core_response else {
            return Ok(None);
        };

        let value = match serde_json::to_value(response) {
            Ok(value) => value,
            Err(err) => {
                error!(error = %err, "failed to serialize goto_definition response");
                return Ok(None);
            }
        };

        let response = match serde_json::from_value(value) {
            Ok(response) => response,
            Err(err) => {
                error!(error = %err, "failed to convert core goto_definition to tower type");
                return Ok(None);
            }
        };

        Ok(Some(response))
    }

    async fn completion(&self, params: CompletionParams) -> Result<Option<CompletionResponse>> {
        let uri = params.text_document_position.text_document.uri;
        let position = params.text_document_position.position;
        debug!(
            uri = uri.as_str(),
            line = position.line,
            character = position.character,
            "completion request"
        );

        let core_uri = match Self::tower_uri_to_core(&uri) {
            Ok(uri) => uri,
            Err(err) => {
                error!(error = %err, "failed to convert completion URI");
                return Ok(None);
            }
        };

        let Some(entry) = self.vfs.get(&core_uri) else {
            return Ok(None);
        };

        if self.cache.get(&core_uri).is_none() {
            let _ = self.cache.analyze(&core_uri, &self.vfs, &self.config);
        }

        let Some(analysis) = self.cache.get(&core_uri) else {
            return Ok(None);
        };
        let Some(module) = analysis.module else {
            return Ok(None);
        };

        let core_response = core_completions(
            module.as_ref(),
            &entry.content,
            position.line,
            position.character,
        );

        let value = match serde_json::to_value(core_response) {
            Ok(value) => value,
            Err(err) => {
                error!(error = %err, "failed to serialize completion response");
                return Ok(None);
            }
        };

        let response = match serde_json::from_value(value) {
            Ok(response) => response,
            Err(err) => {
                error!(error = %err, "failed to convert core completion to tower type");
                return Ok(None);
            }
        };

        Ok(Some(response))
    }

    async fn document_symbol(
        &self,
        params: DocumentSymbolParams,
    ) -> Result<Option<DocumentSymbolResponse>> {
        let uri = params.text_document.uri;
        debug!(uri = uri.as_str(), "document_symbol request");

        let core_uri = match Self::tower_uri_to_core(&uri) {
            Ok(uri) => uri,
            Err(err) => {
                error!(error = %err, "failed to convert document_symbol URI");
                return Ok(Some(DocumentSymbolResponse::Nested(Vec::new())));
            }
        };

        if self.cache.get(&core_uri).is_none() && self.vfs.get(&core_uri).is_some() {
            let _ = self.cache.analyze(&core_uri, &self.vfs, &self.config);
        }

        let Some(analysis) = self.cache.get(&core_uri) else {
            return Ok(Some(DocumentSymbolResponse::Nested(Vec::new())));
        };

        let Some(module) = analysis.module else {
            return Ok(Some(DocumentSymbolResponse::Nested(Vec::new())));
        };

        let core_symbols = core_document_symbols(module.as_ref());
        let symbols = match Self::core_document_symbols_to_tower(core_symbols) {
            Ok(symbols) => symbols,
            Err(err) => {
                error!(error = %err, "failed to convert document symbols to tower types");
                return Ok(Some(DocumentSymbolResponse::Nested(Vec::new())));
            }
        };

        Ok(Some(DocumentSymbolResponse::Nested(symbols)))
    }
}

fn init_tracing() {
    let filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new("ash_lsp=info,ash_lsp_core=info"));
    tracing_subscriber::fmt().with_env_filter(filter).init();
}

async fn run_stdio() {
    info!("starting ash-lsp over stdio");
    let (service, socket) = LspService::new(AshLanguageServer::new);
    Server::new(stdin(), stdout(), socket).serve(service).await;
}

async fn run_tcp(port: u16) -> std::result::Result<(), Box<dyn std::error::Error>> {
    let addr = SocketAddr::from(([127, 0, 0, 1], port));
    info!(%addr, "starting ash-lsp TCP listener");
    let listener = TcpListener::bind(addr).await?;
    let (stream, peer_addr) = listener.accept().await?;
    info!(%peer_addr, "accepted TCP LSP connection");

    let (read, write) = tokio::io::split(stream);
    let (service, socket) = LspService::new(AshLanguageServer::new);
    Server::new(read, write, socket).serve(service).await;
    Ok(())
}

#[tokio::main]
async fn main() -> std::result::Result<(), Box<dyn std::error::Error>> {
    init_tracing();
    let args = Args::parse();

    match args.port {
        Some(port) => {
            if let Err(err) = run_tcp(port).await {
                error!(error = %err, port, "ash-lsp TCP server failed");
                return Err(err);
            }
        }
        None => run_stdio().await,
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use futures::StreamExt;
    use serde_json::json;
    use tower::{Service, ServiceExt};
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
        let symbols: DocumentSymbolResponse =
            serde_json::from_value(result).expect("decode symbols");
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
}
