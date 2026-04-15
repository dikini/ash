# PLAN-035: Incremental Analysis Engine

## Phase: 89

## Goal

Replace the simple per-request cache in `ash-lsp-core` with a `salsa`-based incremental query engine.

## Specification

- [SPEC-043: Incremental Analysis Engine](../spec/SPEC-043-INCREMENTAL-ANALYSIS.md)

## Tasks

| Task | Description | Estimate | Status |
|------|-------------|----------|--------|
| [TASK-576](../tasks/TASK-576-ash-lsp-salsa.md) | Integrate `salsa` into `ash-lsp-core` for parse/type/symbol queries (includes `Eq + Hash` sub-task) | 48h | 📝 Planned |

## Deliverable

- `salsa` database defining `parse_file`, `module_graph`, `type_check_file`, `symbol_index`
- VFS updates feed into salsa inputs atomically
- Cross-file invalidation works correctly
- `didClose` and `didChangeWatchedFiles` keep inputs current
- Performance improvement measurable on 10-file workspace

## Timeline

1.5–2 weeks (~48 hours)

## Risks

- Salsa trait requirements may force refactoring of `TypeEnv` or `ModuleGraph`.
- Debugging invalidation bugs is notoriously difficult.
- Cycle recovery may require redesigning query boundaries if the salsa API is insufficient.
