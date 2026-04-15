# PLAN-035: Incremental Analysis Engine

## Phase: 89

## Goal

Replace the simple per-request cache in `ash-lsp-core` with a `salsa`-based incremental query engine.

## Specification

- [SPEC-043: Incremental Analysis Engine](../spec/SPEC-043-INCREMENTAL-ANALYSIS.md)

## Tasks

| Task | Description | Estimate | Status |
|------|-------------|----------|--------|
| [TASK-576](../tasks/TASK-576-ash-lsp-salsa.md) | Integrate `salsa` into `ash-lsp-core` for parse/type/symbol queries | 32h | 📝 Planned |

## Deliverable

- `salsa` database defining `parse_file`, `module_graph`, `type_check_file`, `symbol_index`
- VFS updates feed into salsa inputs
- Cross-file invalidation works correctly
- Performance improvement measurable on 10-file workspace

## Timeline

1.5–2 weeks (~32 hours)

## Risks

- Salsa trait requirements may force refactoring of `TypeEnv` or `ModuleGraph`.
- Debugging invalidation bugs is notoriously difficult.
