# PLAN-140: MCP Agent Intelligence Spike

## Phase: 140

## Status: ✅ Implemented

## Goal

Make `ash-mcp` the default analysis backend for Ash agent tasks by exposing
reliable, queryable language intelligence through the Model Context Protocol,
and measure whether that intelligence is good enough to replace text-search
fallbacks.

## Background

`crates/ash-lsp-core` already contains a Salsa-backed incremental analysis
cache, diagnostics, hover, go-to-definition, completion, and document symbols.
`crates/ash-mcp` already wraps those capabilities as MCP tools. The
infrastructure exists; this spike is about making it **agent-grade**: robust,
measurable, and wired into the agent loop.

## Scope

### In scope

1. Harden the MCP server for external launch (CLI flags, health check, no
   stdout leakage).
2. Add `ash_workspace_symbols` — directory-wide symbol search.
3. Replace the `ash_find_references` placeholder with a same-file reference
   finder.
4. Build an evaluation harness that scores tool accuracy on agent-style
   queries.
5. Configure Hermes to discover and call `ash-mcp`.

### Out of scope

- Cross-file module graph / source-root resolution.
- Type-checker diagnostics (`ash-typeck` public API still pending).
- Editor incremental sync (we read files from disk).

## Specification

- [SPEC-038: Rust LSP / MCP Research 2025](../spec/SPEC-038-RUST-LSP-MCP-RESEARCH-2025.md)
- [SPEC-043: Incremental Analysis Engine](../spec/SPEC-043-INCREMENTAL-ANALYSIS.md)

## Tasks

| Task | Description | Estimate | Status |
|------|-------------|----------|--------|
| [TASK-1399](tasks/TASK-1399-mcp-server-hardening.md) | MCP server hardening and health check | 3h | ✅ Complete |
| [TASK-1400](tasks/TASK-1400-workspace-symbol-search.md) | Workspace symbol search via MCP | 6h | ✅ Complete |
| [TASK-1401](tasks/TASK-1401-single-file-find-references.md) | Single-file find-references via MCP | 4h | ✅ Complete |
| [TASK-1402](tasks/TASK-1402-agent-evaluation-harness.md) | Agent evaluation harness for MCP tools | 4h | ✅ Complete |
| [TASK-1403](tasks/TASK-1403-hermes-mcp-config.md) | Hermes MCP server configuration | 2h | ✅ Complete |
| [TASK-1404](tasks/TASK-1404-mcp-spike-closeout.md) | Spike closeout and evaluation report | 3h | ✅ Complete |

## Deliverable

- `ash-mcp` starts reliably from Hermes and exposes health, diagnostics,
  symbols, goto, references, completion, hover, and workspace-symbol tools.
- Evaluation harness reports query pass rate.
- Spike report documents what worked, what did not, and the recommended next
  phase.

## Timeline

~22 hours (bounded spike).

## Risks

| Risk | Mitigation |
|------|------------|
| `rmcp` API changes | Pinned to `1.5` in `Cargo.toml`. |
| Workspace scan slow on large trees | Limit to `.ash` files; measure before optimizing. |
| Symbol names don't match agent expectations | Return raw name + file/line; evaluate on real fixtures. |
| Cross-file false positives | Keep references single-file; document limitation honestly. |

## Decision Log

| Date | Decision | Rationale |
|------|----------|-----------|
| 2026-06-11 | Start with file-scanning workspace symbols instead of full salsa workspace index | Lower risk; defers module-graph work. |
| 2026-06-11 | Same-file references only | Cross-file resolution needs source roots and import graph. |
| 2026-06-11 | Use `#[doc(hidden)]` public wrappers for integration test access to private `#[tool_router]` methods | Macro-generated methods are private; wrappers keep API clean while enabling external tests. |
| 2026-06-11 | Use release binary path in Hermes config instead of `cargo run` | Hermes MCP connection test times out on `cargo run` due to compilation delay. |

## Notes

- This plan was created after inspecting `crates/ash-lsp-core` and
  `crates/ash-mcp` and confirming they compile and pass existing tests.
- The existing `ash_find_references`, `ash_workspace_symbols`, and
  `ash_code_action` tools are placeholders; this spike replaces two of them.
