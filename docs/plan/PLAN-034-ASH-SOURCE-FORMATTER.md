# PLAN-034: Ash Source Formatter

## Phase: 88

## Goal

Provide a source formatter for Ash that pretty-prints any valid `ModuleFile` while preserving user comments and blank lines.

## Specification

- [SPEC-042: Ash Source Formatter](../spec/SPEC-042-ASH-SOURCE-FORMATTER.md)

## Tasks

| Task | Description | Estimate | Status |
|------|-------------|----------|--------|
| [TASK-575](../tasks/TASK-575-ash-source-formatter.md) | Implement Ash source formatter with comment preservation | 48h | 📝 Planned |

## Deliverable

- `crates/ash-formatter` crate
- `FormatConfig { indent_width, max_width }`
- `format_module(&ModuleFile, &FormatConfig) -> String`
- Preserves comments via `CommentTable`
- Normalizes blank lines (top-level max 1, nested 0)
- Formats all `surface::Workflow` variants with keyword + indentation rules
- Formats all `PolicyExpr` variants and `ConstraintBlock`
- Round-trip parse equality for all example files
- `ash fmt` CLI subcommand

## Timeline

2 weeks (~48 hours)

## Risks

- Comment placement edge cases (comments between record fields, after trailing commas).
- Expression parenthesization must respect Ash operator precedence exactly.
- Idempotency requires stable comment re-attachment across passes.
