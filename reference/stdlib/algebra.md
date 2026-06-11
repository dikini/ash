# Standard Algebra

Ash exposes algebra interfaces under `std::algebra`. These modules define source-visible interfaces and evidence requirements; carrier-specific helper functions and instances live with the carrier modules that own the data type or runtime carrier.

## Modules

- `std::algebra::semigroup`: `Semigroup<A>` with `append`.
- `std::algebra::monoid`: `Monoid<A>` with `empty` and `append`; `Monoid<A>` requires `Semigroup<A>` evidence.
- `std::algebra::functor`: `Functor<F>` with generic `map(F<A>, A -> B) -> F<B>`.
- `std::algebra::applicative`: `Applicative<F>` with generic `pure(A) -> F<A>` and `apply(F<A -> B>, F<A>) -> F<B>`; `Applicative<F>` requires `Functor<F>` evidence.
- `std::algebra::monad`: `Monad<M>` with generic `unit(A) -> M<A>` and `bind(M<A>, A -> M<B>) -> M<B>`; `Monad<M>` requires `Applicative<M>` evidence.
- `std::algebra::comonad`: `Comonad<W>` with generic `extract(W<A>) -> A` and `extend(W<A>, W<A> -> B) -> W<B>`.
- `std::algebra::kleisli`: currently publishes no concrete carrier wrappers. A lawful carrier-polymorphic Kleisli helper surface depends on selected `Monad<M>` method dispatch from source code and remains deferred until that surface exists.

## Instances

Pure carrier instances are defined next to the carrier implementations:

- `std::option` defines `Functor<Option>`, `Applicative<Option>`, and `Monad<Option>`.
- `std::result` defines `Functor<Result<_, E>>`, `Applicative<Result<_, E>>`, and `Monad<Result<_, E>>`.
- `std::list` defines `Functor<List>`, `Semigroup<List<A>>`, and `Monoid<List<A>>`.
- `std::string` defines `Semigroup<String>` and `Monoid<String>`.

Carrier-local helper functions remain in their carrier modules (`std::option`, `std::result`, `std::list`, `std::string`, `std::act`, `std::proc`, `std::workflow`). Hidden runtime state remains opaque.

`Comonad` currently has no stdlib carrier instances. `Option`, `Result`, ordinary `List`, `Act`, `Proc`, and `Workflow` are intentionally not Comonad instances: extraction would be partial for empty/error/unfocused pure carriers or would violate runtime opacity for tower carriers.

## `do:` and comprehensions

Generalized `do:K` and explicit-target comprehensions select visible `Monad<K>` evidence. The canonical return/unit method is `unit`, not `return`.

Example imports should reference algebra interfaces from `std::algebra` and concrete operations from the carrier modules:

```ash
use algebra::monad::{Monad}
use option::{Option, Some, None}

workflow main {
  ret 0
}
```

A `do:Option` block lowers through selected `Monad<Option>` evidence. A `do:Result<_, E>` block uses the public `Monad<Result<_, E>>` surface and the runtime intrinsic shim for current result failure behavior.

## Helper scope

`std::algebra` does not publish concrete Option/Result/List/String wrapper functions. More general higher-rank helpers such as fully generic `then`, `join`, `compose`, Kleisli composition, and generated algebra law checks are deferred to owned follow-up work until Ash has an honest selected-evidence method-dispatch surface for them.

## Dual/context helpers

SPEC-079 / PLAN-129 own the design history for `Comonad`, Kleisli helpers, Cokleisli helpers, and the Coapplicative decision gate. Phase 135 supersedes the temporary concrete Option/Result Kleisli wrapper surface: Cokleisli helpers remain deferred because no lawful Comonad carrier exists in the current stdlib, Coapplicative is explicitly deferred with no source module, and the broader `std::category` hierarchy remains out of scope.

## Source-visible law declarations

Law declarations are source-visible in `std/src/algebra/*.ash`:

- `std::algebra::semigroup` — `law associativity(a, b, c, eq)`
- `std::algebra::monoid` — `law left_identity(a, eq)`, `law right_identity(a, eq)`
- `std::algebra::functor` — `law identity(value, eq)`, `law composition(value, f, g, eq)`
- `std::algebra::applicative` — `law identity(value, eq)`, `law homomorphism(x, f, eq)`, `law interchange(u, y, eq)`, `law composition(u, v, w, eq)`
- `std::algebra::monad` — `law left_identity(a, f, eq)`, `law right_identity(m, eq)`, `law associativity(m, f, g, eq)`

Each law takes explicit `Eq` evidence and states an equivalence between two expressions. Law declarations are checked by the parser and typechecker; they are distinct from proofs.

## Proof declarations

`std::option` and `std::result` carry `by test "..."` proof declarations inside their `impl Functor`, `impl Applicative`, and `impl Monad` blocks. These proofs delegate to generated/synthetic law tests rather than claiming manual proof-checker validation. This is the honest first surface: `by_definition` proofs are deferred until the proof checker can validate them semantically against the law proposition.

## Law profiles

Normative law profiles for `Semigroup`, `Monoid`, `Functor`, `Applicative`, and `Monad` are recorded in `docs/plan/audits/TASK-1026-algebra-law-test-handoff.md`. Comonad, Kleisli, and Cokleisli law-profile ownership is recorded in `docs/plan/audits/TASK-1036-comonad-law-test-handoff.md` and extends `docs/plan/tasks/TASK-1029-generated-algebra-law-tests.md`. They are contracts for future generated tests, not proof obligations executed by Phase 134 or Phase 135.
