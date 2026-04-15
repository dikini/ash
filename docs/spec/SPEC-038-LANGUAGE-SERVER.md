# SPEC-038: Ash Language Server Protocol (LSP) & MCP Interface

## Status: Draft

## 1. Goal

Define a production-quality Language Server Protocol (LSP) implementation for the Ash workflow language, plus an embedded Model Context Protocol (MCP) interface that exposes the same semantic intelligence to AI coding agents (Hermes, Codex, Claude, Open Code, OpenClaw, etc.).

The server must work out-of-the-box with VSCode and Neovim, and it must be agent-first: every LSP capability should be queryable programmatically so that AI assistants can reason about Ash code with the same precision as a human IDE user.

## 2. Non-Goals

- **Not** a generic IDE or GUI.
- **Not** a replacement for `ash check` / `ash run` CLI commands.
- **Not** a remote build system (the server is local-only).

## 3. Architecture

### 3.1 High-Level Design

```
┌─────────────────────────────────────────────────────────────────────┐
│                         ash-lsp (crate)                             │
├─────────────────────────────────────────────────────────────────────┤
│  LSP Layer (tower-lsp-server)                                       │
│   ├─ textDocument/* handlers                                        │
│   ├─ workspace/* handlers                                           │
│   └─ notification dispatch                                          │
├─────────────────────────────────────────────────────────────────────┤
│  Analysis Layer (ash-lsp-core)                                      │
│   ├─ VFS: virtual file system (in-memory overlays + fs watcher)     │
│   ├─ Salsa-like query cache (parse → surface AST → check → symbols) │
│   ├─ Diagnostic aggregator (parse + type + lint + custom rules)     │
│   └─ Index: symbols, references, scopes                             │
├─────────────────────────────────────────────────────────────────────┤
│  Compiler Front-End (existing crates)                               │
│   ├─ ash-parser  (lexer → surface AST, spans, errors)               │
│   ├─ ash-typeck  (types, effects, names, obligations)               │
│   ├─ ash-lint    (custom Ash lints: OODA, provenance, policy)       │
│   └─ ash-engine  (module loading, crate graphs)                     │
├─────────────────────────────────────────────────────────────────────┤
│  MCP Bridge (ash-lsp-mcp)                                           │
│   ├─ mcp/tools/ash_hover                                            │
│   ├─ mcp/tools/ash_complete                                         │
│   ├─ mcp/tools/ash_diagnostics                                      │
│   ├─ mcp/tools/ash_goto_definition                                  │
│   ├─ mcp/tools/ash_find_references                                  │
│   ├─ mcp/tools/ash_symbol_search                                    │
│   └─ mcp/tools/ash_apply_edit (code actions)                        │
└─────────────────────────────────────────────────────────────────────┘
```

### 3.2 Crate Layout

| Crate | Purpose | New or Existing |
|-------|---------|-----------------|
| `crates/ash-lsp` | Binary + LSP wire protocol | **New** |
| `crates/ash-lsp-core` | VFS, analysis cache, index, diagnostic aggregation | **New** |
| `crates/ash-parser` | Lexer, surface AST, spans, error recovery | Existing |
| `crates/ash-typeck` | Type checker, name resolution, effect inference | Existing |
| `crates/ash-lint` | Custom lints (currently CLI-only; needs lib extraction) | Existing, refactor |
| `crates/ash-engine` | Crate graph, module loader, entry verification | Existing |
| `crates/ash-mcp` | MCP server wrapper over `ash-lsp-core` (optional binary) | **New** |

> **Rationale:** Splitting `ash-lsp-core` from `ash-lsp` keeps the analysis engine testable without spinning up a full LSP client/server pair. The MCP bridge can reuse `ash-lsp-core` directly.

## 4. Document Model & Virtual File System

### 4.1 Why a VFS is Mandatory

Ash parsers today read from `&str` or files on disk. LSP clients send `textDocument/didChange` with unsaved buffer contents. Completion requests arrive **before** the file is ever written to disk. Therefore the server must maintain an in-memory overlay.

### 4.2 VFS Design (Minimal but Correct)

```rust
pub struct Vfs {
    files: DashMap<VfsPath, FileSnapshot>,
}

pub struct FileSnapshot {
    pub source: String,
    pub version: i32,
    pub changed_at: Instant,
}

pub type VfsPath = String; // URI as received from LSP client
```

- **Concurrency:** `DashMap` gives lock-free reads so multiple LSP requests can query the same file simultaneously.
- **Incremental sync:** Support `TextDocumentSyncKind::INCREMENTAL`. Clients like VSCode default to incremental. A helper `apply_text_edits(source, &[TextDocumentContentChangeEvent]) -> String` normalizes changes into a new snapshot.
- **File watching:** On `workspace/didChangeWatchedFiles`, update disk-backed entries so that modifying `foo.ash` in a terminal refreshes the index without restarting the server.

### 4.3 Change Application

```rust
pub fn apply_changes(text: &str, changes: &[TextDocumentContentChangeEvent]) -> String {
    let mut result = text.to_string();
    // If any change lacks a range, treat as full replacement
    if changes.iter().any(|c| c.range.is_none()) {
        return changes.last().unwrap().text.clone();
    }
    // Apply incremental edits in reverse order by start position
    // to keep earlier indices stable.
    ...
}
```

## 5. Analysis Cache & Query Engine

### 5.1 Query Granularity

The analysis engine exposes these pure functions over the VFS:

```rust
// parse
pub fn parse_file(vfs: &Vfs, path: VfsPath) -> (SurfaceAST, Vec<ParseError>)

// type-check (requires module graph)
pub fn check_file(vfs: &Vfs, path: VfsPath, graph: &ModuleGraph)
    -> (TypedSurface, Vec<TypeError>)

// symbols
pub fn document_symbols(vfs: &Vfs, path: VfsPath) -> Vec<Symbol>

// index
pub fn goto_definition(vfs: &Vfs, path: VfsPath, pos: Position) -> Option<Location>
pub fn find_references(vfs: &Vfs, path: VfsPath, pos: Position) -> Vec<Location>
```

### 5.2 Caching Strategy (Phase 1: Simple; Phase 2: Salsa)

**Phase 1 (MVP):** Per-request recomputation with a short-lived LRU cache keyed by `(path, version)`.

**Phase 2 (Polish):** Introduce `salsa` or a hand-rolled query system so that:
- Parsing a file is cached.
- If file `A.ash` changes but `B.ash` does not, `B.ash`'s AST remains valid.
- Name-resolution and type-checking are incremental across the module graph.

> **Reference:** rust-analyzer uses `salsa` for this exact purpose. For Ash, which has no macros and a simpler module system, a lightweight custom cache may be sufficient. Start simple, measure, then upgrade.

## 6. LSP Capabilities

Capabilities are grouped by **priority**. The MVP must ship Priority 1; everything else is stretch.

### 6.1 Priority 1 — MVP (Week 1–2)

| Capability | LSP Method | Ash Implementation Notes |
|------------|------------|--------------------------|
| **Diagnostics** | `textDocument/publishDiagnostics` | Aggregate `ParseError` (ash-parser), `TypeError` (ash-typeck), and lint diagnostics. Run on every `didChange` / `didOpen` / `didSave`. Debounce 200 ms. |
| **Hover** | `textDocument/hover` | Keyword docs + type info from `ash-typeck`. If the cursor is on a capability name, show its signature. |
| **Go to Definition** | `textDocument/definition` | Use `ash-parser` name bindings and `ash-typeck` `NameEnv`. Surface AST already has spans for every binding site. |
| **Document Sync** | `textDocument/didOpen/Change/Close` | Incremental sync. VFS overlay. |
| **Document Symbols** | `textDocument/documentSymbol` | Walk surface AST; emit `SymbolKind::Function` for workflows, `SymbolKind::Interface` for capabilities, etc. |

### 6.2 Priority 2 — Agent Power-Ups (Week 3–4)

| Capability | LSP Method | Notes |
|------------|------------|-------|
| **Completion** | `textDocument/completion` | Trigger chars: `.`, `:`, `(`, `{`. Suggest: keywords, in-scope variables, capability names, policy names, record fields. |
| **Find References** | `textDocument/references` | Requires a cross-file reference index. Build it by scanning the surface AST for `Name` usages and matching against the binding table. |
| **Formatting** | `textDocument/formatting` | Ash has **no formatter yet**. This requires building one. See §9. |
| **Code Actions** | `textDocument/codeAction` | Quick fixes: "Import missing capability", "Add missing match arm" (from exhaustiveness checker). |

### 6.3 Priority 3 — Polish (Week 5+)

| Capability | LSP Method | Notes |
|------------|------------|-------|
| **Workspace Symbols** | `workspace/symbol` | Search across all `.ash` files in the workspace. |
| **Semantic Tokens** | `textDocument/semanticTokens/full` | Full semantic highlighting (capabilities, policies, workflow keywords, types). |
| **Signature Help** | `textDocument/signatureHelp` | Show parameter names when typing inside a capability call. |
| **Rename** | `textDocument/rename` | Safe rename across the module graph using the reference index. |

## 7. Diagnostic Pipeline

### 7.1 Sources

1. **Parser** (`ash-parser`) — syntax errors, unexpected tokens.
2. **Type Checker** (`ash-typeck`) — type mismatches, unbound variables, effect errors.
3. **Linter** (`ash-lint`) — OODA loop violations, missing provenance, policy conflicts.
4. **Custom Rules** (future) — deprecation warnings, style nits.

### 7.2 Conversion to LSP Diagnostics

All Ash error types already carry `Span { start, end, line, column }`. Conversion is mechanical:

```rust
fn ash_error_to_diagnostic(err: &AshError, source: &str) -> Diagnostic {
    let range = span_to_lsp_range(err.span(), source);
    Diagnostic {
        range,
        severity: Some(err.severity().into()),
        code: Some(NumberOrString::String(err.code().into())),
        source: Some("ash".into()),
        message: err.message(),
        ..Default::default()
    }
}
```

### 7.3 Debouncing

Typing generates a flood of `didChange` notifications. Diagnostics must be **debounced** (200 ms) so that the server doesn't re-parse the world on every keystroke. Use `tokio::time::sleep` in a cancellable task.

```rust
async fn schedule_validation(&self, uri: Url) {
    // Cancel any in-flight validation for this URI
    if let Some(old) = self.pending_validations.insert(uri.clone(), token) {
        old.cancel();
    }
    tokio::time::sleep(Duration::from_millis(200)).await;
    if !token.is_cancelled() {
        self.validate_and_publish(uri).await;
    }
}
```

## 8. MCP Interface

### 8.1 Design Principle

The MCP bridge is **not** a separate language server. It is a thin wrapper over `ash-lsp-core` that speaks the Model Context Protocol. This guarantees that human IDE users and AI agents see the *exact same* analysis.

### 8.2 Transport

- **Default:** `stdio` (standard MCP transport).
- **Optional:** TCP or HTTP for remote agent orchestration.

### 8.3 Exposed Tools

Each tool maps 1:1 to an `ash-lsp-core` query.

| MCP Tool | Input | Output | LSP Equivalent |
|----------|-------|--------|----------------|
| `ash_get_diagnostics` | `{"file": "..."}` | JSON array of diagnostics | `publishDiagnostics` |
| `ash_hover` | `{"file": "...", "line": 12, "column": 5}` | Markdown string | `textDocument/hover` |
| `ash_goto_definition` | `{"file": "...", "line": 12, "column": 5}` | Location JSON | `textDocument/definition` |
| `ash_find_references` | `{"file": "...", "line": 12, "column": 5}` | Location[] JSON | `textDocument/references` |
| `ash_complete` | `{"file": "...", "line": 12, "column": 5}` | CompletionItem[] JSON | `textDocument/completion` |
| `ash_document_symbols` | `{"file": "..."}` | Symbol[] JSON | `textDocument/documentSymbol` |
| `ash_workspace_symbols` | `{"query": "temperature_alert"}` | Symbol[] JSON | `workspace/symbol` |
| `ash_code_action` | `{"file": "...", "startLine": ..., "endLine": ...}` | CodeAction[] JSON | `textDocument/codeAction` |
| `ash_apply_edit` | `{"file": "...", "edits": [...]}` | success boolean | `workspace/applyEdit` |

### 8.4 Agent-Friendly Response Format

MCP clients are LLMs. Responses should be **token-efficient** and **structured**:

```json
{
  "diagnostics": [
    {
      "severity": "error",
      "message": "Unknown capability 'send_alrt'",
      "line": 42,
      "column": 5,
      "suggestion": "Did you mean 'send_alert'?"
    }
  ],
  "summary": "1 error found in examples/simple_workflow.ash"
}
```

- Always include a one-line `summary` field.
- Use line/column numbers (1-indexed) because most LLM prompts are trained on human-readable coordinates.
- Include `suggestion` when the underlying error type provides an expected value (e.g., typo correction from the parser).

### 8.5 Context-Aware File Opening

MCP tools should **not** require the caller to manually open files first. The bridge internally calls `vfs.open(file_path)` before querying, and keeps the file in memory for the lifetime of the MCP session. This matches how `lsp-mcp` and `cclsp` behave.

## 9. Formatter

Ash currently has **no source formatter**. This is the largest greenfield work item.

### 9.1 Formatter Strategy

Because Ash's surface syntax is relatively small (no macros, no complex precedence), a **naive pretty-printer** over the surface AST is sufficient for MVP.

```rust
pub fn format_surface_ast(ast: &ModuleFile, indent: usize) -> String {
    // Walk the AST and emit formatted text.
    // Use the original spans only for preserving comments.
}
```

### 9.2 Comment Preservation

The `ash-parser` lexer already tokenizes comments. The formatter must thread comment trivia through the surface AST nodes. If comment trivia is not currently stored, add a `leading_comments: Vec<Comment>` and `trailing_comments: Vec<Comment>` field to key AST nodes (or store them in a side-table keyed by `Span`).

### 9.3 Long-Term

Once the formatter exists, `textDocument/formatting` and `textDocument/rangeFormatting` become trivial one-line handlers.

## 10. Editor Integration

### 10.1 VSCode

- Extension name: `ash-vscode`
- Location: `editors/vscode/`
- Client uses `vscode-languageclient/node`.
- Server path: `${workspaceFolder}/target/release/ash-lsp` (or bundled binary).

```json
// package.json contributes
"activationEvents": ["onLanguage:ash"],
"main": "./out/extension.js"
```

### 10.2 Neovim

- No plugin required; native `vim.lsp.config` (Neovim 0.11+) or `lspconfig`.

```lua
-- minimal init.lua
vim.lsp.config['ash-lsp'] = {
  cmd = { 'ash-lsp' },
  filetypes = { 'ash' },
  root_markers = { '.ash.toml', 'ash.toml', '.git' },
}
vim.lsp.enable('ash-lsp')
```

### 10.3 TextMate Grammar (Bonus)

VSCode requires a basic TextMate grammar for syntax highlighting before semantic tokens kick in. Provide `syntaxes/ash.tmLanguage.json` with scopes for:
- `comment.line.double-dash`
- `keyword.control.ash` (`workflow`, `observe`, `act`, `decide`, `if`, `let`, etc.)
- `storage.type.ash` (`capability`, `policy`, `role`)
- `entity.name.function.ash` (capability calls)

## 11. Implementation Phases

### Phase 1 — LSP Skeleton (Week 1)
- Create `crates/ash-lsp` and `crates/ash-lsp-core`.
- Implement `initialize`, `shutdown`, `didOpen/Change/Close`.
- VFS with incremental sync.
- Wire up `ash-parser` for syntax validation + diagnostic publishing.

### Phase 2 — Core Intelligence (Week 2)
- Hover with keyword + type info.
- Go to definition using existing name binding data.
- Document symbols from surface AST.
- Integrate `ash-typeck` errors into diagnostics.

### Phase 3 — Agent Interface (Week 3)
- Create `crates/ash-mcp` MCP bridge.
- Implement tools: `ash_diagnostics`, `ash_hover`, `ash_goto_definition`, `ash_find_references`, `ash_complete`.
- Test with Claude Code / Codex CLI.

### Phase 4 — Advanced Features (Week 4)
- Formatter (see §9).
- Code actions (quick fixes).
- Workspace symbols.
- VSCode extension skeleton.

### Phase 5 — Polish (Week 5+)
- Semantic tokens.
- Reference index caching.
- Salsa-based incremental analysis.
- Rename symbol support.

## 12. Dependencies

```toml
[dependencies]
# LSP framework (actively maintained fork of tower-lsp)
tower-lsp-server = "0.23"

# Async runtime
tokio = { version = "1.42", features = ["rt-multi-thread", "io-std", "time"] }

# Concurrency
parking_lot = "0.12"
dashmap = "6.1"

# Serialization
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"

# MCP SDK (Rust implementation)
rmcp = "0.1"          # or equivalent community crate
# If no mature Rust MCP crate exists, hand-roll JSON-RPC 2.0 over stdio.

# Ash internal crates
ash-core = { path = "../ash-core" }
ash-parser = { path = "../ash-parser" }
ash-typeck = { path = "../ash-typeck" }
ash-lint = { path = "../ash-lint" }
ash-engine = { path = "../ash-engine" }
```

## 13. Testing Strategy

1. **Unit tests** for VFS change application.
2. **LSP conformance tests** using `lsp-types` + manual JSON-RPC request/response pairs.
3. **Integration tests** that open real `.ash` files from `examples/` and assert on diagnostics, hover text, and symbol counts.
4. **MCP end-to-end tests** that spawn `ash-mcp`, send tool calls, and verify JSON output.

## 14. Security & Sandboxing

- The MCP bridge must validate all file paths to prevent path traversal outside the workspace root.
- `ash_apply_edit` should only write to `.ash` files within the opened workspace.
- No arbitrary code execution through the LSP or MCP interface.

## 15. Relationship to Existing Tasks

- **TASK-059** (`cli-lsp`): The MVP LSP skeleton. This spec supersedes and expands TASK-059.
- **TASK-270** / **TASK-457** (`mcp-provider`): Those tasks define an *outgoing* MCP client inside `ash-engine` (Ash workflows calling external MCP servers). This spec defines an *incoming* MCP server that exposes Ash language intelligence to external agents. The two directions are complementary.

## 16. Open Questions

1. Should the LSP server support multi-crate workspaces (like Cargo workspaces) or single-crate roots only?
2. Does `ash-parser` already preserve comment trivia, or do we need to add it for the formatter?
3. Should the MCP bridge be a separate binary (`ash-mcp`) or a subcommand (`ash lsp --mcp`)?

---

**Next Step:** Create `docs/plan/tasks/TASK-XXX-lsp-mcp-implementation.md` and begin Phase 1 (skeleton + VFS) using subagent-driven development.
