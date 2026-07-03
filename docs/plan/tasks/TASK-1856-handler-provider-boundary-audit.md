# TASK-1856: Audit handler/provider semantics boundaries

## Description

Audit target specs and implementation seams for handler/provider frame semantics, raise/handle behavior, admission proofs, and shadowing.

## Requirements

- Identify current CPS handler/provider frame types and dispatch behavior.
- Identify current admission-side operation row discharge behavior.
- Record any mismatch between target frame order and implementation order.

## Completion criteria

- [x] Audit evidence names affected source/test/spec files.
- [x] Audit distinguishes existing CPS behavior from admission gaps.
- [x] Audit records the bounded implementation decision.

## Evidence

- Audited `crates/ash-core/src/cps.rs`, `crates/ash-interp/src/cps/mod.rs`, Phase 159 CPS handler/provider tests, `crates/ash-engine/src/row_admission.rs`, SPEC-096b, SPEC-097b, SPEC-098b, SPEC-099b, and SPEC-100. Existing CPS behavior had handler/provider execution but used handler-first lookup; admission lacked explicit frame proof evidence. Phase 184 fixes frame-ordered dispatch and adds admission proof metadata.

## Depends on

- TASK-1855.
