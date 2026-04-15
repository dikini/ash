# PLAN-036: LSP & MCP Interface

## Phase: 87

## Goal

Implement a production-quality Language Server Protocol (LSP) server for Ash with an embedded Model Context Protocol (MCP) bridge. The server provides real-time diagnostics, hover, go-to-definition, completion, find references, document symbols, and code actions for human editors (VSCode, Neovim) while exposing the same semantic intelligence as MCP tools for AI coding agents.

## Specification

- [SPEC-038: Ash Language Server Protocol (LSP) & MCP Interface](../spec/SPEC-038-LANGUAGE-SERVER.md)
- [SPEC-005: CLI Specification — LSP section](../spec/SPEC-005-CLI.md)

## Tasks

| Task | Description | Estimate | Status |
|------|-------------|----------|--------|
| [TASK-569](../tasks/TASK-569-lsp-mcp-implementation.md) | LSP & MCP interface for Ash | 180h | 📝 Planned |

## Hard Prerequisites (Must Complete First)

These blockers are defined in SPEC-038 §18:

1. **Local variable spans** — `Expr::Variable { name: Name, span: Span }` and `Pattern::Variable { name: Name, span: Span }` (Phase 84, TASK-570)
2. **Type-checker error spans** — all `TypeEnvError`, `ConstructorError`, `NameError`, `ResolutionError`, and `TypeError` variants carry `span` (Phase 85, TASK-572)
3. **Unified error trait** — `AshLspError` trait implemented for all error types (Phase 85, TASK-573)
4. **`ash-lint` library extraction** — `lint_module` API available (Phase 86, TASK-574)
5. **`parse_surface_file` API** — top-level parser entry point returning `ModuleFile` with `CommentTable` (Phase 84, TASK-571)

## Deliverable

- `crates/ash-lsp` and `crates/ash-mcp` crates
- LSP handlers: diagnostics, hover, goto-definition, completion, find references, document symbols, code actions
- MCP tools: `ash_get_diagnostics`, `ash_hover`, `ash_goto_definition`, `ash_find_references`, `ash_complete`, `ash_document_symbols`, `ash_workspace_symbols`, `ash_code_action`
- VSCode extension skeleton and Neovim setup documentation
- `ash lsp --stdio`, `ash lsp --port <n>`, `ash lsp --mcp` CLI interfaces

## Timeline

5 weeks (~180 hours)

| Phase | Work | Hours |
|-------|------|-------|
| Week 1 | Skeleton + VFS + parser diagnostics | 32 |
| Week 2 | Typeck diagnostics + hover + symbols | 36 |
| Week 3 | Go-to-definition + completion + references | 40 |
| Week 4 | MCP bridge + VSCode skeleton | 40 |
| Week 5 | Polish, tests, docs, CHANGELOG | 32 |

## Risks

- `tower-lsp-server` and `rmcp` are moving targets; URI handling and transport APIs may shift.
- Cross-file reference index invalidation on `didChange` is error-prone.
- No prior LSP skeleton exists; Phase 87 is truly greenfield.

## Parallelization

- Phase 87 cannot start until all five hard prerequisites are resolved.
- Phase 88 (Formatter core crate) can begin in parallel with Weeks 4–5 of Phase 87 because it only needs `CommentTable` from Phase 84.
- Phase 89 (Salsa) should wait until `ash-lsp-core` VFS and diagnostic pipeline are stable (Week 3+ of Phase 87).
