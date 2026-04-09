# PLAN-016: Capability Call Dispatch Split and Operational Call Sugar

## Status: 📝 Planned

## Overview

Implement DESIGN-016 by splitting operational execution into explicit `provider` and `action`
components, then adding act-less workflow sugar for operational capability calls. This phase keeps
symbolic capability names as a resolver concern while making runtime dispatch use one canonical
`lookup(provider) -> execute(action_name, args)` mechanism.

## Design Reference

- [DESIGN-016: Capability Call Dispatch Split and Operational Call Sugar](../design/DESIGN-016-CAPABILITY-CALL-DISPATCH.md)

## Goals

1. Split `Workflow::Act` into explicit provider and action fields.
2. Add surface support for `capability(args)` / `provider:action(args)` workflow sugar with
   optional `when guard`.
3. Make symbolic capability names resolve to `(provider, action)` targets.
4. Refactor runtime/provider execution so provider lookup and action dispatch are distinct steps.
5. Update the relevant specs together, including `SPEC-025`.
6. Ensure the final canonical semantic contract lives in `docs/spec/`, not in design or plan docs.

## Scope

**In Scope**:
- spec updates across `SPEC-001`, `SPEC-002`, `SPEC-003`, `SPEC-004`, `SPEC-010`, `SPEC-017`,
  and `SPEC-025`
- parser and surface-AST support for explicit and symbolic operational call sugar
- lowering and core AST migration to split provider/action ACT targets
- resolver/typechecker support for symbolic capability target pairs
- provider trait update to provider-local `execute(action_name, args)` dispatch
- interpreter and engine migration to the split runtime dispatch model
- docs/examples and final verification

**Out of Scope**:
- redesigning `observe`, `set`, or `send`
- dynamic provider discovery
- broad module-system redesign beyond what is required for symbolic capability target resolution

## Canonical Document Rule

This phase is not complete until the final normative contract lives in `docs/spec/`.

- `docs/design/` records rationale and migration intent.
- `docs/plan/` and `docs/plans/` record sequencing, task decomposition, and implementation
  history.
- `docs/spec/` must be the lasting source of truth for language/runtime behavior after the phase
  closes.

In particular, no completion claim for PLAN-016 is valid if the implemented behavior only matches
`DESIGN-016` / `PLAN-016` while the active specs still describe the old overloaded ACT model.

## Implementation Guardrails

The following constraints are mandatory for every implementation task in this phase:

1. Do not reintroduce a flat one-name ACT contract anywhere in parser, lowering, interpreter,
   engine, or provider APIs.
2. Do not treat `provider:action(...)` as cosmetic syntax over one overloaded runtime string.
3. Do not treat symbolic capability calls as direct provider names. They must resolve to
   `(provider, action)` pairs through resolver-owned metadata.
4. Do not make the provider trait take its own provider name as an argument. Provider lookup must
   happen before trait dispatch.
5. Do not mark docs/design work complete unless the normative `docs/spec/` files have been updated
   accordingly.

## Phases

### Phase 1: Spec Contract Freeze

**Goal**: Freeze one explicit cross-spec contract before implementation work starts.

**Tasks**:
- [TASK-463](tasks/TASK-463-spec-capability-call-dispatch-contract.md): Update the normative spec
  set for split provider/action ACT dispatch and operational call sugar

**Deliverable**: The active spec set states one canonical provider/action operational call model.

### Phase 2: Surface and Core Shape

**Goal**: Add surface syntax support and make the canonical ACT shape explicit in parser/lowering.

**Tasks**:
- [TASK-464](tasks/TASK-464-surface-operational-call-sugar.md): Add parser and surface-AST support
  for act-less operational call sugar and explicit `provider:action(...)`
- [TASK-465](tasks/TASK-465-core-act-provider-action-shape.md): Split core `Workflow::Act` and
  lowering into provider/action fields

**Deliverable**: Surface and core AST layers carry one explicit ACT target shape.

### Phase 3: Resolver and Runtime Dispatch

**Goal**: Resolve symbolic capability names into target pairs and align runtime dispatch.

**Tasks**:
- [TASK-466](tasks/TASK-466-resolver-capability-target-pairs.md): Teach resolution/typechecking to
  represent symbolic operational capability targets as `(provider, action)` pairs
- [TASK-467](tasks/TASK-467-provider-local-execute-dispatch.md): Refactor interpreter/runtime and
  provider trait boundaries to use provider-local action dispatch

**Deliverable**: Symbolic operational calls resolve to provider/action pairs, and runtime dispatch
  executes them via explicit provider lookup then provider-local action dispatch.

### Phase 4: Engine Migration and Closeout

**Goal**: Migrate engine providers and close out docs/examples/verification.

**Tasks**:
- [TASK-468](tasks/TASK-468-engine-provider-split-dispatch.md): Migrate engine providers and engine
  wiring to the split dispatch contract
- [TASK-469](tasks/TASK-469-capability-call-docs-and-examples.md): Update docs/examples/tutorials
  to the new operational call model
- [TASK-470](tasks/TASK-470-capability-call-dispatch-verification.md): Run final integration and
  quality-gate verification

**Deliverable**: Engine/runtime/docs are aligned with the split provider/action contract and the
new sugar forms, with `docs/spec/` holding the canonical contract.

## Critical Path

```text
TASK-463
  -> TASK-464 / TASK-465
  -> TASK-466
  -> TASK-467
  -> TASK-468
  -> TASK-469 / TASK-470
```

Parallel paths:
- `TASK-464` and `TASK-465` can overlap once the spec contract is frozen.
- `TASK-469` can begin once the surface/core/runtime contract is stable.

## Risks

### Risk 1: Surface Syntax Lands Before Resolver Contract

**Probability**: Medium  
**Impact**: High  
**Mitigation**: make `TASK-463` spec-first and keep symbolic target resolution explicit in
`TASK-466`.

### Risk 2: Provider Trait Migration Leaves Runtime Overload Intact

**Probability**: Medium  
**Impact**: High  
**Mitigation**: require `TASK-467` to remove the remaining one-name lookup/dispatch path rather
than just wrapping it.

### Risk 3: Small-step Spec Drifts from Big-step/Runtime

**Probability**: Low  
**Impact**: Medium  
**Mitigation**: include `SPEC-025` in the first task rather than as a follow-up note.

## Success Criteria

1. `Workflow::Act` carries separate provider and action identifiers.
2. Runtime execution performs provider lookup before provider-local action dispatch.
3. The provider trait no longer depends on overloaded `Action.name`.
4. The surface language supports both symbolic capability calls and explicit
   `provider:action(...)` operational calls.
5. The relevant specs and docs agree on the same dispatch contract.
6. `docs/spec/` is the final semantic authority; design/plan docs are no longer required to
   understand the normative behavior.

## Timeline

| Phase | Duration | Start Date | End Date |
|-------|----------|------------|----------|
| Phase 1 | 0.5 day | TBD | TBD |
| Phase 2 | 1 day | TBD | TBD |
| Phase 3 | 1 day | TBD | TBD |
| Phase 4 | 1 day | TBD | TBD |
| **Total** | **3.5 days** | TBD | TBD |

## Next Steps

1. Approve DESIGN-016 and PLAN-016.
2. Execute [TASK-463](tasks/TASK-463-spec-capability-call-dispatch-contract.md) first to freeze
   the contract.
3. Land surface/core changes before trait/runtime migration so the split is visible end-to-end.
4. Finish with engine migration, docs, and verification.
5. Do not close the phase until the canonical documents in `docs/spec/` reflect the landed
   behavior.

---

*Document Version: 1.0*  
*Status: Planned*  
*Author: codex*  
*Date: 2026-04-09*
