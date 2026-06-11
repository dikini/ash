# TASK-1399: MCP server hardening and health check

## Status: 📝 Planned

## Description

Stabilize the `ash-mcp` binary so it can be launched reliably by external agents (Hermes, Codex, Claude Code) and report its own health. This is the substrate for all subsequent MCP-based agent tooling.

## Specification Reference

- [PLAN-140: MCP Agent Intelligence Spike](../PLAN-140-MCP-AGENT-INTELLIGENCE-SPIKE.md)
- [SPEC-038: Rust LSP / MCP Research 2025](../../spec/SPEC-038-RUST-LSP-MCP-RESEARCH-2025.md) §8.5 (VFS session state)

## Dependencies

- Existing `crates/ash-mcp` and `crates/ash-lsp-core` compile and pass tests.

## Requirements

### Functional Requirements

- Add `--version` and `--help` CLI flags to `ash-mcp`.
- Ensure stdio transport emits no stray stdout (tracing to stderr only); verify under launch from a parent process.
- Add an `ash_mcp_health` tool that returns:
  - Server status (`ok`).
  - List of available tool names.
  - `ash-lsp-core` version / workspace version.
- Add unit tests for health tool output.

### Non-Functional Requirements

- No breaking changes to existing MCP tools.
- `cargo clippy --all-targets` clean.
- `cargo fmt --check` clean.

## Files

- Modify: `crates/ash-mcp/src/main.rs`
- Modify: `crates/ash-mcp/src/lib.rs`
- Modify: `crates/ash-mcp/src/tests.rs`

## TDD Steps

1. Write test expecting `ash_mcp_health` to return status `ok` and a non-empty tool list.
2. Implement health tool.
3. Write test for `--version` returning the workspace version.
4. Implement CLI flags.
5. Verify no stdout leakage by capturing stdio during a mock handshake.

## Verification

- [ ] `cargo test -p ash-mcp` passes.
- [ ] `cargo clippy -p ash-mcp --all-targets -- -D warnings` passes.
- [ ] `cargo fmt --check` passes.
- [ ] Health tool returns expected JSON shape.
- [ ] CHANGELOG.md updated.
