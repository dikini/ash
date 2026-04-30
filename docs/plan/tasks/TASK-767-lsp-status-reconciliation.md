# TASK-767: LSP Status Reconciliation and Syntax/Semantics Drift Audit

## Status: ✅ Complete

## Description

Reconcile the LSP/tooling planning corpus against the live implementation in `crates/ash-lsp` and `crates/ash-lsp-core` before any further LSP work. This task is a documentation/status correction task: it does not implement new LSP behavior. It records the verified local LSP MVP, downgrades or splits over-broad claims, and captures the syntax/semantics drift that accumulated after later Ash language phases.

## Specification Reference

- [SPEC-038: Ash Language Server Protocol (LSP) & MCP Interface](../../spec/SPEC-038-LANGUAGE-SERVER.md)
- [SPEC-043: Incremental Analysis Engine](../../spec/SPEC-043-INCREMENTAL-ANALYSIS.md)
- [PLAN-036: LSP & MCP Interface](../PLAN-036-LSP-MCP-INTERFACE.md)
- [PLAN-035: Incremental Analysis Engine](../PLAN-035-INCREMENTAL-ANALYSIS.md)
- [TASK-569: LSP & MCP Interface for Ash](TASK-569-lsp-mcp-implementation.md)
- [TASK-576: Integrate Salsa into ash-lsp-core](TASK-576-ash-lsp-salsa.md)

## Dependencies

- ✅ `crates/ash-lsp` exists and builds.
- ✅ `crates/ash-lsp-core` exists and builds.
- ✅ Phase 87/89 planning artifacts exist.
- ✅ Later Ash language development has progressed through Phase 107 planning, so LSP syntax/semantic assumptions must be re-audited before adding new LSP features.

## Ground Truth Verified

The live code provides a local single-document LSP MVP:

- `crates/ash-lsp-core`:
  - VFS with open/change/close lifecycle.
  - Full-document and incremental text-change application.
  - Line/column ↔ byte-offset helpers.
  - DashMap-backed per-URI/per-version analysis cache.
  - Parser + lint diagnostic aggregation.
  - Keyword/top-level hover.
  - Same-file go-to-definition by token/name matching.
  - Keyword/top-level definition completion.
  - Document symbols from `ModuleFile`.
- `crates/ash-lsp`:
  - `tower-lsp-server` binary.
  - stdio transport by default and one-shot TCP via `--port`.
  - `initialize`, `shutdown`, `didOpen`, `didChange`, `didClose`.
  - diagnostic publish/clear.
  - `textDocument/hover`, `textDocument/documentSymbol`, `textDocument/definition`, and `textDocument/completion` handlers.

## Discrepancies Found and Reconciled

| Claim Surface | Previous Claim | Reconciled State |
|---------------|----------------|------------------|
| `PLAN-INDEX.md` Phase 87 | Full production LSP/MCP interface done, including references and VSCode skeleton | Phase 87 is complete only for the local LSP MVP. MCP and advanced LSP surfaces remain planned/follow-up work. |
| `PLAN-INDEX.md` Phase 89 | Salsa incremental engine done | Phase 89 is planned/blocked/rescoped. No `salsa` dependency or database exists in `ash-lsp-core`; the simple DashMap cache remains. |
| `PLAN-036` / `TASK-569` | TASK-569 planned, but also broad 180h production scope | TASK-569 is complete for the implemented local MVP only; pending surfaces are explicitly split out. |
| `TASK-576` vs `PLAN-INDEX` | Task file planned but PLAN-INDEX done | `TASK-576` remains planned and blocked on typecheck/module graph/query prerequisites. |
| `SPEC-038` | MVP includes find references and MCP bridge | Implemented MVP is narrower: diagnostics, document sync, hover, document symbols, goto-definition, completion. Find references, MCP bridge, code actions, workspace symbols, config/debounce/panic isolation remain pending. |
| Diagnostic pipeline | parse + typeck + lint | Actual pipeline is parse + lint. Typecheck diagnostic integration is pending. |
| Navigation | NameBinder/TypeEnv/cross-file semantic navigation | Actual goto-definition is same-file token/name matching over parsed declarations. |
| Incremental analysis | Salsa database and cross-file invalidation | Actual analysis cache is per-URI/per-version DashMap; no cross-file invalidation. |

## Syntax and Semantics Drift Identified After Later Ash Development

The current LSP implementation was built against the Phase 84-89 language/tooling shape. Later Ash work added or changed major language surfaces that the LSP intelligence has not been re-audited against:

1. Capability/resource declarations:
   - `capability interface`
   - `capability impl`
   - `resource type`
   - workflow `owns` headers
   - workflow `uses` bindings
2. Runtime authority and capability implementation semantics:
   - Ash-defined implementation bodies
   - internal/derived authority provenance
   - resource/capability binding admission metadata
3. Operational/process tower surfaces:
   - `fail`
   - `with_error`
   - `Proc<T>` / `P<T>` constructors
   - `proc::unit`, `proc::bind`, `proc::then`, `proc::par`, `proc::await`, `proc::join`, `proc::gather`, `proc::from_act`
4. Generalized typed do notation and comprehension syntax:
   - `do:Act { ... }`
   - `do:Proc { ... }`
   - new-form `act { ... }` compatibility through generalized do
   - bracket comprehensions with explicit target annotations
5. Corpus/module syntax drift from Phase 107 planning:
   - stdlib import/export behavior
   - semicolonless ordinary imports vs multiline import scanning
   - comment syntax and opaque parser diagnostics
   - current expected-pass/expected-fail stdlib and example corpus classifications

Current LSP keyword docs, completion snippets, hover text, syntax examples, and semantic assumptions therefore need a dedicated refresh before adding advanced LSP features. The parser may accept many new constructs, but LSP semantic intelligence does not yet expose correct type/authority/process/comprehension-aware behavior for them.

## Downgraded / Split Claims

Completed:

- ✅ Local LSP MVP: document sync, parser/lint diagnostics, hover, document symbols, same-file goto-definition, keyword/top-level completion.

Pending follow-up work:

- 📝 Typecheck diagnostics and expression-level type hover.
- 📝 Cross-file workspace/module graph index.
- 📝 `textDocument/references`, `textDocument/codeAction`, `workspace/symbol`.
- 📝 Config ingestion, diagnostic debouncing, and request panic isolation.
- 📝 MCP bridge completion/verification where not already covered by `ash-mcp`'s independent implementation surface.
- 📝 Syntax/semantics refresh for post-Phase-89 Ash language surfaces.
- 📝 Salsa incremental analysis, pending prerequisite spike and possible rescope.

## Files Updated

- `docs/spec/SPEC-038-LANGUAGE-SERVER.md`
- `docs/spec/SPEC-043-INCREMENTAL-ANALYSIS.md`
- `docs/plan/PLAN-036-LSP-MCP-INTERFACE.md`
- `docs/plan/tasks/TASK-569-lsp-mcp-implementation.md`
- `docs/plan/tasks/TASK-576-ash-lsp-salsa.md`
- `docs/plan/PLAN-INDEX.md`
- `CHANGELOG.md`

## Verification Steps

- [x] Inspected live `ash-lsp` / `ash-lsp-core` source layout and TODO/deferred markers.
- [x] Verified targeted LSP crates before reconciliation: `cargo check -p ash-lsp-core -p ash-lsp` passed.
- [x] Verified targeted LSP tests before reconciliation: `cargo test -p ash-lsp-core -p ash-lsp` passed with 57 tests.
- [x] Verified targeted LSP clippy before reconciliation: `cargo clippy -p ash-lsp-core -p ash-lsp --all-targets --all-features -- -D warnings` passed.
- [x] Updated status surfaces to distinguish implemented local MVP from pending production/workspace/Salsa work.
- [x] Recorded post-Phase-89 Ash syntax/semantics drift that must be audited before further LSP feature implementation.

## Required Next Tasks

Future implementation should create explicit task files for each follow-up before coding:

1. LSP syntax/semantics refresh for current Ash language constructs.
2. Typecheck diagnostic integration and type hover.
3. Workspace/module graph index and cross-file navigation.
4. References, workspace symbols, and code actions.
5. Configuration/debounce/panic-isolation hardening.
6. Salsa prerequisite spike and final go/no-go/rescope decision.
