# TASK-1735: Close out Phase 169 with verification and review

## Status: 📝 Planned

## Summary

Close Phase 169 by reconciling task status, plan status, changelog entries, docs/index surfaces, and
verification evidence after implementation and independent review.

## Specification Reference

- PLAN-169: `docs/plan/PLAN-169-SURFACE-EXPANSION-AND-NOTATION-ELABORATION.md`
- SPEC-095c: `docs/spec/SPEC-095c-SURFACE-AST-MACROS-AND-NOTATION.md`
- SPEC-098c: `docs/spec/SPEC-098c-SURFACE-TO-CORE-LOWERING.md`

## Dependencies

- 📝 TASK-1728 through TASK-1734 complete

## Deferral / Planned-Feature Reconciliation

| Prior item | Source | Original reason | Prereqs now? | Decision | Gate |
|---|---|---|---|---|---|
| Phase 169 completion | PLAN-169 | Implementation tasks must finish first | Pending | Reconcile after verification/review | All required gates and review pass on final diff |
| Macro/imported-notation/generalized-mixfix/full-lowering follow-up | PLAN-169 non-goals | Out of scope | No | Record next owners honestly | Closeout evidence lists deferrals without completion language |

## Files

- `docs/plan/PLAN-169-SURFACE-EXPANSION-AND-NOTATION-ELABORATION.md`
- `docs/plan/PLAN-INDEX.md`
- `docs/plan/tasks/TASK-1728-phase-169-plan-packet.md` through `TASK-1735-phase-169-closeout.md`
- `CHANGELOG.md`
- Any review/audit artifact created for closeout

## Requirements

1. Rerun focused Phase 169 tests and broad affected-crate gates on the final diff.
2. Run `cargo check --workspace` if any shared parser/surface carriers changed during the phase.
3. Run independent review focused on parser lookahead, traversal completeness, notation-table honesty,
   operator-section elaboration, and lowering-gate bypasses.
4. Fix review blockers and rerun focused and broad verification after the final fix.
5. Reconcile all status surfaces only after final verification.

## Work steps

1. Re-read the Phase 169 plan and every task file.
2. Run the verification command set and record evidence.
3. Dispatch independent review with exact changed files and expected boundaries.
4. Patch blockers, rerun verification, and update docs/changelog.
5. Mark Phase 169 complete only when code, docs, review, and status surfaces agree.

## Verification

```yaml
strictness: clean
commands:
  - cargo fmt --check
  - cargo test -p ash-parser
  - cargo test -p ash-typeck
  - cargo test -p ash-engine
  - cargo check --workspace
  - cargo clippy -p ash-parser -p ash-typeck -p ash-engine --all-targets --all-features -- -D warnings
  - git diff --check
  - python3 tools/docs/validate_orientation_indexes.py --self-test
  - bash scripts/check-docs-gate.sh
checklist:
  - [ ] Focused Phase 169 tests pass.
  - [ ] Broad affected crate tests pass.
  - [ ] Workspace check passes if shared carriers changed.
  - [ ] Independent review completed and blockers addressed.
  - [ ] PLAN-INDEX, PLAN-169, task files, and CHANGELOG agree.
```

## Dispatch

```yaml
agent: hermes
reasoning: high
max_turns: 25
toolsets: [terminal, file]
```

## Dependencies for next task

Produces a verified handoff to later macro expansion, imported-notation propagation, generalized
mixfix, and full `SPEC-098c` lowering packets.
