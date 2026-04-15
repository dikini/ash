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
2. Implement `Formatter` state machine that walks `ModuleFile`.
3. Query `CommentTable` for leading/trailing comments before/after every span.
4. Emit consistently formatted output for all Ash surface syntax.
5. Preserve blank lines by checking line distances between spans.
6. Configurable indent width (default 4 spaces).

## LSP/CLI Integration

- LSP: `textDocument/formatting` handler in `ash-lsp`
- CLI: `ash fmt [options] <file.ash>` subcommand

## Testing

1. Round-trip parse equality for all example files.
2. Comment preservation tests.
3. Idempotency: `format(format(source)) == format(source)`.

## Completion Checklist

- [ ] `crates/ash-formatter` crate created
- [ ] Formatter handles all surface syntax
- [ ] Comments preserved via `CommentTable`
- [ ] Round-trip and idempotency tests passing
- [ ] `ash fmt` CLI integrated
- [ ] LSP formatting handler implemented
- [ ] `cargo test --all` passing
- [ ] Clippy and fmt clean
