# TASK-1871: CLI Entry Boundary Audit

**Status:** Complete
**Plan:** [PLAN-186](../PLAN-186-SURFACE-FUNCTION-CLI-ENTRY.md)

## Description

Audit the CLI `run` and `check` paths after Phase 185 to identify where function-first entry files still route through legacy workflow assumptions.

## Requirements

- Inspect `crates/ash-cli/src/commands/run.rs`.
- Inspect `crates/ash-cli/src/commands/check.rs`.
- Identify the smallest CLI-facing gap that prevents ordinary `fn main` sources from behaving like entry files.

## TDD Steps

This is an audit task; implementation tests belong to TASK-1872.

## Completion Checklist

- [x] CLI run/check entry paths inspected.
- [x] Implementation gap selected.
- [x] No implementation began before TASK-1872 existed.

## Evidence

- `ash check` can fall back to module-file checking for non-workflow `.ash` files.
- `ash run` normal execution uses the ordinary file engine path for sources without workflow entry headers.
- `ash run --dry-run` still forces `parse_runnable_workflow(..., WorkflowSourceKind::Entry)`, which routes to the runtime-entry workflow parser and rejects function-first `fn main` sources.
- TASK-1872 targets the dry-run mismatch so the CLI verification path matches the engine execution path.
