# TASK-2033 Target-Spec Parity and Evidence Policy Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Enforce target-spec parity and separate implementation status from evidence.

**Architecture:** `AGENTS.md` and the Canonical Core define the policy. Semantic records and the traceability graph encode separate status axes. Python validators reject invalid records and their tests demonstrate each rejection.

**Tech Stack:** Markdown, JSON, Python unittest, existing documentation validators.

---

### Task 1: Add validator RED tests

**Files:**
- Modify: `tools/docs/tests/test_validate_semantic_task_records.py`
- Modify: `tools/docs/tests/test_validate_semantic_traceability.py`

1. Add tests for the three-axis schema and invalid conflated status values.
2. Run the tests and confirm the current validators fail them.

### Task 2: Implement schema and validator support

**Files:**
- Modify: `tools/docs/validate_semantic_task_records.py`
- Modify: `tools/docs/validate_semantic_traceability.py`
- Modify: `docs/plan/semantic-task-records.json`
- Modify: `docs/spec/SEMANTIC-TRACEABILITY.json`

1. Add the three report axes and fail-closed validation.
2. Migrate active records and traceability metadata.
3. Run the focused validators and tests.

### Task 3: Migrate policy documentation

**Files:**
- Modify: `AGENTS.md`
- Modify: `docs/spec/CANONICAL-CORE.md`
- Modify: `docs/spec/SPEC-INDEX.md`
- Modify: `docs/plan/SEMANTIC-RULE-COVERAGE.md`
- Modify: `docs/plan/PLAN-202-FORMAL-SEMANTICS-AND-VERIFICATION-PROGRAMME.md`
- Modify: `docs/plan/PLAN-203-RUNNABLE-ASH-SEMANTIC-REALIZATION.md`
- Modify: `docs/plan/PLAN-INDEX.md`
- Modify: `CHANGELOG.md`

1. Replace the retired meaning of `bounded`.
2. State the reporting rule and migration result.
3. Run documentation and orientation gates.

### Task 4: Review and final verification

1. Review policy language and validator behavior.
2. Run focused Python tests, semantic-record and traceability validators, orientation validation, the documentation gate, formatter, and the staged pre-commit hook.
3. Update this task status and commit with its changelog entry.
