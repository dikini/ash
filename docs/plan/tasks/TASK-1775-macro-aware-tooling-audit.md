# TASK-1775: Audit macro-aware tooling, LSP, and summary-identity seams

## Status: ✅ Complete

## Description

Audit the live parser, macro summary, LSP, module-loader, and typechecker-facing surfaces that consume macro declarations or summaries. The audit must identify every place that still presents macros as ordinary functions, every cache or parse-summary field that can miss macro-significant edits, and every inference path that would need a callable identity proof before TASK-1780.

## Specification Reference

- [PLAN-174: Macro-Aware Tooling, Summary Identity, and Inference Readiness](../PLAN-174-MACRO-AWARE-TOOLING-SUMMARY-IDENTITY-AND-INFERENCE-READINESS.md)
- [SPEC-095c: Surface AST, Macro Expansion, and Notation](../../spec/SPEC-095c-SURFACE-AST-MACROS-AND-NOTATION.md)
- [SPEC-098c: Surface-to-Core Lowering](../../spec/SPEC-098c-SURFACE-TO-CORE-LOWERING.md)

## Dependencies

- ✅ TASK-1774: Phase 174 plan packet (complete)

## Deferral / Planned-Feature Reconciliation

| Prior item | Source | Original reason | Prereqs now? | Decision | Gate |
|---|---|---|---|---|---|
| Macro-aware LSP UX | Phase 173 audit | LSP macro treatment was function-like | Yes | Audit all affected LSP surfaces before code changes | Audit artifact names file/line owners |
| Callable identity for inference | TASK-1772 | Ordinary calls could fabricate summaries | Partial | Audit but do not implement inference here | TASK-1779 owns the formal identity decision |

## Requirements

### Functional Requirements

1. Create `docs/audit/phase-174-macro-aware-tooling-audit.md`.
2. Inventory macro consumers in `crates/ash-lsp-core/src/{completion,hover,goto,symbols,db}.rs`.
3. Inventory macro summary and typed-signature carriers in `crates/ash-parser/src/surface.rs` and module-loader transport points.
4. Classify each surface as user-facing tooling, cache identity, symbol identity, same-file navigation, imported-summary navigation, or inference prerequisite.
5. Assign each gap to TASK-1776 through TASK-1781.

### Property Requirements

- Macros must remain syntax-phase metadata in the audit language.
- No audit row may propose treating macro summaries as runtime callables.
- Every proposed implementation task must have at least one positive and one negative gate.

## TDD Steps

### Step 1: Inspect live surfaces

Use `read_file`, `search_files`, and rust-analyzer symbol lookup for `MacroSummary`, `MacroTypeSignatureSummary`, `ParseSummary`, `SymbolKind`, `Definition::Macro`, and macro inference helpers.

### Step 2: Write the audit artifact

Record current state, gap table, task ownership, and explicit non-goals in `docs/audit/phase-174-macro-aware-tooling-audit.md`.

### Step 3: Patch downstream tasks if needed

If the audit discovers a different task order or missing target file, patch TASK-1776 through TASK-1781 before implementation begins.

### Step 4: Verify audit coverage

Run a structural assertion that the audit mentions `completion.rs`, `hover.rs`, `goto.rs`, `symbols.rs`, `db.rs`, `MacroSummary`, `ParseSummary`, and `TASK-1780`.

## Dispatch

```yaml
agent: hermes
reasoning: high
max_turns: 15
toolsets: [terminal, file]
```

## Verification

```yaml
strictness: clean
commands:
  - python3 -c 'from pathlib import Path; s=Path("docs/audit/phase-174-macro-aware-tooling-audit.md").read_text(); assert "completion.rs" in s and "hover.rs" in s and "goto.rs" in s and "symbols.rs" in s and "db.rs" in s and "MacroSummary" in s and "ParseSummary" in s and "TASK-1780" in s'
  - git diff --check
checklist:
  - [x] Audit artifact created
  - [x] All LSP macro surfaces classified
  - [x] Cache identity and callable identity gaps assigned
```

## Dependencies for Next Task

This task outputs the ownership map for TASK-1776 through TASK-1781.

## Completion Evidence

- Created `docs/audit/phase-174-macro-aware-tooling-audit.md`; mapped LSP macro-as-function surfaces, parse-summary gaps, goto/reference limits, and callable-identity prerequisites.
