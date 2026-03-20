# Runtime-Reasoner Implementation Planning Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Review the existing convergence queue against the new runtime-reasoner specs and produce a revised implementation-planning surface before opening any new code-facing work.

**Architecture:** This phase stays in docs/planning. It first audits already-planned convergence tasks for impact from the new runtime-reasoner corpus, then defines the concrete implementation-planning surface implied by the new interaction contract, and finally synthesizes a revised convergence map showing which tasks are unchanged, which need reference/scope updates, and which new code-facing tasks should be introduced later.

**Tech Stack:** Markdown planning docs, existing task files in `docs/plan/tasks/`, current spec/reference corpus in `docs/spec/`, `docs/reference/`, and `docs/design/`, plus `PLAN-INDEX.md` and `CHANGELOG.md`.

---

### Task 1: Audit Planned Convergence Tasks Against the Runtime-Reasoner Spec Corpus

**Files:**
- Create: `docs/plan/tasks/TASK-196-audit-planned-convergence-tasks-against-runtime-reasoner-specs.md`
- Modify: `docs/plan/PLAN-INDEX.md`
- Modify: `CHANGELOG.md`

**Purpose:**
Review the already-planned convergence tasks against the new runtime-reasoner docs to classify which tasks are unchanged, which need updated references or scope notes, and which are blocked pending new implementation-planning work.

**Scope:**
- review at least `TASK-164` through `TASK-173`
- compare each task against the runtime-reasoner handoff corpus
- classify each task as unchanged, reference-update-only, scope-adjustment-needed, or blocked
- record concrete findings with task-file references

**Parallelization note:**
This task must complete before the next tasks begin.

### Task 2: Define the Runtime-Reasoner Implementation-Planning Surface

**Files:**
- Create: `docs/plan/tasks/TASK-197-define-runtime-reasoner-implementation-planning-surface.md`

**Purpose:**
Define the concrete implementation-planning surface implied by the new interaction contract without yet creating code-facing tasks.

**Scope:**
- identify runtime entry-point classes likely to need projection or advisory-boundary handling
- identify which concerns stay strictly docs-only
- identify which future implementation areas are likely runtime, tooling, or docs-facing
- keep the result implementation-planning only, not implementation execution

**Parallelization note:**
Can run after Task 1, but before final synthesis.

### Task 3: Synthesize Revised Convergence Map and Next Task Openings

**Files:**
- Create: `docs/plan/tasks/TASK-198-synthesize-revised-runtime-reasoner-convergence-map.md`
- Modify: `docs/plan/PLAN-INDEX.md`
- Modify: `CHANGELOG.md`

**Purpose:**
Turn the impact audit and the implementation-planning surface into one revised convergence map that states what existing tasks need updates and where new code-facing tasks should later be introduced.

**Scope:**
- merge findings from Tasks 196 and 197
- state which existing planned tasks should be updated in place
- state which new task clusters should be added later
- keep this as planning output only

**Parallelization note:**
Depends on Tasks 196 and 197 completing.

## Execution order

1. Complete Task 1 and freeze the impact classification of existing planned work.
2. Complete Task 2 to define the implementation-planning surface implied by the new docs corpus.
3. Complete Task 3 to synthesize the revised convergence map and later task-opening boundary.

## Review rules to preserve

- Do not reopen the runtime-only versus interaction-layer separation rules.
- Do not create new implementation tasks in this phase.
- Existing planned tasks should be preserved when still valid; prefer updating references or scope rather than replacing them.
- Runtime-only constructs remain protected and must not be reinterpreted as projection mechanisms.
- Any proposed new implementation work must be derived from the stable handoff corpus, not from fresh design improvisation.

## Deliverables

- One audit of the existing convergence queue against the new runtime-reasoner specs
- One definition of the concrete implementation-planning surface
- One revised convergence map for later code-facing task creation

## Suggested parallel agent split

- Main agent: Task 1, then Task 3
- Agent A or main agent: Task 2 after Task 1

## Completion criteria

- Existing planned tasks have an explicit impact classification
- The implementation-planning surface is explicit
- The revised convergence map states what is unchanged, what needs updating, and what later task clusters should exist
- No implementation tasks are opened prematurely
