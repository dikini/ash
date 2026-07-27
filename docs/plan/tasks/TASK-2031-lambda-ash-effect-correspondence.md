# TASK-2031: λAsh-Effect Correspondence

**Status:** In progress
**Phase:** [PLAN-203](../PLAN-203-RUNNABLE-ASH-SEMANTIC-REALIZATION.md)
**Depends on:** TASK-2030, TASK-2013, TASK-2014, and the target operational semantics

**Semantic task record:** [TASK-2031](../semantic-task-records.json)

**Semantic coverage map:** [TASK-2031 λAsh-Effect correspondence record](../SEMANTIC-RULE-COVERAGE.md#task-2031-λash-effect-correspondence-record)

## Description

Define `λAsh-Effect` as the complete conservative extension of `λAsh-CPS₀` for the target
effectful CPS constructs. It is a mathematical semantics of the existing CPS layer and a
refinement target for the Rust Engine executor; it is neither a new production IR nor a second
execution route.

## Handoffs

**Declared domain**: general for the target effectful CPS subset defined here. Parser, Core,
  admission, and Engine execution layers remain explicitly unowned and non-authorizing.
- **Run-route impact:** `prerequisite`. The task defines the effectful CPS/operational/Rust
  correspondence required before a generic effectful Engine expansion; it does not itself admit
  or execute a source program.
- **Consumes:** `CORE-CPS-SYNTAX-001`, `SEM-TARGET-CORE-CPS-001`, checked handler facts from
  TASK-2013, and Path-B admission/frame authority from TASK-2014.
- **Produces:** a complete λAsh-Effect grammar, configuration state, well-formedness and typing
  judgments, frame lookup and provider-boundary relations, small-step and terminal rules,
  canonical examples, and an explicit correspondence table for CPS artifacts, operational state,
  Engine state view, and terminal projection.
- **Downstream owner:** the PLAN-203 shared-execution-seam and feature-realization tasks consume
  this correspondence when they generalize effectful admission and execution.
- **Does not own:** parser acceptance, surface-to-Core lowering, provider implementations,
  daemon transport, generic frame installation, or an Engine executor implementation.
- **Integration/proof responsibility:** a later PLAN-203 integration task demonstrates an active
  admitted route through the shared Engine and both clients. Selected Verus pilots may refine
  named rules, but unproved obligations remain deferred in traceability and do not block this
  semantic definition or its later executable realization.

## Semantic workflow record

The active record binds this task's target-effect subset to `CORE-CPS-SYNTAX-001`,
`SEM-TARGET-CORE-CPS-001`, `OBS-TARGET-PROJECTION-001`, and the TASK-2031 `SEM-EFFECT-*` rules in
[semantic-task-records.json](../semantic-task-records.json). Its positive, negative, and mutation
evidence is documentation-validator evidence; **parity is not applicable** because this is a
prerequisite handoff with no active Engine/CLI/daemon route. TASK-2032 owns any later route and
client-parity proof.

## Task-owned evidence plan

- **Positive:** `TEST-DOCS-TASK-2031-EFFECT-CORRESPONDENCE-CONTRACT` validates the complete,
  rule-indexed contract.
- **Negative:** `TEST-DOCS-TASK-2031-EFFECT-CORRESPONDENCE-INCOMPLETE-REJECTION` rejects a
  missing per-rule mapping field.
- **Mutation:** `TEST-DOCS-TASK-2031-EFFECT-CORRESPONDENCE-MISMAPPING-REJECTION` rejects a stable
  rule-index retarget.
- **Parity:** not applicable: no active generic route or CLI/daemon parity claim is made here.

The selected authorization, affine-use, and terminal-projection Verus entries are
deferred/unproved traceability dispositions. The separate innermost-lookup entry is a scoped
TASK-2031 correspondence bridge, explicitly distinct from the limited existing
`PROOF-CPS-FRAME-LOOKUP-001` model proof; it is also deferred/unproved and does not reclassify that
proof. None of these entries is a proof.

## Requirements

- State the complete syntax and configuration model for `Raise`, `Handle`, discharge recording,
  ordered handler/provider frames, continuation consumption, residual closed rows, and external
  outcomes in scope for the target effect semantics.
- Define the required judgments and transitions, including innermost-first lookup, missing
  discharge, deep affine resumption, handler-body traps, provider invocation, timeout, and
  cancellation terminalization.
- Make the correspondence to SPEC-099b and the checked CPS/admission artifact explicit. Rows are
  requirements only; only separately authorized admission instructions install frames.
- Distinguish target semantic rules from implementation-specific Engine storage or scheduling.
- Add canonical examples and rule-indexed conformance obligations for normal return, missing
  admission, malformed/unchecked CPS, handler-body trap, timeout, and cancellation.
- Record only selected, high-value Verus pilot candidates. A pilot becomes proved only after a
  verified artifact is linked from semantic traceability.

## TDD and activation steps

1. Before changing the calculus, add this task to the active semantic-task record scope with its
   rule IDs, coverage section, traceability edges, and positive, negative, mutation, and parity
   evidence plan.
2. Add failing documentation/validation fixtures for the new grammar, rule identifiers, and
   correspondence completeness before editing the canonical artifact.
3. Define the calculus and examples; then add focused conformance tests only for the separately
   admitted execution slices that consume the resulting handoff.
4. Run the calculus, traceability, orientation, documentation, and affected Rust test gates.

## Completion checklist

- [ ] λAsh-Effect is complete for the target effectful CPS subset and retains λAsh-CPS₀ as a
      conservative control fragment.
- [ ] Every rule has a stable identity and an explicit CPS, operational, Engine-view, and terminal
      correspondence.
- [ ] Handler/provider authority, deep affine resumption, timeout, and cancellation are specified
      without adding an alternative execution path.
- [ ] The semantic workflow record, coverage map, traceability graph, canonical indexes, and
      evidence plan are active and consistent.
- [ ] Selected Verus candidates and deferred obligations are visibly distinguished from verified
      proofs.
