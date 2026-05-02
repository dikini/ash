# TASK-804: SPEC-B Closeout, Docs, and Verification

## Status: 📝 Planned

## Description

Reconcile docs, status surfaces, changelog, and verification evidence for Phase 110 after the implementation tasks are green.

## Specification Reference

- [SPEC-058](../../spec/SPEC-058-CANONICAL-TYPE-EXPRESSION-IR-PROJECTION-IDS-KIND-ARITY-SUBSTRATE.md)
- [PLAN-106](../PLAN-106-TYPE-EXPRESSION-IR-PROJECTION-IDS-KIND-ARITY-SUBSTRATE.md)
- [DESIGN-034](../../design/DESIGN-034-TOTAL-TYPE-COMPUTATION.md)

## Dependencies

- ✅ [TASK-803](TASK-803-spec-b-diagnostics-negative-tests-and-non-interference.md)

## Objective

Close Phase 110 honestly after focused and broad verification succeeds or any residual failure is explicitly classified.

## Requirements

1. Reconcile SPEC-058, PLAN-106, PLAN-INDEX, task files, and CHANGELOG.
2. Record exact focused and broad verification commands, with one-line pass/fail summaries, in a `## Verification Evidence` section in this task file.
3. Record the exact carried-forward non-Phase-110 suites by test target name and why each belongs to closeout.
4. Record independent-review handoff status in a `## Self-Review / Review Handoff` section in this task file. Do not mark the phase complete while controller review findings remain open.

## Files

- Modify: `docs/spec/README.md`
- Modify: `docs/plan/PLAN-INDEX.md`
- Modify: `docs/spec/SPEC-058-CANONICAL-TYPE-EXPRESSION-IR-PROJECTION-IDS-KIND-ARITY-SUBSTRATE.md` if needed for final status wording
- Modify: `docs/plan/PLAN-106-TYPE-EXPRESSION-IR-PROJECTION-IDS-KIND-ARITY-SUBSTRATE.md`
- Modify: `docs/plan/tasks/TASK-804-spec-b-closeout-docs-and-verification.md` with final `## Completion Notes`, `## Verification Evidence`, and `## Self-Review / Review Handoff` sections
- Modify: `CHANGELOG.md`

## TDD Steps

1. Assemble the final focused verification command set from the exact Phase 110 task-numbered suites before editing status surfaces.
2. Run focused verification first.
3. Run broad verification before claiming closeout and record any failure with the exact command, failing target, and ownership.
4. Only then update final status/checklist surfaces.

## Verification Steps

- [ ] `git diff --check`
- [ ] `cargo fmt --check`
- [ ] `cargo check --workspace`
- [ ] all focused Phase 110 suites are run and listed by exact target name in `## Verification Evidence`
- [ ] all carried-forward regression suites are listed by exact target name and rationale in `## Verification Evidence`
- [ ] broad verification command and result summary are recorded honestly in `## Verification Evidence`

## Notes

This task is docs/verification only. If broad verification fails, keep the phase open and document the failure honestly.
