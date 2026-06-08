# TASK-1029: Generated Algebra Law Tests

## Status: 📝 Planned

## Description

Implement generated algebra law tests for the public `std::algebra` interfaces using the SPEC-077 generated-test runner framework. This task consumes the handoffs from TASK-1026 and TASK-1036 and turns law profiles into executable generated tests.

## Owner

Future generated-test/law-profile phase owner. This is explicitly out of scope for Phase 133 implementation, but no longer unowned.

## Specification References

- `docs/spec/SPEC-077-ASH-TEST-RUNNER-SYNTHESIZED-AND-SMALLWORLD-COMPLETION.md`
- `docs/spec/SPEC-078-STANDARD-ALGEBRA-LIBRARY-AND-MONAD-REMEDIATION.md`
- `docs/spec/SPEC-079-STANDARD-ALGEBRA-COMONAD-AND-KLEISLI-HELPERS.md`
- `docs/plan/audits/TASK-1026-algebra-law-test-handoff.md`
- `docs/plan/audits/TASK-1036-comonad-law-test-handoff.md`

## Requirements

1. Add generated-test law profile data structures for Semigroup, Monoid, Functor, Applicative, Monad, Comonad, Kleisli, and Cokleisli.
2. Add metadata hooks for generators, equivalence relations, and diagnostic rendering.
3. Generate and execute pure carrier laws for `String`, `List`, `Option`, and `Result<_, E>` first.
4. Keep tower carrier laws gated until bounded equivalence metadata exists for `Act`, `Proc`, and `Workflow`.
5. Emit diagnostics that identify interface, instance key, law name, seed, and minimized counterexample.
6. Do not expose hidden runtime state for tower carriers.
7. Gate Cokleisli law execution until a lawful Comonad carrier exists.

## Acceptance Rows

| Area | Acceptance |
|---|---|
| Semigroup | Generated associativity tests run for String/List instances. |
| Monoid | Generated left/right identity tests run for String/List instances. |
| Functor | Generated identity/composition tests run for Option/Result/List instances. |
| Applicative | Generated identity/homomorphism/interchange/composition tests run for Option/Result instances. |
| Monad | Generated left identity/right identity/associativity tests run for Option/Result instances. |
| Comonad | Generated extend/extract law tests run for lawful Comonad instances once a lawful carrier exists. |
| Kleisli | Generated identity/associativity tests remain deferred until a lawful generic Kleisli helper surface exists; carrier-owned Option/Result operations can inform future metadata but are not `std::algebra` wrappers. |
| Cokleisli | Generated identity/associativity tests are explicitly gated until lawful Comonad carrier metadata exists. |
| Tower | Act/Proc/Workflow law tests are either executable with bounded equivalence metadata or explicitly gated with fail-closed diagnostics. |
| Runner | SPEC-077 runner discovers algebra law families and reports non-zero generated test counts. |

## Verification Seed

```bash
RUSTC_WRAPPER= cargo test -p ash-engine generated_algebra_laws -- --nocapture
RUSTC_WRAPPER= cargo test -p ash-cli generated_algebra_laws -- --nocapture
```
