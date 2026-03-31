# S57-6 Entry Workflow Typing Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Add the normative entry-workflow typing contract for `main` to the Ash specs and align the blocked downstream task references with that contract.

**Architecture:** Put the canonical entry-workflow judgment and failure conditions in `SPEC-022`, and add a short type-system cross-reference in `SPEC-003` so workflow validation stays centralized while remaining connected to the general typing model. Then update the task tracker, downstream blocked task text, and changelog to reflect the resolved contract.

**Tech Stack:** Markdown specifications, Common Changelog, Rust workspace verification via Cargo

---

### Task 1: Add the canonical entry-workflow typing rule to SPEC-022

**Files:**

- Modify: `docs/spec/SPEC-022-WORKFLOW-TYPING.md`
- Reference: `docs/spec/SPEC-017-CAPABILITY-INTEGRATION.md`

**Step 1: Write the failing test**

Use the task acceptance criteria as the failing check: SPEC-022 currently lacks a normative rule that says entry `main` must return exactly `Result<(), RuntimeError>` and may only use `cap X` parameters.

**Step 2: Run test to verify it fails**

Run: review the current `SPEC-022` text and confirm no “Entry Workflow Typing” section exists.
Expected: FAIL because the contract is missing.

**Step 3: Write minimal implementation**

Add a dedicated section that defines:

- `main` as the only entry workflow name
- exact return type `Result<(), RuntimeError>`
- zero-or-more parameters, each typed `cap X`
- a judgment or rule for validating entry workflows
- error cases for wrong name, wrong return type, and non-capability parameters
- note that effects remain inferred from the body

**Step 4: Run test to verify it passes**

Re-read the section and confirm each acceptance criterion is covered once, with no contradiction against `SPEC-017`.

**Step 5: Commit**

Do not commit yet; batch with the other S57-6 spec edits.

### Task 2: Add the type-system cross-reference in SPEC-003

**Files:**

- Modify: `docs/spec/SPEC-003-TYPE-SYSTEM.md`
- Reference: `docs/spec/SPEC-022-WORKFLOW-TYPING.md`

**Step 1: Write the failing test**

Use the consistency requirement as the failing check: SPEC-003 currently does not point readers to the specialized entry-workflow typing judgment.

**Step 2: Run test to verify it fails**

Run: inspect the overview / phase-boundary text in `SPEC-003`.
Expected: FAIL because there is no explicit cross-reference for entry-workflow typing.

**Step 3: Write minimal implementation**

Add a concise note that entry-workflow validation is a specialized workflow-typing rule owned by `SPEC-022`, and that runtime availability remains outside pure typing.

**Step 4: Run test to verify it passes**

Re-read both specs and confirm the ownership split is clear and non-duplicative.

**Step 5: Commit**

Do not commit yet; batch with the other S57-6 spec edits.

### Task 3: Align planning/task documents and changelog

**Files:**

- Modify: `docs/plan/tasks/TASK-S57-6-spec-003-022-entry-typing.md`
- Modify: `docs/plan/tasks/TASK-364-main-verification.md`
- Modify: `docs/plan/PLAN-INDEX.md`
- Modify: `CHANGELOG.md`
- Create: `docs/plans/2026-03-31-s57-6-entry-typing-design.md`
- Create: `docs/plans/2026-03-31-s57-6-entry-typing-implementation.md`

**Step 1: Write the failing test**

Use document consistency as the failing check: the task file and TASK-364 still contain unresolved questions and pre-S57-5 examples.

**Step 2: Run test to verify it fails**

Run: inspect the task docs for unresolved questions, stale `capability Args` usage, or blocked wording that assumes S57-6 is incomplete.
Expected: FAIL because those unresolved placeholders are still present.

**Step 3: Write minimal implementation**

Update the docs to reflect the resolved contract:

- mark S57-6 complete once the spec text is in place
- record that `main()` is valid
- use `cap Args` in examples
- remove or resolve open questions in TASK-364
- update `PLAN-INDEX` status
- add a changelog entry summarizing the entry-workflow typing contract

**Step 4: Run test to verify it passes**

Re-read the changed planning docs and confirm they match `SPEC-022` exactly.

**Step 5: Commit**

Stage all S57-6 files and commit with a conventional message, for example:
`docs(spec): define entry workflow typing contract`

### Task 4: Review and verify the complete S57-6 change

**Files:**

- Review all files changed in Tasks 1-3

**Step 1: Spec review**

Have a spec-focused review confirm that:

- `SPEC-022` is the canonical location of the rule
- `SPEC-003` only cross-references and does not duplicate semantics
- the zero-or-more capability-parameter rule is consistent everywhere

**Step 2: Quality review**

Have a quality review confirm the task and downstream docs no longer contain stale assumptions.

**Step 3: Run verification**

Run: `git diff --check && cargo test -q`
Expected: PASS

**Step 4: Commit**

If review feedback required changes after the first commit, amend or add a follow-up commit, then rerun verification.
