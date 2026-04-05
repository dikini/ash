# TASK-402: Residual Control, Blocked-State, and Completion Realization (MCE-006)

## Status: ✅ Complete

## Description

Build on TASK-401's frozen carrier-mapping baseline by explaining how the current interpreter/runtime realizes residual workflow control, blocked/suspended waiting states, and completion/control authority boundaries. This task is documentation/planning/runtime-correspondence work. It must convert the high-level open questions in MCE-006 into one concrete operational correspondence story without redesigning the accepted MCE-005 semantics.

## Specification Reference

- [SPEC-001: Intermediate Representation](../../spec/SPEC-001-IR.md)
- [SPEC-004: Operational Semantics](../../spec/SPEC-004-SEMANTICS.md)
- [MCE-005: Small-Step Semantics](../../ideas/minimal-core/MCE-005-SMALL-STEP.md)
- [MCE-006: Align Small-Step Semantics with IR Execution](../../ideas/minimal-core/MCE-006-SMALL-STEP-IR.md)
- [MCE-007: Full Layer Alignment](../../ideas/minimal-core/MCE-007-FULL-ALIGNMENT.md)
- [TASK-401: Runtime Carrier Inventory and Semantic Mapping Table](TASK-401-runtime-carrier-inventory-and-semantic-mapping-table.md)

## Dependencies

- ✅ [TASK-401: Runtime Carrier Inventory and Semantic Mapping Table](TASK-401-runtime-carrier-inventory-and-semantic-mapping-table.md)
- ✅ Accepted MCE-005 backbone in `docs/ideas/minimal-core/MCE-005-SMALL-STEP.md`
- 📝 Current interpreter/runtime evidence in `crates/ash-interp/`

## Requirements

### Functional Requirements

1. Explain the runtime control representation for residual workflow execution and classify it concretely against the accepted MCE-006 execution-model choices.
2. Define the current runtime carrier story for blocked/suspended states, covering at minimum:
   - blocking `Receive`,
   - mailbox/control waits,
   - yield/proxy suspension,
   - completion observation waits if present.
3. Distinguish clearly between:
   - active residual execution state,
   - blocked/suspended state,
   - terminal outcome state,
   - invalid/runtime-failure boundary,
   and make explicit where the current runtime does not materialize those classes as four fully separate concrete result carriers.
4. Define the current completion/control-authority realization story for `ControlLink`-style lifecycle and any available completion-sealing or retained-completion structures.
5. Record where the current runtime has:
   - direct realization,
   - distributed/partial realization,
   - missing or weakly evidenced realization.
6. Record concrete implementation evidence sources (files, types, functions) for each sub-area.
7. Produce one explicit operational correspondence section in `MCE-006-SMALL-STEP-IR.md` that later tasks can build on mechanically.

### Non-Functional Requirements

1. Keep scope limited to docs/planning/runtime-correspondence; no runtime redesign.
2. Preserve accepted MCE-005 semantics:
   - workflow-first execution,
   - blocked vs stuck distinction,
   - no new user-visible syntax,
   - no semantic reopening of `Receive` / spawn / control behavior.
3. Use repo-relative links throughout.
4. Be conservative: when evidence is partial, say so explicitly instead of overclaiming full realization.

## TDD Evidence

### Red

Before this task:

- TASK-401 identifies residual control, blocked-state, and completion realization as key remaining gaps;
- the corpus has a mapping table, but not yet one explicit operational story for how the runtime carries these nontrivial state classes;
- MCE-007 therefore lacks a clean runtime-evidence packet for these areas.

### Green

This task is complete when:

- MCE-006 contains one explicit section explaining residual control, blocked-state carriers, and completion/control realization;
- blocked/suspended versus terminal versus invalid states are distinguished in runtime terms;
- direct vs partial vs missing realization is explicit;
- the result is conservative enough for MCE-007 to consume without reinterpreting runtime ownership boundaries.

## Files

- Modify: `docs/ideas/minimal-core/MCE-006-SMALL-STEP-IR.md`
- Modify: `docs/plan/PLAN-INDEX.md`
- Optional Modify: `docs/ideas/README.md`
- Optional Modify: `docs/ideas/IMPLEMENTABILITY-REPORT.md`
- Reference: `docs/plan/tasks/TASK-401-runtime-carrier-inventory-and-semantic-mapping-table.md`
- Evidence source: current interpreter/runtime implementation files

## TDD Steps

### Step 1: Re-read the TASK-401 baseline (Red)

Confirm the remaining control/waiting gaps are still open:

- residual control representation needs a sharper operational story,
- blocked/suspended class is mixed and non-uniform,
- completion-payload / control-authority realization remains partial.

### Step 2: Freeze the control-state story

Document how residual execution state is carried at runtime and how that relates to the hybrid control classification.

### Step 3: Freeze the blocked-state story

Document how blocked/suspended runtime states are represented today and where the representation remains partial or indirect.

### Step 4: Freeze the completion/control story

Document how supervision/control lifecycle is represented and how far completion sealing/retention is currently realized.

### Step 5: Verify GREEN

Expected pass condition:

- a reader can tell how the current runtime carries active, blocked, terminal, and invalid execution states and where completion/control authority is explicit versus still weakly realized.

## Completion Checklist

- [x] TASK-402 task file created
- [x] dedicated operational correspondence section added to `MCE-006-SMALL-STEP-IR.md`
- [x] blocked/suspended vs terminal vs invalid runtime states documented
- [x] completion/control-authority realization documented
- [x] evidence sources cited for each sub-area
- [x] PLAN-INDEX updated

## Completion Notes

Completed 2026-04-05.

- Added a dedicated MCE-006 operational correspondence section covering residual control representation, blocked/suspended realization, runtime state-classification boundaries, and completion/control-authority interpretation.
- Classified the current runtime using four runtime-facing correspondence classes: active residual execution, blocked/suspended waiting, terminal outcome, and invalid/runtime-failure boundary, while making explicit that the current runtime does not materialize all four as distinct concrete result carriers.
- Recorded the conservative realization split: direct for ordinary residual AST execution and control-link lifecycle, distributed/partial for blocking receive and the overall blocked-state story, and weak/missing for retained completion observation/payload carriers.
- Preserved the no-overclaim boundary for `SPEC-004` completion payloads: current runtime evidence clearly supports control-link lifecycle retention, but not a dedicated completion-payload registry on the inspected main path.

## Dependencies for Next Task

This task outputs:

- one explicit runtime control/waiting correspondence story,
- one sharper boundary for completion/control realization,
- one evidence packet for follow-on `Par` and observable-preservation tasks.

Required by:

- TASK-403: `Par` Interleaving, Branch-Local State, and Aggregation Correspondence
- TASK-404: Observable Preservation, Gap Classification, and MCE-007 Handoff

## Notes

Important constraints:

- Do not turn this into a runtime rewrite proposal.
- Do not collapse blocked/suspended and invalid/failure states.
- Do not overclaim completion payload realization if the runtime only has partial carrier support today.
- Keep MCE-007 as the downstream consumer of this evidence.

Edge cases to keep visible:

- blocking receive parked in polling/wait loops,
- explicit yield/proxy suspension continuations,
- reusable vs terminal control-link lifecycle,
- retained completion observation behavior if present only indirectly.
