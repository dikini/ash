# TASK-513: Synthesized Tests from Contracts, Policies, and Obligations

## Status: Complete (Phase 76B)

## Description

Add explicit, opt-in synthesized test planning and execution for contracts, policies, and obligations, preserving clear labeling and separation from authored tests.

## Specification Reference

- [PLAN-024: Ash Test Runner V1](../PLAN-024-ASH-TEST-RUNNER-V1.md)
- [DESIGN-021: Ash Test Runner V1](../../design/DESIGN-021-ASH-TEST-RUNNER-V1.md)
- [DESIGN-022: Synthesized Contract / Policy / Obligation Cases](../../design/DESIGN-022-SYNTHESIZED-CONTRACT-POLICY-OBLIGATION-CASES.md)
- [TASK-1010: Phase 76B Rescope and Spec-Hardening Packet](TASK-1010-phase-76b-rescope-spec-hardening-packet.md)

## Dependencies

- [TASK-510](TASK-510-test-execution-isolation-and-panic-capture.md)
- [TASK-511](TASK-511-ash-test-library-surface.md)
- [TASK-1010](TASK-1010-phase-76b-rescope-spec-hardening-packet.md)

## Requirements

1. Consume the TASK-1010 `RunnerIntrospectionSnapshot` contract instead of raw-source pattern scans for executable synthesized cases.
2. Support opt-in executable synthesized tests from function/workflow contracts when contract metadata exposes parameter/return types, lowered `requires`/`ensures`, runtime postconditions, and bounded generation hints.
3. Support opt-in executable synthesized tests from policies when policy metadata exposes bounded input domains, terminal outcome support, authority requirements, and oracle shape.
4. Support opt-in executable synthesized tests from obligations when obligation metadata exposes lifecycle transitions, introduction/discharge/check sites, and terminal expectations.
5. Ensure synthesized tests are labeled as synthesized in output and JSON results.
6. Ensure synthesized tests are excluded from default authored-test discovery.
7. Preserve the rule that synthesized tests complement, not replace, authored tests.
8. Emit TASK-1010-compatible `ReproArtifact` data for each executed synthesized case.
9. Preserve honest reporting: unsupported/planning-only cases must remain `skip`/deferred, and `pass` requires an executed oracle.

## Likely Files

- Modify: runner synthesis/planning/execution code
- Modify: CLI option parsing/output classification for synthesized tests
- Add tests covering synthesized contract/policy/obligation execution paths

## TDD Steps

### Red

- Add failing runner tests showing synthesized tests are either missing or incorrectly mixed into authored discovery/output.

### Green

- Implement explicit, opt-in synthesized planning/execution and labeled reporting for contracts, policies, and obligations.

## Implementation Reality Check

The runner now has a TASK-1010-shaped synthesized-case substrate at the CLI-runner layer:
`RunnerIntrospectionSnapshot`, source-specific metadata records, exact bounded type-generator
descriptors, executable `SynthesizedCase` records, and `ReproArtifact` output on generated and
executed synthesized rows. `SuiteConfig::synthesized_snapshots` is the current runner-facing
integration seam: tests and future checked-summary producers can pass structured snapshots directly
to `run_suite`, and that path executes before raw-source compatibility scans. The user-facing CLI
flags are wired for opt-in source selection, but the CLI command does not yet produce live checked
snapshots from source files.

The executable slice is intentionally narrow. Structured contract metadata can execute simple
integer `requires` boundary cases such as `x > 0` only when metadata explicitly declares
`PreconditionBoundary` and provides exact valid and invalid representatives via contract or snapshot
generator descriptors. Structured policy metadata can execute `TerminalEquals` allow/deny cases when
lowered policy identity, exact finite input-domain values, and supported `Allow`/`Deny` terminals are
present. Structured obligation metadata can execute finite lifecycle expectations for
introduced/discharged/missing-discharge/double-discharge when lifecycle model plus
introduction/discharge/check sites are present. TASK-1011 tightened this slice so obligation
lifecycle pass rows require evaluated explicit finite lifecycle world state; wrong lifecycle state
fails and missing/unsupported lifecycle metadata defers. Raw-source contract/policy/obligation
pattern recognition remains a compatibility fallback and reports explicit `skip`/`deferred` rows
with repro context instead of successful execution.

## Explicit Deferred Follow-Up Items

Deferred until later implementation tasks or metadata integration:
- wiring live lowered typechecker/runtime snapshots into the CLI command instead of raw-source fallback scans
- executable contract postcondition cases that call real runtime targets and check lowered `ensures`
- broader policy execution beyond exact `TerminalEquals` allow/deny metadata
- broader obligation execution beyond explicit finite lifecycle world metadata and future runtime-backed lifecycle execution

## Completion Checklist

- [x] narrow contract-derived synthesized `requires` boundary cases implemented and verified over structured metadata
- [x] runner-facing structured snapshot seam wired through `SuiteConfig`/`run_suite`
- [x] narrow policy-derived `TerminalEquals` allow/deny cases implemented over exact metadata; unavailable metadata defers with repro context
- [x] narrow obligation lifecycle cases implemented over exact finite lifecycle world metadata; wrong lifecycle state fails and unavailable metadata defers with repro context
- [x] explicit CLI opt-in implemented
- [x] output preserves authored vs synthesized distinction under verified smoke coverage
