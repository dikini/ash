# Tooling and Surface Implementation Planning Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Define the next code-facing planning surface for CLI, REPL, trace output, and explanatory surface guidance without changing runtime authority or opening implementation tasks prematurely.

**Architecture:** This phase stays in docs/planning. It audits the user-facing tooling surfaces separately from the authoritative runtime boundary work, identifies where later implementation planning may need wording, ordering, or presentation changes, and ends in one steering brief that states what tooling and surface task clusters should later exist and what must remain explanatory only.

**Tech Stack:** Markdown planning docs, runtime-observable and runtime-reasoner reference corpus in `docs/reference/`, current CLI/REPL specs in `docs/spec/`, existing convergence planning docs in `docs/plan/`, and CLI/REPL/provenance surfaces in the Rust workspace as planning evidence only.

---

### Task 1: Audit CLI and REPL Surfaces for Runtime-Reasoner-Aware Planning

**Files:**
- Create: `docs/plan/tasks/TASK-202-audit-cli-and-repl-surfaces-for-interaction-planning.md`
- Modify: `docs/plan/PLAN-INDEX.md`
- Modify: `CHANGELOG.md`

**Purpose:**
Map the current CLI and REPL surfaces that may later need wording, ordering, or presentation alignment with the new runtime-reasoner docs corpus without turning user-facing output into semantic authority.

**Scope:**
- audit `ash run`, `ash trace`, `ash repl`, and related REPL command/reporting surfaces
- classify what is runtime-observable behavior versus explanatory-only guidance
- identify where later implementation planning may need output or prompt adjustments
- record the result as an audit, not a redesign

**Parallelization note:**
Can run in parallel with Task 2.

### Task 2: Audit Trace Export and Presentation Surfaces for Tooling Planning

**Files:**
- Create: `docs/plan/tasks/TASK-203-audit-trace-export-and-presentation-surfaces.md`

**Purpose:**
Map trace formatting, export, and presentation surfaces that may later need stage-aware or interaction-aware wording while preserving runtime-owned provenance and observability.

**Scope:**
- review CLI trace formatting, provenance export helpers, and related presentation surfaces
- identify where later planning may need advisory/gated/committed presentation distinctions
- keep such distinctions explanatory or presentation-level unless the runtime contract says otherwise
- record the result as an audit that the synthesis task can consume

**Parallelization note:**
Can run in parallel with Task 1.

### Task 3: Synthesize Tooling and Surface Steering Brief

**Files:**
- Create: `docs/plan/tasks/TASK-204-synthesize-tooling-and-surface-steering-brief.md`
- Modify: `docs/plan/PLAN-INDEX.md`
- Modify: `CHANGELOG.md`

**Purpose:**
Turn the tooling and presentation audits into one steering brief that defines what user-facing task clusters should later exist, what remains explanatory-only, and what must not be mistaken for runtime semantic change.

**Scope:**
- merge the output of Tasks 202 and 203
- define later tooling/surface task clusters without opening them
- call out review checkpoints and explicit steering questions
- protect runtime-only semantics from being silently shifted by wording or prompt design

**Parallelization note:**
Depends on Tasks 202 and 203.

## Execution order

1. Run Tasks 202 and 203 in parallel.
2. Complete Task 204 as the synthesis and review-brief step.

## Review checkpoint

Review this phase after Task 204 completes and before opening any tooling or surface code-facing tasks.

Steering questions:

1. Are the planned tooling and surface adjustments clearly presentation-level rather than semantic changes?
2. Is the boundary between runtime-observable behavior and explanatory stage guidance still explicit?
3. Did any CLI, REPL, or trace surface accidentally become a proxy for projection or hidden reasoner state?

## Review rules to preserve

- Do not add new syntax or stage markers in this phase.
- Do not redefine runtime authority through CLI, REPL, or trace wording.
- Keep explanatory stage guidance separate from monitorability and runtime observability.
- Preserve the canonical runtime-observable contracts as the authority for user-visible behavior.
- Do not open implementation tasks in this phase.

## Deliverables

- One audit of CLI and REPL planning surfaces
- One audit of trace export and presentation planning surfaces
- One tooling/surface steering brief for later task creation

## Suggested parallel agent split

- Agent A: Task 202
- Agent B: Task 203
- Main agent: Task 204

## Completion criteria

- Tooling and presentation planning surfaces are explicitly audited
- The next tooling/surface task clusters are clear without being opened
- The review checkpoint and steering questions are explicit
- Presentation-level work remains cleanly separated from runtime semantic authority
