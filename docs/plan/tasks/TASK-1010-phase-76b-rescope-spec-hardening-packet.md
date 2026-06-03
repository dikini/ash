# TASK-1010: Phase 76B Rescope and Spec-Hardening Packet

## Status: ✅ Complete

## Description

Create the Phase 76B rescope/spec-hardening packet before implementation resumes. This task turns the deferred Phase 76B follow-ups into implementation-grade prerequisites by defining the stable runner-facing introspection APIs needed for executable synthesized tests, generated property inputs, true small-world exploration, and reproducible artifacts.

This is a docs/spec/planning task only. It must not implement runner code.

## Scope

- Harden [DESIGN-022](../../design/DESIGN-022-SYNTHESIZED-CONTRACT-POLICY-OBLIGATION-CASES.md) with a stable runner introspection snapshot and source-specific metadata APIs.
- Harden [DESIGN-023](../../design/DESIGN-023-SMALL-WORLD-EXPLORATION-SUBSTRATE.md) with stable world/domain/enumerator/repro artifact contracts.
- Update [PLAN-024](../PLAN-024-ASH-TEST-RUNNER-V1.md) so Phase 76B implementation tasks depend on this hardening gate.
- Update [PLAN-INDEX](../PLAN-INDEX.md) Phase 76B rows so the planning gate is visible before TASK-513/TASK-514/TASK-515 implementation work.
- Update TASK-513, TASK-514, and TASK-515 dependencies and requirements so they do not proceed from raw-source scans or bounded rerun loops.
- Add a matching CHANGELOG.md entry.

## Non-Goals

- Do not implement `RunnerIntrospectionSnapshot`, `SynthesizedCase`, `TypeGeneratorDescriptor`, `SmallWorldState`, `SmallWorldDomain`, or repro artifact code.
- Do not mark TASK-513, TASK-514, TASK-515, or Phase 76B complete.
- Do not create Phase 132.
- Do not broaden Phase 76A's completed v1 substrate claims.

## Requirements

1. Define a stable runner-facing introspection snapshot that can enumerate contracts, policies, obligations, generator descriptors, small-world models, and reproducible artifact context.
2. Define contract introspection metadata for callable identity, parameter/return types, lowered `requires`/`ensures`, runtime postconditions, generation hints, and executable-case eligibility.
3. Define policy introspection metadata for policy identity, input/domain descriptors, supported terminal outcomes, oracle shape, authority requirements, and bounded materialization rules.
4. Define obligation introspection metadata for obligation identity, scope, lifecycle transitions, discharge/check behavior, terminal expectations, and small-world derivation hints.
5. Define type/contract-derived input-generation descriptors for authored examples, finite domains, valid contract domains, invalid-nearby contract domains, and unsupported-deferred cases.
6. Define a small-world state model with deterministic finite-domain enumeration, stable world identity, state-transition traces, and world-specific oracles.
7. Define reproducible artifact requirements that include seed, case/world index, generated input or world snapshot, source artifact identity, check summary identity, runner schema version, and replay command.
8. Preserve honest reporting: planning-only synthesized cases remain `skip`/deferred, and `pass` is reserved for executed cases whose oracle passed.
9. Run scoped docs verification after editing.

## Implementation Notes

The hardening packet uses existing live code names as grounding points without committing to code changes in this task:

- `crates/ash-typeck/src/type_env.rs::StoredFnContract`
- `crates/ash-cli/src/test_runner/synthesized.rs`
- `crates/ash-cli/src/test_runner/property.rs`
- `crates/ash-cli/src/test_runner/types.rs`

Current runner code still has the expected limitation: synthesized tests are source-scan/planning records, and property/small-world modes boundedly rerun authored bodies. TASK-513 and TASK-514 must replace those limitations only after the stable API contracts in DESIGN-022 and DESIGN-023 are implemented.

## Verification

- Scoped docs gate: `bash scripts/check-docs-gate.sh`
- Drift/search checks:
  - `grep -R "TASK-1010" docs/plan docs/design CHANGELOG.md`
  - `grep -R "RunnerIntrospectionSnapshot\|ReproArtifact\|SmallWorldModel" docs/design/DESIGN-022-SYNTHESIZED-CONTRACT-POLICY-OBLIGATION-CASES.md docs/design/DESIGN-023-SMALL-WORLD-EXPLORATION-SUBSTRATE.md docs/plan/PLAN-024-ASH-TEST-RUNNER-V1.md`

## Completion Checklist

- [x] DESIGN-022 defines stable runner-facing contract/policy/obligation introspection APIs.
- [x] DESIGN-022 defines generated input descriptors and synthesized repro artifacts.
- [x] DESIGN-023 defines stable small-world state/domain/enumerator/repro contracts.
- [x] PLAN-024 records the Phase 76B hardening gate before implementation.
- [x] PLAN-INDEX records TASK-1010 before TASK-513/TASK-514/TASK-515.
- [x] TASK-513, TASK-514, and TASK-515 depend on TASK-1010.
- [x] CHANGELOG.md updated.
- [x] Scoped docs verification run.
