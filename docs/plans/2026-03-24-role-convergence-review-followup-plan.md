# Role Convergence Review Follow-up Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Fix the remaining inline-module honesty blocker and reconcile the nearby role-convergence review follow-ups without widening scope beyond the touched parser and docs surface.

**Architecture:** First add a failing regression for the remaining silent-skip path, then adjust inline-module recovery in `parse_module.rs` so unsupported canonical items are rejected even after resynchronization. After the parser fix is green, refresh the affected audit/task wording so it matches the current test-only lowering story and the post-fix implementation boundary.

**Tech Stack:** Rust (`ash-parser`), Markdown plans/audits/specs, focused parser unit tests, `cargo fmt`, `cargo test`, and `cargo clippy`.

---

## Planning assumptions

- The blocker is limited to inline-module parser honesty in `ash-parser`.
- The flat role contract and `RoleObligationRef` work are already in acceptable shape.
- The module role-lowering helpers remain intentionally test-only unless a real non-test consumer is discovered during execution.
- This plan should not broaden into a parser architecture rewrite.

## Sequencing rules

1. Fix the silent-skip parser path before changing any closeout wording.
2. Use focused regressions to prove the blocker before and after the fix.
3. Keep the role-lowering surface bounded; prefer honest wording over API expansion.
4. Finish with evidence-backed verification only.

## Verification defaults

- `cargo fmt --check`
- `cargo test -p ash-parser`
- `cargo clippy -p ash-parser --all-targets --all-features -- -D warnings`

---

### Task 1: Reproduce the remaining silent-skip path

**Files:**

- Modify: `crates/ash-parser/src/parse_module.rs`
- Test: `crates/ash-parser/src/parse_module.rs`

**Step 1: Add a failing parser regression**

Add a new unit test in the `parse_module` test module covering this shape:

- unknown non-canonical item
- then unsupported canonical inline item such as `workflow`
- then a supported `role` or `capability`

The test should assert that parsing fails instead of silently reaching the later supported item.

Suggested case:

- `mod governance { weird thing { noop } workflow main { done } role reviewer { authority: [approve] } }`

**Step 2: Run the new focused test to verify RED**

Run:

```bash
cargo test -p ash-parser parse_module::tests::test_parse_inline_module_rejects_unsupported_workflow_after_unknown_item -- --nocapture
```

Expected: FAIL because recovery currently skips far enough to reach the later supported item.

**Step 3: Add one companion regression**

Add a second failing case using another unsupported canonical item, such as `policy` or `datatype`, after an unknown recovered item.

**Step 4: Run the second focused test to verify RED**

Run:

```bash
cargo test -p ash-parser parse_module::tests::test_parse_inline_module_rejects_unsupported_policy_after_unknown_item -- --nocapture
```

Expected: FAIL for the same reason.

**Step 5: Commit the failing-test checkpoint if desired**

```bash
git add crates/ash-parser/src/parse_module.rs
```

No commit required yet if keeping RED local.

---

### Task 2: Fix inline-module recovery honesty

**Files:**

- Modify: `crates/ash-parser/src/parse_module.rs`
- Test: `crates/ash-parser/src/parse_module.rs`
- Test: `crates/ash-parser/src/lib.rs` (only if an existing integration test needs adjustment)

**Step 1: Adjust recovery boundaries**

Update `skip_unknown_definition()` and/or the surrounding `parse_definitions()` control flow so resynchronization stops before:

- supported inline items: `role`, `capability`
- unsupported canonical inline items currently tracked by `starts_with_unsupported_inline_definition()`

The explicit-rejection branch in `parse_definitions()` should then fire naturally.

**Step 2: Keep genuine unknown-item recovery intact**

Preserve the ability to skip truly unknown non-canonical items until the next definition boundary.

**Step 3: Run the two focused regressions**

Run:

```bash
cargo test -p ash-parser parse_module::tests::test_parse_inline_module_rejects_unsupported_workflow_after_unknown_item -- --nocapture
cargo test -p ash-parser parse_module::tests::test_parse_inline_module_rejects_unsupported_policy_after_unknown_item -- --nocapture
```

Expected: PASS.

**Step 4: Re-run the existing honesty regressions**

Run:

```bash
cargo test -p ash-parser parse_module::tests::test_parse_inline_module_rejects_unsupported_inline_workflow_before_role -- --nocapture
cargo test -p ash-parser parse_module::tests::test_parse_inline_module_rejects_unsupported_inline_workflow_before_capability_and_role -- --nocapture
cargo test -p ash-parser parse_module::tests::test_parse_inline_module_rejects_unsupported_canonical_datatype_definition -- --nocapture
cargo test -p ash-parser parse_module::tests::test_parse_inline_module_rejects_unsupported_canonical_policy_definition -- --nocapture
```

Expected: PASS.

**Step 5: Re-run the same-module lowering regression**

Run:

```bash
cargo test -p ash-parser lib_tests::test_module_decl_lowers_inline_module_roles_after_parse -- --nocapture
```

Expected: PASS.

**Step 6: Commit the parser fix**

```bash
git add crates/ash-parser/src/parse_module.rs crates/ash-parser/src/lib.rs
git commit -m "fix: reject unsupported inline module items after recovery"
```

---

### Task 3: Reconcile review wording and stale task text

**Files:**

- Modify: `docs/audit/2026-03-23-role-convergence-closeout-audit.md`
- Modify: `docs/plan/tasks/TASK-218-implement-source-role-definition-parsing-and-lowering.md`
- Modify: `docs/plan/tasks/TASK-225-inline-module-role-honesty-fix.md` (only if wording needs refinement)
- Modify: `CHANGELOG.md`

**Step 1: Fix stale placeholder wording**

In `TASK-218`, replace wording that still says the parsed role shape lowers into “current placeholder core role metadata”. Update it to reflect the actual `RoleObligationRef` carrier and non-placeholder state.

**Step 2: Refresh the closeout audit wording**

Update the audit so it states the post-fix parser honesty result precisely:

- unsupported canonical inline items are rejected explicitly, including after recovery from earlier unknown items;
- the module role-lowering path remains a maintained test-only helper surface.

If the audit still says “no blocker-class issues remain”, keep that only after verification is complete.

**Step 3: Add a changelog note**

In `CHANGELOG.md`, add or refine an `Unreleased` entry summarizing:

- the inline-module recovery honesty fix;
- the review-driven wording cleanup.

**Step 4: Review the affected docs for consistency**

Check the touched wording against:

- `docs/spec/SPEC-009-MODULES.md`
- `docs/spec/SPEC-001-IR.md`
- `docs/spec/SPEC-002-SURFACE.md`

No wording should imply a broader production lowering API than the code actually provides.

**Step 5: Commit the doc/task cleanup**

```bash
git add docs/audit/2026-03-23-role-convergence-closeout-audit.md docs/plan/tasks/TASK-218-implement-source-role-definition-parsing-and-lowering.md docs/plan/tasks/TASK-225-inline-module-role-honesty-fix.md CHANGELOG.md
git commit -m "docs: reconcile role convergence review follow-up"
```

---

### Task 4: Final verification and closeout evidence

**Files:**

- No new files required unless verification exposes another wording mismatch

**Step 1: Run formatter verification**

Run:

```bash
cargo fmt --check
```

Expected: PASS.

**Step 2: Run crate verification**

Run:

```bash
cargo test -p ash-parser
```

Expected: PASS.

**Step 3: Run lint verification**

Run:

```bash
cargo clippy -p ash-parser --all-targets --all-features -- -D warnings
```

Expected: PASS.

**Step 4: Run focused text audits**

Run:

```bash
rg -n "placeholder core role metadata|silentl|skipped silently|test-only" docs/audit docs/plan/tasks CHANGELOG.md
rg -n "starts_with_unsupported_inline_definition|skip_unknown_definition|lower_role_definitions\(" crates/ash-parser/src
```

Expected:

- no stale placeholder wording remains in the touched tasks;
- parser code references align with the implemented recovery logic;
- doc wording matches the final implementation boundary.

**Step 5: Commit the final verified state if verification changed files**

```bash
git status
```

If verification caused no edits, no extra commit is needed.

---

## Execution handoff

Plan complete and saved to `docs/plans/2026-03-24-role-convergence-review-followup-plan.md`.

Two execution options:

1. **Subagent-Driven (this session)** - I dispatch a fresh subagent per task, review between tasks
2. **Parallel Session (separate)** - Open a new session with executing-plans and batch the work with checkpoints
