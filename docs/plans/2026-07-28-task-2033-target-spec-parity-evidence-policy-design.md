# TASK-2033 Target-Spec Parity and Evidence Policy Design

## Goal

Use the target Ash specification as the complete feature domain and report implementation, evidence, and parity independently.

## Decision

The policy is anchored in `AGENTS.md` and `docs/spec/CANONICAL-CORE.md`. The machine-readable semantic records and traceability graph carry separate implementation, evidence, and parity fields. Validators reject the retired status vocabulary and inconsistent claims.

## Data model

Each target-spec feature records:

```text
implementation: implemented | partial | not_implemented
evidence: proved | tested | none
parity: matches_spec | below_spec
```

`implemented` requires evidence. `matches_spec` requires `implemented`. Behavior beyond the target
specification is rejected before implementation pending a specification update. A proof records
its theorem and refinement scope; a model proof is not a production-runtime proof without a
checked refinement bridge.

## Migration

Existing records which use finite fixture or layer slices become `partial` and `below_spec` unless their target-spec owner explicitly defines that complete finite feature. Existing tests remain test evidence. Existing model-only Verus proofs remain proof evidence for their models and leave runtime parity below specification.
