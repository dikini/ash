# Surface and Observability Reasoner Boundary Review

Date: 2026-03-20
Task: TASK-189

## Scope

Reviewed the workflow-facing and observability-facing documents against
[runtime-reasoner-separation-rules.md](/home/dikini/Projects/ash/docs/reference/runtime-reasoner-separation-rules.md)
with the explicit question: does each feature still make sense without a reasoner present?

Targets:

- [SPEC-002: Surface Language](/home/dikini/Projects/ash/docs/spec/SPEC-002-SURFACE.md)
- [SPEC-021: Runtime Observable Behavior](/home/dikini/Projects/ash/docs/spec/SPEC-021-RUNTIME-OBSERVABLE-BEHAVIOR.md)
- [RUNTIME_REASONER_INTERACTION_MODEL](/home/dikini/Projects/ash/docs/design/RUNTIME_REASONER_INTERACTION_MODEL.md)
- [LANGUAGE-TERMINOLOGY](/home/dikini/Projects/ash/docs/design/LANGUAGE-TERMINOLOGY.md)
- [TASK-186: Monitor Authority and Exposed Workflow View](/home/dikini/Projects/ash/docs/plan/tasks/TASK-186-monitor-authority-and-exposed-workflow-view.md)

## Findings

### 1. Monitor views are correctly runtime-only, but the terminology around them is not yet frozen

Severity: medium

Status: silent gap

`SPEC-002` and `SPEC-021` already keep `exposes { ... }` and monitor views in the runtime-only bucket. `SPEC-002` states that `exposes` is externally monitorable, read-only, and does not imply control or messaging authority
([SPEC-002](/home/dikini/Projects/ash/docs/spec/SPEC-002-SURFACE.md#L204),
[SPEC-002](/home/dikini/Projects/ash/docs/spec/SPEC-002-SURFACE.md#L212)).
`SPEC-021` repeats the same runtime story: monitor views are read-only, gated by `MonitorLink`, and do not confer control authority
([SPEC-021](/home/dikini/Projects/ash/docs/spec/SPEC-021-RUNTIME-OBSERVABLE-BEHAVIOR.md#L138),
[SPEC-021](/home/dikini/Projects/ash/docs/spec/SPEC-021-RUNTIME-OBSERVABLE-BEHAVIOR.md#L146)).

The gap is that `LANGUAGE-TERMINOLOGY` reserves `policy`, `source scheduling modifier`, `scheduler`, `behaviour`, `stream`, `InstanceAddr`, and `ControlLink`, but does not yet reserve `monitorability`, `projection`, or `exposed workflow view` as distinct terms
([LANGUAGE-TERMINOLOGY](/home/dikini/Projects/ash/docs/design/LANGUAGE-TERMINOLOGY.md#L6),
[LANGUAGE-TERMINOLOGY](/home/dikini/Projects/ash/docs/design/LANGUAGE-TERMINOLOGY.md#L43)).

Result: the runtime contract is sound, but later wording can still drift because the terminology guide does not yet lock the boundary between runtime monitorability and reasoner projection.

### 2. `observe` is overloaded as a general access verb and could blur input acquisition with monitor-view access

Severity: low

Status: tension

`SPEC-002` uses `observe` for behaviour input and `exposes` for the monitor view ([SPEC-002](/home/dikini/Projects/ash/docs/spec/SPEC-002-SURFACE.md#L202)).
`SPEC-021` then says holders of `MonitorLink` may `observe` the exposed monitor view
([SPEC-021](/home/dikini/Projects/ash/docs/spec/SPEC-021-RUNTIME-OBSERVABLE-BEHAVIOR.md#L143)).

That is not a semantic conflict, but it is a terminology collision: one verb is now carrying both workflow input acquisition meaning and generic monitor-view access meaning.
The design is still runtime-only, but the wording can mislead later surface-syntax or documentation work into treating monitor access as if it were the same concept as workflow `OBSERVE`.

### 3. The reasoner interaction model stays separate, but it still needs an explicit non-overlap statement for monitors

Severity: low

Status: silent gap

The interaction model defines projection as a runtime-to-reasoner map and treats advisory outputs as separate from committed state
([RUNTIME_REASONER_INTERACTION_MODEL](/home/dikini/Projects/ash/docs/design/RUNTIME_REASONER_INTERACTION_MODEL.md#L97),
[RUNTIME_REASONER_INTERACTION_MODEL](/home/dikini/Projects/ash/docs/design/RUNTIME_REASONER_INTERACTION_MODEL.md#L121)).
It does not, however, explicitly state that monitor views and `exposes { ... }` are not part of that projection machinery.

This matters because the interaction model and the runtime-observable specs both talk about visibility, but they are different kinds of visibility:

- runtime observability and monitoring belong to the runtime contract
- projection belongs to the runtime-to-reasoner contract

The documents are currently aligned by intent, but the non-overlap is only implicit.

## Aligned Areas

- `SPEC-002` keeps `exposes { ... }` read-only and separate from control or messaging authority.
- `SPEC-021` keeps `MonitorLink` visible as a separate runtime role and keeps monitor views policy-gated and non-controlling.
- `TASK-186` correctly classifies monitor authority as runtime-facing behavior rather than reasoner-facing projection.

## Conclusion

No blocking contradiction was found in the runtime-observable or surface docs.
The main issue is terminology hygiene: monitorability is correctly runtime-only, but the vocabulary for projection, observation, and exposed views is not yet frozen tightly enough to prevent later drift.

The next review step should use this report to decide whether the terminology guide needs a small explicit reservation pass or whether the separation can be carried forward unchanged into the synthesis task.
