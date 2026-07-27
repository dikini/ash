# Semantic-Rule Coverage Workflow Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Require semantic implementation work to start from canonical rules and declared coverage rather than fixture-first development.

**Architecture:** Add a planning/review coverage map linked to specs and semantic traceability, then make it a mandatory prerequisite in the project workflow. TDD remains evidence for a declared rule and domain.

**Tech Stack:** Markdown policy/docs, task records, CHANGELOG, documentation gate.

---

### Task 1: Establish rule coverage

**Files:**
- Create: `docs/plan/SEMANTIC-RULE-COVERAGE.md`
- Create: `docs/plan/tasks/TASK-2027-semantic-rule-coverage-workflow.md`
- Modify: `docs/plan/PLAN-INDEX.md`

1. Record canonical rule families, layer statuses, bounded domains, and next gaps.
2. Link the workflow task from PLAN-INDEX.
3. Run `bash scripts/check-docs-gate.sh`.

### Task 2: Require rule-first delivery

**Files:**
- Modify: `AGENTS.md`
- Modify: `CHANGELOG.md`

1. Require a coverage row and rule-to-evidence chain before semantic implementation.
2. Require bounded claims to state non-goals and remaining coverage.
3. Run `python3 tools/docs/validate_orientation_indexes.py --self-test` and `bash scripts/check-docs-gate.sh`.
