# PLAN-147: Law Coverage and Mutation Testing

**Status:** ✅ Complete
**Spec:** [SPEC-083: Law Coverage and Mutation Testing](../spec/SPEC-083-LAW-COVERAGE-AND-MUTATION-TESTING.md)
**Depends on:** [PLAN-145: Law Test Evidence Substrate](PLAN-145-LAW-TEST-EVIDENCE-SUBSTRATE.md)
**Task range:** TASK-1466 through TASK-1473
**Estimated effort:** 32h

## Overview

Add `ash test` reporting for law/test coverage and bounded mutation testing so weak or missing law evidence is visible from the Ash CLI.

## Goals

- [x] Make law/test coverage visible.
- [x] Add bounded mutation testing for runner-visible law propositions.
- [x] Report mutation survival honestly with no-Cargo fixtures.

## Non-Goals

- shrinker implementation beyond consuming Phase 146 artifacts
- distributed mutation execution
- symbolic proofs
- automatic unrestricted generator synthesis

## Orchestrator Guidance

- Create a dedicated worktree before implementation, for example `.worktrees/phase-147-law-coverage-and-mutation-testing`.
- Load `rust-skills`, `ash-language-feature-spec-writing`, `test-driven-development`, `systematic-debugging`, and `verification-before-completion` before code work.
- Use rust-analyzer MCP/LSP for Rust symbol tracing before broad text search.
- Keep tasks small and sequential where schema/result formats are dependencies.
- Require direct `$ASH_UNDER_TEST test ...` evidence for user-facing runner behavior; Rust tests alone are bridge evidence.
- Update `CHANGELOG.md` and relevant `reference/tools/test.md` wording in the closeout task.

## Task Plan

| Task | Title | Estimate | Status |
|---|---|---:|---|
| [TASK-1466](tasks/TASK-1466-coverage-mutation-audit.md) | Audit coverage and mutation seams | 4h | ✅ Complete |
| [TASK-1467](tasks/TASK-1467-law-test-coverage-schema.md) | Define law/test coverage schema | 4h | ✅ Complete |
| [TASK-1468](tasks/TASK-1468-coverage-cli-json-output.md) | Expose coverage in CLI/JSON output | 4h | ✅ Complete |
| [TASK-1469](tasks/TASK-1469-coverage-final-surface-fixtures.md) | Add coverage final-surface fixtures | 4h | ✅ Complete |
| [TASK-1470](tasks/TASK-1470-mutation-operator-catalog.md) | Define bounded mutation operator catalog | 4h | ✅ Complete |
| [TASK-1471](tasks/TASK-1471-mutation-execution-loop.md) | Implement mutation execution loop | 4h | ✅ Complete |
| [TASK-1472](tasks/TASK-1472-mutation-reporting-fixtures.md) | Add mutation reporting fixtures | 4h | ✅ Complete |
| [TASK-1473](tasks/TASK-1473-coverage-mutation-closeout.md) | Close out coverage/mutation phase | 4h | ✅ Complete |

## Decision Gates

- D1: coverage schema lands before CLI coverage output.
- D2: mutation operators remain bounded and pure until mutation execution loop is verified.
- D3: mutation survival must not be reported as pass/fail test outcome without a distinct mutation status.

## Verification Strategy

Each implementation task must include:

1. Focused Rust tests for new parser/runner/schema behavior.
2. Focused Ash fixture tests where the behavior is user-facing.
3. Direct Ash-under-test command evidence:

   ```bash
   export ASH_UNDER_TEST=/absolute/path/to/candidate/ash
   "$ASH_UNDER_TEST" test fixtures/phase147-... --format json
   ```

4. `cargo fmt --check`, focused `cargo test`, and focused `cargo clippy` for touched crates.

The closeout task owns broad gates and documentation drift checks.
