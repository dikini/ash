# TASK-513: Synthesized Tests from Contracts, Policies, and Obligations

## Status: Planned (Phase 76B)

## Description

Add explicit, opt-in synthesized test planning and execution for contracts, policies, and obligations, preserving clear labeling and separation from authored tests.

## Specification Reference

- [PLAN-024: Ash Test Runner V1](../PLAN-024-ASH-TEST-RUNNER-V1.md)
- [DESIGN-021: Ash Test Runner V1](../../design/DESIGN-021-ASH-TEST-RUNNER-V1.md)

## Dependencies

- [TASK-510](TASK-510-test-execution-isolation-and-panic-capture.md)
- [TASK-511](TASK-511-ash-test-library-surface.md)

## Requirements

1. Support opt-in synthesized test planning from function/workflow contracts.
2. Support opt-in synthesized test planning from policies.
3. Support opt-in synthesized test planning from obligations.
4. Ensure synthesized tests are labeled as synthesized in output and JSON results.
5. Ensure synthesized tests are excluded from default authored-test discovery.
6. Preserve the rule that synthesized tests complement, not replace, authored tests.

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

The runner now supports explicit source-scoped synthesized selection and clearly labels synthesized
results in JSON/human output. However, current contract/policy/obligation synthesis still produces
planning-level labeled records rather than executable end-to-end cases. In particular, policy and
obligation synthesis should not be treated as passing execution when nothing was actually run.

## Explicit Deferred Follow-Up Items

Deferred until after spec work improvement:
- stable runner-facing introspection APIs for lowered contracts, policies, and obligations
- executable synthesized contract cases instead of planning-level scans
- executable synthesized policy cases instead of labeled allow/deny placeholders
- executable synthesized obligation lifecycle cases instead of labeled planning records

## Completion Checklist

- [ ] contract-derived synthesized tests implemented and verified end-to-end
- [ ] policy-derived synthesized tests implemented and verified end-to-end
- [ ] obligation-derived synthesized tests implemented and verified end-to-end
- [x] explicit CLI opt-in implemented
- [x] output preserves authored vs synthesized distinction under verified smoke coverage
