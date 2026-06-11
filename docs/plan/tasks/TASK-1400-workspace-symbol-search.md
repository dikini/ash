# TASK-1400: Workspace symbol search via MCP

## Status: 📝 Planned

## Description

Add cross-file symbol discovery so agents can ask "where is `helper` defined?" across a directory tree without reading every file. This is a bounded, file-scanning implementation; true incremental cross-file indexing is deferred.

## Specification Reference

- [PLAN-140: MCP Agent Intelligence Spike](../PLAN-140-MCP-AGENT-INTELLIGENCE-SPIKE.md)
- [SPEC-038: Rust LSP / MCP Research 2025](../../spec/SPEC-038-RUST-LSP-MCP-RESEARCH-2025.md)
- [SPEC-043: Incremental Analysis Engine](../../spec/SPEC-043-INCREMENTAL-ANALYSIS.md) (deferred true incremental indexing)

## Dependencies

- TASK-1399 complete.

## Requirements

### Functional Requirements

- In `ash-lsp-core`, add `workspace_symbols(root: &Path, query: &str) -> Vec<WorkspaceSymbol>`.
- Recursively scan `root` for `*.ash` files.
- Parse each file and collect top-level symbols from `symbols::document_symbols`.
- Match names case-insensitively by substring (MVP; exact/fuzzy deferred).
- Return: name, kind, file path, line, column.
- Expose as MCP tool `ash_workspace_symbols` with params `{ "root": string, "query": string }`.
- Return honest empty result with summary when no matches.

### Non-Functional Requirements

- Handle parse failures gracefully: skip file, log at debug, continue scan.
- Respect IO errors: return a clear MCP error if `root` is unreadable.
- No `unsafe` code.

## Files

- Modify: `crates/ash-lsp-core/src/symbols.rs` (or new `crates/ash-lsp-core/src/workspace.rs`)
- Modify: `crates/ash-mcp/src/lib.rs`
- Modify: `crates/ash-mcp/src/tests.rs`

## TDD Steps

1. Write unit test for `workspace_symbols` on a temp directory with two `.ash` files.
2. Write test for query matching substring case-insensitively.
3. Write test for parse-failed file being skipped.
4. Implement scanning and symbol collection.
5. Expose via MCP and add integration test.

## Verification

- [ ] `cargo test -p ash-lsp-core -p ash-mcp` passes.
- [ ] `cargo clippy -p ash-lsp-core -p ash-mcp --all-targets -- -D warnings` passes.
- [ ] `cargo fmt --check` passes.
- [ ] Workspace symbol query finds symbols across multiple files.
- [ ] CHANGELOG.md updated.
