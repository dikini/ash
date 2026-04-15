# PLAN-035: Incremental Analysis Engine

## Phase: 89

## Goal

Replace the simple per-request cache in `ash-lsp-core` with a `salsa`-based incremental query engine.

## Specification

- [SPEC-043: Incremental Analysis Engine](../spec/SPEC-043-INCREMENTAL-ANALYSIS.md)

## Tasks

| Task | Description | Estimate | Status |
|------|-------------|----------|--------|
| [TASK-576](../tasks/TASK-576-ash-lsp-salsa.md) | Integrate `salsa` into `ash-lsp-core` for parse/type/symbol queries (includes `Eq + Hash` sub-task and `SymbolIndex` definition) | 48h | 📝 Planned |

## Deliverable

- `salsa` database defining `parse_file`, `module_graph`, `type_check_file`, `symbol_index`
- `SymbolIndex` struct defined (document symbols, reference locations, cross-file usage index)
- VFS updates feed into salsa inputs atomically
- Cross-file invalidation works correctly
- `didClose` and `didChangeWatchedFiles` keep inputs current
- Manifest-edit over-invalidation risk documented and accepted for MVP
- Performance improvement measurable on 10-file workspace

## Timeline

1.5–2 weeks (~48 hours)

## Risks

- `ash-lsp-core` does not yet exist; blocked until SPEC-038 Phase 2 is complete.
- Salsa trait requirements may force refactoring of `TypeEnv`, `ModuleGraph`, or many `surface.rs` AST types.
- Debugging invalidation bugs is notoriously difficult.
- Cycle recovery may require redesigning query boundaries if the salsa API is insufficient.
- Manifest edits will invalidate `type_check_file` for every file (acceptable for MVP, but must be documented).
