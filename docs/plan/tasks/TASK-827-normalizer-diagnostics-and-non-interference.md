# TASK-827: Normalizer Diagnostics and Non-Interference

## Status: ✅ Complete

## Description

Add diagnostics, negative tests, and non-interference coverage for the Phase 112 normalizer/equality core.

## Specification Reference

- [SPEC-060](../../spec/SPEC-060-NORMALIZER-DEFINITIONAL-EQUALITY-CORE.md)
- [PLAN-108](../PLAN-108-NORMALIZER-DEFINITIONAL-EQUALITY-CORE.md)
- [DESIGN-034](../../design/DESIGN-034-TOTAL-TYPE-COMPUTATION.md) §16.4
- [SPEC-058](../../spec/SPEC-058-CANONICAL-TYPE-EXPRESSION-IR-PROJECTION-IDS-KIND-ARITY-SUBSTRATE.md)
- [SPEC-059](../../spec/SPEC-059-SEALED-TYPE-LEVEL-DOMAINS.md)

## Dependencies

- ✅ [TASK-816](TASK-816-spec-d-spec-plan-packet.md)
- ✅ [TASK-826](TASK-826-typeenv-forcing-point-rollout.md)

## Dispatch

```
agent: hermes
provider: openai-codex
model: gpt-5.5
profile: default
reasoning: medium
max_turns: 15
toolsets: [terminal, file]
```

## Objective

Add diagnostics, negative tests, and non-interference coverage for the Phase 112 normalizer/equality core.

## Requirements

Completion evidence: added structured normalizer diagnostics and non-interference tests without adding public syntax or widening TypeEnv rollout.

1. Add tests for neutral note, neutral associated projection note, concrete-normal-form-required, equality-blocked-by-neutrality, normalized mismatch, and fuel/cycle guard diagnostics.
2. Add non-interference tests for Phase 109 ordinary summaries, Phase 110 projection canonicalization, and Phase 111 sealed-domain registration.
3. Verify module summaries do not export fixture equations.
4. Add a real parser negative test proving public source `type fn` syntax is still rejected; do not rely on a zero-test cargo filter.
5. Record focused verification targets for TASK-828.

## Files

- Modify diagnostics in `crates/ash-typeck/src/error.rs` / `diagnostic.rs` as needed
- Test: `crates/ash-typeck/tests/task_827_normalizer_diagnostics.rs`
- Test: existing Phase 109/110/111 focused suites
- Test: `crates/ash-parser/tests/task_827_no_public_type_fn_syntax.rs`

## TDD Steps

1. Write focused failing tests for the task-owned behavior.
2. Run the focused test and confirm it fails for the expected reason.
3. Implement the smallest compiling change that passes the focused test.
4. Re-run focused tests and nearby regression suites.
5. Run formatting and the verification commands below.

## Verification

```
strictness: clean
commands:
  - cargo test -p ash-typeck --test task_827_normalizer_diagnostics
  - cargo test -p ash-engine semantic_summary
  - cargo test -p ash-parser --test task_827_no_public_type_fn_syntax
  - cargo fmt --check
checklist:
  - [x] Diagnostic tests pass
  - [x] Non-interference tests pass
  - [x] No fixture summary export leakage
  - [x] No public parser type fn support appears and the parser negative test runs nonzero tests
```

## Notes

Task type: Diagnostics/Tests. Estimated effort: 6 hours. Keep the slice compilable and do not widen beyond SPEC-060 scope.


## Completion Notes

Completed in Phase 112 implementation. Added `NormalizerDiagnostic` evidence for neutral/stuck normalization, neutral associated projections, concrete-normal-form requirements, equality blocked by neutrality, non-inverting equality notes, normalized mismatches, fuel guards, and legacy fallback boundaries. Added focused parser negative tests for public `type fn` syntax and non-interference coverage for Phase 109 summaries, Phase 110 projections, Phase 111 sealed-domain registration, fixture registry serialization, ordinary ADT constructor boundaries, and TASK-826 guarded TypeEnv rollout.
