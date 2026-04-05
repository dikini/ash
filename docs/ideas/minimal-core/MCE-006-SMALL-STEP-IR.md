---
status: drafting
created: 2026-03-30
last-revised: 2026-04-05
related-plan-tasks: [TASK-396]
tags: [small-step, ir, execution, alignment, interpreter]
---

# MCE-006: Align Small-Step Semantics with IR Execution

## Status Summary

MCE-006 is no longer blocked on an undefined small-step target.

Phase 61 fixed the upstream semantic backbone in [MCE-005](MCE-005-SMALL-STEP.md):

- workflow-first configurations are canonical;
- ambient context is `(C, P)`;
- dynamic state is expressed in `Γ`, `Ω`, `π`, cumulative trace, cumulative effect summary, and residual workflow terms;
- the main relation is `A ⊢ κ —μ→ κ'`;
- observables are split across configuration state and step labels;
- pure expressions/patterns remain atomic in v1;
- blocked/suspended configurations are distinct from stuckness;
- `Par` uses interleaving plus helper-backed terminal aggregation.

What remains open here is the runtime/interpreter realization of that accepted backbone.

## Problem Statement

Small-step semantics must correspond to executable IR evaluation without collapsing the semantic/runtime boundary.

The problem is now concrete:

- MCE-005 defines the semantic contract;
- MCE-006 must explain how the interpreter/runtime realizes that contract;
- any mismatch must be described as an implementation mapping issue, not as lingering ambiguity in the semantic backbone.

## Scope

In scope:

- mapping the accepted MCE-005 configurations onto interpreter/runtime structures;
- identifying where the current runtime already matches the backbone and where it diverges;
- defining the abstract-machine or equivalent execution view implied by the runtime;
- validating that runtime observables preserve the small-step and `SPEC-004` contracts.

Out of scope:

- revising the semantic backbone fixed by MCE-005;
- inventing new surface syntax or new canonical workflow forms;
- backend/JIT/distributed execution design.

Related but separate:

- [MCE-005](MCE-005-SMALL-STEP.md): accepted semantic backbone
- [MCE-007](MCE-007-FULL-ALIGNMENT.md): full five-layer closeout
- [MCE-008](MCE-008-RUNTIME-CLEANUP.md): adjacent runtime inventory/cleanup work

## Accepted Upstream Contract Consumed from MCE-005

The following are now fixed inputs for this exploration.

### 1. Configuration vocabulary

MCE-006 must map some runtime representation to the semantic carriers:

- ambient authority/policy context `A = (C, P)`;
- environment `Γ`;
- obligation state `Ω`;
- provenance state `π`;
- cumulative trace `T`;
- cumulative effect summary `ε̂`;
- residual canonical workflow or terminal result state.

A runtime may represent these indirectly, compactly, or across multiple data structures, but the mapping must preserve their observable meaning.

### 2. Step granularity

The runtime target is workflow-first, not expression-first.

So MCE-006 should assume:

- small-step transitions are anchored at workflow boundaries;
- pure expressions and patterns stay atomic in v1;
- helper-backed actions such as receive selection, policy checks, provenance joins, and parallel aggregation are semantic boundaries, not accidental implementation details.

### 3. Observable contract

MCE-006 must preserve the accepted split:

- configuration state is authoritative for cumulative obligations, provenance, traces, effects, and terminal outcome class;
- labels carry step-local trace/effect deltas.

An implementation may not silently discard that distinction if doing so would break trace/effect/provenance reconstruction or make blocking behavior unobservable.

### 4. Blocking contract

A runtime realization must distinguish:

- terminal completion;
- progress transitions;
- blocked/suspended waiting states;
- invalid/stuck states that should instead be owned by a declared failure boundary.

This matters especially for `Receive` and for runtime-owned control/completion interactions.

### 5. Concurrency contract

`Par` remains semantically interleaving-based with helper-backed terminal aggregation.

MCE-006 therefore owns the operational question:

- how does the runtime choose the next branch to step,
- and how does it encode/helper-call the combined terminal outcome,

without reinterpreting the semantic contract as simple left-to-right sequential execution.

## Remaining Runtime/Interpreter Alignment Questions

The open questions are now implementation-facing.

### 1. Residual workflow representation

How does the runtime represent the residual workflow component of `κ`?

Possible realizations include:

- direct residual AST values;
- explicit continuation frames;
- a hybrid control stack plus current node pointer.

This question belongs here, not in MCE-005.

### 2. Branch-local state for `Par`

The semantic model allows interleaved branch progress plus helper-backed aggregation. The runtime must decide:

- whether branch state is stored as separate workflow instances, frames, tasks, or scheduler entries;
- where branch-local traces/effects/obligations/provenance accumulate before aggregation;
- how helper-owned concurrent combination is surfaced.

### 3. Blocked-state carrier

The runtime needs an explicit representation for blocked/suspended states, especially for:

- blocking `Receive`;
- mailbox/control waits;
- runtime-owned completion observation boundaries.

This may be a scheduler state, queue registration, parked task record, or equivalent.

### 4. Effect and trace accumulation

How are the cumulative `T` and `ε̂` carriers realized operationally?

Key alignment questions:

- append-only trace log vs incremental event sink;
- per-branch effect summaries vs shared accumulator with rollback-free updates;
- projection of runtime records back into `SPEC-004` terminal outcome fields.

### 5. Provenance and obligation mutation boundaries

The semantic model speaks in terms of state transitions over `Ω` and `π`. MCE-006 must explain:

- where those state carriers live at runtime;
- when updates are committed;
- how helper-owned joins/discharges map to concrete operations.

### 6. ControlLink and completion realization

`SPEC-004` already fixes spawned-child completion semantics. MCE-006 must map that contract to runtime structures such as:

- control handles;
- completion tombstones or sealed payload records;
- supervisor-owned observation channels.

This remains runtime/helper work, not a new user-level syntax design.

## Alignment Strategy

The conservative strategy for MCE-006 is:

1. start from the accepted MCE-005 backbone;
2. inventory the current interpreter/runtime structures that correspond to each semantic carrier;
3. document gaps explicitly as representation or scheduling mismatches;
4. decide whether the current interpreter already realizes the backbone, needs a thin abstract-machine description, or needs contract-preserving refactoring.

This keeps theory and implementation aligned without moving runtime design pressure back upstream into MCE-005.

## Expected Outputs

MCE-006 should eventually produce:

1. a semantic-carrier-to-runtime mapping table;
2. a branch/interleaving realization story for `Par`;
3. a blocked-state realization story for `Receive` and control-owned waits;
4. a terminal observable preservation checklist for traces, effects, obligations, provenance, and rejection/return outcomes;
5. a statement of whether the current interpreter already realizes the accepted small-step backbone or only approximates it.

## Relationship to MCE-007

Once MCE-006 is mature, [MCE-007](MCE-007-FULL-ALIGNMENT.md) can treat the main remaining work as full-stack verification rather than unresolved semantic design.

## Decision Log

| Date | Decision | Rationale |
|------|----------|-----------|
| 2026-03-30 | Exploration created | Need to close theory-practice gap |
| 2026-04-05 | Reframed around the accepted MCE-005 backbone | Phase 61 fixed the semantic target; remaining work is runtime/interpreter alignment rather than upstream semantic ambiguity |
