# TASK-1492: Law/Property Enforcement and Cache Schema

## Status: 📝 Planned

## Description

Split law vs property outcome consequences and add a versioned law-evidence cache schema.

## Specification Reference

- [SPEC-086: QuickCheck Arbitrary and Strategy Property Testing](../../spec/SPEC-086-QUICKCHECK-ARBITRARY-STRATEGY.md)
- [PLAN-150: QuickCheck Arbitrary and Strategy Property Testing](../PLAN-150-QUICKCHECK-ARBITRARY-STRATEGY.md)
- [DESIGN-NOTE: QuickCheck-Style Property Testing and Future Evidence Families](../../design/DESIGN-NOTE-QUICKCHECK-PROPERTY-TESTING.md)

## Dependencies

- 📝 TASK-1490: prerequisite task in this phase
- 📝 TASK-1491: prerequisite task in this phase

## Required Skills

Implementation agents must load:

- `rust-skills` for Rust changes and test quality.
- `ash-language-feature-spec-writing` for live Ash syntax and runner/typechecker boundaries.
- `test-driven-development` for RED-GREEN-REFACTOR implementation.
- `verification-before-completion` before closeout.
- `systematic-debugging` for unexpected failures.

## Files / Surfaces

- `crates/ash-cli/src/test_runner/types.rs`
- `crates/ash-cli/src/test_runner/output.rs`
- `docs/spec/SPEC-086-QUICKCHECK-ARBITRARY-STRATEGY.md`

## Requirements

1. Mark QuickCheck evidence rows with law/property classification.
2. For property failures, report test failure only.
3. For law refutation, report invalid/refuted law evidence suitable for strict compile/admission policy.
4. Define cache key material: compiler version, stdlib backend version, source hash, law/proof hash, strategy/Arbitrary hashes, seed/size/caps.
5. Implement schema-only, write-only, or read/write cache according to TASK-1485/D4 decision.

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
  - cargo test -p ash-cli law_property_quickcheck
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
