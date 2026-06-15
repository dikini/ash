# SPEC-083: Law Coverage and Mutation Testing

**Status:** Planned
**Date:** 2026-06-15
**Builds on:** [SPEC-081](SPEC-081-LAW-TEST-EVIDENCE-SUBSTRATE.md)
**Plan:** [PLAN-147: Law Coverage and Mutation Testing](../plan/PLAN-147-LAW-COVERAGE-AND-MUTATION-TESTING.md)

## Summary

Add `ash test` reporting for law/test coverage and bounded mutation testing so weak or missing law evidence is visible from the Ash CLI.

## Motivation

Phase 145 made empirical law evidence explicit but intentionally left several important `ash test` gaps visible. This specification defines the next scoped slice for those gaps while preserving the project rule that Ash law/test/proof authors and executors validate supported behavior through an Ash executable, not through Cargo or Rust test harnesses.

## Evidence Boundary

Implementation agents must distinguish three command classes:

1. **Implementation health** — Rust commands such as `cargo test`, `cargo clippy`, and `cargo fmt`; useful and required for implementers.
2. **Candidate Ash final surface** — direct invocations of an Ash-under-test executable:

   ```bash
   ${ASH_UNDER_TEST:?set Ash candidate binary} test fixtures/<phase-fixture> --format json
   ```

3. **Release/install parity** — ordinary `ash` on PATH after install/release catches up; closeout must either prove parity or record an explicit handoff.

`cargo run -p ash-cli -- test ...` is never final-surface evidence.

## Scope

### In Scope

- Opt-in law/test coverage reporting.
- Coverage JSON/human output and uncovered-law reporting.
- Bounded pure-expression mutation operators.
- Mutation execution/reporting with killed/survived/deferred statuses.
- No-Cargo coverage and mutation fixtures.

### Non-Goals

- shrinker implementation beyond consuming Phase 146 artifacts
- distributed mutation execution
- symbolic proofs
- automatic unrestricted generator synthesis

## Required Agent Skills

Implementation agents must load and follow:

- `rust-skills` for Rust code, public APIs, proptest coverage, error handling, and clippy-clean implementation.
- `ash-language-feature-spec-writing` for Ash surface/parser/typechecker/runner contracts and final-surface Ash examples.
- `test-driven-development` when implementing code slices: write failing Rust/Ash-facing tests before production changes.
- `verification-before-completion` before marking any task complete.
- `systematic-debugging` for any unexpected runner, parser, or property failure.

## Examples

```bash
$ASH_UNDER_TEST test fixtures/phase147-coverage --coverage --format json
$ASH_UNDER_TEST test fixtures/phase147-mutation --mutation --mutation-limit 20 --format json
```

Coverage should report uncovered law/proof declarations. Mutation output should distinguish killed, survived, equivalent/deferred, and errored mutants.

## Result and Reporting Requirements

- JSON output must remain machine-readable and stable enough for later orchestration.
- Unsupported cases must be `deferred`, `untested`, or explicit errors; they must not be counted as passing evidence.
- Repro artifacts must include enough data for a direct `$ASH_UNDER_TEST test ...` replay when the phase owns execution behavior.
- Human output should summarize the new capability without hiding caveats.

## Implementation Tasks

- [TASK-1466](../plan/tasks/TASK-1466-coverage-mutation-audit.md): Audit coverage and mutation seams
- [TASK-1467](../plan/tasks/TASK-1467-law-test-coverage-schema.md): Define law/test coverage schema
- [TASK-1468](../plan/tasks/TASK-1468-coverage-cli-json-output.md): Expose coverage in CLI/JSON output
- [TASK-1469](../plan/tasks/TASK-1469-coverage-final-surface-fixtures.md): Add coverage final-surface fixtures
- [TASK-1470](../plan/tasks/TASK-1470-mutation-operator-catalog.md): Define bounded mutation operator catalog
- [TASK-1471](../plan/tasks/TASK-1471-mutation-execution-loop.md): Implement mutation execution loop
- [TASK-1472](../plan/tasks/TASK-1472-mutation-reporting-fixtures.md): Add mutation reporting fixtures
- [TASK-1473](../plan/tasks/TASK-1473-coverage-mutation-closeout.md): Close out coverage/mutation phase

## Changelog

### 2026-06-15

- Created this planning specification and registered PLAN-147 / TASK-1466 through TASK-1473.
