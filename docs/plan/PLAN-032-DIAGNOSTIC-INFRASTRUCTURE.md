# PLAN-032: Diagnostic Infrastructure

## Phase: 85

## Goal

Make all Ash compiler errors LSP-diagnostic-ready by adding source spans to every error variant and defining a uniform error trait.

## Specification

- [SPEC-040: Diagnostic Infrastructure](../spec/SPEC-040-DIAGNOSTIC-INFRASTRUCTURE.md)

## Tasks

| Task | Description | Estimate | Status |
|------|-------------|----------|--------|
| [TASK-572](../tasks/TASK-572-typeck-error-spans.md) | Add spans to `TypeEnvError`, `NameError`, `ResolutionError`, `TypeError`, and all spanless `ConstructorError` variants | 12h | 📝 Planned |
| [TASK-573](../tasks/TASK-573-ash-lsp-error-trait.md) | Create `crates/ash-diagnostic` per SPEC-040 §5.4, define `AshLspError` trait, `Severity`, and `DiagnosticCode`, and implement for all error types | 6h | 📝 Planned |

## Deliverable

- Every `TypeEnvError`, `NameError`, `ResolutionError`, and `TypeError` variant carries a `span` (some may be approximate `Span::default()` until AST span gaps are resolved)
- All spanless `ConstructorError` variants (`UnknownConstructor`, `MissingField`, `UnknownField`, `FieldTypeMismatch`, `TupleFieldTypeMismatch`, `TupleArityMismatch`, `NonExhaustiveMatch`) and `TypeError::NotAConstructor` carry a `span`
- New `crates/ash-diagnostic` crate provides `AshLspError` trait, `Severity`, and `DiagnosticCode` newtype, with explicit dependency constraints (may depend on `ash-parser`, must not depend on `ash-typeck`)
- Mechanical diagnostic conversion from any Ash error to LSP `Diagnostic`
- Lexer errors (if any) are handled separately by the parser/LSP front-end and do not implement `AshLspError`

## Timeline

1 week (~18 hours)

## Risks

- `TypeEnvError` is constructed in many call sites; missing one will cause compilation failures.
- Some errors originate deep in helper functions where span propagation requires threading an extra argument.
- AST span gaps (`TypeDef`/`InterfaceDef`/`ImplDef`, `Expr::Variable`/`Literal`, `Pattern` variants, and `NameBinder` APIs taking `&str` without `Span`) may force temporary use of `Span::default()` or parent spans until SPEC-039 lands.

## Parallelization

- Phase 85 can run in parallel with `TASK-571` (comment trivia) from Phase 84, but only after `TASK-570` (binding spans) is complete.
- Phase 85 is independent of Phase 86.
