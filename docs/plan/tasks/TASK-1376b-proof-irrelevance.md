# TASK-1376b: Proof Irrelevance

## Status: ✅ Complete

## Description

Implement Stage 3 local/static proof irrelevance: all checked proofs of the same proposition are definitionally equal after typechecker-owned proof erasure.

## Requirements

1. Add proof irrelevance rule to typechecker
2. `proof1 : P` and `proof2 : P` are equal
3. Proofs can be erased at runtime

## Acceptance Criteria

- [x] Proof irrelevance represented for the Stage 3 local/static typechecker slice via `TypeEnv::erase_proof_for_proposition` and `TypeEnv::proofs_definitionally_equal_for_proposition`
- [x] Runtime/codegen proof erasure remains explicitly out of scope for TASK-1376b and deferred to TASK-1376c/later runtime work; this task provides only the typechecker-owned erased proof carrier
- [x] Inconclusive proof totality is not erased
- [x] Test passes

## Implementation Notes

- Added an `ErasedProof` carrier owned by `ash-typeck` that retains the proved `TypeProposition` while discarding proof declaration name, proof body, and witness identity.
- `TypeEnv::erase_proof_for_proposition` reuses the default Stage 3 proof totality checker before producing the erased carrier; `TypeEnv::erase_proof_for_proposition_with_fuel` exposes the same rule with explicit fuel for deterministic checking of inconclusive cases.
- `TypeEnv::proofs_definitionally_equal_for_proposition` erases both proofs and compares the retained proposition boundary, so different propositions do not collapse.
- This task does not implement full theorem proving, proof-term typing, code generation erasure, or TASK-1376c runtime escape prevention.

## Verification

- `CARGO_BUILD_RUSTC_WRAPPER= RUSTC_WRAPPER= cargo fmt --check`
- `CARGO_BUILD_RUSTC_WRAPPER= RUSTC_WRAPPER= cargo test -p ash-typeck --test task_1376b_proof_irrelevance -- --nocapture`
- `CARGO_BUILD_RUSTC_WRAPPER= RUSTC_WRAPPER= cargo check --workspace`

## Related

- [TASK-1376](TASK-1376-stage3-prop-kind.md)
