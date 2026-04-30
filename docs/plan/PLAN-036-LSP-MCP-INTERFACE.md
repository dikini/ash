# PLAN-036: LSP & MCP Interface

## Phase: 87

## Status: ✅ Complete (Local LSP MVP Only; Production Follow-Ups Planned)

## Goal

Establish an Ash Language Server Protocol implementation and shared `ash-lsp-core` analysis substrate. After TASK-767 reconciliation, this plan is scoped honestly as a completed **local LSP MVP** plus explicitly pending follow-up tracks for production/workspace/MCP features.

## Specification

- [SPEC-038: Ash Language Server Protocol (LSP) & MCP Interface](../spec/SPEC-038-LANGUAGE-SERVER.md)
- [SPEC-005: CLI Specification — LSP section](../spec/SPEC-005-CLI.md)

## Tasks

| Task | Description | Estimate | Status |
|------|-------------|----------|--------|
| [TASK-569](tasks/TASK-569-lsp-mcp-implementation.md) | Local LSP MVP: VFS/cache, parser+lint diagnostics, hover, document symbols, same-file goto-definition, completion, stdio/TCP binary | 180h original / implemented subset | ✅ Complete (Local MVP) |
| [TASK-767](tasks/TASK-767-lsp-status-reconciliation.md) | Reconcile LSP planning/status claims against live code and record syntax/semantics drift before further LSP work | 2-4h | ✅ Complete |

## Hard Prerequisites

The original blockers from SPEC-038 §18 have been partially resolved by Phases 84-86:

1. ✅ Local variable spans — Phase 84 / TASK-570.
2. ✅ Type-checker error spans — Phase 85 / TASK-572.
3. ✅ Unified error trait — Phase 85 / TASK-573.
4. ✅ `ash-lint` library extraction — Phase 86 / TASK-574.
5. ✅ `parse_surface_file` API — Phase 84 / TASK-571.

However, these prerequisites do **not** mean the production LSP is complete. The current `ash-lsp-core` diagnostic path still does not call typecheck, and navigation/completion remain parser/top-level oriented.

## Implemented Deliverable

- `crates/ash-lsp-core` crate.
- `crates/ash-lsp` binary crate.
- VFS with open/change/close and incremental edit application.
- Per-URI/per-version analysis cache.
- Parser + lint diagnostics.
- `textDocument/hover` for keywords/top-level declarations.
- `textDocument/documentSymbol`.
- `textDocument/definition` for same-file declaration lookup.
- `textDocument/completion` for keywords/snippets and top-level names.
- stdio launch and TCP `--port` launch on the `ash-lsp` binary.
- Focused tests: 49 `ash-lsp-core` tests and 8 `ash-lsp` tests passed during the TASK-767 audit.

## Pending Follow-Up Tracks

These were formerly overclaimed by the broad Phase 87 wording and are now explicitly pending:

1. Typecheck diagnostics and expression-level type hover.
2. Cross-file workspace/module graph index.
3. `textDocument/references`, `workspace/symbol`, and `textDocument/codeAction`.
4. Config ingestion, diagnostic debouncing, max-diagnostic limiting, watched-file handling, and panic isolation.
5. Current-Ash syntax/semantics refresh for capability/resource declarations, operational failure, Proc, generalized `do:K`, bracket comprehensions, and std/module syntax drift.
6. MCP parity hardening if `ash-mcp` is to share exactly the same LSP-core query semantics.
7. Salsa incremental engine follow-up through Phase 89 / TASK-576 after prerequisite spike and possible rescope.

## Syntax/Semantics Drift Warning

Ash language development after Phase 89 materially changed the surface and semantic vocabulary. Any new LSP feature work must first audit current parser/typechecker/runtime behavior for post-Phase-89 features rather than assuming the original SPEC-038 examples and keyword/completion lists are current.

## Timeline

Original estimate: 5 weeks / ~180h for a production LSP+MCP interface.

Reconciled status:

- Local LSP MVP: implemented.
- Production/workspace/MCP hardening: unestimated follow-up work pending new task files.
- Salsa: separate Phase 89 task remains planned/blocked.

## Risks

- Planning documents can overclaim implementation maturity if local MVP and production LSP are not separated.
- Later Ash syntax/semantics drift can make hover/completion snippets stale even if the parser still accepts the source.
- Cross-file LSP features depend on module graph and typecheck APIs that the current `ash-lsp-core` does not use.
