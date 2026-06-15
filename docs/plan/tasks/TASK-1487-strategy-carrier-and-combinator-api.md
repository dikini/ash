# TASK-1487: Strategy Carrier and Combinator API

## Status: 📝 Planned

## Description

Define `Strategy<T>` as the explicit generated-domain/shrinker carrier and add core combinator API docs/tests.

## Specification Reference

- [SPEC-086: QuickCheck Arbitrary and Strategy Property Testing](../../spec/SPEC-086-QUICKCHECK-ARBITRARY-STRATEGY.md)
- [PLAN-150: QuickCheck Arbitrary and Strategy Property Testing](../PLAN-150-QUICKCHECK-ARBITRARY-STRATEGY.md)
- [DESIGN-NOTE: QuickCheck-Style Property Testing and Future Evidence Families](../../design/DESIGN-NOTE-QUICKCHECK-PROPERTY-TESTING.md)

## Dependencies

- 📝 TASK-1485: prerequisite task in this phase
- 📝 TASK-1486: prerequisite task in this phase

## Required Skills

Implementation agents must load:

- `rust-skills` for Rust changes and test quality.
- `ash-language-feature-spec-writing` for live Ash syntax and runner/typechecker boundaries.
- `test-driven-development` for RED-GREEN-REFACTOR implementation.
- `verification-before-completion` before closeout.
- `systematic-debugging` for unexpected failures.

## Files / Surfaces

- `std/src/test/quickcheck.ash`
- `crates/ash-cli/src/test_runner/`

## Requirements

1. Define first-slice `Strategy<T>` representation: opaque runner carrier or library wrapper.
2. Add core conceptual operations: `gen`, `shrink`, `finite`, `map`, `map2`, `one_of`, `list_of`, `option_of` as supported/stubbed honestly.
3. Document strategy laws: typed generated values, typed shrink candidates, domain preservation obligation, bounded shrinking.
4. Ensure unsupported combinators fail closed/defer rather than silently generating empty pass evidence.

## Examples

Illustrative target surface; TASK-1485 must validate exact live syntax before implementation relies on it:

```ash
use test::quickcheck::{Arbitrary, Strategy}

pub interface Arbitrary<T> {
    arbitrary() -> Strategy<T>
    gen(Int, Int) -> List<T>
    shrink(T) -> List<T>
}

proof example_property(x: Int) {
    by test quickcheck
}
```

Strategy override example:

```ash
proof sorted_binary_search(xs: List<Int>, x: Int) {
    by test quickcheck with {
        xs <- strategy test::gens::sorted_int_lists
        x <- strategy test::quickcheck::ints
    }
}
```

If parser-level override syntax is deferred, use metadata bridge fixtures instead:

```ash
-- @test strategy xs: test::gens::sorted_int_lists
```

## TDD Steps

1. Write focused failing tests or fixtures for this task's surface.
2. Implement the smallest change that makes the focused tests pass.
3. Add negative tests proving missing/unsupported evidence fails closed rather than passing.
4. Run the focused command listed below and record output in the task file before marking complete.
5. Re-run relevant no-Cargo `$ASH_UNDER_TEST test ...` fixtures when this task affects final surface behavior.

## Dispatch

```
agent: hermes
reasoning: high
max_turns: 20
toolsets: [terminal, file, search]
```

## Verification

```
strictness: clean
commands:
  - cargo test -p ash-cli strategy
  - cargo fmt --check
  - git diff --check
checklist:
  - [ ] Focused tests or docs checks pass
  - [ ] Negative/fail-closed behavior covered when applicable
  - [ ] No-Cargo final-surface evidence recorded when applicable
  - [ ] CHANGELOG.md updated if implementation/docs policy changed
```

## Dependencies for Next Task

This task feeds:

- TASK-1496

## Notes

- Keep `property` and `law` enforcement distinct: properties fail tests; laws can invalidate evidence under policy.
- Strategy overrides exist because `Arbitrary<T>` is only a default, not every useful domain for `T`.
- Missing generators, strategies, or Arbitrary evidence must never count as passing evidence.
