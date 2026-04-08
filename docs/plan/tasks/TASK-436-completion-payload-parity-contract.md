# TASK-436: Completion-Payload Parity Contract

## Status: ✅ Complete

## Description

Freeze the exact contract for retained completion observation versus full `CompletionPayload` parity so later runtime work stops accreting payload slices without one explicit target. This task should define what exact parity means, which retained slices may remain conservative, which dimensions are required for conformance versus optional enrichment, and how the retained-completion surface relates to the broader semantic execution-record contract.

This remains contract/spec/reference work only.

## Specification Reference

- [SPEC-004: Operational Semantics](../../spec/SPEC-004-SEMANTICS.md)
- [SPEC-021: Runtime Observable Behavior](../../spec/SPEC-021-RUNTIME-OBSERVABLE-BEHAVIOR.md)
- [MCE-004: Big-Step Semantics Alignment](../../ideas/minimal-core/MCE-004-BIG-STEP-ALIGNMENT.md)
- [MCE-007: Full Layer Alignment](../../ideas/minimal-core/MCE-007-FULL-ALIGNMENT.md)
- [TASK-406: Retained Completion-Payload Observation](TASK-406-retained-completion-payload-observation.md)
- [TASK-407: Spawned-Child Execution Substrate and Completion Sealing](TASK-407-spawned-child-execution-substrate-and-completion-sealing.md)
- [TASK-408: Richer Retained Completion Payload Contents](TASK-408-richer-retained-completion-payload-contents.md)
- [TASK-409: Retained Completion Effect-Summary Contents](TASK-409-retained-completion-effect-summary-contents.md)
- [TASK-410: Retained Completion Obligations Contents](TASK-410-retained-completion-obligations-contents.md)
- [TASK-411: Retained Completion Provenance Contents](TASK-411-retained-completion-provenance-contents.md)
- [TASK-412: Dedicated Completion-Wait Carrier](TASK-412-dedicated-completion-wait-carrier.md)
- [TASK-432: Semantic Execution Record and Terminal Projection Contract](TASK-432-semantic-execution-record-and-terminal-projection-contract.md)

## Dependencies

- ✅ [TASK-412: Dedicated Completion-Wait Carrier](TASK-412-dedicated-completion-wait-carrier.md)
- ✅ [TASK-432: Semantic Execution Record and Terminal Projection Contract](TASK-432-semantic-execution-record-and-terminal-projection-contract.md)

## Requirements

### Functional Requirements

1. Define one explicit retained-completion parity contract for the current runtime corpus.
2. Distinguish clearly between:
   - full `CompletionPayload` parity,
   - conservative retained-completion slices,
   - execution-record terminal projection,
   - control tombstone / termination observation.
3. State for each semantic dimension whether retained completion requires:
   - exact transport,
   - conservative summary,
   - terminal-visible subset only,
   - or remains intentionally out of scope for the current stage.
4. Make explicit how retained completion relates to conformance requirements versus optional runtime enrichment.
5. Keep the contract compatible with the current retained-completion surfaces from TASK-406 through TASK-412 without pretending they already provide full parity.
6. Update planning/reporting/reference surfaces and `CHANGELOG.md`.

### Non-Functional Requirements

1. Do not implement runtime changes here.
2. Do not silently merge retained completion with the full execution-record contract; the relationship must be explicit.
3. Keep current-runtime claims conservative and evidence-based.
4. Use repo-relative links throughout.
5. Mark complete only if later retained-completion work can cite one parity contract directly.

## TDD Evidence

### Red

Before this task:
- retained completion has grown through multiple narrow runtime tasks, but there is still no single explicit parity contract defining the target boundary;
- the corpus distinguishes full `CompletionPayload` parity from current retained slices, but not yet as one centralized contract;
- later follow-on work would otherwise keep choosing payload slices ad hoc.

### Green

This task is complete when:
- one explicit retained-completion parity contract exists;
- exact vs conservative vs subset-only retained dimensions are stated clearly;
- later retained-completion work can cite the contract directly instead of reconstructing it from TASK-406 through TASK-412.

## Files

- Create: `docs/reference/retained-completion-parity-contract.md`
- Modify: `docs/reference/semantic-execution-record-contract.md`
- Modify: `docs/plan/PLAN-INDEX.md`
- Optional Modify: `docs/ideas/IMPLEMENTABILITY-REPORT.md`
- Modify: `CHANGELOG.md`

## Completion Checklist

- [x] retained-completion parity contract created
- [x] full parity vs conservative slices distinguished clearly
- [x] relation to execution-record contract stated explicitly
- [x] current runtime surfaces from TASK-406..412 incorporated honestly
- [x] planning/reference surfaces updated
- [x] `CHANGELOG.md` updated

## Completion Notes

TASK-436 is complete as the explicit retained-completion parity contract pass for Phase 67.

The new reference
[docs/reference/retained-completion-parity-contract.md](../../reference/retained-completion-parity-contract.md)
now freezes one centralized boundary between:

- full semantic `CompletionPayload` parity,
- conservative retained-completion slices,
- terminal-visible subset-only retained slices,
- and dimensions that remain outside retained-completion parity itself.

The contract states explicitly that retained completion is downstream terminal observation rather than
a replacement for the broader semantic execution record, and it records the current honest runtime
classification of TASK-406 through TASK-412:

- retained `result` is exact for that one dimension;
- retained `effects` remains conservative;
- retained `obligations` remains terminal-visible subset only;
- retained `provenance` remains conservative;
- waiting for retained completion is an observation surface, not parity by itself.

The execution-record reference now links to this parity contract directly so later work can keep the
broader execution-record contract distinct from retained `CompletionPayload` parity.

This task remains contract/spec/reference work only. It does not implement a new runtime fidelity
slice; that is the follow-on role of TASK-437.

## Dependencies for Next Task

This task outputs:
- the retained-completion parity contract for later runtime follow-on work.

Required by:
- TASK-437: Retained-Completion Parity Follow-On
- TASK-439: Differential Conformance Harness (Rust First)

## Notes

Important constraints:
- Do not let retained completion become an accidental substitute for the full semantic execution record.
- Keep control tombstones and child completion payloads conceptually distinct.
- Prefer explicit fidelity labels over implied parity.
