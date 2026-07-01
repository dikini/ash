# TASK-1776: Add macro-specific symbol kinds and cache-summary invalidation keys

## Status: ✅ Complete

## Description

Introduce a macro-specific LSP symbol/cache model so macro declaration edits, typed signature edits, and template-shape edits invalidate analysis and no longer appear as ordinary functions in internal symbol identity. This task builds infrastructure only; user-facing completion/hover wording lands in TASK-1777.

## Specification Reference

- [PLAN-174: Macro-Aware Tooling, Summary Identity, and Inference Readiness](../PLAN-174-MACRO-AWARE-TOOLING-SUMMARY-IDENTITY-AND-INFERENCE-READINESS.md)
- TASK-1775 audit artifact

## Dependencies

- ✅ TASK-1774: Phase 174 plan packet (complete)
- ✅ TASK-1775: Macro-aware tooling audit

## Deferral / Planned-Feature Reconciliation

| Prior item | Source | Original reason | Prereqs now? | Decision | Gate |
|---|---|---|---|---|---|
| LSP parse summaries omit macro detail | Phase 173 audit | Only broad counts were tracked | Yes | Add lightweight macro summary keys | Same definition count with changed macro signature invalidates cache |
| Macro symbols use function kind | Phase 173 audit/live code | No macro-specific symbol kind | Yes | Add macro-specific kind in internal model | Tests assert macro is not `Function` internally |

## Requirements

### Functional Requirements

1. Add a macro-specific variant to `crates/ash-lsp-core/src/db.rs::SymbolKind`.
2. Extend `ParseSummary` with lightweight macro-sensitive data, such as a macro declaration count and stable macro summary key list or hash.
3. Compute the summary from parsed `Definition::Macro` fields that affect tooling: name, visibility, parameter count, typed signature shape, input/output kind if available, and template fingerprint where available.
4. Update `compute_summary_from_cache` so cached AST entries validate against the same macro-sensitive summary.
5. Update symbol-index construction to record macro symbols with the macro-specific kind.

### Property Requirements

- The salsa tracked summary must remain lightweight and `Eq + Hash`; do not store full `ModuleFile` or full `Expr` values in it.
- The macro summary key must not include runtime authority, rows, contracts, or callable export information.
- Same-count macro edits must invalidate; unrelated whitespace-only edits may still reparse but should not force semantic overclaims.

## TDD Steps

### Step 1: Write failing cache tests

Add tests in `crates/ash-lsp-core/src/db.rs` or an integration test proving two sources with the same definition count but different macro signatures produce different `ParseSummary` values.

### Step 2: Write failing symbol-kind tests

Add a test proving `macro m(x) => x;` indexes `m` as the new macro symbol kind, not `Function`.

### Step 3: Implement the minimal model

Update `ParseSummary`, cache comparison, symbol enum, and symbol construction with the smallest macro-specific fields needed by the tests.

### Step 4: Verify focused crate

Run `cargo test -p ash-lsp-core macro` and the exact new test filters.

## Dispatch

```yaml
agent: hermes
reasoning: medium
max_turns: 15
toolsets: [terminal, file]
```

## Verification

```yaml
strictness: clean
commands:
  - cargo test -p ash-lsp-core macro
  - cargo test -p ash-lsp-core
  - cargo fmt --check
  - cargo clippy -p ash-lsp-core --all-targets --all-features -- -D warnings
checklist:
  - [x] Macro symbol kind added
  - [x] Macro-sensitive parse summary added
  - [x] Same-count macro edits invalidate cache summaries
```

## Dependencies for Next Task

TASK-1777 and TASK-1778 depend on the macro symbol/cache model from this task.

## Completion Evidence

- Added `ParseSummary::macro_count`, lightweight `MacroSummaryKey` cache data, and internal `SymbolKind::Macro`; added db regressions for same-count macro edits and macro symbol identity.
