# Runtime-Reasoner Design Review Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Define and execute a design-review phase that separates runtime-only concerns from runtime-to-reasoner interaction concerns before revising the Ash language and runtime specifications.

**Architecture:** This review phase freezes the separation rules first, then audits the existing canonical specs in parallel from two angles: core runtime semantics versus surface/observability and workflow-facing docs. A final synthesis task turns those audit results into an ordered spec-delta program that later implementation and convergence work can follow without reintroducing concern-mixing.

**Tech Stack:** Markdown planning docs, canonical spec set in `docs/spec/`, existing design docs in `docs/design/`, planning index and task tracking in `docs/plan/`.

---

### Task 1: Freeze Separation Rules and Review Protocol

**Files:**
- Create: `docs/plan/tasks/TASK-187-freeze-runtime-reasoner-separation-rules.md`
- Modify: `docs/plan/PLAN-INDEX.md`
- Modify: `CHANGELOG.md`

**Purpose:**
Define the classification rule set for the review phase so later audit tasks do not improvise their own boundaries.

**Scope:**
- Freeze the “does this make sense without a reasoner present?” test
- Define the three classification outcomes:
  - runtime-only
  - interaction-layer
  - split concern with explicit separation required
- Define review questions and evidence expectations for later tasks
- Identify explicit non-goals such as monitor views and `exposes` being reused as projection machinery

**Parallelization note:**
This task must complete before the audit tasks begin.

### Task 2: Audit Core Runtime and Verification Specs

**Files:**
- Create: `docs/plan/tasks/TASK-188-audit-runtime-and-verification-specs-for-reasoner-boundaries.md`
- Review targets:
  - `docs/spec/SPEC-001-IR.md`
  - `docs/spec/SPEC-004-SEMANTICS.md`
  - `docs/spec/SPEC-017-CAPABILITY-INTEGRATION.md`
  - `docs/spec/SPEC-018-CAPABILITY-MATRIX.md`

**Purpose:**
Audit the core runtime-facing specs against the separation rules and identify where projection, advisory output, acceptance boundaries, and runtime-only responsibilities are currently explicit, implicit, or blurred.

**Scope:**
- Check whether each construct has complete meaning without a reasoner present
- Identify runtime-only features that must remain free of interaction-layer meaning
- Identify missing but needed interaction-boundary statements in runtime docs
- Produce a findings list with concrete file/section references and severity

**Parallelization note:**
Can run in parallel with Task 3 once Task 1 is complete.

### Task 3: Audit Surface, Observability, and Workflow-Facing Docs

**Files:**
- Create: `docs/plan/tasks/TASK-189-audit-surface-and-observability-docs-for-reasoner-boundaries.md`
- Review targets:
  - `docs/spec/SPEC-002-SURFACE.md`
  - `docs/spec/SPEC-021-RUNTIME-OBSERVABLE-BEHAVIOR.md`
  - `docs/design/RUNTIME_REASONER_INTERACTION_MODEL.md`
  - `docs/design/LANGUAGE-TERMINOLOGY.md`
  - `docs/plan/tasks/TASK-186-monitor-authority-and-exposed-workflow-view.md`

**Purpose:**
Audit the human-facing language and observability contracts to ensure runtime-only constructs such as monitoring and exposed workflow views are not overloaded with runtime-to-reasoner meaning.

**Scope:**
- Distinguish monitorability from projection
- Identify where human-facing docs need future guidance without contaminating runtime-only features
- Find terminology collisions between exposure, observation, projection, and advisory interaction
- Produce a findings list with concrete file/section references and severity

**Parallelization note:**
Can run in parallel with Task 2 once Task 1 is complete.

### Task 4: Synthesize Spec Delta Program and Downstream Impact

**Files:**
- Create: `docs/plan/tasks/TASK-190-synthesize-runtime-reasoner-spec-delta-program.md`
- Modify: `docs/plan/PLAN-INDEX.md`
- Modify: `CHANGELOG.md`

**Purpose:**
Turn the rule-freeze task and the two audits into an ordered review outcome that states what to revise, what to leave untouched, and how the work affects later convergence phases.

**Scope:**
- Merge findings from Tasks 187-189
- Produce one prioritized delta list by document
- Mark which deltas are:
  - pure framing additions
  - normative contract changes
  - new document requirements
- Identify which existing planned tasks or phases are blocked, unchanged, or need follow-up tasks

**Parallelization note:**
This task depends on Tasks 187-189 completing.

## Execution order

1. Complete Task 1 and freeze the separation rules.
2. Run Tasks 2 and 3 in parallel.
3. Complete Task 4 to synthesize the deltas and downstream task impact.

## Review rules to preserve

- A feature is `runtime-only` if it still has complete meaning in a reasoner-free execution model.
- A feature is `interaction-layer` if it only exists to govern runtime-to-reasoner participation.
- A feature is `split` if it has both a runtime core meaning and a separate interaction-facing interpretation that must be specified independently.
- Monitor views, `exposes`, and workflow observability remain runtime-only unless a separate contract proves otherwise.
- Tool calls are a concrete current interaction boundary, not the sole canonical meaning of advisory interaction.
- The core IR and runtime semantics remain execution-neutral and must not assume full visibility into reasoner history or internal state.

## Deliverables

- A frozen review protocol for runtime-only versus interaction-layer classification
- Two audit reports covering core semantics and workflow-facing docs
- A prioritized spec-delta program for later spec revision work
- Explicit guidance on which future tasks can run in parallel using sub-agents

## Suggested parallel agent split

- Agent A: Task 2, core runtime and verification audit
- Agent B: Task 3, surface and observability audit
- Main agent: Task 1 first, then Task 4 synthesis

## Completion criteria

- The separation test is explicit and reusable
- Review scope is fully enumerated
- Tasks are large enough to produce meaningful audit output
- Parallel execution boundaries are clear
- The plan does not yet change normative spec meaning
