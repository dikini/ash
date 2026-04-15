# TASK-574: Extract `ash-lint` into a Library Crate

**Phase:** 86
**Spec:** SPEC-041
**Related:** None
**Estimate:** 12 hours
**Status:** 📝 Planned

## Description

Convert `crates/ash-lint` from a CLI-only binary into a dual crate (library + binary) with a public API for lint diagnostics.

## Requirements

1. Add `[lib]` section to `crates/ash-lint/Cargo.toml`. Add `walkdir` under `[bin.dependencies]` so it is a binary-only dependency (keeps the library dependency graph lean).
2. Create `src/lib.rs` exporting:
   - `LintDiagnostic`, `LintCode`, `LintFix`, `LintSeverity`, `LintCategory`, `RuleLevel`
   - `LintConfig` (with per-rule `HashMap<LintCode, RuleLevel>`)
   - `lint_source(source: &str, config: &LintConfig) -> Vec<LintDiagnostic>`
   - `lint_module(module: &ModuleFile, config: &LintConfig) -> Vec<LintDiagnostic>`
   - `lint_definition(def: &Definition, config: &LintConfig, diagnostics: &mut Vec<LintDiagnostic>)`
3. Add optional `serde` feature and `#[derive(Serialize)]` to lint types (or define a separate CLI DTO).
4. Refactor existing string-matching lints into AST visitors.
   - `L001` (`ooda-missing-decide`): Workflow lacks `observe` or `act` call
   - `L002` (`ooda-missing-orient`): `act` without preceding `orient`
   - `L004`: Policy conflict not checked — recursive CPS condition over `Workflow::Decide`, `Expr::Policy`, and `Workflow::Check { target: CheckTarget, .. }`
   - `L003` is **deferred** until provenance syntax is defined.
5. Make `src/main.rs` a CLI wrapper that preserves existing flags (`--allow`, `--deny-warnings`, `--format`) and accepts legacy rule ID aliases (`ooda-missing-decide` → `L001`, `ooda-missing-orient` → `L002`).

## Lint Rules (MVP)

- `L001`: Workflow lacks `observe` or `act` call (OODA)
- `L002`: `act` without preceding `orient` (OODA)
- `L004`: Policy conflict not checked (Policy)

## Completion Checklist

- [ ] `[lib]` and `[[bin]]` both present in Cargo.toml; `walkdir` scoped to `[bin.dependencies]` (binary-only)
- [ ] `src/lib.rs` exports public API including `lint_definition`
- [ ] Optional `serde` feature present and lint types derive `Serialize` when enabled
- [ ] Lint rules are AST visitors, not string searches
- [ ] CLI binary wraps library, preserves legacy flags, and accepts rule ID aliases (`ooda-missing-decide` → `L001`, `ooda-missing-orient` → `L002`)
- [ ] Unit tests for each lint rule
- [ ] Property tests: idempotence of `lint_module`, non-default spans on diagnostics
- [ ] Integration tests on example files
- [ ] `cargo test --all` passing
- [ ] Clippy and fmt clean
