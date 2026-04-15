# PLAN-034: Ash Source Formatter

## Phase: 88

## Goal

Provide a source formatter for Ash that pretty-prints any valid `ModuleFile` while preserving user comments and blank lines.

## Specification

- [SPEC-042: Ash Source Formatter](../spec/SPEC-042-ASH-SOURCE-FORMATTER.md)

## Tasks

| Task | Description | Estimate | Status |
|------|-------------|----------|--------|
| [TASK-575](../tasks/TASK-575-ash-source-formatter.md) | Implement Ash source formatter with comment preservation | 40h | 📝 Planned |

## Deliverable

- `crates/ash-formatter` crate
- `format_module(&ModuleFile, indent_width) -> String`
- Preserves comments via `CommentTable`
- Round-trip parse equality for all example files
- `ash fmt` CLI subcommand

## Timeline

2 weeks (~40 hours)

## Risks

- Comment placement edge cases (comments between record fields, after trailing commas).
- Expression parenthesization must respect Ash operator precedence exactly.
