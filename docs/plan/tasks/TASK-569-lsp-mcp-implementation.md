# TASK-569: LSP & MCP Interface for Ash

## Status: Planned

## Description

Implement a Language Server Protocol (LSP) server for the Ash workflow language with an embedded Model Context Protocol (MCP) interface. The server will provide real-time diagnostics, hover, go-to-definition, completion, and code actions for human editors (VSCode, Neovim) while exposing the same semantic intelligence as MCP tools for AI coding agents (Hermes, Codex, Claude, Open Code, OpenClaw).

## Specification Reference

- [SPEC-038: Ash Language Server Protocol (LSP) & MCP Interface](../../spec/SPEC-038-LANGUAGE-SERVER.md)
- [SPEC-005: CLI Specification — LSP section](../../spec/SPEC-005-CLI.md)

## Requirements

### 1. LSP Core (`crates/ash-lsp`)
- Use `tower-lsp-server` for the JSON-RPC protocol layer.
- Support stdio and TCP transports.
- Implement `initialize`, `shutdown`, `didOpen`, `didChange`, `didClose`.
- Advertise capabilities: text document sync (incremental), hover, completion, diagnostics, document symbols, formatting (when available).

### 2. Analysis Engine (`crates/ash-lsp-core`)
- Build a Virtual File System (VFS) with in-memory overlays and incremental change application.
- Cache parsed surface AST per file version.
- Aggregate diagnostics from `ash-parser`, `ash-typeck`, and `ash-lint`.
- Debounce diagnostic publishing (200 ms) after `didChange`.

### 3. LSP Handlers (Priority Order)
1. **Diagnostics** — publish on open, change, save.
2. **Hover** — keyword docs + type information from `ash-typeck`.
3. **Go to Definition** — using existing name-binding tables.
4. **Document Symbols** — outline view from surface AST.
5. **Completion** — keywords, in-scope variables, capability/policy names, record fields.
6. **Find References** — cross-file symbol usage index.
7. **Formatting** — requires Ash formatter (greenfield work).
8. **Code Actions** — quick fixes (imports, match arm completion).

### 4. MCP Bridge (`crates/ash-mcp`)
- Expose MCP tools that wrap `ash-lsp-core` queries:
  - `ash_get_diagnostics`
  - `ash_hover`
  - `ash_goto_definition`
  - `ash_find_references`
  - `ash_complete`
  - `ash_document_symbols`
  - `ash_workspace_symbols`
  - `ash_code_action`
  - `ash_apply_edit`
- Respond with token-efficient JSON including a one-line `summary`.
- Auto-open files on first tool call; keep them in memory for the session.

### 5. Editor Support
- Provide a minimal VSCode extension skeleton (`editors/vscode/`).
- Provide Neovim `lspconfig` / `vim.lsp.config` setup documentation.

### 6. Testing
- Unit tests for VFS change application.
- LSP conformance tests (manual JSON-RPC request/response pairs).
- Integration tests against real `.ash` files in `examples/`.
- MCP end-to-end tests spawning `ash-mcp` and asserting tool outputs.

## TDD Steps

### Red
- No LSP or MCP server exists for Ash beyond the stub in TASK-059.
- `ash-lint` is a CLI binary; its diagnostic logic must be extracted into a library.

### Green
- `ash-lsp` binary responds to `initialize` with correct capabilities.
- `ash-lsp-core` parses an open file and returns diagnostics mapped from `ParseError` spans.
- Hover and go-to-definition return accurate results for example files.
- MCP tools return structured JSON that agents can consume.

## Completion Checklist

- [ ] `crates/ash-lsp` created with basic LSP skeleton
- [ ] `crates/ash-lsp-core` created with VFS and diagnostic aggregator
- [ ] Incremental text document sync implemented and tested
- [ ] Diagnostic pipeline wired to `ash-parser`, `ash-typeck`, `ash-lint`
- [ ] Hover handler implemented
- [ ] Go-to-definition handler implemented
- [ ] Document symbol handler implemented
- [ ] Completion handler implemented
- [ ] Find references handler implemented
- [ ] Formatter implemented (or deferred with clear follow-up task)
- [ ] `crates/ash-mcp` created with MCP tool bridge
- [ ] VSCode extension skeleton provided
- [ ] Neovim setup documented
- [ ] Unit and integration tests passing
- [ ] `cargo test --workspace` clean for affected crates
- [ ] `cargo clippy --all-targets --all-features` clean
- [ ] `cargo fmt --check` clean
- [ ] CHANGELOG.md updated

## Estimated Effort

4–5 weeks (1 engineer full-time)

## Dependencies

- `tower-lsp-server`
- `tokio`
- `dashmap`
- `serde` / `serde_json`
- `ash-parser`
- `ash-typeck`
- `ash-lint` (refactored to lib)
- `ash-engine`

## Blocked By

- TASK-059 (CLI LSP command skeleton — can be merged into this work)
- `ash-lint` library extraction

## Blocks

- Future IDE-first workflows
- Agent-driven Ash development (Hermes/Codex deep integration)
