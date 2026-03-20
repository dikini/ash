# Runtime-Reasoner Spec Follow-Up Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Complete the docs-only follow-up phase that turns the runtime-reasoner delta program into clean, reviewable contracts and references without mixing in implementation-convergence work.

**Architecture:** This phase keeps all work in docs, specs, and reference notes. It first defines the missing runtime-to-reasoner interaction contract, then applies the minimum framing and terminology deltas that depend on that contract, then resolves the human-facing surface-guidance boundary, and finally synthesizes the resulting corpus into an implementation-readiness handoff for later convergence planning.

**Tech Stack:** Markdown design notes, canonical specs in `docs/spec/`, reference notes in `docs/reference/`, planning/task tracking in `docs/plan/`, and changelog/index updates only.

---

### Task 1: Define Runtime-to-Reasoner Interaction Contract

**Files:**
- Create: `docs/plan/tasks/TASK-191-define-runtime-to-reasoner-interaction-contract.md`
- Modify: `docs/plan/PLAN-INDEX.md`
- Modify: `CHANGELOG.md`

**Purpose:**
Write the missing interaction contract as a separate authoritative note so projection, advisory outputs, and runtime acceptance boundaries are defined without overloading runtime-only specs.

**Scope:**
- define projected or injected context
- define advisory outputs as non-authoritative artifacts
- define acceptance boundaries for artifacts returning to runtime
- state explicitly that monitor views and `exposes` are not projection machinery
- keep the transport abstract while acknowledging current tool-call boundaries

**Parallelization note:**
This task must complete before the next tasks begin.

### Task 2: Add Minimal Runtime-Authority Framing to SPEC-004

**Files:**
- Create: `docs/plan/tasks/TASK-192-add-runtime-authority-framing-to-spec-004.md`

**Purpose:**
Add the smallest possible framing delta to `SPEC-004` so the runtime semantics explicitly acknowledge runtime authority and advisory interaction boundaries without changing the canonical operational rules.

**Scope:**
- add a short framing section near the front of `SPEC-004`
- clarify authoritative runtime state ownership
- clarify that advisory interaction remains outside authoritative state transition until accepted
- preserve execution neutrality

**Parallelization note:**
Can run in parallel with Task 3 after Task 1 is complete.

### Task 3: Tighten Projection and Monitorability Terminology

**Files:**
- Create: `docs/plan/tasks/TASK-193-tighten-projection-and-monitorability-terminology.md`

**Purpose:**
Reserve and clarify the terms needed to keep monitorability, exposed workflow views, observation, and runtime-to-reasoner projection distinct.

**Scope:**
- update `docs/design/LANGUAGE-TERMINOLOGY.md`
- add a small non-overlap clarification to the runtime-reasoner interaction design/reference material if needed
- reserve `projection`, `monitorability`, and `exposed workflow view`
- document the overloaded use of `observe` and constrain its interpretation

**Parallelization note:**
Can run in parallel with Task 2 after Task 1 is complete.

### Task 4: Define Human-Facing Surface Guidance Boundary

**Files:**
- Create: `docs/plan/tasks/TASK-194-define-human-facing-surface-guidance-boundary.md`

**Purpose:**
Decide what human-facing guidance belongs in the surface-language documentation for advisory, gated, and committed stages, without introducing syntax unless the docs-only review proves it is necessary.

**Scope:**
- decide whether the needed surface guidance is explanatory only or normative
- document what stage guidance belongs in `SPEC-002`
- explicitly avoid overloading `exposes` or monitor visibility
- defer syntax work unless a later task is intentionally opened for it

**Parallelization note:**
Should start after Task 1 and benefits from Task 3 being complete first.

### Task 5: Synthesize Spec/Reference Handoff for Later Convergence Planning

**Files:**
- Create: `docs/plan/tasks/TASK-195-synthesize-runtime-reasoner-spec-handoff.md`
- Modify: `docs/plan/PLAN-INDEX.md`
- Modify: `CHANGELOG.md`

**Purpose:**
Merge the results of Tasks 191-194 into one implementation-readiness handoff that states which docs are now authoritative, which implementation-facing follow-up tasks should be planned later, and which runtime-only areas remain protected.

**Scope:**
- list the authoritative interaction/runtime docs after this phase
- summarize unresolved non-goals
- identify the later implementation-planning surface
- confirm that implementation convergence has not yet begun

**Parallelization note:**
Depends on Tasks 191-194 completing.

## Execution order

1. Complete Task 1 and freeze the interaction contract.
2. Run Tasks 2 and 3 in parallel.
3. Complete Task 4 after Task 1, preferably after Task 3.
4. Complete Task 5 to produce the implementation-readiness handoff.

## Review rules to preserve

- Runtime-only concerns stay runtime-only unless a later contract proves a true split concern.
- Monitor views, `exposes`, workflow observability, capability verification, approval routing, and effect execution are protected runtime-only areas.
- `SPEC-004` receives framing only, not a new operational model for reasoners.
- Human-facing surface guidance must not be used to smuggle in unstable semantics.
- This phase remains docs-only and does not create implementation-convergence tasks.

## Deliverables

- One runtime-to-reasoner interaction contract
- One minimal `SPEC-004` framing update
- One terminology pass protecting monitorability versus projection
- One resolved surface-guidance boundary for later user-facing docs
- One handoff note for later implementation-convergence planning

## Suggested parallel agent split

- Main agent: Task 1, then Task 5
- Agent A: Task 2 after Task 1
- Agent B: Task 3 after Task 1
- Main agent or later subagent: Task 4 after Task 3

## Completion criteria

- The interaction contract exists as a standalone document
- Runtime-only and interaction-layer terms are no longer drifting
- `SPEC-004` is framed without being overloaded
- The surface-guidance boundary is explicit
- The phase ends with a clean handoff for later implementation planning
