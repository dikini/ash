# TASK-1011: Phase 76B Final Remediation and DESIGN-022/023 Completion Planning

## Status: Complete

## Description

Fix the final Phase 76B review blockers in the narrow structured-snapshot runner substrate, reconcile phase documentation with the implemented reality, and create implementation-grade follow-on spec/plan work for completing DESIGN-022 and DESIGN-023 beyond the Phase 76B slice.

Phase 76B remains narrow: raw-source scans are compatibility discovery only and must emit deferred skip rows, ordinary `ash test` CLI source files do not yet produce live checked/lowered `RunnerIntrospectionSnapshot` values, and executable synthesized/generated/small-world rows require explicit finite structured metadata injected through `SuiteConfig` / `run_suite`.

## Specification Reference

- [PLAN-024: Ash Test Runner V1](../PLAN-024-ASH-TEST-RUNNER-V1.md)
- [DESIGN-022: Synthesized Contract / Policy / Obligation Cases](../../design/DESIGN-022-SYNTHESIZED-CONTRACT-POLICY-OBLIGATION-CASES.md)
- [DESIGN-023: Small-World Exploration Substrate](../../design/DESIGN-023-SMALL-WORLD-EXPLORATION-SUBSTRATE.md)
- [TASK-513: Synthesized Tests from Contracts, Policies, and Obligations](TASK-513-synthesized-tests-from-contracts-policies-and-obligations.md)
- [TASK-514: Property and Small-World Execution](TASK-514-property-and-smallworld-execution.md)
- [TASK-515: Ash Test Runner Docs and Phase Verification](TASK-515-ash-test-runner-docs-and-phase-verification.md)

## Requirements

1. Prevent synthesized obligation lifecycle rows from reporting `pass` unless a real finite lifecycle/world oracle is evaluated successfully.
2. Ensure wrong, unsupported, or incomplete lifecycle metadata produces failure or deferred skip outcomes, not unconditional passes.
3. Include explicit lifecycle world snapshots in repro artifacts for evaluated obligation lifecycle rows.
4. Prevent uncapped `BoundedInt` small-world domains from materializing huge ranges when neither CLI `max_worlds` nor metadata `max_worlds_default` supplies a safe cap.
5. Preserve existing truncation behavior where explicit `max_worlds` bounds materialization before huge ranges are collected.
6. Apply synthesized `kind_filter`, `tag_filter`, and `fail_fast` semantics when straightforward; otherwise document any residual behavior as deferred.
7. Update `CHANGELOG.md`, PLAN-024, PLAN-INDEX, and affected task/design docs so Phase 76B claims match the narrow implemented state.
8. Create follow-on spec and plan documents for completing DESIGN-022 and DESIGN-023 after Phase 76B, including task files for future implementation slices.

## Non-Goals

- Do not implement live checked/lowered snapshot production from ordinary CLI files in this task.
- Do not implement broad contract target execution, broad policy oracle execution, or runtime-backed obligation execution beyond the narrow structured metadata oracle fix.
- Do not implement rich small-world domains beyond the current explicit finite slice and bounded-int safety remediation.
- Do not commit; the controller will verify and commit.

## TDD Steps

### Red

- Add failing tests proving wrong obligation lifecycle metadata does not pass.
- Add failing tests proving uncapped huge `BoundedInt` domains are capped or deferred and do not materialize the full range.
- Add focused tests for synthesized filter/fail-fast semantics if the implementation remains small.

### Green

- Implement the minimal lifecycle oracle and bounded-int cap/defer fixes.
- Implement or document synthesized filter/fail-fast behavior.
- Run the targeted tests and record GREEN evidence.

## Dispatch

User instruction for this remediation explicitly disables sub-agent spawning. Work is performed directly in the current worktree while preserving the AGENTS.md requirements for task-file-first workflow, strict TDD, changelog updates, and plan/status hygiene.

## Verification

Minimum commands before handoff:

- `cargo fmt --check`
- `CARGO_BUILD_RUSTC_WRAPPER= cargo test -p ash-cli test_runner -- --nocapture`
- `CARGO_BUILD_RUSTC_WRAPPER= cargo test -p ash-cli --test test_command -- --nocapture`
- `CARGO_BUILD_RUSTC_WRAPPER= cargo clippy -p ash-cli --all-targets -- -D warnings`
- `CARGO_BUILD_RUSTC_WRAPPER= cargo check --workspace`
- `git diff --check`

Optional broad commands if time allows:

- `CARGO_BUILD_RUSTC_WRAPPER= cargo clippy --workspace --all-targets --all-features -- -D warnings`
- `CARGO_BUILD_RUSTC_WRAPPER= cargo test --workspace`

## TDD Evidence

- RED: `CARGO_BUILD_RUSTC_WRAPPER= cargo test -p ash-cli test_runner -- --nocapture` failed before implementation with 47 passed / 5 failed active runner tests. Failing regressions were:
  - `test_runner::synthesized::tests::obligation_lifecycle_oracle_fails_when_world_state_disagrees_with_expectation`
  - `test_runner::synthesized::tests::uncapped_bounded_int_world_enumeration_defers_instead_of_materializing_range`
  - `test_runner::executor::tests::synthesized_snapshot_results_honor_kind_filter`
  - `test_runner::executor::tests::synthesized_snapshot_results_honor_tag_filter`
  - `test_runner::executor::tests::synthesized_snapshot_results_honor_fail_fast`
- GREEN: `CARGO_BUILD_RUSTC_WRAPPER= cargo test -p ash-cli test_runner -- --nocapture` passed after implementation with 52 passed / 0 failed active runner tests.
- REVIEW-FIX GREEN: after independent review found the initial obligation lifecycle fix still generated worlds from expectations, `CARGO_BUILD_RUSTC_WRAPPER= cargo test -p ash-cli test_runner::synthesized -- --nocapture` passed with 23 passed / 0 failed. Added regressions for missing explicit lifecycle worlds, normal snapshot world/expectation disagreement, binding/world-snapshot disagreement, and unsupported-expectation/world alignment.

## Final Verification Evidence

Fresh evidence collected on 2026-06-03:

- `cargo fmt --check`: exited 0 after `cargo fmt` applied formatting.
- `CARGO_BUILD_RUSTC_WRAPPER= cargo test -p ash-cli test_runner -- --nocapture`: exited 0; `56 passed; 0 failed; 0 ignored; 0 measured; 51 filtered out` in `ash_cli` unit tests, with downstream filtered integration targets also reporting 0 active failures.
- `CARGO_BUILD_RUSTC_WRAPPER= cargo test -p ash-cli --test test_command -- --nocapture`: exited 0; `22 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out`.
- `CARGO_BUILD_RUSTC_WRAPPER= cargo clippy -p ash-cli --all-targets -- -D warnings`: first run caught two `collapsible_if` warnings in the new synthesized filter helper; after cleanup, rerun exited 0 with `Finished dev profile`.
- `CARGO_BUILD_RUSTC_WRAPPER= cargo check --workspace`: exited 0 with `Finished dev profile`.
- `git diff --check`: exited 0.
- `CARGO_BUILD_RUSTC_WRAPPER= cargo clippy --workspace --all-targets --all-features -- -D warnings`: exited 0 with `Finished dev profile`.
- `CARGO_BUILD_RUSTC_WRAPPER= cargo test --workspace`: exited 0 across workspace unit, integration, and doctest suites.

## Completion Checklist

- [x] Obligation lifecycle rows only pass after evaluated finite metadata oracle success.
- [x] Wrong lifecycle metadata fails or defers under focused regression coverage.
- [x] BoundedInt worlds require a safe explicit cap or defer before huge materialization.
- [x] Synthesized filter/fail-fast behavior implemented or explicitly deferred.
- [x] PLAN-024 and PLAN-INDEX include TASK-1011 and correct Phase 76B counts.
- [x] TASK-513/TASK-514/TASK-515 reality/evidence updated where behavior or evidence changed.
- [x] DESIGN-022 and DESIGN-023 status/acceptance text reconciled with Phase 76B narrow slice and future full-completion work.
- [x] Follow-on SPEC/PLAN/task files created for completing DESIGN-022 and DESIGN-023.
- [x] CHANGELOG.md updated under `[Unreleased]`.
- [x] Required verification commands run and recorded.
