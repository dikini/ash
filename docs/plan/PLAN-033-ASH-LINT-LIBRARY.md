# PLAN-033: Ash Lint Library Extraction

## Phase: 86

## Goal

Convert `crates/ash-lint` from a CLI-only binary into a reusable library crate with a public API.

## Specification

- [SPEC-041: Ash Lint Library](../spec/SPEC-041-ASH-LINT-LIBRARY.md)

## Tasks

| Task | Description | Estimate | Status |
|------|-------------|----------|--------|
| [TASK-574](../tasks/TASK-574-ash-lint-library.md) | Extract `ash-lint` CLI into a library + binary wrapper | 12h | 📝 Planned |

## Deliverable

- `crates/ash-lint/Cargo.toml` has `[lib]`, `[[bin]]`, and `walkdir` scoped to `[bin.dependencies]` (binary-only)
- `src/lib.rs` exports `LintDiagnostic`, `LintConfig`, `lint_source`, `lint_module`, `lint_definition`
- Optional `serde` feature enables `Serialize` on lint types
- `src/main.rs` is a thin CLI wrapper supporting legacy rule ID aliases
- Lint rules are AST visitors, not string searches

## Timeline

1 week (~12 hours)

## Risks

- Current lint logic is extremely primitive; effectively rebuilding from scratch.
- Must define `LintDiagnostic` shape carefully so it integrates cleanly with `AshLspError`.
- SPEC-039 must deliver `parse_surface_file()` as an explicit, gated deliverable before this plan can execute.

## Parallelization

- Phase 86 can run in parallel with Phase 85 (Diagnostic Infrastructure) and with `TASK-571` (comment trivia) from Phase 84.
- Phase 86 is **blocked by `TASK-570`** (binding spans) because lint visitors will pattern-match on `Expr::Variable`.
- Phase 86 is **also blocked by SPEC-039** (specifically the `parse_surface_file` API and stable `ModuleFile` AST). Implementation cannot begin until SPEC-039 §4.6 acceptance criteria are satisfied: the AST is frozen, parser tests pass, and the API is public and documented.
