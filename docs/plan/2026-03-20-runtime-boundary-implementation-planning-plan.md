# Runtime Boundary Implementation Planning Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Define the next code-facing planning surface for authoritative runtime execution boundaries without opening implementation tasks prematurely.

**Architecture:** This phase stays in docs/planning. It separates authoritative runtime execution boundaries from tooling and surface concerns, audits the concrete runtime-facing entry points and trace/provenance touch points identified in the runtime-reasoner planning corpus, and ends in one steering brief that states what code-facing runtime tasks should later exist and what remains out of scope.

**Tech Stack:** Markdown planning docs, runtime-reasoner reference corpus in `docs/reference/`, current specs in `docs/spec/`, existing convergence planning docs in `docs/plan/`, and runtime implementation entry points in the Rust workspace as planning evidence only.

---

### Task 1: Audit Runtime Execution Entry Points and Acceptance Boundaries

**Files:**
- Create: `docs/plan/tasks/TASK-199-audit-runtime-execution-boundaries-for-interaction-planning.md`
- Modify: `docs/plan/PLAN-INDEX.md`
- Modify: `CHANGELOG.md`

**Purpose:**
Map the current authoritative runtime execution entry points and identify where later implementation planning may need to account for acceptance, rejection, commitment, and injected-context handling without redefining runtime-only behavior.

**Scope:**
- audit execution entry points such as engine and interpreter workflow runners
- identify authoritative acceptance/rejection boundaries already present in code or implied by the specs
- classify what is runtime-only, what may need interaction-aware handling later, and what remains out of scope
- record the result as an audit, not an implementation plan

**Parallelization note:**
Can run in parallel with Task 2.

### Task 2: Audit Runtime Trace and Provenance Surfaces for Interaction-Aware Planning

**Files:**
- Create: `docs/plan/tasks/TASK-200-audit-runtime-trace-and-provenance-surfaces.md`

**Purpose:**
Map the current trace, provenance, and workflow wrapper surfaces that may later need stage-aware or acceptance-aware treatment without changing authoritative runtime ownership.

**Scope:**
- review trace recorder, trace event, provenance export, and workflow wrapper surfaces
- identify what later planning may need for advisory versus committed visibility
- keep monitorability and observability runtime-only
- record the result as an audit that the synthesis task can consume

**Parallelization note:**
Can run in parallel with Task 1.

### Task 3: Synthesize Runtime Boundary Steering Brief

**Files:**
- Create: `docs/plan/tasks/TASK-201-synthesize-runtime-boundary-steering-brief.md`
- Modify: `docs/plan/PLAN-INDEX.md`
- Modify: `CHANGELOG.md`

**Purpose:**
Turn the runtime-boundary audits into one steering brief that defines what runtime code-facing task clusters should later exist, what should remain docs-only, and what must not be overloaded with runtime-to-reasoner meaning.

**Scope:**
- merge the output of Tasks 199 and 200
- define later runtime-boundary task clusters without opening them
- call out review checkpoints and explicit steering questions
- preserve runtime-only semantics for monitors, observability, provenance, and effect execution

**Parallelization note:**
Depends on Tasks 199 and 200.

## Execution order

1. Run Tasks 199 and 200 in parallel.
2. Complete Task 201 as the synthesis and review-brief step.

## Review checkpoint

Review this phase after Task 201 completes and before opening any runtime code-facing tasks.

Steering questions:

1. Are the identified runtime boundaries still fully meaningful without any reasoner present?
2. Are acceptance, rejection, commitment, and provenance boundaries separated cleanly from tooling and surface concerns?
3. Did any runtime-only feature get overloaded with projection or advisory semantics by accident?

## Review rules to preserve

- Do not reopen the core runtime-versus-reasoner separation rules.
- Do not create implementation tasks in this phase.
- Keep monitorability, `exposes`, workflow observability, and approval routing runtime-only.
- Treat tool calls or interaction requests as current runtime-boundary evidence, not as the only abstract model.
- Preserve the difference between runtime authority and user-facing presentation.

## Deliverables

- One audit of runtime execution entry points and acceptance boundaries
- One audit of trace/provenance and wrapper surfaces
- One runtime-boundary steering brief for later task creation

## Suggested parallel agent split

- Agent A: Task 199
- Agent B: Task 200
- Main agent: Task 201

## Completion criteria

- Runtime execution and trace/provenance planning surfaces are explicitly audited
- The next runtime-boundary task clusters are clear without being opened
- The review checkpoint and steering questions are explicit
- Runtime-only features remain protected from interaction-layer overload
