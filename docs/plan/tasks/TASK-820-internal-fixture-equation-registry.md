# TASK-820: Internal Fixture Equation Registry

## Status: 📝 Planned

## Description

Add an internal, non-exported fixture equation registry for test-only / explicit compiler-internal test setup computation heads.

## Specification Reference

- [SPEC-060](../../spec/SPEC-060-NORMALIZER-DEFINITIONAL-EQUALITY-CORE.md)
- [PLAN-108](../PLAN-108-NORMALIZER-DEFINITIONAL-EQUALITY-CORE.md)
- [DESIGN-034](../../design/DESIGN-034-TOTAL-TYPE-COMPUTATION.md) §16.4
- [SPEC-058](../../spec/SPEC-058-CANONICAL-TYPE-EXPRESSION-IR-PROJECTION-IDS-KIND-ARITY-SUBSTRATE.md)
- [SPEC-059](../../spec/SPEC-059-SEALED-TYPE-LEVEL-DOMAINS.md)

## Dependencies

- ✅ [TASK-816](TASK-816-spec-d-spec-plan-packet.md)
- 📝 [TASK-819](TASK-819-typeck-normalizer-api-skeleton.md) (planned predecessor)

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

Add an internal, non-exported fixture equation registry for test-only / explicit compiler-internal test setup computation heads.

## Requirements

1. Define fixture equation/pattern/result data structures in ash-typeck normalizer code or test support; if production code can construct them, document that they are not source/module-summary driven and are used only by explicit compiler-internal test setup in this phase.
2. Support first-order constructor patterns over sealed-domain constructor normal forms plus variables.
3. Provide helpers to register an Append-style fixture in tests.
4. Ensure fixtures are not serialized into ModuleSemanticSummary or accepted from parser syntax.
5. Add tests proving fixture registration, deterministic lookup, and no module-summary leakage.

## Files

- Modify: `crates/ash-typeck/src/normalizer.rs`
- Test: `crates/ash-typeck/tests/task_820_fixture_equation_registry.rs`

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
  - cargo test -p ash-typeck --test task_820_fixture_equation_registry
  - cargo test -p ash-engine semantic_summary
  - cargo fmt --check
checklist:
  - [ ] Fixture registry tests pass
  - [ ] No source parser, production module summary, or non-test compiler path claims public type fn support
  - [ ] Append fixture setup can be reused by TASK-821/822
```

## Notes

Task type: Type/Test Substrate. Estimated effort: 5 hours. Keep the slice compilable and do not widen beyond SPEC-060 scope.
