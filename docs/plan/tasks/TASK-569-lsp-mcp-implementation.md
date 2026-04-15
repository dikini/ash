# TASK-569: LSP & MCP Interface for Ash

## Status: Planned

## Description

Implement a Language Server Protocol (LSP) server for the Ash workflow language with an embedded Model Context Protocol (MCP) interface. The server will provide real-time diagnostics, hover, go-to-definition, completion, and code actions for human editors (VSCode, Neovim) while exposing the same semantic intelligence as MCP tools for AI coding agents (Hermes, Codex, Claude).

This task implements the MVP defined in SPEC-038. Source formatting and Salsa-based incremental caching are explicitly out of scope and deferred to follow-up work.

## Specification Reference

- [SPEC-038: Ash Language Server Protocol (LSP) & MCP Interface](../../spec/SPEC-038-LANGUAGE-SERVER.md)
- [SPEC-005: CLI Specification — LSP section](../../spec/SPEC-005-CLI.md)

## Hard Prerequisites (Must Complete First)

These blockers are defined in SPEC-038 §18. Engineering must not begin until they are resolved.

1. **Local variable spans** — Add `span: ash_parser::token::Span` to `Expr::Variable` and `Pattern::Variable` in `surface.rs` and `ast.rs`.
2. **Type-checker error spans** — Add `span: ash_parser::token::Span` to every variant of `TypeEnvError`, `ExhaustivenessError`, and `NameError`.
3. **Unified error trait** — Define an `AshLspError` trait (or wrapper enum) with `span()`, `severity()`, `code()`, and `message()` methods.
4. **`ash-lint` library extraction** — Convert `crates/ash-lint` from a CLI-only binary into a library crate with a public `lint_module(source: &str) -> Vec<LintDiagnostic>` API.

## Requirements

### 1. LSP Core (`crates/ash-lsp`)
- Use `tower-lsp-server = "0.23"` for the JSON-RPC protocol layer.
- Support stdio and TCP transports per SPEC-005 (`ash lsp --stdio` and `ash lsp --port <n>`).
- Implement `initialize`, `shutdown`, `didOpen`, `didChange`, `didClose`.
- Advertise capabilities: text document sync (incremental), hover, completion, diagnostics, document symbols.
- **Do not** advertise `textDocument/formatting` (deferred).
- Wrap every request handler in panic isolation (`catch_unwind` or equivalent) so parser/type-checker panics do not crash the server.

### 2. Analysis Engine (`crates/ash-lsp-core`)
- Build a Virtual File System (VFS) with `lsp_types::Url` keys, in-memory overlays, and incremental change application.
- Cache parsed `ash_parser::surface::ModuleFile` per file version.
- Aggregate diagnostics from `ash-parser`, `ash-typeck`, and `ash-lint`.
- Debounce diagnostic publishing (200 ms, configurable) after `didChange` using `tokio::time::sleep` + `CancellationToken`.
- Build a cross-file reference index for "Find References" and workspace symbols.
- Support multi-crate workspaces by discovering `ash.toml` / `.ash.toml` files and loading crate graphs via `ash-engine`.

### 3. LSP Handlers (Priority Order)
1. **Diagnostics** — publish on open, change, save.
2. **Document Symbols** — outline view from surface AST.
3. **Hover** — keyword docs + type information from `ash-typeck`.
4. **Go to Definition** — using `NameBinder` and `TypeEnv`.
5. **Completion** — keywords, in-scope variables, interface/policy names, record fields.
6. **Find References** — cross-file symbol usage index.
7. **Code Actions** — quick fixes (imports, match arm completion).
8. **Workspace Symbols** — search across all `.ash` files.

### 4. MCP Bridge (`crates/ash-mcp`)
- Built on `rmcp = "1.4"` (official Anthropic Rust SDK).
- Expose MCP tools that wrap `ash-lsp-core` queries:
  - `ash_get_diagnostics`
  - `ash_hover`
  - `ash_goto_definition`
  - `ash_find_references`
  - `ash_complete`
  - `ash_document_symbols`
  - `ash_workspace_symbols`
  - `ash_code_action`
- **No direct file-write tool.** Edits are returned as `CodeAction` / `TextEdit` arrays only.
- Respond with token-efficient JSON including a one-line `summary`.
- Auto-open files on first tool call; keep them in memory for the session.
- Launch mode: `ash lsp --mcp` (per SPEC-005).

### 5. Editor Support
- Provide a minimal VSCode extension skeleton (`editors/vscode/`).
- Provide Neovim `lspconfig` / `vim.lsp.config` setup documentation.
- Provide a basic TextMate grammar (`syntaxes/ash.tmLanguage.json`) for syntax highlighting.

### 6. Configuration & Observability
- Read LSP config from `initialize` params and workspace `.ash.toml`:
  - `ash.lsp.debounce_ms` (default 200)
  - `ash.lsp.max_diagnostics` (default 100)
- Use `tracing` for structured logging at `INFO`/`DEBUG`/`TRACE` levels.

### 7. Testing
- Unit tests for VFS change application and `Url` normalization.
- LSP conformance tests (manual JSON-RPC request/response pairs).
- Integration tests against real `.ash` files in `examples/`.
- MCP end-to-end tests spawning `ash lsp --mcp` and asserting tool outputs.

## TDD Steps

### Red
- No LSP or MCP server exists for Ash beyond the design doc in TASK-059.
- `ash-lint` is a CLI binary; its diagnostic logic must be extracted into a library.
- `TypeEnvError`, `ExhaustivenessError`, and `NameError` lack source spans.

### Green
- `ash-lsp` binary responds to `initialize` with correct capabilities.
- `ash-lsp-core` parses an open file and returns diagnostics mapped from `ParseError` spans.
- Hover and go-to-definition return accurate results for example files.
- MCP tools return structured JSON that agents can consume.

## Completion Checklist

- [ ] All 4 hard prerequisites resolved
- [ ] `crates/ash-lsp` created with basic LSP skeleton
- [ ] `crates/ash-lsp-core` created with VFS and diagnostic aggregator
- [ ] Incremental text document sync implemented and tested
- [ ] Diagnostic pipeline wired to `ash-parser`, `ash-typeck`, `ash-lint`
- [ ] Hover handler implemented
- [ ] Go-to-definition handler implemented
- [ ] Document symbol handler implemented
- [ ] Completion handler implemented
- [ ] Find references handler implemented
- [ ] Code actions handler implemented
- [ ] Workspace symbols handler implemented
- [ ] `crates/ash-mcp` created with MCP tool bridge
- [ ] VSCode extension skeleton provided
- [ ] Neovim setup documented
- [ ] Unit and integration tests passing
- [ ] `cargo test --workspace` clean for affected crates
- [ ] `cargo clippy --all-targets --all-features` clean
- [ ] `cargo fmt --check` clean
- [ ] CHANGELOG.md updated

## Estimated Effort

**~180 hours (5 weeks, 1 engineer full-time)**

| Phase | Work | Hours |
|-------|------|-------|
| Prerequisites | Spans, error trait, `ash-lint` lib | 24–36 |
| Phase 1 | Skeleton + VFS + parser diagnostics | 32 |
| Phase 2 | Typeck diagnostics + hover + symbols | 36 |
| Phase 3 | Go-to-definition + completion + references | 40 |
| Phase 4 | MCP bridge + VSCode skeleton | 40 |
| Phase 5 | Polish, tests, docs, CHANGELOG | 32 |

## Dependencies

- `tower-lsp-server = "0.23"`
- `tokio = "1.52"`
- `dashmap = "6.1"`
- `serde` / `serde_json`
- `rmcp = "1.4"`
- `tracing = "0.1"`
- `ash-parser`
- `ash-typeck`
- `ash-lint` (refactored to lib)
- `ash-engine`

## Blocked By

- Span addition to `Expr::Variable` and `Pattern::Variable`
- Span addition to `TypeEnvError`, `ExhaustivenessError`, and `NameError`
- Creation of `AshLspError` trait/uniform diagnostic wrapper
- `ash-lint` library extraction

## Blocks

- Future IDE-first workflows
- Agent-driven Ash development (Hermes/Codex deep integration)
- Source formatter (depends on LSP skeleton existing)
