# TASK-1956: Examples Current-Syntax Refresh

**Status:** Planned
**Phase:** [PLAN-200: Tooling And Migration Polish](../PLAN-200-TOOLING-AND-MIGRATION-POLISH.md)

## Description

Refresh the examples corpus so productive examples use current syntax and legacy examples are
explicitly classified, demoted, or removed.

## Requirements

- Audit `examples/`, template-generated examples, and example README references using the
  TASK-1952 classification model.
- Migrate productive examples to current syntax.
- Move retained legacy examples into clearly labeled compatibility or historical paths.
- Add or update executable gates for productive examples.

## TDD Steps

1. Add or update an example corpus gate that fails on unclassified deprecated forms.
2. Confirm current productive examples with old syntax fail the gate.
3. Migrate, demote, or remove old examples.
4. Re-run example corpus checks and record evidence.

## Completion Checklist

- [ ] Productive examples use current syntax.
- [ ] Retained legacy examples are compatibility-only or historical-reference-only.
- [ ] Example README paths teach current syntax first.
- [ ] Example corpus gates pass.
