# TASK-1727: Close out Phase 168 with verification and status reconciliation

## Status: ✅ Completed

## Summary

Close out Phase 168 by reconciling implementation evidence, task status, plan/index state,
changelog, and follow-on lowering/macro/notation work. This task must not claim full macro or
surface-to-Core completion unless the earlier tasks actually implemented it.

## Specification Reference

- PLAN-168: `docs/plan/PLAN-168-SURFACE-AST-NOTATION-SUBSTRATE.md`
- TASK-1721 through TASK-1726
- SPEC-095c: `docs/spec/SPEC-095c-SURFACE-AST-MACROS-AND-NOTATION.md`
- SPEC-098c: `docs/spec/SPEC-098c-SURFACE-TO-CORE-LOWERING.md`

## Dependencies

- ✅ TASK-1726: Surface-to-Core lowering inventory

## Deferral / Planned-Feature Reconciliation

| Prior item | Source | Original reason | Prereqs now? | Decision | Gate |
|---|---|---|---|---|---|
| Full macro system | SPEC-095c | Out of Phase 168 scope | No | Keep deferred with concrete follow-on owner | Closeout evidence names future owner or explicit no-owner status |
| Full `SPEC-098c` lowering implementation | SPEC-098c | Requires substrate and next packet | Partial | Plan next implementation packet | Lowering inventory and PLAN-INDEX status agree |

## Files

- `docs/plan/PLAN-168-SURFACE-AST-NOTATION-SUBSTRATE.md`
- `docs/plan/PLAN-INDEX.md`
- `docs/plan/tasks/TASK-1721-*.md` through `TASK-1727-*.md`
- `CHANGELOG.md`
- Phase 168 audit/design artifacts created by earlier tasks

## Requirements

1. Verify all Phase 168 focused tests and docs gates on the final diff.
2. Reconcile every task status in the task files, phase plan, and `PLAN-INDEX.md`.
3. Record honest closeout evidence: commands run, artifacts created, implemented substrate, and
   explicitly deferred work.
4. Update `CHANGELOG.md` under `[Unreleased]`.
5. Add follow-on implementation packet notes for macro expansion, notation resolution, type
   inference, or lowering only if Phase 168 evidence supports them.
6. Run an independent review or explicitly leave the phase in planned/in-progress status until review
   blockers are resolved.

## Work steps

1. Re-read all Phase 168 task files and plan/index rows.
2. Run focused and broad-enough verification commands.
3. Patch statuses and closeout evidence only after verification.
4. Run an independent review pass and address blockers.

## Verification

```yaml
strictness: clean
commands:
  - git diff --check
  - python3 tools/docs/validate_orientation_indexes.py --self-test
  - bash scripts/check-docs-gate.sh
  - cargo fmt --check
  - cargo clippy -p ash-parser -p ash-typeck -p ash-engine --all-targets --all-features -- -D warnings
  - cargo test -p ash-parser
  - cargo test -p ash-typeck
  - cargo test -p ash-engine
  - python3 -c 'from pathlib import Path; p=Path("docs/plan/PLAN-168-SURFACE-AST-NOTATION-SUBSTRATE.md"); s=p.read_text(); assert "## Closeout evidence" in s and "TASK-1727" in s'
checklist:
  - [x] Focused parser/lowering tests pass.
  - [x] Docs gate passes.
  - [x] Phase plan, PLAN-INDEX, task files, and changelog agree.
  - [x] Deferred macro/notation/lowering work is named honestly.
  - [x] Independent review completed; review-triggered parser and expanded-boundary blockers addressed.
```

## Dispatch

```yaml
agent: hermes
reasoning: high
max_turns: 25
toolsets: [terminal, file]
```

## Dependencies for next task

Produces the reviewed Phase 168 substrate and follow-on implementation packet recommendations.
