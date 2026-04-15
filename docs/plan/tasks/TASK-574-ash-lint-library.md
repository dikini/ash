# TASK-574: Extract `ash-lint` into a Library Crate

**Phase:** 86
**Spec:** SPEC-041
**Related:** None
**Estimate:** 12 hours
**Status:** 📝 Planned

## Description

Convert `crates/ash-lint` from a CLI-only binary into a dual crate (library + binary) with a public API for lint diagnostics.

## Requirements

1. Add `[lib]` section to `crates/ash-lint/Cargo.toml`.
2. Create `src/lib.rs` exporting:
   - `LintDiagnostic`, `LintSeverity`, `LintCategory`
   - `LintConfig`
   - `lint_source(source: &str, config: &LintConfig) -> Vec<LintDiagnostic>`
   - `lint_module(module: &ModuleFile, config: &LintConfig) -> Vec<LintDiagnostic>`
3. Refactor existing string-matching lints into AST visitors.
4. Make `src/main.rs` a thin CLI wrapper over the library.

## Lint Rules (MVP)

- `L001`: Workflow lacks `observe` or `act` call (OODA)
- `L002`: `act` without preceding `orient` (OODA)
- `L003`: Missing provenance block (Provenance)
- `L004`: Policy conflict not checked (Policy)

## Completion Checklist

- [ ] `[lib]` and `[[bin]]` both present in Cargo.toml
- [ ] `src/lib.rs` exports public API
- [ ] Lint rules are AST visitors, not string searches
- [ ] CLI binary wraps library
- [ ] Unit tests for each lint rule
- [ ] Integration tests on example files
- [ ] `cargo test --all` passing
- [ ] Clippy and fmt clean
