# S57-7 Post-SPEC Review Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Audit Phase 57B task files against completed S57-1 through S57-6 specs, correct stale downstream assumptions, and record the review outcome in TASK-S57-7.

**Architecture:** Use `TASK-S57-7` as the canonical review report. First identify every Phase 57B task that still conflicts with the completed 57A specs, then make the smallest necessary task-file edits to remove stale syntax, citations, status drift, and invalid examples. Finally, mark S57-7 complete and update the plan index and changelog.

**Tech Stack:** Markdown planning/spec docs, Common Changelog, Rust workspace verification via Cargo

---

### Task 1: Audit all 57B task files for post-SPEC drift

**Files:**

- Review: `docs/plan/tasks/TASK-359-*.md` through `docs/plan/tasks/TASK-369-*.md`
- Modify: `docs/plan/tasks/TASK-S57-7-post-spec-review.md`

**Step 1: Write the failing test**

Use the audit requirement as the failing check: S57-7 currently predicts likely issues, but does not yet record an actual review outcome against completed S57-1 through S57-6.

**Step 2: Run test to verify it fails**

Run: inspect `TASK-S57-7` and the downstream task files.
Expected: FAIL because the review report and validated task status are not yet complete.

**Step 3: Write minimal implementation**

Audit each Phase 57B task and classify it as either:

- requires edits, or
- reviewed with no change required.

Update `TASK-S57-7` to record:

- which tasks were reviewed
- which tasks required edits
- what classes of drift were fixed
- which tasks were already aligned

**Step 4: Run test to verify it passes**

Re-read `TASK-S57-7` and confirm it now serves as a complete review record rather than a prediction list.

**Step 5: Commit**

Do not commit yet; batch with the stale-task fixes.

### Task 2: Fix stale capability and entry-typing assumptions in downstream tasks

**Files:**

- Modify any stale files found during the audit, likely including:
  - `docs/plan/tasks/TASK-361-args-capability.md`
  - `docs/plan/tasks/TASK-363b-runtime-main-verification.md`
  - `docs/plan/tasks/TASK-364-main-verification.md` if further drift remains
  - `docs/plan/tasks/TASK-367-cli-error-reporting.md`

**Step 1: Write the failing test**

Use spec-alignment as the failing check: stale downstream tasks still contain outdated forms like old capability parameter syntax, outdated invocation examples, or unresolved entry-workflow assumptions.

**Step 2: Run test to verify it fails**

Run: inspect the audited task files for:

- parameter syntax other than `cap X`
- method-style capability usage treated as normative
- stale blocked wording after S57-6
- stale or speculative validation questions now answered by completed specs
Expected: FAIL wherever such drift remains.

**Step 3: Write minimal implementation**

For each stale task file, make the smallest edit needed to:

- align examples and wording to `cap X`
- align invocation examples with explicit effect-form semantics where needed
- align entry-workflow assumptions with `main() -> Result<(), RuntimeError>` and zero-or-more capability parameters
- update status/blocking wording where the completed 57A specs have removed ambiguity

**Step 4: Run test to verify it passes**

Re-read the edited task files and confirm they no longer contradict S57-4, S57-5, or S57-6.

**Step 5: Commit**

Do not commit yet; batch with the review-record and tracker updates.

### Task 3: Update S57-7 status, trackers, and changelog

**Files:**

- Modify: `docs/plan/tasks/TASK-S57-7-post-spec-review.md`
- Modify: `docs/plan/PLAN-INDEX.md`
- Modify: `CHANGELOG.md`
- Create: `docs/plans/2026-03-31-s57-7-post-spec-review-design.md`
- Create: `docs/plans/2026-03-31-s57-7-post-spec-review-implementation.md`

**Step 1: Write the failing test**

Use project bookkeeping as the failing check: S57-7 is still pending and the changelog does not yet record the 57B post-spec alignment pass.

**Step 2: Run test to verify it fails**

Run: inspect `TASK-S57-7`, `PLAN-INDEX`, and `CHANGELOG`.
Expected: FAIL because they do not yet reflect completion of the review.

**Step 3: Write minimal implementation**

- mark `TASK-S57-7` complete with a concise completion summary
- update `PLAN-INDEX` to show S57-7 complete
- add a changelog entry describing the 57B task-alignment review and corrections

**Step 4: Run test to verify it passes**

Re-read the three files and confirm they agree on S57-7 completion.

**Step 5: Commit**

Stage all S57-7 files and commit with a conventional message, for example:
`docs(plan): align 57B tasks with updated entry specs`

### Task 4: Review and verify the complete S57-7 change

**Files:**

- Review all files changed in Tasks 1-3

**Step 1: Spec review**

Have a spec-focused review confirm the downstream tasks no longer contradict completed S57-1 through S57-6 rules.

**Step 2: Quality review**

Have a quality review confirm the audit record, task statuses, and review scope are internally consistent.

**Step 3: Run verification**

Run: `git diff --check && cargo test -q`
Expected: PASS

**Step 4: Commit**

If review feedback required additional fixes after the first pass, update the staged files and rerun verification before final commit.
