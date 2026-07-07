# TASK-1954: Formatter Current-Syntax Polish

**Status:** Planned
**Phase:** [PLAN-200: Tooling And Migration Polish](../PLAN-200-TOOLING-AND-MIGRATION-POLISH.md)

## Description

Polish formatter behavior for current target syntax and quarantine formatter behavior for legacy or
deprecated forms.

## Requirements

- Add formatter coverage for current functions, do blocks, rows, contracts/evidence,
  providers/profiles, process/channel helpers, testing helpers, templates, records, matches, and
  imports.
- Prevent productive formatter fixtures from preserving deprecated syntax.
- Keep any retained old-form formatter behavior explicitly compatibility-only.
- Ensure formatting output is stable and idempotent.

## TDD Steps

1. Add failing formatter/idempotence tests for current syntax cases from Phase 199 productive
   examples and templates.
2. Add a compatibility-only fixture for any old-form formatter behavior retained by policy.
3. Implement or repair formatter handling.
4. Verify formatting is stable, idempotent, and current-syntax-first.

## Completion Checklist

- [ ] Current-syntax formatter fixtures cover Phase 199 app/library surfaces.
- [ ] Formatter output is idempotent.
- [ ] Deprecated-form formatter behavior is compatibility-only or removed.
- [ ] Productive formatter fixtures contain no unlabeled old syntax.
