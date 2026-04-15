# TASK-573: Define `AshLspError` Diagnostic Trait

**Phase:** 85
**Spec:** SPEC-040 §5
**Related:** TASK-572
**Estimate:** 6 hours
**Status:** 📝 Planned

## Description

Define a uniform `AshLspError` trait that `ash-lsp-core` can use to convert any Ash error into an LSP `Diagnostic`.

## Requirements

1. Define `AshLspError` trait with:
   - `fn span(&self) -> Option<Span>`
   - `fn severity(&self) -> Severity`
   - `fn code(&self) -> Option<String>`
   - `fn message(&self) -> String`
2. Define `Severity` enum mapping to LSP severities.
3. Implement `AshLspError` for:
   - `ParseError`
   - `ConstructorError`
   - `TypeEnvError`
   - `TypeError`
   - `NameError`
   - `ResolutionError`
   - `PurityError`
4. Provide `ash_error_to_diagnostic(err: &dyn AshLspError, source: &str) -> Option<Diagnostic>` helper.

> **Note:** `ExhaustivenessError` is unused in the active type-checking pipeline and does **not** receive an implementation.

## Location

Place the trait in `crates/ash-typeck/src/diagnostic.rs` (new file) because `ash-lsp-core` does not exist yet. Migrate it to `ash-lsp-core` during SPEC-038.

## Completion Checklist

- [ ] `AshLspError` trait defined
- [ ] Implementations for all required error types
- [ ] Conversion helper tested with sample errors
- [ ] `cargo test --all` passing
- [ ] Clippy and fmt clean
