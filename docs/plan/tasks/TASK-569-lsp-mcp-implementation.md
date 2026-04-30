# TASK-569: Local LSP MVP for Ash

## Status: ✅ Complete (Local LSP MVP; Production Follow-Ups Split Out)

## Description

Implement the first usable Language Server Protocol server for Ash. After TASK-767 reconciliation, this task is considered complete only for the implemented local LSP MVP: document synchronization, VFS/cache, parser+lint diagnostics, hover, document symbols, same-file goto-definition, completion, and the `ash-lsp` stdio/TCP binary. The broader production LSP/MCP/workspace scope originally described here has been split into explicit pending follow-up work.

## Specification Reference

- [SPEC-038: Ash Language Server Protocol (LSP) & MCP Interface](../../spec/SPEC-038-LANGUAGE-SERVER.md)
- [SPEC-005: CLI Specification — LSP section](../../spec/SPEC-005-CLI.md)
- [TASK-767: LSP Status Reconciliation and Syntax/Semantics Drift Audit](TASK-767-lsp-status-reconciliation.md)

## Hard Prerequisites

- ✅ TASK-570: Local variable spans.
- ✅ TASK-571: `parse_surface_file` and `CommentTable` substrate.
- ✅ TASK-572: Typechecker/name-resolution error spans.
- ✅ TASK-573: `AshLspError` diagnostic trait.
- ✅ TASK-574: `ash-lint` library extraction.

## Implemented Requirements

### 1. `crates/ash-lsp-core`

- ✅ VFS with `lsp_types::Uri` keys.
- ✅ In-memory overlay for open documents.
- ✅ Full-document changes.
- ✅ Incremental range changes.
- ✅ Line/column ↔ byte-offset helpers.
- ✅ Per-version analysis cache keyed by document URI.
- ✅ Parser + lint diagnostic aggregation.
- ✅ Hover for keywords and top-level declarations.
- ✅ Document symbols from `ModuleFile`.
- ✅ Same-file goto-definition over parsed declarations.
- ✅ Keyword/snippet and top-level-name completion.

### 2. `crates/ash-lsp`

- ✅ `tower-lsp-server = "0.23"` based server.
- ✅ stdio transport by default.
- ✅ TCP transport via `--port`.
- ✅ `initialize`, `shutdown`, `didOpen`, `didChange`, `didClose`.
- ✅ Diagnostic publish on open/change and clear on close.
- ✅ Advertises and handles hover, completion, definition, and document symbols.
- ✅ Focused JSON-RPC style unit tests.

## Explicitly Pending Follow-Ups

The following requirements from the original broad task are **not** completed by TASK-569 and must receive separate task files before implementation:

- 📝 Typecheck diagnostics from `ash-typeck`.
- 📝 Expression-level type hover.
- 📝 `NameBinder`/`TypeEnv`-powered semantic goto-definition.
- 📝 Cross-file reference/workspace index.
- 📝 `textDocument/references`.
- 📝 `workspace/symbol`.
- 📝 `textDocument/codeAction`.
- 📝 `textDocument/formatting` integration, if formatter work is reintroduced through LSP.
- 📝 Diagnostic debounce and max-diagnostic limiting.
- 📝 Configuration from `initialize` params and `.ash.toml`.
- 📝 Panic isolation for LSP request handlers.
- 📝 Watched-file handling and workspace root/crate graph discovery.
- 📝 VSCode extension skeleton and Neovim user docs.
- 📝 Current-Ash syntax/semantics refresh for post-Phase-89 language features.
- 📝 MCP parity hardening through `ash-mcp` / shared `ash-lsp-core` queries.

## Syntax/Semantics Drift to Audit Before More LSP Work

Later Ash phases introduced language constructs not covered by the original LSP MVP assumptions:

- Capability interfaces and Ash-defined capability implementations.
- Runtime resources, workflow `owns`, and workflow `uses` bindings.
- Authority provenance and implementation-backed capability binding metadata.
- `fail` / `with_error` operational failure syntax and semantics.
- `Proc<T>`, `P<T>`, and the `std::proc` surface.
- Generalized `do:Act` / `do:Proc` notation and new-form `act { ... }` compatibility.
- Bracket comprehension syntax with explicit computation targets.
- Stdlib/module import/export drift and corpus classification from Phase 107 planning.

## Verification Evidence

Fresh TASK-767 audit commands against the live crates:

- ✅ `cargo check -p ash-lsp-core -p ash-lsp`
- ✅ `cargo test -p ash-lsp-core -p ash-lsp` — 57 tests passed.
- ✅ `cargo clippy -p ash-lsp-core -p ash-lsp --all-targets --all-features -- -D warnings`
- ✅ `cargo fmt --check`
- ✅ `cargo run -p ash-lsp -- --help`

## Completion Checklist

- [x] `crates/ash-lsp` created with basic LSP skeleton.
- [x] `crates/ash-lsp-core` created with VFS and diagnostic aggregator.
- [x] Incremental text document sync implemented and tested.
- [x] Parser diagnostic path wired.
- [x] Lint diagnostic path wired.
- [ ] Typecheck diagnostic path wired. Deferred follow-up.
- [x] Hover handler implemented for keywords/top-level declarations.
- [x] Go-to-definition handler implemented for same-file parsed declarations.
- [x] Document symbol handler implemented.
- [x] Completion handler implemented for keywords/top-level names.
- [ ] Find references handler implemented. Deferred follow-up.
- [ ] Code actions handler implemented. Deferred follow-up.
- [ ] Workspace symbols handler implemented. Deferred follow-up.
- [ ] Full production MCP parity complete. Deferred follow-up.
- [ ] VSCode extension skeleton provided. Deferred follow-up.
- [ ] Neovim setup documented. Deferred follow-up.
- [x] Unit and focused integration tests passing for implemented local MVP.
- [x] Targeted `cargo test -p ash-lsp-core -p ash-lsp` clean.
- [x] Targeted clippy clean.
- [x] `cargo fmt --check` clean.
- [x] CHANGELOG.md updated by TASK-767 reconciliation.

## Required Follow-Up Task Seeds

1. LSP current-syntax refresh and corpus-backed examples.
2. Typecheck diagnostics + type hover.
3. Workspace/module graph index.
4. References/workspace-symbol/code-action feature set.
5. LSP hardening: config, debounce, panic isolation, watched files.
6. Editor integration docs/skeleton.
7. MCP parity hardening.
