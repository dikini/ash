# TASK-S57-5 Runtime Capability Syntax Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Define normative runtime-provided capability parameter syntax and invocation semantics in SPEC-017, then record completion metadata.

**Architecture:** Extend SPEC-017 with a usage-site capability type form `cap X`, a runtime-injection section, and explicit effect-first invocation rules that keep capability use in the `observe`/`receive`/`set`/`send` model. Update task tracking and changelog in the same change set.

**Tech Stack:** Markdown specifications, task tracking docs, Common Changelog

---

### Task 1: Update SPEC-017 capability typing and invocation rules

**Files:**

- Modify: `docs/spec/SPEC-017-CAPABILITY-INTEGRATION.md`

**Step 1: Write the failing test**

For this documentation task, the failing condition is that SPEC-017 currently defines capability declarations but does not define:

- `cap X` as a parameter type form
- runtime-provided capability injection
- effect-first invocation rules for runtime capabilities such as `Args`

**Step 2: Run test to verify it fails**

Inspect `docs/spec/SPEC-017-CAPABILITY-INTEGRATION.md` and confirm those three rules are absent.

Expected: the spec lacks a normative section for runtime-provided capability parameter syntax.

**Step 3: Write minimal implementation**

Add normative wording that:

- defines `capability_type ::= "cap" IDENTIFIER`
- states `cap X` is a usage-site type form, not a declaration form
- defines runtime injection for capabilities such as `Args`, `Stdout`, and `Stdin`
- specifies that capability invocation remains effect-first and explicit, with `observe Args 0` as the canonical read-like example

**Step 4: Run test to verify it passes**

Re-read the updated SPEC-017 sections and confirm the syntax, injection, and invocation rules are explicit and consistent.

**Step 5: Commit**

Stage the spec update with the rest of the task changes.

### Task 2: Update task tracking and changelog

**Files:**

- Modify: `docs/plan/tasks/TASK-S57-5-spec-017-runtime-capabilities.md`
- Modify: `docs/plan/PLAN-INDEX.md`
- Modify: `CHANGELOG.md`

**Step 1: Write the failing test**

The failing condition is stale project metadata: TASK-S57-5 is still pending and there is no changelog entry for the runtime capability syntax clarification.

**Step 2: Run test to verify it fails**

Inspect the task file, PLAN-INDEX, and changelog.

Expected:

- task status is pending
- PLAN-INDEX row is pending
- `CHANGELOG.md` has no TASK-S57-5 entry

**Step 3: Write minimal implementation**

Update the task file with the resolved design, mark it complete, update PLAN-INDEX to complete, and add a Common Changelog entry under `Unreleased`.

**Step 4: Run test to verify it passes**

Re-read all three files and confirm they reflect the completed spec clarification.

**Step 5: Commit**

Stage the metadata updates with the rest of the task changes.
