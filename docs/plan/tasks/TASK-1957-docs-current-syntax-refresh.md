# TASK-1957: Docs Current-Syntax Refresh

**Status:** Planned
**Phase:** [PLAN-200: Tooling And Migration Polish](../PLAN-200-TOOLING-AND-MIGRATION-POLISH.md)

## Description

Refresh tutorials, reference docs, and orientation paths so productive documentation teaches current
syntax and quarantines old syntax in migration notes only.

## Requirements

- Audit docs and snippets using the TASK-1952 classification model.
- Migrate productive tutorials and getting-started paths to current syntax.
- Add migration notes for deprecated forms that remain user-visible.
- Update orientation indexes when read paths change.
- Add docs gates that prevent unlabeled old syntax from returning to productive docs.

## TDD Steps

1. Add a docs audit/test that fails on unlabeled old syntax in productive docs.
2. Confirm current stale docs fail where applicable.
3. Migrate productive docs or move old syntax into labeled migration sections.
4. Re-run docs gates and orientation-index validation.

## Completion Checklist

- [ ] Productive tutorials and getting-started docs teach current syntax.
- [ ] Old syntax appears only in labeled migration/historical sections.
- [ ] Orientation indexes are updated where read paths change.
- [ ] Docs gates pass.
