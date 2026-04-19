# TASK-613: Phase 90 Status/Corpus Reconciliation and TASK-595 End-to-End Validation

## Status: ✅ Complete

## Resolution Note

This task audited and reconciled the Phase 90 planning/status corpus against the actual code on `main`.

Historical note: the regex-specific downgrade recorded here was accurate when TASK-613 landed, but has since been superseded by Phase 92 Track E work (TASK-627/TASK-628/TASK-630/TASK-629). `std::regex` is now proven end-to-end from imported Ash source on the builtin path alone.

**Findings:**
- Phase 90 PLAN-INDEX listed 13 tasks (TASK-590–603) as `✅ Complete`; all corresponding task files said `📝 Planned`. No Track A (file collector, plan-index parser, spec-links validator, changelog checker, report formatter) or Track C (integration/meta-validation) code existed on `main`.
- TASK-595 (std::regex) had only partial runtime support on main at the time of this audit, and the Ash-language surface (`use regex::{find,matches,replace}`) was not yet functional end-to-end because `std/src/regex.ash` still used `act execute` inside `fn` bodies.

**Actions taken:**
1. Cherry-picked the then-current TASK-595 Rust regex backend code from the discarded `phase-90` worktree onto `main` (commit `a94dce4`).
2. Downgraded TASK-595 from `Complete` to `🟡 Partial` in PLAN-INDEX and its task file.
3. Downgraded TASK-590–594, TASK-596–598, TASK-600–603 from `Complete` to `📝 Planned` in PLAN-INDEX (no code exists on main).
4. Discarded the `phase-90` worktree and branch (superseded by Phase 91 substrate).
5. Fixed stale file-path and error-handling wording in TASK-595 task file.
6. Added a limitation regression test codifying the then-current import failure; that test target has since been repurposed into honest positive/complementary coverage by TASK-630.

**Remaining honest gaps at TASK-613 close:**
- Ash-language regex import/use was still blocked on parser support for `act execute` in `fn` body expression position. This gap is now closed by TASK-627/TASK-628/TASK-630.
- Track A and Track C tasks had no code on `main`; they required Ash-language features not yet implemented.

## Description

Close the gap between the documented Phase 90 completion claims and the current repository reality. This task had two tightly-scoped goals: (1) reconcile Phase 90 planning/task-state surfaces so they are not contradictory on `main`, and (2) validate TASK-595 (`std::regex`) end-to-end from the Ash-language surface rather than only through Rust-side provider tests. The required outcome is a Phase 90 state that is internally consistent with no contradictory docs or unproven stdlib surface claims.

## Specification Reference

- [PLAN-090: Spec Processor](../PLAN-090-SPEC-PROCESSOR.md)
- [DESIGN-SPEC-PROCESSOR](../../design/DESIGN-SPEC-PROCESSOR.md)
- [TASK-595: std::regex Interface and Rust Backend](TASK-595-std-regex.md)
- `docs/spec/SPEC-002-SURFACE.md`
- `docs/spec/SPEC-003-TYPE-SYSTEM.md`
- `docs/spec/SPEC-010-EMBEDDING.md`
- `docs/spec/SPEC-017-CAPABILITY-INTEGRATION.md`

## Dependencies

- ✅ Phase 91 merged to `main`
- ✅ TASK-612 alignment remediation merged to `main`
- ✅ Phase 90 worktree discarded after cherry-picking TASK-595 Rust provider

## Completion Checklist

- [x] Phase 90 status/task-state corpus is internally consistent
- [x] TASK-595 completion claim is either proven end-to-end or downgraded honestly → downgraded to Partial
- [x] Former `phase-90` worktree has been discarded (superseded by Phase 91 substrate)
- [x] `PLAN-INDEX`, PLAN-090, and task files agree after reconciliation
- [x] No contradictory completion claims remain for Phase 90 on `main`
- [x] Verification commands pass
