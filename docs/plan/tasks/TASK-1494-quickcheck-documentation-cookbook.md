# TASK-1494: QuickCheck Documentation Cookbook

## Status: ✅ Complete

## Description

Document user-facing QuickCheck examples: Arbitrary, Strategy composition, overrides, shrinking, and law/property semantics.

## Specification Reference

- [SPEC-086: QuickCheck Arbitrary and Strategy Property Testing](../../spec/SPEC-086-QUICKCHECK-ARBITRARY-STRATEGY.md)
- [PLAN-150: QuickCheck Arbitrary and Strategy Property Testing](../PLAN-150-QUICKCHECK-ARBITRARY-STRATEGY.md)
- [DESIGN-NOTE: QuickCheck-Style Property Testing and Future Evidence Families](../../design/DESIGN-NOTE-QUICKCHECK-PROPERTY-TESTING.md)

## Dependencies

- ✅ TASK-1493: prerequisite task in this phase

## Required Skills

Implementation agents must load:

- `rust-skills` for Rust changes and test quality.
- `ash-language-feature-spec-writing` for live Ash syntax and runner/typechecker boundaries.
- `test-driven-development` for RED-GREEN-REFACTOR implementation.
- `verification-before-completion` before closeout.
- `systematic-debugging` for unexpected failures.

## Files / Surfaces

- `reference/tools/test.md`
- `docs/design/DESIGN-NOTE-QUICKCHECK-PROPERTY-TESTING.md`

## Requirements

1. Add a QuickCheck section to the test reference.
2. Include examples for default `Arbitrary<T>`, sorted-list override, safe-expression override, and shrink trace output.
3. Explain law vs property failure consequences.
4. Explain evidence cache invalidation at a user level.

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
  - python3 -c "from pathlib import Path; assert 'quickcheck' in Path('reference/tools/test.md').read_text().lower()"
  - cargo fmt --check
  - git diff --check
checklist:
  - [x] Focused tests or docs checks pass
  - [x] Negative/fail-closed behavior covered when applicable
  - [x] No-Cargo final-surface evidence recorded when applicable
  - [x] CHANGELOG.md updated if implementation/docs policy changed
```

## Dependencies for Next Task

This task feeds:

- TASK-1496

## Notes

- Keep `property` and `law` enforcement distinct: properties fail tests; laws can invalidate evidence under policy.
- Strategy overrides exist because `Arbitrary<T>` is only a default, not every useful domain for `T`.
- Missing generators, strategies, or Arbitrary evidence must never count as passing evidence.


## Completion Notes

Phase 150 landed the first QuickCheck-like implementation slice:

- `test::quickcheck` stdlib namespace with `Strategy<T>` and `Arbitrary<T>` surface docs/laws.
- Metadata bridge for explicit strategy overrides: `-- @test strategy <binding>: <strategy-path>`.
- Runner resolution order: explicit strategy override first, otherwise default bounded `Arbitrary<T>` representatives.
- Domain-preserving shrink candidates for explicit strategies.
- Version-moderated empirical law-evidence cache schema.
- No-Cargo `$ASH_UNDER_TEST test ...` fixtures for default/override/failing/fail-closed cases.

Parser-level `by test quickcheck with { ... }`, automatic `Arbitrary<T>` derivation, full SmallCheck, and solver/proof backends remain future work per SPEC-086 and the design note.
