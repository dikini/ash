# TASK-613: Phase 90 Status/Corpus Reconciliation and TASK-595 End-to-End Validation

## Status: 🟡 Ready

## Description

Close the gap between the documented Phase 90 completion claims and the current repository reality. This task has two tightly-scoped goals: (1) reconcile Phase 90 planning/task-state surfaces so they are not contradictory on `main` or after merging the remaining Phase 90 worktree, and (2) validate TASK-595 (`std::regex`) end-to-end from the Ash-language surface rather than only through Rust-side provider tests. The required outcome is a Phase 90 state that is safe to merge into current `main` without contradictory docs, broken task-status claims, or unproven stdlib surface behavior.

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
- 🟡 Remaining Phase 90 worktree at `~/Projects/ash/.worktrees/phase-90/` requires validation/reconciliation before merge

## Requirements

### Functional Requirements

1. Reconcile Phase 90 status surfaces so `PLAN-INDEX`, individual `TASK-590`–`TASK-603` files, and any phase-level deliverable text agree on what is actually complete, deferred, or still blocked.
2. Verify whether TASK-595 is genuinely complete from the Ash-language surface:
   - importing `regex::{find,matches,replace}` from Ash code must work, or
   - docs/task status must be updated honestly to mark the feature incomplete/blocked.
3. If the remaining Phase 90 worktree is stale relative to `main`, document and implement the safest integration path:
   - narrow cherry-pick/reapply of valid changes, or
   - explicit discard/supersession of stale branch-only scaffolding.
4. Ensure Phase 90 docs/specs are not contradictory internally and do not contradict current `main` after merge.
5. Update `CHANGELOG.md` if repository policy requires documenting planning/status or stdlib-surface corrections.

### Non-Functional Requirements

1. Do not broaden Phase 90 scope into new spec-processor features beyond validation/reconciliation.
2. Do not claim TASK-595 complete unless an end-to-end Ash callsite is verified.
3. Prefer explicit status correction over hand-wavy “complete enough” wording.
4. Treat broken or contradictory task-state surfaces as real bugs.
5. If the stale worktree cannot merge safely, prefer transplanting the narrow valid diff onto current `main` over direct branch merge.

## Files

### Planning/task-state reconciliation
- Modify: `docs/plan/PLAN-INDEX.md`
- Modify: `docs/plan/PLAN-090-SPEC-PROCESSOR.md` if its narrative overclaims or lags reality
- Modify: `docs/plan/tasks/TASK-590-file-collector.md`
- Modify: `docs/plan/tasks/TASK-591-plan-index-parser.md`
- Modify: `docs/plan/tasks/TASK-592-spec-links-validator.md`
- Modify: `docs/plan/tasks/TASK-593-changelog-checker.md`
- Modify: `docs/plan/tasks/TASK-594-report-formatter.md`
- Modify: `docs/plan/tasks/TASK-595-std-regex.md`
- Modify: `docs/plan/tasks/TASK-596-std-markdown.md`
- Modify: `docs/plan/tasks/TASK-597-std-json.md`
- Modify: `docs/plan/tasks/TASK-598-std-process.md`
- Modify: `docs/plan/tasks/TASK-599-std-diff.md` only if phase summary wording needs consistency
- Modify: `docs/plan/tasks/TASK-600-example-conformance.md`
- Modify: `docs/plan/tasks/TASK-601-capability-boundary.md`
- Modify: `docs/plan/tasks/TASK-602-meta-validation.md`
- Modify: `docs/plan/tasks/TASK-603-ci-gate.md`

### TASK-595 validation/fix surface
- Inspect/modify if needed: `std/src/regex.ash`
- Inspect/modify if needed: `std/src/lib.ash`
- Inspect/modify if needed: `crates/ash-engine/src/providers/regex.rs`
- Inspect/modify if needed: `crates/ash-engine/src/providers/mod.rs`
- Inspect/modify if needed: `crates/ash-engine/src/lib.rs`
- Inspect/modify if needed: `crates/ash-engine/src/module_loader.rs`
- Add/modify tests under `crates/ash-engine/tests/`
- Add/modify CLI/integration tests if required to prove Ash-language import/use actually works

### Optional merge-hygiene documentation
- Add/modify a short note in docs/plan or task notes documenting whether the remaining `phase-90` worktree should be merged directly, transplanted, or discarded

## TDD Steps

### Step 1: Status/corpus audit (Red)

Audit current `main` and the remaining `phase-90` worktree for contradictions across:
- Phase 90 header status in `PLAN-INDEX`
- task row statuses for TASK-590–603
- task file statuses/checklists
- PLAN-090 narrative and deliverable wording

Write down the exact mismatches before changing anything.

### Step 2: TASK-595 end-to-end failing test (Red)

Add a real end-to-end test proving the Ash-language surface works, for example:
- import `regex::{find}` in an Ash file and call it from a workflow/function
- verify it parses, typechecks, and executes through engine/CLI/module loading

If current substrate cannot support this surface, the failing test should make the limitation explicit.

### Step 3: Implement the smallest honest fix (Green)

Do one of the following, but only one honest path:
1. Make `std::regex` actually importable/usable from Ash code end-to-end, or
2. If the current substrate cannot support the wrapper shape honestly, update TASK-595/Phase 90 status/docs so the repo stops claiming completion.

### Step 4: Reconcile Phase 90 statuses/docs (Green)

Update `PLAN-INDEX`, PLAN-090, and TASK-590–603 files so they reflect the real completion state after Step 3. Completed tasks should have honest status/checklists; incomplete or blocked tasks should not be marked complete.

### Step 5: Worktree merge-safety determination

Assess the remaining `~/Projects/ash/.worktrees/phase-90/` branch against current `main` and document the safe path:
- direct merge safe, or
- transplant/cherry-pick specific commits/files, or
- discard stale worktree after porting valid changes.

### Step 6: Final verification

Run targeted verification for TASK-595 and the spec-processor planning surfaces, then workspace checks as needed.

## Verification Steps

### TASK-595 / Phase 90 targeted checks
- [ ] `cargo test -p ash-engine --test regex_capability`
- [ ] one real Ash-language import/use test for `regex::{find|matches|replace}` passes
- [ ] if spec-processor app wiring is involved, relevant `apps/spec_processor` verification commands are documented and runnable

### Merge-compatibility / corpus checks
- [ ] `PLAN-INDEX` Phase 90 header/status agrees with TASK-590–603 state
- [ ] TASK-590–603 files have honest statuses/checklists
- [ ] Phase 90 worktree integration path is documented (merge vs transplant vs discard)
- [ ] no contradictory Phase 90 completion claims remain on `main`

### Workspace quality gates
- [ ] `cargo test --workspace`
- [ ] `cargo clippy --workspace --all-targets --all-features -- -D warnings`
- [ ] `cargo fmt --check`

## Completion Checklist

- [ ] Phase 90 status/task-state corpus is internally consistent
- [ ] TASK-595 completion claim is either proven end-to-end or downgraded honestly
- [ ] Remaining `phase-90` worktree has a documented safe integration path
- [ ] `PLAN-INDEX`, PLAN-090, and task files agree after merge
- [ ] No spec/doc/task contradictions remain for Phase 90 on `main`
- [ ] Verification commands pass

## Dependencies for Next Task

This task outputs:
- a merge-safe, corpus-consistent Phase 90 state
- an honest answer about whether `std::regex` is truly landed end-to-end
- a documented integration/discard plan for the remaining Phase 90 worktree

Required by:
- any future claim that Phase 90 is complete on `main`
- any merge of the remaining `phase-90-spec-processor` worktree
- downstream Phase 90 continuation work that depends on a trustworthy task/status baseline

## Notes

This is a remediation/validation task, not a new feature phase. The key principle is repository honesty: after this task, Phase 90 should not overclaim completion, and any remaining merge should be safe relative to current `main` and non-contradictory with current docs/specs.