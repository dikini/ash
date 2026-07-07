# TASK-1959: Tooling/Migration Polish Closeout

**Status:** Planned
**Phase:** [PLAN-200: Tooling And Migration Polish](../PLAN-200-TOOLING-AND-MIGRATION-POLISH.md)

## Description

Close out Phase 200 with full gates, stale-claim sweeps, docs, changelog, PLAN-INDEX reconciliation,
and review remediation.

## Requirements

- Run all Phase 200 focused tests and broad verification gates.
- Reconcile PLAN-200, task files, PLAN-INDEX, CHANGELOG, and relevant docs.
- Run stale-claim sweeps for legacy syntax, deprecated-form teaching paths, formatter/LSP legacy
  leakage, and authority-bypassing wording.
- Address code review findings before marking complete.

## TDD Steps

1. Run focused Phase 200 gates and fix failures.
2. Run broad workspace and docs gates.
3. Update status/evidence docs.
4. Complete review remediation.

## Completion Checklist

- [ ] Phase 200 focused gates pass.
- [ ] Workspace and docs gates pass.
- [ ] PLAN-INDEX and CHANGELOG are reconciled.
- [ ] Stale-syntax, deprecated-form, and stale-authority sweeps are recorded.
- [ ] Review remediation is complete.
