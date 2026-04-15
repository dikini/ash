# TASK-575: Implement Ash Source Formatter

**Phase:** 88
**Spec:** SPEC-042
**Related:** TASK-571
**Estimate:** 48 hours
**Status:** 📝 Planned

## Description

Build a source formatter for Ash that pretty-prints `ModuleFile` while preserving comments and blank lines.

## Requirements

1. Create `crates/ash-formatter` crate.
2. Implement `Formatter` state machine that walks `ModuleFile` and accumulates `Vec<FormatCmd>`.
3. Query `CommentTable` for leading/trailing comments before/after every span.
4. Emit consistently formatted output for all Ash surface syntax.
5. Normalize blank lines numerically:
   - at most 1 blank line between top-level defs/decls,
   - collapse to 0 inside workflow bodies and block expressions.
6. Introduce `FormatConfig { indent_width: usize, max_width: Option<usize> }` and use it as the configuration type for the formatter.
7. On parse errors, return original source unchanged (CLI exits 2, LSP returns empty edits).
8. Semicolons are **not emitted** (Ash surface syntax does not use them).
9. Apply keyword + indentation rules for all `surface::Workflow` variants.
10. Apply formatting rules for all `PolicyExpr` variants and `ConstraintBlock`.
11. Use a direct recursive walk with a small formatting IR (no full pretty-printing library); define `FormatCmd { Token, Space, Newline, Indent, Dedent }`.
12. Provide `render(cmds: &[FormatCmd], config: &FormatConfig) -> String` to convert the IR to output text.
13. Ensure idempotency via a two-pass or span-independent comment re-attachment algorithm.
14. `write_workflow_def` must emit the `WorkflowDef` header (name, params, roles, capabilities, contract) before delegating to `write_workflow` for the body.

## LSP/CLI Integration

- LSP: `textDocument/formatting` handler in `ash-lsp`
- CLI: `ash fmt [options] <file.ash>` subcommand per SPEC-005 (`--check`, `--write`, `--stdin`, `--indent` with range `1..=16`)

## Testing

1. Round-trip parse equality modulo spans for all example files.
2. Comment preservation tests.
3. Idempotency: `format(format(source)) == format(source)`, verified with a stable comment re-attachment scheme.
4. Proptest: generate random valid ASTs and assert round-trip equality modulo spans.

## Completion Checklist

- [ ] `crates/ash-formatter` crate created
- [ ] `FormatConfig` defined and used by `format_module(&ModuleFile, &FormatConfig) -> String`
- [ ] `FormatCmd` enum defined (Token, Space, Newline, Indent, Dedent)
- [ ] `render(cmds: &[FormatCmd], config: &FormatConfig) -> String` implemented
- [ ] Formatter handles all surface syntax (definitions, module_decls, workflow)
- [ ] `write_workflow_def` emits header before body
- [ ] Comments preserved via `CommentTable`
- [ ] Blank-line normalization implemented (top-level max 1, nested 0)
- [ ] Workflow variant formatting rules implemented
- [ ] PolicyExpr and ConstraintBlock formatting rules implemented
- [ ] Direct recursive walk with small formatting IR committed
- [ ] Two-pass / span-independent idempotency mechanism in place
- [ ] Round-trip and idempotency tests passing
- [ ] `ash fmt` CLI integrated per SPEC-005
- [ ] LSP formatting handler implemented
- [ ] `cargo test --all` passing
- [ ] Clippy and fmt clean
