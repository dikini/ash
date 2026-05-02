# TASK-805: Phase 110 Review Remediation

## Status: 📝 Planned

## Description

Fix the blocking and non-blocking findings from the independent Phase 110 review after TASK-804. This task reserves the usual post-closeout hardening slice.

## Specification Reference

- [SPEC-058](../../spec/SPEC-058-CANONICAL-TYPE-EXPRESSION-IR-PROJECTION-IDS-KIND-ARITY-SUBSTRATE.md)
- [PLAN-106](../PLAN-106-TYPE-EXPRESSION-IR-PROJECTION-IDS-KIND-ARITY-SUBSTRATE.md)
- [DESIGN-034](../../design/DESIGN-034-TOTAL-TYPE-COMPUTATION.md)

## Dependencies

- ✅ [TASK-793](TASK-793-spec-b-spec-plan-packet.md) through [TASK-804](TASK-804-spec-b-closeout-docs-and-verification.md)
- ✅ Independent Phase 110 review findings from the controller review pass

## Objective

Remediate review findings with targeted regressions and re-verification, without widening the phase into later packets.

## Requirements

1. Fix only the findings raised by independent review or honest closeout re-checks.
2. Add targeted regression tests for each real bug found.
3. Re-run focused verification and any required broad verification.
4. Keep later-packet work deferred unless the review proves a real mis-scoping problem.

## Files

- Modify only the files touched by the actual review findings
- Add targeted regression tests near the affected crates
- Modify `CHANGELOG.md`, `PLAN-106`, `PLAN-INDEX`, and task status surfaces if closeout evidence changes

## TDD Steps

1. Write a failing regression for each blocking review finding.
2. Implement the minimal fix for each finding.
3. Re-run focused verification after each fix.
4. Run final closeout verification again before marking complete.

## Verification Steps

- [ ] targeted regression tests for the review findings
- [ ] `cargo fmt --check`
- [ ] `git diff --check`
- [ ] focused and broad verification as required by the finding set

## Notes

This task intentionally stays generic until review findings exist. It should not be used as a dumping ground for deferred SPEC-C/D/E/F/G work.
