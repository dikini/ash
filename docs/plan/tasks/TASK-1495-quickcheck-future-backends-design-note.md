# TASK-1495: QuickCheck Future Backends Design Note

## Status: 📝 Planned

## Description

Validate and link the future-backend design note covering SmallCheck, solver/proofs, coverage, mutation, flake quarantine, and distributed orchestration.

## Specification Reference

- [SPEC-086: QuickCheck Arbitrary and Strategy Property Testing](../../spec/SPEC-086-QUICKCHECK-ARBITRARY-STRATEGY.md)
- [PLAN-150: QuickCheck Arbitrary and Strategy Property Testing](../PLAN-150-QUICKCHECK-ARBITRARY-STRATEGY.md)
- [DESIGN-NOTE: QuickCheck-Style Property Testing and Future Evidence Families](../../design/DESIGN-NOTE-QUICKCHECK-PROPERTY-TESTING.md)

## Dependencies

- 📝 TASK-1494: prerequisite task in this phase

## Required Skills

Implementation agents must load:

- `rust-skills` for Rust changes and test quality.
- `ash-language-feature-spec-writing` for live Ash syntax and runner/typechecker boundaries.
- `test-driven-development` for RED-GREEN-REFACTOR implementation.
- `verification-before-completion` before closeout.
- `systematic-debugging` for unexpected failures.

## Files / Surfaces

- `docs/design/DESIGN-NOTE-QUICKCHECK-PROPERTY-TESTING.md`
- `docs/spec/SPEC-086-QUICKCHECK-ARBITRARY-STRATEGY.md`
- `docs/plan/PLAN-150-QUICKCHECK-ARBITRARY-STRATEGY.md`

## Requirements

1. Ensure the design note covers SmallCheck, solver/proof-producing synthesis, coverage/mutation, flake quarantine, and distributed orchestration.
2. Ensure the note clearly separates empirical test evidence from proof evidence.
3. Link the note from spec, plan, and reference docs.
4. Run scoped markdown link and trailing-whitespace checks.

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
  - python3 -c "from pathlib import Path; s=Path('docs/design/DESIGN-NOTE-QUICKCHECK-PROPERTY-TESTING.md').read_text().lower(); assert all(k in s for k in ['smallcheck','solver','proof','mutation','distributed'])"
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
