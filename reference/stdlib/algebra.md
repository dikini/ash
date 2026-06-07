# Standard Algebra

Ash exposes algebra interfaces under `std::algebra`. The modules define source-visible interfaces and helper functions; carrier evidence lives with the carrier modules that own the data type or runtime carrier.

## Modules

- `std::algebra::semigroup`: `Semigroup<A>` with `append`.
- `std::algebra::monoid`: `Monoid<A>` with `empty` and `append`, plus `concat_string` and `concat_list` helpers.
- `std::algebra::functor`: `Functor<F>` with `map`, plus current helper wrappers `map_option`, `map_result`, and `map_list`.
- `std::algebra::applicative`: `Applicative<F>` with `pure` and `apply`, plus current helper wrappers for `Option` and `Result`.
- `std::algebra::monad`: `Monad<M>` with `unit` and `bind`, plus current helper wrappers for `Option` and `Result`.
- `std::algebra::comonad`: `Comonad<W>` with `extract` and `extend`. Current source uses the same monomorphic `Int` payload style as the Phase 133 algebra MVP.
- `std::algebra::kleisli`: concrete Option/Result Kleisli helper wrappers `id_option`, `compose_option`, `id_result`, and `compose_result`.

## Instances

Pure carrier instances are defined next to the carrier implementations:

- `std::option` defines `Functor<Option>`, `Applicative<Option>`, and `Monad<Option>`.
- `std::result` defines `Functor<Result<_, E>>`, `Applicative<Result<_, E>>`, and `Monad<Result<_, E>>`.
- `std::list` defines `Functor<List>`, `Semigroup<List<A>>`, and `Monoid<List<A>>`.
- `std::string` defines `Semigroup<String>` and `Monoid<String>`.

Tower carrier helpers remain in their carrier modules (`std::act`, `std::proc`, `std::workflow`) and are wired as public evidence where the runtime can honestly support them. Hidden runtime state remains opaque.

`Comonad` currently has no stdlib carrier instances. `Option`, `Result`, ordinary `List`, `Act`, `Proc`, and `Workflow` are intentionally not Comonad instances: extraction would be partial for empty/error/unfocused pure carriers or would violate runtime opacity for tower carriers.

## `do:` and comprehensions

Generalized `do:K` and explicit-target comprehensions select visible `Monad<K>` evidence. The canonical return/unit method is `unit`, not `return`.

Examples and examples imported by tests:

```ash
use algebra::monad::{Monad, unit_option, bind_option}
use option::{Option, Some, None}

workflow main {
  ret 0
}
```

A `do:Option` block lowers through selected `Monad<Option>` evidence. A `do:Result<_, E>` block uses the public `Monad<Result<_, E>>` surface and the runtime intrinsic shim for current result failure behavior.

## Current helper scope

Phase 133 added only helpers that the current Ash surface can express honestly. More general higher-rank helpers such as fully generic `then`, `join`, `compose`, and generated algebra law checks are deferred to the owned follow-up `TASK-1029-generated-algebra-law-tests.md` rather than implemented as fake Rust builtins.

## Dual/context helpers

SPEC-079 / PLAN-129 own the follow-on for `Comonad`, Kleisli helpers, Cokleisli helpers, and the Coapplicative decision gate. Phase 134 implements the `Comonad` interface and concrete Option/Result Kleisli helper module only. Cokleisli helpers remain deferred because no lawful Comonad carrier exists in the current stdlib, and Coapplicative is explicitly deferred with no source module. The broader `std::category` hierarchy remains out of scope.

## Law profiles

Normative law profiles for `Semigroup`, `Monoid`, `Functor`, `Applicative`, and `Monad` are recorded in `docs/plan/audits/TASK-1026-algebra-law-test-handoff.md`. Comonad, Kleisli, and Cokleisli law-profile ownership is recorded in `docs/plan/audits/TASK-1036-comonad-law-test-handoff.md` and extends `docs/plan/tasks/TASK-1029-generated-algebra-law-tests.md`. They are contracts for future generated tests, not proof obligations executed by Phase 134.
