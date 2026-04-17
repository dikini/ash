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
    DocumentSymbol, DocumentSymbolParams, DocumentSymbolResponse, Hover, HoverParams,
    HoverProviderCapability, InitializeParams, InitializeResult, InitializedParams, MessageType,
    OneOf, ServerCapabilities, ServerInfo, TextDocumentContentChangeEvent,
    TextDocumentSyncCapability, TextDocumentSyncKind, Uri,
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

    async fn completion(&self, _: CompletionParams) -> Result<Option<CompletionResponse>> {
        Ok(None)
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
