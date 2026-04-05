# TASK-397: Five-Layer Alignment Matrix and Closure Contract (MCE-007)

## Status: 📝 Planned

## Description

Create the execution-ready planning backbone for MCE-007 by defining one canonical five-layer alignment matrix over construct families, one evidence model for each layer boundary, and one explicit closure contract for what it will mean to mark full-layer alignment complete later. This task is documentation/planning only. It must not claim that runtime/interpreter correspondence is closed before MCE-006 provides the required runtime-mapping evidence.

## Specification Reference

- [SPEC-001: Intermediate Representation](../../spec/SPEC-001-IR.md)
- [SPEC-004: Operational Semantics](../../spec/SPEC-004-SEMANTICS.md)
- [MCE-004: Big-Step Semantics Alignment](../../ideas/minimal-core/MCE-004-BIG-STEP-ALIGNMENT.md)
- [MCE-005: Small-Step Semantics](../../ideas/minimal-core/MCE-005-SMALL-STEP.md)
- [MCE-006: Align Small-Step Semantics with IR Execution](../../ideas/minimal-core/MCE-006-SMALL-STEP-IR.md)
- [MCE-007: Full Layer Alignment](../../ideas/minimal-core/MCE-007-FULL-ALIGNMENT.md)
- [TASK-393: Big-Step Semantics Alignment (MCE-004)](TASK-393-big-step-semantics-alignment.md)
- [TASK-394: Small-Step Semantics Scope and Configuration Contract](TASK-394-small-step-semantics-scope-and-configuration-contract.md)
- [TASK-395: Canonical Workflow Small-Step Rule Set and Concurrency Semantics](TASK-395-canonical-workflow-small-step-rule-set-and-concurrency-semantics.md)
- [TASK-396: Small-Step / Big-Step Correspondence and MCE-006 Handoff](TASK-396-small-step-big-step-correspondence-and-mce-006-handoff.md)

## Dependencies

- ✅ [TASK-393: Big-Step Semantics Alignment (MCE-004)](TASK-393-big-step-semantics-alignment.md)
- ✅ [TASK-394: Small-Step Semantics Scope and Configuration Contract](TASK-394-small-step-semantics-scope-and-configuration-contract.md)
- ✅ [TASK-395: Canonical Workflow Small-Step Rule Set and Concurrency Semantics](TASK-395-canonical-workflow-small-step-rule-set-and-concurrency-semantics.md)
- ✅ [TASK-396: Small-Step / Big-Step Correspondence and MCE-006 Handoff](TASK-396-small-step-big-step-correspondence-and-mce-006-handoff.md)
- ✅ [TASK-396: Small-Step / Big-Step Correspondence and MCE-006 Handoff](TASK-396-small-step-big-step-correspondence-and-mce-006-handoff.md)
- Note: TASK-397 is executable now; only later MCE-007 runtime-evidence ingestion tasks are gated on MCE-006 maturity

## Requirements

### Functional Requirements

1. Reframe MCE-007 as a five-layer alignment closeout/meta-verification phase rather than an open-ended exploration.
2. Define one canonical alignment matrix keyed by construct families, not by surface sugar variants, helper implementation details, or per-rule proof obligations.
3. Define the minimum construct families the matrix must cover:
   - sequencing / binding / branching,
   - pattern-driven control,
   - receive / blocking behavior,
   - parallel composition,
   - capability / policy / obligation workflows,
   - spawn / completion observation contracts.
4. Define one evidence model for each layer boundary:
   - Surface → IR,
   - IR → Big-step,
   - Big-step → Small-step,
   - Small-step → Interpreter.
5. Define one canonical row status vocabulary that distinguishes at least:
   - closed,
   - closed pending packaging,
   - pending MCE-006,
   - residual drift / follow-up required.
6. Record which alignment cells are already closed by accepted corpus inputs from MCE-004 and Phase 61, and which cells remain gated on MCE-006.
7. Define explicit phase exit criteria for eventual MCE-007 completion.
8. Define explicit non-goals so MCE-007 does not absorb runtime design, abstract-machine design, scheduler theory, or reopening of MCE-004/MCE-005 decisions.
9. Update PLAN-INDEX with a dedicated MCE-007 phase entry that is clearly gated on MCE-006 maturity.

### Non-Functional Requirements

1. Keep scope limited to docs/planning/alignment; no Rust/runtime/interpreter changes.
2. Preserve accepted current-corpus terminology from SPEC-001, SPEC-004, MCE-004, and MCE-005.
3. Use repo-relative links throughout.
4. Be precise enough that later tasks can consume MCE-006 mechanically rather than reinterpret MCE-007 from scratch.
5. Keep the matrix at construct-family granularity so the phase remains tractable.

## TDD Evidence

### Red

Before this task:

- `MCE-007` describes deliverables and gaps at a high level but does not define one execution-ready matrix artifact;
- there is no dedicated PLAN-INDEX phase/task scaffold for MCE-007;
- there is no explicit closure contract explaining what evidence is sufficient to mark full-layer alignment complete;
- the boundary between MCE-006 runtime mapping work and MCE-007 full-stack closeout is easy to blur.

### Green

This task is complete when:

- MCE-007 reads as an execution-ready closeout/meta-alignment phase;
- one canonical matrix structure is defined;
- one evidence model per layer boundary is defined;
- closed-versus-pending-on-MCE-006 status is explicit;
- later MCE-007 tasks can be written against the frozen matrix/closure contract without guessing scope.

## Files

- Modify: `docs/ideas/minimal-core/MCE-007-FULL-ALIGNMENT.md`
- Modify: `docs/plan/PLAN-INDEX.md`
- Modify: `docs/ideas/README.md` (if status/notes need planning linkage)
- Modify: `docs/ideas/IMPLEMENTABILITY-REPORT.md` (if MCE-007 assessment wording should mention the new planning phase)
- Reference: `docs/ideas/minimal-core/MCE-006-SMALL-STEP-IR.md`
- Reference: `docs/ideas/minimal-core/MCE-005-SMALL-STEP.md`
- Reference: `docs/ideas/minimal-core/MCE-004-BIG-STEP-ALIGNMENT.md`

## TDD Steps

### Step 1: Identify the current planning gap (Red)

Confirm that MCE-007 still lacks:

- a canonical five-layer matrix artifact,
- a row status vocabulary,
- explicit closeout criteria,
- explicit gating language for MCE-006-owned runtime evidence.

### Step 2: Freeze the alignment frame

Document:

- construct-family granularity,
- the four adjacency boundaries,
- evidence expectations for each boundary,
- explicit phase non-goals.

### Step 3: Freeze the closeout contract

Define:

- the row status vocabulary,
- what counts as a closed row,
- what remains pending MCE-006,
- what counts as residual drift requiring follow-on work.

### Step 4: Update planning surfaces

Add a dedicated MCE-007 phase entry to `PLAN-INDEX` with explicit gating and downstream tasks.

### Step 5: Verify GREEN

Expected pass condition:

- a reader can tell exactly what “full-layer alignment” means, what evidence is required, which rows are already closed, and what remains blocked on MCE-006.

## Completion Checklist

- [ ] TASK-397 task file created
- [ ] MCE-007 reframed as a closeout/meta-alignment phase
- [ ] canonical construct-family matrix shape defined
- [ ] evidence model per boundary defined
- [ ] row status vocabulary defined
- [ ] MCE-006 gating language made explicit
- [ ] PLAN-INDEX updated with dedicated MCE-007 phase

## Dependencies for Next Task

This task outputs:

- one canonical five-layer matrix frame,
- one closure contract for MCE-007,
- one explicit boundary between MCE-006 runtime mapping and MCE-007 closeout.

Required by:

- TASK-398: Runtime/Interpreter Correspondence Ingestion for MCE-007
- TASK-399: Five-Layer Drift Resolution and Residual Gap Classification
- TASK-400: MCE-007 Closeout Summary and Drift-Prevention Checklist

## Notes

Important constraints:

- Do not reopen accepted MCE-004 / MCE-005 decisions.
- Do not let MCE-007 become a shadow runtime-design phase.
- Do not switch from construct-family alignment to per-rule proof obligations unless a later explicit task chooses that intentionally.
- Keep runtime representation ambiguity fenced into MCE-006 until MCE-006 matures.

Edge cases to keep visible in the matrix:

- blocked receive versus semantic stuckness,
- helper-backed `Par` aggregation and branch interleaving,
- spawn/completion authority boundaries,
- capability/policy/obligation flows whose runtime realization may be indirect.
