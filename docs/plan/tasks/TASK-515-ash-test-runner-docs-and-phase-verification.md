# TASK-515: Ash Test Runner Docs and Phase Verification

## Status: Complete (Phase 76B closeout)

## Description

Finalize planning/bookkeeping/docs for the Ash test runner phase and run the final verification gate, including targeted `ash test` smoke runs for authored and synthesized test paths.

## Specification Reference

- [PLAN-024: Ash Test Runner V1](../PLAN-024-ASH-TEST-RUNNER-V1.md)
- [DESIGN-021: Ash Test Runner V1](../../design/DESIGN-021-ASH-TEST-RUNNER-V1.md)
- [DESIGN-022: Synthesized Contract / Policy / Obligation Cases](../../design/DESIGN-022-SYNTHESIZED-CONTRACT-POLICY-OBLIGATION-CASES.md)
- [DESIGN-023: Small-World Exploration Substrate](../../design/DESIGN-023-SMALL-WORLD-EXPLORATION-SUBSTRATE.md)
- [TASK-1010: Phase 76B Rescope and Spec-Hardening Packet](TASK-1010-phase-76b-rescope-spec-hardening-packet.md)

## Dependencies

- [TASK-509](TASK-509-ash-test-runner-substrate.md)
- [TASK-510](TASK-510-test-execution-isolation-and-panic-capture.md)
- [TASK-511](TASK-511-ash-test-library-surface.md)
- [TASK-512](TASK-512-authored-test-metadata-and-execution-model.md)
- [TASK-1010](TASK-1010-phase-76b-rescope-spec-hardening-packet.md)
- [TASK-513](TASK-513-synthesized-tests-from-contracts-policies-and-obligations.md)
- [TASK-514](TASK-514-property-and-smallworld-execution.md)

## Requirements

1. Update `PLAN-INDEX.md` and related task bookkeeping.
2. Update `CHANGELOG.md`.
3. Run the final verification gate for the affected workspace.
4. Run targeted `ash test` smoke cases for authored tests.
5. Run targeted synthesized-test smoke cases for contracts/policies/obligations.
6. Report residual limitations explicitly if any remain.
7. Verify TASK-513 and TASK-514 implemented the TASK-1010 introspection, generated-input, small-world, and repro artifact contracts before Phase 76B closeout.

## Likely Files

- Modify: `docs/plan/PLAN-INDEX.md`
- Modify: `CHANGELOG.md`
- Modify: active test-runner design/plan/task docs touched during implementation

## TDD Steps

### Red

- Identify stale docs/bookkeeping and missing verification evidence after the implementation tasks land.

### Green

- Update docs/bookkeeping and run the verification/smoke gate until the recorded closeout state matches the actual repository state.

## Closeout Scope

TASK-513 and TASK-514 are closed for the narrow Phase 76B structured snapshot
substrate. The runner-facing substrate now supports injected `RunnerIntrospectionSnapshot`
metadata, executable synthesized cases for the documented contract/policy/obligation
slices, exact finite generated property cases, deterministic explicit small-world state
enumeration, reproducible artifacts, and `--max-worlds` truncation over actual metadata
worlds.

This closeout does not claim that ordinary CLI source files produce live checked/lowered
`RunnerIntrospectionSnapshot` values. Raw-source synthesized paths remain compatibility
fallbacks that report explicit deferred `skip` rows unless structured snapshots are
injected through runner internals/tests.

## Explicit Deferred Follow-Up Items

Deferred until later spec and metadata integration work:
- live checked/lowered snapshot production from ordinary CLI source files
- executable contract postcondition cases beyond the narrow structured contract `requires`
  boundary slice
- richer policy oracles beyond exact `TerminalEquals` allow/deny metadata
- richer obligation execution beyond exact finite lifecycle metadata
- richer property oracles beyond the narrow metadata-backed `property_holds` shape
- richer small-world domain families such as product, list, role/capability, protocol,
  and state-machine descriptors
- broader synthesized execution beyond the narrow TASK-513/TASK-514 metadata slices

## Baseline Already Satisfied by Phase 76A

- [x] PLAN-INDEX updated for the Phase 76A substrate closeout
- [x] CHANGELOG updated for the Phase 76A substrate closeout
- [x] verification commands run successfully against the bounded v1 implementation
- [x] authored `ash test` smoke cases run successfully against the bounded v1 implementation
- [x] synthesized smoke coverage preserved explicit opt-in planning-level behavior
- [x] residual limitations recorded honestly for the bounded v1 implementation

## Phase 76B Completion Checklist

- [x] TASK-513 executable synthesized contract/policy/obligation cases complete for the
  narrow structured snapshot substrate
- [x] TASK-514 exact finite generated-input and explicit small-world exploration cases
  complete for the narrow structured snapshot substrate
- [x] final verification commands run successfully against the Phase 76B implementation,
  with the global sccache wrapper disabled where required by the sandbox
- [x] targeted authored and synthesized `ash test` smoke cases run against the Phase 76B
  implementation and raw-source deferred fallback
- [x] PLAN-INDEX, PLAN-024, task files, and CHANGELOG reflect the final Phase 76B state
- [x] residual limitations recorded honestly

## Final Verification Evidence

Fresh evidence collected on 2026-06-03:

- `cargo fmt --check`: exited 0.
- `CARGO_BUILD_RUSTC_WRAPPER= cargo test -p ash-cli test_runner -- --nocapture`: exited 0; `47 passed; 0 failed;
  0 ignored; 0 measured; 51 filtered out` in `ash_cli` unit tests, with downstream
  filtered integration targets also reporting 0 active failures.
- `CARGO_BUILD_RUSTC_WRAPPER= cargo test -p ash-cli --test test_command -- --nocapture`: exited 0; `22 passed;
  0 failed; 0 ignored; 0 measured; 0 filtered out`.
- `cargo clippy -p ash-cli --all-targets -- -D warnings`: initial sandbox run failed
  before compilation because the user Cargo config invokes `/home/dikini/.cargo/bin/sccache-30g`
  and sccache returned `Operation not permitted (os error 1)`.
- `CARGO_BUILD_RUSTC_WRAPPER= cargo clippy -p ash-cli --all-targets -- -D warnings`:
  exited 0; `Finished dev profile`.
- `CARGO_BUILD_RUSTC_WRAPPER= cargo check --workspace`: exited 0; `Finished dev profile`.
- `git diff --check`: exited 0 after the closeout documentation updates.

Smoke evidence:

- `target/debug/ash test /tmp/ash-phase76b-task515-smoke/tests/ash/unit/authored_smoke.ash --format json`:
  exited 0; JSON reported `total: 1`, `passed: 1`, `failed: 0`, `skipped: 0`, with
  `authored_smoke` as an authored unit pass.
- `target/debug/ash test /tmp/ash-phase76b-task515-smoke --only-synthesized contracts,policies,obligations --format json`:
  exited 0; JSON reported `total: 9`, `passed: 0`, `failed: 0`, `skipped: 9`, covering
  raw-source fallback contract, policy, and obligation rows with explicit deferred messages
  and repro artifacts. This smoke intentionally does not claim live checked/lowered snapshot
  production from ordinary CLI source files.
