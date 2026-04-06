# TASK-419: Effect Inference and Runtime Verification Alignment

## Status: ✅ Complete

## Description

Implement the first code follow-on after TASK-414 by aligning the current effect inference and
runtime-verification paths with the promoted coarse effect-typing contract.

This task is not a redesign of Ash's effect system. It is an alignment task that ensures the code
matches the now-promoted contract:
- effect classification comes from workflow forms and source-level contracts
- provider effect metadata is compatibility/validation metadata, not the primary source of effect typing
- composition remains join-based over the current coarse lattice

## Specification Reference

- [TASK-414: Effect Typing Contract Promotion and Vocabulary Cleanup](TASK-414-effect-typing-contract-promotion.md)
- [SPEC-001: Intermediate Representation](../../spec/SPEC-001-IR.md)
- [SPEC-003: Type System](../../spec/SPEC-003-TYPE-SYSTEM.md)
- [SPEC-004: Operational Semantics](../../spec/SPEC-004-SEMANTICS.md)
- [SPEC-010: Embedding](../../spec/SPEC-010-EMBEDDING.md)
- [SPEC-017: Capability Integration](../../spec/SPEC-017-CAPABILITY-INTEGRATION.md)
- [Type-to-Runtime Contract](../../reference/type-to-runtime-contract.md)

## Dependencies

- ✅ TASK-414 complete

## Requirements

### Functional Requirements

1. Audit and align workflow-form effect inference with the promoted coarse classification contract.
2. Ensure runtime verification consumes type-derived workflow effect classification rather than treating provider metadata as source-of-truth effect typing.
3. Preserve join-based composition across the current coarse effect lattice.
4. Add/update tests covering:
   - workflow-form effect classification
   - join-based composition
   - provider metadata compatibility checks
   - source-level classification winning over provider-metadata overreach

### Non-Functional Requirements

1. Do not add `Pure` as a new normative lattice element in this task.
2. Do not redesign the effect system into rows/associated effects/effect polymorphism.
3. Prefer targeted alignment over broad rewrites.
4. Update `CHANGELOG.md`.

## Files

- Modify: `crates/ash-typeck/src/effect.rs`
- Modify: `crates/ash-typeck/src/runtime_verification.rs`
- Modify: `crates/ash-typeck/src/requirements.rs`
- Modify any closely related tests under `crates/ash-typeck/tests/`
- Modify: `CHANGELOG.md`

## TDD Steps

### Step 1: Write failing alignment tests

Add tests that codify the promoted effect-typing contract and expose any mismatch between source-form inference and provider metadata.

### Step 2: Implement effect/runtime-verification alignment

Make the minimal changes needed so source-form classification drives typing/verification and provider metadata remains secondary compatibility metadata.

### Step 3: Verify affected crate quality

Run at least:
- `cargo test -p ash-typeck`
- `cargo clippy -p ash-typeck --all-targets -- -D warnings`
- `cargo fmt --check`

## Completion Checklist

- [x] effect inference aligned with promoted workflow-form contract
- [x] runtime verification alignment complete
- [x] tests added/updated
- [x] `CHANGELOG.md` updated

## Notes

This task should leave the door open for a later `Pure` follow-on, but should not silently add it.

Completion note:
- workflow-form inference now aligns the promoted coarse contract by keeping governance/control forms
  (`oblige`, `ret`) epistemic and by letting `for` inherit its body effect instead of adding an
  extra operational grade;
- runtime verification now checks explicitly type-derived workflow effects, while provider metadata
  remains compatibility/validation metadata rather than the source of effect typing;
- join-based composition over the existing coarse lattice remains unchanged, leaving `Pure` and any
  richer effect-system redesign to later follow-on work.
