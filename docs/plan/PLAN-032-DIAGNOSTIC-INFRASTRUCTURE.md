# PLAN-032: Diagnostic Infrastructure

## Phase: 85

## Goal

Make all Ash compiler errors LSP-diagnostic-ready by adding source spans to every error variant and defining a uniform error trait.

## Specification

- [SPEC-040: Diagnostic Infrastructure](../spec/SPEC-040-DIAGNOSTIC-INFRASTRUCTURE.md)

## Tasks

| Task | Description | Estimate | Status |
|------|-------------|----------|--------|
| [TASK-572](../tasks/TASK-572-typeck-error-spans.md) | Add spans to `TypeEnvError`, `ExhaustivenessError`, `NameError`, and `ConstructorError::UnknownConstructor` | 12h | 📝 Planned |
| [TASK-573](../tasks/TASK-573-ash-lsp-error-trait.md) | Define `AshLspError` trait and implement it for all error types | 6h | 📝 Planned |

## Deliverable

- Every `TypeEnvError`, `ExhaustivenessError`, `NameError`, and `ConstructorError` variant carries a `span`
- `AshLspError` trait provides `span()`, `severity()`, `code()`, `message()`
- Mechanical diagnostic conversion from any Ash error to LSP `Diagnostic`

## Timeline

1 week (~18 hours)

## Risks

- `TypeEnvError` is constructed in many call sites; missing one will cause compilation failures.
- Some errors originate deep in helper functions where span propagation requires threading an extra argument.

## Parallelization

- Phase 85 can run in parallel with `TASK-571` (comment trivia) from Phase 84, but only after `TASK-570` (binding spans) is complete.
- Phase 85 is independent of Phase 86.
