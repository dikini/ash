# SPEC-038: Ash Language Server Protocol (LSP) & MCP Interface

## Status: Draft (Partially Implemented — Local LSP MVP)

## 1. Goal

Define a production-quality Language Server Protocol (LSP) implementation for the Ash workflow language, plus an embedded Model Context Protocol (MCP) interface that exposes the same semantic intelligence to AI coding agents (Hermes, Codex, Claude, etc.).

The server must work out-of-the-box with VSCode and Neovim, and it must be agent-first: every LSP capability should be queryable programmatically so that AI assistants can reason about Ash code with the same precision as a human IDE user.

**Scope:** The implemented local LSP MVP covers document sync, parser/lint diagnostics, hover, same-file go-to-definition, document symbols, and keyword/top-level completion. Find references, code actions, workspace symbols, typecheck diagnostics, full MCP parity, config/debounce/panic-isolation hardening, and Salsa-based incremental caching are follow-up work.


## 1.1 Implementation Status Snapshot (TASK-767)

This specification was reconciled against the live code in `crates/ash-lsp` and `crates/ash-lsp-core` by TASK-767. The codebase contains a working **local LSP MVP**, but not the full production/workspace/MCP surface described by the original implementation-grade draft.

Implemented today:

- `ash-lsp-core` VFS, incremental text edit application, position helpers, and per-URI/per-version analysis cache.
- Parser + lint diagnostic aggregation. Typecheck diagnostics are not yet wired into the LSP diagnostic pipeline.
- Keyword/top-level hover, document symbols, same-file token/name based goto-definition, and keyword/top-level definition completion.
- Macro-aware local tooling presentation: macro completions and symbols are not presented as ordinary runtime functions; macro hover displays syntax-phase typed signatures when present; same-file `m!(...)` goto prefers a macro declaration over a same-named function.
- Phase 175 local semantic identity: compact parser/LSP identity keys distinguish syntax-phase macro declarations from ordinary `fn`/`builtin fn` callable declarations, and same-file references split `m!(...)` macro uses from `m()` callable uses when identity is proven. Identity-free names keep the older limited lexical fallback.
- `ash-lsp` stdio transport, one-shot TCP `--port` mode, basic lifecycle notifications, diagnostic publishing, hover, document symbols, definition, and completion.

Not implemented by the local MVP:

- Typecheck diagnostics, expression-level type hover, or `NameBinder`/`TypeEnv` powered semantic navigation.
- Cross-file workspace/module graph indexing, references, workspace symbols, code actions, semantic tokens, signature help, or rename.
- Diagnostic debounce, initialize/config ingestion, max-diagnostic limiting, panic isolation, watched-file handling, or workspace root discovery.
- Salsa database / query engine. The code still uses a simple DashMap cache.
- Full syntax/semantics refresh for later Ash features such as capability interfaces/implementations, resources, operational failure, Proc, generalized `do:K`, and bracket comprehensions.

The rest of this document remains the target architecture unless a later spec supersedes it, but implementation status must be read through this snapshot.

## 2. Non-Goals

- **Not** a generic IDE or GUI.
- **Not** a replacement for `ash check` / `ash run` CLI commands.
- **Not** a remote build system (the server is local-only).
- **Not** a source formatter. Ash has no comment-trivia preservation or pretty-printer. Formatting is deferred to a future spec.
- **Not** a Salsa-based incremental analysis engine. The MVP uses a simple per-file cache; migrating to `salsa` is deferred.

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
│  ash-lsp-core                                                       │
│   ├─ VFS: virtual file system (in-memory overlays + fs watcher)     │
│   ├─ Analysis cache: parse → surface AST → diagnostics              │
│   ├─ Diagnostic aggregator (parse + type + lint)                    │
│   └─ Symbol index: document symbols, references                     │
├─────────────────────────────────────────────────────────────────────┤
│  Compiler Front-End (existing crates)                               │
│   ├─ ash-parser  (lexer → surface AST, spans, errors)               │
│   ├─ ash-typeck  (types, effects, names, obligations)               │
│   ├─ ash-lint    (custom Ash lints — must become a library first)   │
│   └─ ash-engine  (crate graphs, module loader)                      │
├─────────────────────────────────────────────────────────────────────┤
│  MCP Bridge (ash-mcp follow-up / parity surface)                    │
│   ├─ mcp/tools/ash_hover                                            │
│   ├─ mcp/tools/ash_complete                                         │
│   ├─ mcp/tools/ash_diagnostics                                      │
│   ├─ mcp/tools/ash_goto_definition                                  │
│   ├─ mcp/tools/ash_find_references                                  │
│   └─ mcp/tools/ash_symbol_search                                    │
└─────────────────────────────────────────────────────────────────────┘
```

### 3.2 Crate Layout

| Crate | Purpose | New or Existing |
|-------|---------|-----------------|
| `crates/ash-lsp` | Binary + LSP wire protocol | **New** |
| `crates/ash-lsp-core` | VFS, analysis cache, index, diagnostic aggregation | **New** |
| `crates/ash-parser` | Lexer, surface AST (`ModuleFile`), spans, error recovery | Existing |
| `crates/ash-typeck` | Type checker, name resolution, effect inference | Existing |
| `crates/ash-lint` | Custom lints reused by `ash-lsp-core` | Existing library + binary |
| `crates/ash-engine` | Crate graph, module loader, entry verification | Existing |
| `crates/ash-mcp` | MCP server wrapper over `ash-lsp-core` (optional binary) | **New** |

> **Rationale:** Splitting `ash-lsp-core` from `ash-lsp` keeps the analysis engine testable without spinning up a full LSP client/server pair. The MCP bridge can reuse `ash-lsp-core` directly.

## 4. Document Model & Virtual File System

### 4.1 Why a VFS is Mandatory

Ash parsers today read from `&str` or files on disk. LSP clients send `textDocument/didChange` with unsaved buffer contents. Completion requests arrive **before** the file is ever written to disk. Therefore the server must maintain an in-memory overlay.

### 4.2 VFS Design

```rust
use dashmap::DashMap;
use lsp_types::Uri;
use std::time::Instant;

pub struct Vfs {
    files: DashMap<Uri, FileSnapshot>,
}

pub struct FileSnapshot {
    pub source: String,
    pub version: i32,
    pub changed_at: Instant,
}
```

- **Keys:** `lsp_types::Uri` (not raw `String`) to avoid `file://` normalization and percent-encoding bugs.
- **Concurrency:** `DashMap` gives lock-free reads so multiple LSP requests can query the same file simultaneously.
- **Incremental sync:** Support `TextDocumentSyncKind::INCREMENTAL`. A helper `apply_text_edits(source, &[TextDocumentContentChangeEvent]) -> String` normalizes changes into a new snapshot.
- **File watching:** On `workspace/didChangeWatchedFiles`, update disk-backed entries so that modifying `foo.ash` in a terminal refreshes the index without restarting the server.

### 4.3 Change Application

```rust
pub fn apply_changes(text: &str, changes: &[TextDocumentContentChangeEvent]) -> String {
    let mut result = text.to_string();
    if changes.iter().any(|c| c.range.is_none()) {
        return changes.last().unwrap().text.clone();
    }
    // Apply incremental edits in reverse order by start position
    // to keep earlier indices stable.
    // ...
}
```

## 5. Analysis Engine

### 5.1 Query API

`ash-typeck` does **not** expose a high-level query API today. `ash-lsp-core` must build its own orchestration layer:

```rust
// Parse a single file from the VFS.
pub fn parse_file(vfs: &Vfs, uri: &Uri) -> (ash_parser::surface::ModuleFile, Vec<ash_parser::error::ParseError>)

// Check a single file. Requires a module graph for cross-file resolution.
pub fn check_file(vfs: &Vfs, uri: &Uri, graph: &ash_core::module_graph::ModuleGraph)
    -> (ash_typeck::TypeCheckResult, Vec<ash_typeck::error::ConstructorError>)

// Document symbols.
pub fn document_symbols(file: &ash_parser::surface::ModuleFile) -> Vec<Symbol>

// Index queries.
pub fn goto_definition(env: &ash_typeck::type_env::TypeEnv, uri: &Uri, pos: Position) -> Option<Location>
pub fn find_references(index: &ReferenceIndex, uri: &Uri, pos: Position) -> Vec<Location>
```

> **Note:** `ash_parser::surface::ModuleFile` is the real parser output. The spec does not invent `SurfaceAST` or `TypedSurface`.

### 5.2 Caching Strategy

**MVP:** Per-request recomputation with a short-lived LRU cache keyed by `(Url, version)`.

- Parse results are cached per file version.
- Type-check results are cached per file version **and** module graph hash.
- No cross-file incremental invalidation in the MVP.

**Future:** A follow-up spec will evaluate `salsa = "0.26"` for true incremental analysis.

## 6. LSP Capabilities

Capabilities are grouped by **priority**. The MVP must ship Priority 1 and 2; Priority 3 is stretch.

### 6.1 Priority 1 — MVP Foundation (Week 1)

| Capability | LSP Method | Ash Implementation Notes |
|------------|------------|--------------------------|
| **Diagnostics** | `textDocument/publishDiagnostics` | Implemented today for parser + lint diagnostics on `didOpen` / `didChange`; typecheck diagnostics, `didSave`, max-diagnostic limiting, and debounce remain pending. |
| **Document Sync** | `textDocument/didOpen/Change/Close` | Incremental sync. VFS overlay. |
| **Document Symbols** | `textDocument/documentSymbol` | Walk `ModuleFile`; emit `SymbolKind::Function` for workflows, `SymbolKind::Interface` for interfaces, etc. |

### 6.2 Priority 2 — Core Intelligence (Week 2–3)

| Capability | LSP Method | Notes |
|------------|------------|-------|
| **Hover** | `textDocument/hover` | Implemented today for keyword and top-level declaration docs; expression-level type info from `TypeEnv` remains pending. |
| **Go to Definition** | `textDocument/definition` | Implemented today as same-file token/name lookup over parsed declarations with macro/callable identity splitting for supported `m!(...)` versus `m()` cases. `NameBinder`/`TypeEnv` semantic resolution and cross-file navigation remain pending. |
| **Completion** | `textDocument/completion` | Implemented today for keywords/snippets and top-level definition names. Context-aware variables, type-position suggestions, record fields, and current-language-surface refresh remain pending. |
| **Find References** | `textDocument/references` | Implemented for same-file lexical references, with Phase 175 semantic filtering for supported macro/callable identity collisions. Cross-file references still require a workspace reference index. |

### 6.3 Priority 3 — Polish (Week 4–5)

| Capability | LSP Method | Notes |
|------------|------------|-------|
| **Workspace Symbols** | `workspace/symbol` | Search across all `.ash` files in the workspace. |
| **Code Actions** | `textDocument/codeAction` | Quick fixes: "Import missing module", "Add missing match arm". |
| **Semantic Tokens** | `textDocument/semanticTokens/full` | Full semantic highlighting. |
| **Signature Help** | `textDocument/signatureHelp` | Show parameter names when typing inside a call. |
| **Rename** | `textDocument/rename` | Safe rename across the module graph using the reference index. Deferred if time runs short. |

## 7. Diagnostic Pipeline

### 7.1 Sources

1. **Parser** (`ash-parser`) — syntax errors, unexpected tokens. `ParseError` carries `ash_parser::token::Span`.
2. **Type Checker** (`ash-typeck`) — type mismatches, unbound variables, effect errors. **Only** `ConstructorError` carries spans today; `TypeEnvError` and `ExhaustivenessError` do not.
3. **Linter** (`ash-lint`) — OODA loop violations, missing provenance, policy conflicts. (Blocked until `ash-lint` becomes a library.)

### 7.2 Span Requirements

Not all Ash error types are LSP-ready:

| Error Type | Has Span? | Action Required |
|------------|-----------|-----------------|
| `ParseError` | ✅ Yes | Directly convertible. |
| `ConstructorError` | ⚠️ Partial | Most variants have `span`, but `UnknownConstructor` does not. Add `span` to all variants. |
| `TypeEnvError` | ❌ No | **Blocker.** Must add `span: ash_parser::token::Span` to every variant. |
| `ExhaustivenessError` | ❌ No | **Blocker.** Must add `span` to the variant. |
| `NameError` | ❌ No | **Blocker.** Must add `span` to every variant. |

### 7.3 Conversion to LSP Diagnostics

```rust
fn ash_error_to_diagnostic(err: &dyn AshLspError, source: &str) -> Diagnostic {
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

> **Blocker:** A new `AshLspError` trait (or enum) must be defined that provides `span()`, `severity()`, `code()`, and `message()` uniformly across `ParseError`, `ConstructorError`, `TypeEnvError`, `ExhaustivenessError`, and `NameError`.

### 7.4 Debouncing

Typing generates a flood of `didChange` notifications. Diagnostics must be **debounced** (200 ms) so that the server doesn't re-parse the world on every keystroke.

Use a `tokio::sync::mpsc` channel with `tokio::select!` and a `tokio::time::sleep` reset pattern, or `tokio_util::sync::CancellationToken`:

```rust
async fn schedule_validation(&self, uri: Uri) {
    let token = CancellationToken::new();
    self.pending_validations.insert(uri.clone(), token.clone());
    tokio::select! {
        _ = tokio::time::sleep(Duration::from_millis(200)) => {
            if !token.is_cancelled() {
                self.validate_and_publish(uri).await;
            }
        }
        _ = token.cancelled() => {}
    }
}
```

## 8. MCP Interface

### 8.1 Design Principle

The MCP bridge is a thin wrapper over `ash-lsp-core` that speaks the Model Context Protocol. This guarantees that human IDE users and AI agents see the *exact same* analysis.

### 8.2 Transport

- **Default:** `stdio` (standard MCP transport).
- **Optional:** TCP for remote agent orchestration.
- **Launch mode:** `ash lsp --mcp` (see §10).

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

> **Removed:** `ash_apply_edit` is **not** an MCP tool. MCP servers should not write files directly. Edits are returned as structured `CodeAction` responses; it is the client's responsibility to apply them.

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
- Include `suggestion` when the underlying error type provides an expected value.

### 8.5 Context-Aware File Opening

MCP tools should **not** require the caller to manually open files first. The bridge internally calls `vfs.open(file_path)` before querying, and keeps the file in memory for the lifetime of the MCP session.

## 9. Editor Integration

### 9.1 VSCode

- Extension name: `ash-vscode`
- Location: `editors/vscode/`
- Client uses `vscode-languageclient/node`.
- Server path: `${workspaceFolder}/target/release/ash-lsp` (or bundled binary).

```json
// package.json contributes
"activationEvents": ["onLanguage:ash"],
"main": "./out/extension.js"
```

### 9.2 Neovim

No plugin required; native `vim.lsp.config` (Neovim 0.11+) or `lspconfig`.

```lua
-- minimal init.lua
vim.lsp.config['ash-lsp'] = {
  cmd = { 'ash-lsp' },
  filetypes = { 'ash' },
  root_markers = { '.ash.toml', 'ash.toml', '.git' },
}
vim.lsp.enable('ash-lsp')
```

### 9.3 TextMate Grammar (Bonus)

VSCode requires a basic TextMate grammar for syntax highlighting before semantic tokens kick in. Provide `syntaxes/ash.tmLanguage.json` with scopes for:
- `comment.line.double-dash`
- `keyword.control.ash` (`workflow`, `observe`, `act`, `decide`, `if`, `let`, etc.)
- `storage.type.ash` (`capability`, `policy`, `role`)
- `entity.name.function.ash` (function calls)

## 10. CLI Interface

Per **SPEC-005**, the LSP server is launched via the `ash` CLI:

```bash
ash lsp [options]
```

**Options:**

| Option | Description |
|--------|-------------|
| `--stdio` | Use stdio for LSP communication (default) |
| `--port <n>` | Use TCP port for LSP communication |
| `--mcp` | Run in MCP mode instead of LSP mode |

- When `--mcp` is provided, the process speaks MCP over stdio (or TCP if `--port` is also given).
- The canonical binary `ash-lsp` may exist as an internal detail, but the user-facing entry point is always `ash lsp`.

## 11. Implementation Phases

**Realistic MVP timeline: 5 weeks, ~180 hours.**

### Phase 1 — Skeleton & VFS (Week 1, ~32h) — implemented local MVP
- Create `crates/ash-lsp` and `crates/ash-lsp-core`.
- Implement `initialize`, `shutdown`, `didOpen/Change/Close`.
- VFS with incremental sync and `Uri`-based keys.
- Wire up `ash-parser` for syntax validation + diagnostic publishing.
- Add `tracing` integration and basic request logging.

### Phase 2 — Diagnostics & Symbols (Week 2, ~36h) — partially implemented
- Build the `AshLspError` trait and add missing spans to `TypeEnvError`, `ExhaustivenessError`, and `NameError`.
- Integrate `ash-typeck` errors into diagnostics. **Pending.**
- Hover with keyword + type info. Keyword/top-level hover is implemented; type info is pending.
- Document symbols from `ModuleFile`.

### Phase 3 — Navigation & Completion (Week 3, ~40h) — partially implemented
- Go to definition using `NameBinder` + `TypeEnv`. **Pending; current code is same-file token/name lookup.**
- Completion: keywords, in-scope variables, interface/policy names, record fields.
- Find references via a cross-file reference index. **Pending.**

### Phase 4 — MCP Bridge & Agent Integration (Week 4, ~40h) — follow-up
- Create `crates/ash-mcp` with `rmcp = "1.5"`.
- Implement all 8 MCP tools.
- End-to-end tests with simulated agent requests.
- VSCode extension skeleton.

### Phase 5 — Polish & Delivery (Week 5, ~32h) — follow-up
- Workspace symbols.
- Code actions (imports, match arms).
- Semantic tokens (if time permits; otherwise defer).
- Neovim docs, integration tests, CHANGELOG.

## 12. Dependencies

```toml
[dependencies]
# LSP framework (actively maintained fork of tower-lsp)
tower-lsp-server = "0.23"
lsp-types = "0.97"

# Async runtime
tokio = { workspace = true, features = ["full"] }

# Concurrency
parking_lot = "0.12"
dashmap = "6.1"

# Serialization
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"

# MCP SDK (official Anthropic Rust SDK)
rmcp = "1.5"

# Observability
tracing = "0.1"

# Ash internal crates
ash-core = { path = "../ash-core" }
ash-parser = { path = "../ash-parser" }
ash-typeck = { path = "../ash-typeck" }
ash-engine = { path = "../ash-engine" }
```

> **Reconciliation note:** `ash-lint` has since been extracted into a library and is now a real `ash-lsp-core` dependency. MCP work should use the current `ash-mcp` crate/dependency versions rather than the historical draft values in this section.

## 13. Testing Strategy

1. **Unit tests** for VFS change application and `Uri` normalization.
2. **LSP conformance tests** using `lsp-types` + manual JSON-RPC request/response pairs.
3. **Integration tests** that open real `.ash` files from `examples/` and assert on diagnostics, hover text, and symbol counts.
4. **MCP end-to-end tests** that spawn `ash lsp --mcp`, send tool calls, and verify JSON output.

## 14. Security & Sandboxing

- The MCP bridge must validate all file paths to prevent path traversal outside the workspace root.
- No file-write operations through the LSP or MCP interface. All edits are returned as `TextEdit` arrays.
- No arbitrary code execution through the LSP or MCP interface.

## 15. Configuration & Observability

### 15.1 Configuration

- **Workspace root discovery:** `ash.toml` or `.ash.toml` in the workspace root. Fall back to `.git` directory.
- **LSP initialization options:** `ash.lsp.debounce_ms` (default 200), `ash.lsp.max_diagnostics` (default 100).
- **Configuration source:** `initialize` params > workspace `.ash.toml` > defaults.

### 15.2 Logging

- Use `tracing` for structured logging.
- Log levels: `INFO` for initialization and workspace discovery, `DEBUG` for cache hits/misses, `TRACE` for individual LSP messages.
- MCP tool calls are logged at `DEBUG` with tool name and duration.

## 16. Crash Recovery & Panic Isolation

- **Request isolation:** Every LSP request handler must be wrapped in `std::panic::catch_unwind` (or a Tower middleware equivalent) so that a panic in the parser or type checker does not crash the server.
- **Recovery strategy:** On panic, return an LSP `InternalError` and clear the cache entry for the affected file. Do not restart the server.
- **Malformed input:** The parser must use `ash_parser::error_recovery::parse_with_recovery` to return a partial AST on error. Hover and completion should fall back to the last known good AST if the current parse fails.

## 17. Multi-Crate Workspace Support

- The server must support multi-crate workspaces because `ash-engine` already manages crate graphs.
- **Discovery:** On `initialize`, scan the workspace root for `ash.toml` files. Load each crate root with `ash_engine::parse_crate_root`.
- **Module graph:** Build a unified `ModuleGraph` across all discovered crates. Use it for cross-crate goto-definition and workspace symbols.
- **File-to-crate mapping:** Map each `Uri` to its containing crate via parent-directory search for `ash.toml`.

## 18. Known Blockers & Prerequisites

These items were original prerequisites before SPEC-038 engineering. TASK-767 reconfirmed that most substrate prerequisites are now complete, but completing prerequisites did not complete the production LSP surface.

| # | Blocker | Action Required | Estimated Effort |
|---|---------|-----------------|------------------|
| 1 | **Local variable spans** | Add `span: ash_parser::token::Span` to `Expr::Variable` and `Pattern::Variable` in `surface.rs` and `ast.rs`. | 4–6h |
| 2 | **Type-checker error spans** | Add `span: ash_parser::token::Span` to every variant of `TypeEnvError`, `ExhaustivenessError`, and `NameError`. | 8–12h |
| 3 | **Unified error trait** | Define an `AshLspError` trait (or wrapper enum) with `span()`, `severity()`, `code()`, and `message()` methods. | 4–6h |
| 4 | **`ash-lint` library extraction** | Completed by Phase 86 / TASK-574. `ash-lsp-core` depends on the library API. | Done |
| 5 | **Comment trivia (optional)** | If formatter work is ever resumed, the lexer must preserve comments. For the LSP MVP, this is **not** required. | — |

## 19. Relationship to Existing Tasks

- **TASK-059** (`cli-lsp`): The original MVP LSP design doc. SPEC-038 supersedes TASK-059 and expands it into a full implementation spec.
- **TASK-569** (`lsp-mcp-implementation`): The implementation task for SPEC-038. See `docs/plan/tasks/TASK-569-lsp-mcp-implementation.md`.
- **TASK-270** / **TASK-457** (`mcp-provider`): Those tasks define an *outgoing* MCP client inside `ash-engine` (Ash workflows calling external MCP servers). SPEC-038 defines an *incoming* MCP server that exposes Ash language intelligence to external agents. The two directions are complementary.

## 20. Next Steps

1. Treat TASK-569 as complete only for the verified local LSP MVP.
2. Before further LSP feature work, run a syntax/semantics refresh against current Ash language constructs introduced after Phase 89.
3. Create separate task files for typecheck diagnostics/type hover, cross-file workspace indexing, references/code actions/workspace symbols, config/debounce/panic isolation, MCP parity hardening, and Salsa rescoping.
4. Keep TASK-576 planned until the Salsa prerequisite spike and module-level typecheck/query APIs are proven.
