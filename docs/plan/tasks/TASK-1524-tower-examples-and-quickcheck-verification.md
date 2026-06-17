# TASK-1524: Tower Examples and QuickCheck Verification

## Status: 📝 Planned

## Description

Verify all tower examples and deferred QuickCheck combinators work with refined closures. This task validates that the closure refinement enables the patterns that were previously blocked.

## Specification Reference

- [SPEC-088: Closure Refinement and Effect-Safe Capture](../../spec/SPEC-088-CLOSURE-REFINEMENT-AND-EFFECT-SAFE-CAPTURE.md)
- [PLAN-152: Closure Refinement and Tower Documentation](../PLAN-152-CLOSURE-REFINEMENT-AND-TOWER-DOCUMENTATION.md)
- [TASK-1523](TASK-1523-runtime-capture-enforcement.md) — Runtime dependency

## Acceptance Criteria

- [ ] Verify `fn make_adder(n) { fn(x) { n + x } }` works
- [ ] Verify `fn compose(f, g) { fn(x) { f(g(x)) } }` works
- [ ] Verify QuickCheck `recursive` combinator with `GenContext` state passing
- [ ] Verify QuickCheck `one_of` with list indexing (if list primitives available)
- [ ] Verify all tower examples from SPEC-072 work
- [ ] Produce verification report

## Verification

- `cargo test -p ash-engine --test phase151_quickcheck_stdlib` passes (all 3 tests)
- New integration tests for combinators pass
- No regressions in existing tests
