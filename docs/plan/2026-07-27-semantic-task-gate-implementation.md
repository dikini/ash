# Semantic Task Conformance Gate Implementation Plan

> **For Codex:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Enforce semantic-task records, evidence links, and targeted integration verification in local gates.

**Architecture:** A checked-in JSON manifest records the semantic workflow fields for each active task. A Python validator cross-checks that manifest against task files, the coverage map, and semantic traceability, then safely executes task-owned verification commands selected from staged semantic changes.

**Tech Stack:** Python 3 standard library, Bash gate scripts, existing Cargo integration tests, Markdown/JSON documentation.

---

### Task 1: Create the semantic-record validator contract

**Files:**
- Create: `docs/plan/semantic-task-records.json`
- Create: `tools/docs/validate_semantic_task_records.py`
- Test: `tools/docs/tests/test_validate_semantic_task_records.py`

**Step 1: Write failing tests**

Cover valid records, missing layer/evidence fields, unknown traceability rule IDs, missing task/map links,
and unsafe verification commands.

**Step 2: Verify RED**

Run `python3 -m unittest tools.docs.tests.test_validate_semantic_task_records` and confirm the
missing validator fails.

**Step 3: Implement the smallest validator**

Validate the manifest schema and cross-document links using only standard-library JSON and path
operations. Emit a stable JSON report.

**Step 4: Verify GREEN**

Re-run the focused test module and the validator against the repository manifest.

### Task 2: Add semantic staged-change and verification runners

**Files:**
- Create: `scripts/check-semantic-task-gate.sh`
- Modify: `scripts/check-pre-commit-gate.sh`
- Modify: `scripts/check-pre-push-gate.sh`
- Test: `scripts/tests/test-semantic-task-gate.sh`

**Step 1: Write failing shell tests**

Cover semantic-path changes without required task evidence, selected task-command execution, and
safe rejection of undeclared commands.

**Step 2: Verify RED**

Run the shell test and confirm it fails before the runner exists.

**Step 3: Implement the runner**

Classify semantic changes, resolve active task records, invoke the validator, run only declared
targeted commands in pre-commit, and run all active commands in pre-push.

**Step 4: Verify GREEN**

Run the shell test and existing pre-commit gate classifier tests.

### Task 3: Migrate active follow-up records and human links

**Files:**
- Modify: `docs/plan/semantic-task-records.json`
- Modify: `docs/plan/SEMANTIC-RULE-COVERAGE.md`
- Modify: `docs/plan/tasks/TASK-{2001,2002,2003,2004,2005,2008,2013,2014,439}-*.md`
- Modify: `docs/plan/PLAN-INDEX.md`

**Step 1: Add records and task links**

Record canonical rule IDs, bounded layers, evidence references, non-goals, next obligations, and
focused commands for each active task. Add concise task-file links and coverage-map links.

**Step 2: Verify document conformance**

Run the new validator, semantic traceability validator, orientation index check, and docs gate.

### Task 4: Repair TASK-2004 boundary evidence

**Files:**
- Modify: `crates/ash-engine/tests/task_2004_core_cps_production_boundary.rs`
- Modify: `crates/ash-engine/tests/task_2004_run_file_checked_cps_boundary.rs`
- Modify: `docs/plan/tasks/TASK-2004-core-cps-production-boundary-decision.md`

**Step 1: Replace obsolete negative controls**

Use an actually unsupported source form and retain the structured checked-Core/CPS admission
rejection assertion.

**Step 2: Verify evidence**

Run both integration test targets; confirm their admitted positive cases and unsupported negative
cases pass.

### Task 5: Quality and review

**Files:**
- Modify: `CHANGELOG.md`
- Modify: `docs/plan/tasks/TASK-2028-semantic-task-conformance-gate.md`
- Modify: `docs/plan/PLAN-INDEX.md`

**Step 1: Run verification**

Run targeted semantic records, all relevant integration tests, `cargo fmt --check`, Clippy, docs
gate, traceability, and `git diff --check`.

**Step 2: Code review**

Dispatch a review agent to inspect the gate for bypasses, unsafe command handling, and rule/evidence
drift.

**Step 3: Record completion evidence**

Update task, plan index, and changelog only after all checks pass.
