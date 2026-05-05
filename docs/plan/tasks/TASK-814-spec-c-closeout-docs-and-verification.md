# TASK-814: SPEC-C Closeout, Docs, and Verification

## Status: 📝 Planned

## Description

Reconcile docs, status surfaces, changelog, and verification evidence for Phase 111 after the implementation tasks are green.

## Specification Reference

- [SPEC-059](../../spec/SPEC-059-SEALED-TYPE-LEVEL-DOMAINS.md)
- [PLAN-107](../PLAN-107-SEALED-TYPE-LEVEL-DOMAINS.md)
- [DESIGN-034](../../design/DESIGN-034-TOTAL-TYPE-COMPUTATION.md)

## Dependencies

- [TASK-813](TASK-813-sealed-domain-diagnostics-and-non-interference.md)

## Dispatch

```
agent: hermes
reasoning: low
max_turns: 12
toolsets: [terminal, file]
```

## Objective

Close Phase 111 honestly after focused and broad verification succeeds or any residual failure is explicitly classified.

## Requirements

1. Reconcile SPEC-059, PLAN-107, PLAN-INDEX, task files, and CHANGELOG.
2. Record exact focused and broad verification commands, with one-line pass/fail summaries, in a `## Verification Evidence` section in this task file.
3. Record carried-forward non-owner suites by exact target name, why each belongs in closeout, and which task originally owned the evidence.
4. Record independent-review handoff status in a `## Self-Review / Review Handoff` section in this task file. Do not mark the phase complete while controller review findings remain open.

## Files

- Modify: `docs/spec/README.md`
- Modify: `docs/plan/PLAN-INDEX.md`
- Modify: `docs/spec/SPEC-059-SEALED-TYPE-LEVEL-DOMAINS.md` if needed for final status wording
- Modify: `docs/plan/PLAN-107-SEALED-TYPE-LEVEL-DOMAINS.md`
- Modify: `docs/plan/tasks/TASK-814-spec-c-closeout-docs-and-verification.md` with final `## Completion Notes`, `## Verification Evidence`, and `## Self-Review / Review Handoff` sections
- Modify: `CHANGELOG.md`

## TDD Steps

1. Assemble the final focused verification command set from the exact Phase 111 task-numbered suites before editing status surfaces.
2. Run focused verification first.
3. Run broad verification before claiming closeout and record any failure with the exact command, failing target, and ownership.
4. Only then update final status/checklist surfaces.

## Verification

```
strictness: clean
commands:
  - cargo fmt --check
  - cargo check --workspace
checklist:
  - [ ] git diff --check passes
  - [ ] All focused Phase 111 suites run and listed by exact target name
  - [ ] Carried-forward suites listed by target name, rationale, and original task
  - [ ] Broad verification command and result summary recorded
  - [ ] SPEC-059, PLAN-107, PLAN-INDEX, CHANGELOG reconciled
```

## Notes

This task is docs/verification only. If broad verification fails, keep the phase open and document the failure honestly.
