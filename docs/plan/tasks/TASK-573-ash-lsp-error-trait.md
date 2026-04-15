# TASK-573: Define `AshLspError` Diagnostic Trait

**Phase:** 85
**Spec:** SPEC-040 §5
**Related:** TASK-572
**Estimate:** 6 hours
**Status:** 📝 Planned

## Description

Define a uniform `AshLspError` trait that `ash-lsp-core` can use to convert any Ash error into an LSP `Diagnostic`. To break the circular dependency between `ash-typeck` and `ash-lsp-core`, the trait lives in a new `crates/ash-diagnostic` crate.

## Requirements

1. Create new crate `crates/ash-diagnostic` per SPEC-040 §5.4.
2. Define a `DiagnosticCode` newtype (`pub struct DiagnosticCode(pub String);`).
3. Define a `Severity` enum mapping to LSP severities.
4. Define `AshLspError` trait with:
   - `fn span(&self) -> Option<Span>`
   - `fn severity(&self) -> Severity`
   - `fn code(&self) -> Option<DiagnosticCode>`
   - `fn message(&self) -> String`
5. Implement `AshLspError` for:
   - `ParseError`
   - `ConstructorError`
   - `TypeEnvError`
   - `TypeError`
   - `NameError`
   - `ResolutionError`
   - `PurityError`
6. Provide `ash_error_to_diagnostic(err: &dyn AshLspError, source: &str) -> Option<Diagnostic>` helper.
7. Enforce the dependency constraint: `ash-diagnostic` may depend on `ash-parser` (for `Span`), but must **not** depend on `ash-typeck`.

> **Note:** `ExhaustivenessError` is unused in the active type-checking pipeline and does **not** receive an implementation.
>
> **Lexer errors:** If the lexer defines a `LexError` type, it is out of scope for `AshLspError` and should be handled separately by the parser or LSP front-end.

## Location

Place the trait, `Severity`, and `DiagnosticCode` in `crates/ash-diagnostic/src/lib.rs` (new crate).

## Completion Checklist

- [ ] `crates/ash-diagnostic` created
- [ ] `AshLspError` trait defined
- [ ] `DiagnosticCode` and `Severity` defined
- [ ] Implementations for all required error types
- [ ] Conversion helper tested with sample errors
- [ ] `cargo test --all` passing
- [ ] Clippy and fmt clean
