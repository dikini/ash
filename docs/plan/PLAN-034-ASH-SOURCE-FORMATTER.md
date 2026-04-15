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
- `FormatCmd { Token, Space, Newline, Indent, Dedent }`
- `format_module(&ModuleFile, &FormatConfig) -> String`
- `format_range(module: &ModuleFile, range: Span, config: &FormatConfig) -> String`
- `render(cmds: &[FormatCmd], config: &FormatConfig) -> String`
- Preserves comments via `CommentTable`
- Normalizes blank lines (top-level max 1, nested 0)
- Formats all `surface::Workflow` variants with keyword + indentation rules
- `write_workflow_def` emits header (name, params, roles, capabilities, contract) before body
- Formats all `PolicyExpr` variants and `ConstraintBlock`
- Formats all missing major AST nodes:
  - `Type` (7 variants), `Pattern` (7 variants), `Guard` (6 variants)
  - `Definition` subtypes (`CapabilityDef`, `PolicyDef`, `RoleDef`, `ProxyDef`, `InterfaceDef`, `ImplDef`, `FnDef`)
  - `ModuleDecl` and `Import` (`Use`, `UsePath`, `UseItem`, `DependencyDecl`)
  - `MatchArm`, `BlockStmt`, `Visibility`, `Constraint`, `Predicate`
- Exact two-pass width-aware layout (`try_single_line` speculative render + multi-line fallback)
- Round-trip parse stability for all example files
- `ash fmt` CLI subcommand
- LSP `textDocument/formatting` and `textDocument/rangeFormatting` handlers

## Timeline

2 weeks (~48 hours)

## Risks

- Comment placement edge cases (comments between record fields, after trailing commas).
- Expression parenthesization must respect Ash operator precedence exactly.
- Round-trip stability requires stable comment re-attachment across passes.
- Two-pass width-aware layout adds complexity to the recursive walker.
