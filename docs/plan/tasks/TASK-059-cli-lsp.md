# TASK-059: CLI LSP Command

## Status: 🟢 Complete

## Description

Implement the `ash lsp` command for Language Server Protocol support, enabling IDE integration.

## Specification Reference

- SPEC-005: CLI Specification - LSP section

## Requirements

### LSP Command

```rust
use clap::Args;

#[derive(Debug, Args)]
pub struct LspArgs {
    /// Use stdio for communication (default)
    #[arg(long)]
    pub stdio: bool,
    
    /// Use TCP port for communication
    #[arg(long, value_name = "PORT")]
    pub port: Option<u16>,
    
    /// Print debug logs
    #[arg(long)]
    pub debug: bool,
}

pub async fn lsp_command(args: LspArgs) -> Result<ExitCode, CliError> {
    if args.debug {
        tracing_subscriber::fmt::init();
    }
    
    let server = AshLanguageServer::new();
    
    if let Some(port) = args.port {
        // TCP mode
        let listener = tokio::net::TcpListener::bind(("127.0.0.1", port)).await
            .map_err(|e| CliError::Io(PathBuf::from("tcp"), e))?;
        
        tracing::info!("LSP server listening on port {}", port);
        
        let (stream, _) = listener.accept().await
            .map_err(|e| CliError::Io(PathBuf::from("tcp"), e))?;
        
        let (read, write) = stream.into_split();
        server.run(read, write).await;
    } else {
        // Stdio mode
        let stdin = tokio::io::stdin();
        let stdout = tokio::io::stdout();
        server.run(stdin, stdout).await;
    }
    
    Ok(ExitCode::SUCCESS)
}
```

### Language Server

```rust
use tower_lsp::jsonrpc::Result;
use tower_lsp::lsp_types::*;
use tower_lsp::{Client, LanguageServer, LspService, Server};

pub struct AshLanguageServer {
    client: Client,
    documents: Arc<Mutex<HashMap<Url, Document>>>,
}

struct Document {
    uri: Url,
    text: String,
    version: i32,
    parsed: Option<Program>,
}

#[tower_lsp::async_trait]
impl LanguageServer for AshLanguageServer {
    async fn initialize(&self, _: InitializeParams) -> Result<InitializeResult> {
        Ok(InitializeResult {
            capabilities: ServerCapabilities {
                text_document_sync: Some(TextDocumentSyncCapability::Kind(
                    TextDocumentSyncKind::FULL,
                )),
                hover_provider: Some(HoverProviderCapability::Simple(true)),
                completion_provider: Some(CompletionOptions {
                    resolve_provider: Some(false),
                    trigger_characters: Some(vec![".".to_string(), ":".to_string()]),
                    ..Default::default()
                }),
                diagnostic_provider: Some(DiagnosticServerCapabilities::Options(
                    DiagnosticOptions {
                        identifier: Some("ash".to_string()),
                        inter_file_dependencies: true,
                        workspace_diagnostics: false,
                        work_done_progress_options: WorkDoneProgressOptions::default(),
                    },
                )),
                document_formatting_provider: Some(OneOf::Left(true)),
                document_symbol_provider: Some(OneOf::Left(true)),
                ..Default::default()
            },
            ..Default::default()
        })
    }
    
    async fn initialized(&self, _: InitializedParams) {
        self.client
            .log_message(MessageType::INFO, "Ash LSP server initialized")
            .await;
    }
    
    async fn shutdown(&self) -> Result<()> {
        Ok(())
    }
    
    async fn did_open(&self, params: DidOpenTextDocumentParams) {
        let doc = Document {
            uri: params.text_document.uri.clone(),
            text: params.text_document.text,
            version: params.text_document.version,
            parsed: None,
        };
        
        self.documents.lock().await.insert(params.text_document.uri.clone(), doc);
        self.validate_document(&params.text_document.uri).await;
    }
    
    async fn did_change(&self, params: DidChangeTextDocumentParams) {
        let mut docs = self.documents.lock().await;
        
        if let Some(doc) = docs.get_mut(&params.text_document.uri) {
            // Apply changes
            for change in params.content_changes {
                if let Some(range) = change.range {
                    // Incremental change
                    doc.text = apply_change(&doc.text, range, &change.text);
                } else {
                    // Full document change
                    doc.text = change.text;
                }
            }
            
            doc.version = params.text_document.version;
            doc.parsed = None;
        }
        
        drop(docs);
        self.validate_document(&params.text_document.uri).await;
    }
    
    async fn did_close(&self, params: DidCloseTextDocumentParams) {
        self.documents.lock().await.remove(&params.text_document.uri);
    }
    
    async fn hover(&self, params: HoverParams) -> Result<Option<Hover>> {
        let uri = params.text_document_position_params.text_document.uri;
        let position = params.text_document_position_params.position;
        
        let docs = self.documents.lock().await;
        
        if let Some(doc) = docs.get(&uri) {
            // Find word at position
            if let Some(word) = get_word_at_position(&doc.text, position) {
                // Return hover info
                return Ok(Some(Hover {
                    contents: HoverContents::Scalar(MarkedString::String(format!(
                        "**{}**\n\nKeyword in Ash workflow language",
                        word
                    ))),
                    range: None,
                }));
            }
        }
        
        Ok(None)
    }
    
    async fn completion(&self, params: CompletionParams) -> Result<Option<CompletionResponse>> {
        let items = vec![
            CompletionItem {
                label: "workflow".to_string(),
                kind: Some(CompletionItemKind::KEYWORD),
                detail: Some("Define a workflow".to_string()),
                ..Default::default()
            },
            CompletionItem {
                label: "observe".to_string(),
                kind: Some(CompletionItemKind::KEYWORD),
                detail: Some("Observation step".to_string()),
                ..Default::default()
            },
            CompletionItem {
                label: "act".to_string(),
                kind: Some(CompletionItemKind::KEYWORD),
                detail: Some("Action step".to_string()),
                ..Default::default()
            },
            CompletionItem {
                label: "decide".to_string(),
                kind: Some(CompletionItemKind::KEYWORD),
                detail: Some("Decision step".to_string()),
                ..Default::default()
            },
            CompletionItem {
                label: "if".to_string(),
                kind: Some(CompletionItemKind::KEYWORD),
                detail: Some("Conditional".to_string()),
                ..Default::default()
            },
            CompletionItem {
                label: "let".to_string(),
                kind: Some(CompletionItemKind::KEYWORD),
                detail: Some("Variable binding".to_string()),
                ..Default::default()
            },
            CompletionItem {
                label: "par".to_string(),
                kind: Some(CompletionItemKind::KEYWORD),
                detail: Some("Parallel execution".to_string()),
                ..Default::default()
            },
        ];
        
        Ok(Some(CompletionResponse::Array(items)))
    }
    
    async fn formatting(&self, params: DocumentFormattingParams) -> Result<Option<Vec<TextEdit>>> {
        let docs = self.documents.lock().await;
        
        if let Some(doc) = docs.get(&params.text_document.uri) {
            match format_source(&doc.text, 2, 80) {
                Ok(formatted) => {
                    let lines = doc.text.lines().count() as u32;
                    
                    return Ok(Some(vec![TextEdit {
                        range: Range {
                            start: Position { line: 0, character: 0 },
                            end: Position { line: lines, character: 0 },
                        },
                        new_text: formatted,
                    }]));
                }
                Err(_) => {}
            }
        }
        
        Ok(None)
    }
}

impl AshLanguageServer {
    pub fn new() -> (Self, LspService<Self>) {
        let (service, socket) = LspService::new(|client| Self {
            client,
            documents: Arc::new(Mutex::new(HashMap::new())),
        });
        
        (service.inner().clone(), service)
    }
    
    pub async fn run<R, W>(self, read: R, write: W)
    where
        R: tokio::io::AsyncRead + Unpin,
        W: tokio::io::AsyncWrite,
    {
        let (service, socket) = LspService::new(|client| Self {
            client,
            documents: Arc::new(Mutex::new(HashMap::new())),
        });
        
        Server::new(read, write, socket).serve(service).await;
    }
    
    async fn validate_document(&self, uri: &Url) {
        let docs = self.documents.lock().await;
        
        if let Some(doc) = docs.get(uri) {
            let diagnostics = match parse(&doc.text) {
                Ok(_) => vec![],
                Err(e) => vec![Diagnostic {
                    range: Range {
                        start: Position { line: 0, character: 0 },
                        end: Position { line: 0, character: 0 },
                    },
                    severity: Some(DiagnosticSeverity::ERROR),
                    message: e.to_string(),
                    ..Default::default()
                }],
            };
            
            self.client.publish_diagnostics(uri.clone(), diagnostics, Some(doc.version)).await;
        }
    }
}

fn apply_change(text: &str, range: Range, new_text: &str) -> String {
    let lines: Vec<&str> = text.lines().collect();
    
    let start_line = range.start.line as usize;
    let start_char = range.start.character as usize;
    let end_line = range.end.line as usize;
    let end_char = range.end.character as usize;
    
    let mut result = String::new();
    
    // Add lines before change
    for i in 0..start_line {
        result.push_str(lines[i]);
        result.push('\n');
    }
    
    // Add partial start line
    if start_line < lines.len() {
        result.push_str(&lines[start_line][..start_char.min(lines[start_line].len())]);
    }
    
    // Add new text
    result.push_str(new_text);
    
    // Add partial end line
    if end_line < lines.len() {
        result.push_str(&lines[end_line][end_char.min(lines[end_line].len())..]);
        result.push('\n');
    }
    
    // Add lines after change
    for i in (end_line + 1)..lines.len() {
        result.push_str(lines[i]);
        result.push('\n');
    }
    
    result
}

fn get_word_at_position(text: &str, position: Position) -> Option<String> {
    let lines: Vec<&str> = text.lines().collect();
    let line = *lines.get(position.line as usize)?;
    
    let char_pos = position.character as usize;
    
    // Find word boundaries
    let start = line[..char_pos.min(line.len())]
        .rfind(|c: char| !c.is_alphanumeric() && c != '_')
        .map(|i| i + 1)
        .unwrap_or(0);
    
    let end = line[char_pos.min(line.len())..]
        .find(|c: char| !c.is_alphanumeric() && c != '_')
        .map(|i| char_pos + i)
        .unwrap_or(line.len());
    
    if start < end {
        Some(line[start..end].to_string())
    } else {
        None
    }
}
```

## TDD Steps

### Step 1: Create LspArgs

Add to CLI args.

### Step 2: Implement Language Server

Create AshLanguageServer.

### Step 3: Implement Handlers

Add handlers for LSP methods.

### Step 4: Write Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_word_at_position() {
        let text = "workflow test {\n  observe read;\n}";
        
        let word = get_word_at_position(text, Position { line: 0, character: 3 });
        assert_eq!(word, Some("workflow".to_string()));
        
        let word = get_word_at_position(text, Position { line: 1, character: 4 });
        assert_eq!(word, Some("observe".to_string()));
    }

    #[test]
    fn test_apply_change() {
        let text = "hello world\nfoo bar";
        let range = Range {
            start: Position { line: 0, character: 6 },
            end: Position { line: 0, character: 11 },
        };
        
        let result = apply_change(text, range, "universe");
        assert!(result.contains("hello universe"));
    }
}
```

## Completion Checklist

- [ ] LspArgs struct
- [ ] lsp_command
- [ ] AshLanguageServer
- [ ] Initialize handler
- [ ] Text sync handlers
- [ ] Hover handler
- [ ] Completion handler
- [ ] Formatting handler
- [ ] Diagnostic publishing
- [ ] Unit tests
- [ ] `cargo fmt` passes
- [ ] `cargo clippy` passes

## Self-Review Questions

1. **LSP compliance**: Are responses correct?
2. **Performance**: Is document validation fast?
3. **Features**: Are all essential features implemented?

## Estimated Effort

12 hours (LSP is complex)

## Dependencies

- tower-lsp
- tokio
- ash-parser

## Blocked By

- ash-parser: Parser
- TASK-058: Formatting

## Blocks

- TASK-060: Integration tests
