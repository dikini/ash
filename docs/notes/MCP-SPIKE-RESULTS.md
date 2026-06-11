# MCP Agent Intelligence Spike — Results Report

## Summary

Phase 140 was a bounded spike to determine whether exposing Ash language
intelligence through an MCP server measurably improves agent development
workflows. The spike delivered four working MCP tools, an evaluation harness,
and Hermes integration. All tasks passed their quality gates.

## What Worked

### MCP Server Hardening (TASK-1399)
- `ash-mcp` now supports `--version` and `--help` CLI flags.
- `ash_mcp_health` tool reports status, version, and available tool names.
- Binary-level tests verify stdio cleanliness (no stray stdout on launch).
- **Result**: Server is stable enough for external agent launch.

### Workspace Symbol Search (TASK-1400)
- `workspace_symbols()` recursively scans `.ash` files, parses them, and
  returns top-level symbols matching a case-insensitive substring query.
- Exposed via `ash_workspace_symbols` MCP tool with `root` and `query` params.
- **Test results**: 8/8 unit tests pass; 2/2 MCP-level integration tests pass.
- **Limitation**: Uses directory scanning instead of Salsa workspace indexing;
  module-graph-aware search is deferred.

### Single-File Find-References (TASK-1401)
- `find_references()` scans source text for token-boundary matches of the
  identifier at the cursor position.
- Returns all occurrences within the same file, including the definition site.
- Exposed via `ash_find_references` MCP tool.
- **Test results**: 5/5 unit tests pass; 2/2 MCP-level integration tests pass.
- **Limitation**: Single-file only; cross-file references deferred to avoid
  import-graph complexity.

### Agent Evaluation Harness (TASK-1402)
- Fixture directory with 3 `.ash` files spanning multiple definitions.
- Integration test suite (`crates/ash-mcp/tests/agent_queries.rs`) covering
  workspace symbol search, find-references, and go-to-definition.
- Metric summary printed per run: 7 queries, 7 passed, 0 failed.
- **Result**: 8/8 integration tests pass (includes summary test).

### Hermes Integration (TASK-1403)
- `.hermes/mcp_servers.yaml` added for project-local server discovery.
- `docs/notes/MCP-HERMES-INTEGRATION.md` documents setup and troubleshooting.
- All 9 ash-mcp tools discovered and enabled via `hermes mcp`.
- **Verification**: `hermes mcp test ash-mcp` connects successfully and lists
  9 tools.

## What Did Not Work / Limitations

1. **Cross-file analysis is deferred**: Both `find_references` and
   `goto_definition` are single-file only. Resolving symbols across modules
   requires import-graph and source-root awareness that was out of scope for
   this spike.

2. **Workspace symbols use directory scanning**: The current implementation
   walks the filesystem and parses each `.ash` file independently. A Salsa-based
   workspace index would be faster and more consistent with the existing
   `ash-lsp-core` architecture, but was deferred to keep the spike bounded.

3. **No typeck diagnostics in MCP**: `ash_get_diagnostics` returns parse and
   lint diagnostics but not typechecker output. Integrating `ash-typeck` into
   the MCP pipeline is a natural next step.

4. **`cargo run` too slow for Hermes connection timeout**: Hermes's `mcp add`
   test times out when launching via `cargo run`. The release binary works.
   Documented in `MCP-HERMES-INTEGRATION.md`.

## Measured Query Accuracy

| Query Type | Tests | Passed | Accuracy |
|---|---|---|---|
| Workspace symbols (helper) | 1 | 1 | 100% |
| Workspace symbols (read) | 1 | 1 | 100% |
| Workspace symbols (sensor) | 1 | 1 | 100% |
| Find references (helper in main) | 1 | 1 | 100% |
| Find references (sensor in main) | 1 | 1 | 100% |
| Goto definition (helper deferred) | 1 | 1 | 100% |
| Goto definition (main workflow) | 1 | 1 | 100% |
| **Total** | **7** | **7** | **100%** |

*Note: These are synthetic fixture-based tests. Real-world accuracy on
production Ash codebases requires broader evaluation.*

## Recommended Next Phase

**Scale cross-file analysis** (estimated 16–24 h):

1. **Module graph integration**: Wire `ash-mcp` into the Salsa-based
   `AshLspDatabase` so workspace symbols are indexed incrementally rather than
   scanned on each query.

2. **Cross-file goto-definition**: Resolve imports using the module graph and
   search across crate boundaries.

3. **Cross-file find-references**: Index symbol usages across the workspace
   (likely via a second Salsa query).

4. **Typeck diagnostics**: Pipe `ash-typeck` output through the MCP
   `ash_get_diagnostics` tool.

**Alternative**: If cross-file analysis proves too complex, **pivot to
IDE-focused LSP improvements** (hover quality, completion depth) which benefit
both human users and agents.

## Evidence

- Commits on branch `feat/phase-140-mcp-spike`:
  - `1a64f2a8` — docs(plan): Phase 140 MCP Agent Intelligence Spike
  - `7ed6db17` — feat(mcp): TASK-1399 MCP server hardening and health check
  - `3a4a7a17` — feat(mcp): TASK-1400 workspace symbol search via MCP
  - `85ea42f7` — feat(mcp): TASK-1401 single-file find-references via MCP
  - `f39fef82` — feat(mcp): TASK-1402 agent evaluation harness for MCP tools
  - `95ec8ae2` — feat(mcp): TASK-1403 Hermes MCP server configuration

- All commits pass the full pre-commit gate (cargo check, clippy, fmt, tests,
  doc-tests, fuzz smoke).
