# TASK-1373: Integration — end-to-end law syntax in `std::algebra`

## Status: ✅ Complete

## Description

Add `law` declarations to at least one `std::algebra` interface and verify full pipeline works.

## Requirements

1. Add laws to `std/src/algebra/semigroup.ash`
2. Add laws to `std/src/algebra/monoid.ash`
3. Verify parser accepts
4. Verify typechecker passes
5. Verify synthetic tests generate

## Acceptance Criteria

- [x] `Semigroup` has `associativity` law
- [x] `Monoid` has `left_identity` and `right_identity` laws
- [x] Full pipeline: parse → typecheck → test generation
- [x] Integration test passes
- [x] No regressions

## Verification

- `cargo fmt --check` — passed
- `cargo test -p ash-engine --test task_1021_std_algebra_namespace_and_interfaces algebra_interface -- --nocapture` — 3 passed
- `cargo test -p ash-cli test_runner::synthesized::tests::extract_laws_returns_std_algebra_law_metadata -- --nocapture` — 1 passed
- `cargo check --workspace` — passed
- `cargo clippy -p ash-engine -p ash-cli --all-targets --all-features -- -D warnings` — passed
- `git diff --check` — passed

## Completion Notes

- Added explicit `Eq<A>` evidence parameters to the `Semigroup` and `Monoid` laws so algebraic equality remains relation-specific rather than overloading `==`.
- `Semigroup` now declares `associativity` over `append`.
- `Monoid` now declares `left_identity` and `right_identity` over `empty` and `append`.
- Added real-stdlib integration coverage that parses/checks the algebra files through `Engine::check_module_file`, asserts parsed interface law names from the actual stdlib source, and verifies synthetic-runner extraction from the real stdlib law declarations.

## Related

- [PLAN-136](../PLAN-136-INTERFACE-LAW-SYNTAX.md)
- [TASK-1360](TASK-1360-parser-law-in-interfaces.md)
- [TASK-1364](TASK-1364-typeck-law-name-checking.md)
- [TASK-1369](TASK-1369-runner-synthetic-test-generation.md)
