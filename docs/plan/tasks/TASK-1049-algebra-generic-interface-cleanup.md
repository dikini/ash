# TASK-1049: Algebra Generic Interface Cleanup

## Status: ✅ Complete

## Description

Remediate Phase 135 post-review algebra surface drift: the final `std::algebra` interfaces must use generic method payload types instead of monomorphic `Int` placeholders, and algebra interface modules must not publish concrete carrier wrapper functions that belong in carrier modules.

## Specification Reference

- [SPEC-080: Interface Evidence Constraints](../../spec/SPEC-080-INTERFACE-EVIDENCE-CONSTRAINTS.md)
- [SPEC-079: Standard Algebra Comonad and Kleisli Helper Surfaces](../../spec/SPEC-079-STANDARD-ALGEBRA-COMONAD-AND-KLEISLI-HELPERS.md)
- [PLAN-130: Interface Evidence Constraints](../PLAN-130-INTERFACE-EVIDENCE-CONSTRAINTS.md)

## Requirements

1. Change `std::algebra::Functor` from `Int`-only `map` to `map(F<A>, A -> B) -> F<B>`.
2. Change `std::algebra::Applicative` from `Int`-only `pure`/`apply` to generic `pure(A) -> F<A>` and `apply(F<A -> B>, F<A>) -> F<B>` while preserving the `Functor` required-evidence constraint.
3. Change `std::algebra::Monad` from `Int`-only `unit`/`bind` to generic `unit(A) -> M<A>` and `bind(M<A>, A -> M<B>) -> M<B>` while preserving the `Applicative` required-evidence constraint.
4. Change `std::algebra::Comonad` from `Int`-only `extract`/`extend` to generic `extract(W<A>) -> A` and `extend(W<A>, W<A> -> B) -> W<B>`.
5. Remove concrete carrier wrappers from `std/src/algebra/{functor,applicative,monad,monoid,kleisli}.ash`; concrete operations remain carrier-owned in `option`, `result`, `list`, `string`, and tower modules.
6. Update tests and reference/spec wording so removed concrete wrappers are not documented or expected as final algebra surface.
7. Verify through final stdlib paths and broad workspace gates.

## TDD Evidence

- RED source audit: pre-fix `std/src/algebra/functor.ash`, `applicative.ash`, `monad.ash`, and `comonad.ash` used monomorphic `Int` payload signatures; `functor`, `applicative`, `monad`, `monoid`, and `kleisli` published concrete carrier wrappers.
- Syntax probe: `ash check` accepts free method-level type variables in interface signatures such as `map(F<A>, A -> B) -> F<B>`; `map<A, B>(...)` method-generic syntax remains unsupported.
- GREEN: focused algebra/Kleisli tests were updated to assert interface-only algebra modules and removed concrete wrapper imports.
- Post-review remediation: implicit method-level payload variables are tracked separately from interface-head variables, selected interface method calls infer payload variables from arguments, and no-inversion evidence lookup remains fail-closed for older constructor-variable method surfaces.

## Verification

Completed on 2026-06-08:

- `cargo fmt --check`
- `RUSTC_WRAPPER= cargo test -p ash-typeck --test task_1022_pure_algebra_instances -- --nocapture`
- `RUSTC_WRAPPER= cargo test -p ash-typeck --test task_908_hkt_evidence_lookup -- --nocapture`
- `RUSTC_WRAPPER= cargo test -p ash-typeck --test task_1022_pure_algebra_instances pure_algebra_instances_monomorphize_generic_functor_method_payloads_at_call_site -- --nocapture`
- `RUSTC_WRAPPER= cargo test -p ash-engine --test task_1025_algebra_combinators -- --nocapture`
- `RUSTC_WRAPPER= cargo test -p ash-engine --test task_1033_stdlib_kleisli -- --nocapture`
- `RUSTC_WRAPPER= cargo test -p ash-cli --test task_1025_algebra_examples -- --nocapture`
- `RUSTC_WRAPPER= cargo test -p ash-cli --test task_1033_kleisli_examples -- --nocapture`
- `RUSTC_WRAPPER= cargo test -p ash-engine --test task_1044_stdlib_monad_constraint -- --nocapture`
- `RUSTC_WRAPPER= cargo test -p ash-engine --test task_1045_stdlib_applicative_constraint -- --nocapture`
- `RUSTC_WRAPPER= cargo test -p ash-engine --test task_1046_stdlib_monoid_constraint -- --nocapture`
- `RUSTC_WRAPPER= cargo check --workspace`
- `RUSTC_WRAPPER= cargo clippy --workspace --all-targets --all-features -- -D warnings`
- `RUSTC_WRAPPER= cargo test --workspace`
- `git diff --check`

## Completion Checklist

- [x] Generic method payload signatures are present in final `std/src/algebra` interface files.
- [x] Required-evidence constraints remain present for `Monad -> Applicative`, `Applicative -> Functor`, and `Monoid -> Semigroup`.
- [x] Concrete carrier wrappers are removed from algebra interface modules.
- [x] Kleisli concrete Option/Result wrappers are removed rather than retained in `std::algebra`.
- [x] Reference/spec wording no longer teaches the removed wrappers as final surface.
- [x] Focused and broad verification commands pass.
- [x] Independent review completed with no blocking issues.
