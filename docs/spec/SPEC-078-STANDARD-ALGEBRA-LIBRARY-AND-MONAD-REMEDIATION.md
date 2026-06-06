# SPEC-078: Standard Algebra Library and Monad Remediation

**Status:** Draft
**Date:** 2026-06-06
**Plan:** [PLAN-128](../plan/PLAN-128-STANDARD-ALGEBRA-LIBRARY-AND-MONAD-REMEDIATION.md)
**Implementation Tasks:** [TASK-1020](../plan/tasks/TASK-1020-stdlib-algebra-audit-gate.md) through [TASK-1028](../plan/tasks/TASK-1028-stdlib-algebra-closeout.md)
**Related:** [SPEC-054](SPEC-054-GENERALIZED-TYPED-DO-NOTATION.md), [SPEC-055](SPEC-055-MONAD-COMPREHENSION-SYNTAX.md), [SPEC-067](SPEC-067-CONSTRUCTOR-KINDED-PARAMETERS-AND-HKT.md), [SPEC-069](SPEC-069-ALPHA-VISIBLE-TOWER-ALGEBRA-AND-DO-LOWERING.md), [SPEC-077](SPEC-077-ASH-TEST-RUNNER-SYNTHESIZED-AND-SMALLWORLD-COMPLETION.md)

## Summary

Ash must expose standard algebraic interfaces as usable Ash standard-library code rather than only as internal Rust/typechecker evidence. This specification adds a `std::algebra` namespace containing source-visible `Semigroup`, `Monoid`, `Functor`, `Applicative`, and `Monad` interfaces, concrete instances for ordinary data carriers, and reconciled standard/prelude evidence for `Act`, `Proc`, and `Workflow`.

The phase does not add syntax. It uses existing Ash language features: modules, imports, interfaces, impls, ordinary functions, builtin opaque carrier declarations, and existing `do:K` / comprehension lowering. Rust runtime specialization remains allowed for opaque tower carriers, but public sequencing authority must be represented by standard Ash algebra surfaces or by explicitly named compiler-prelude evidence tied to those surfaces.

## Motivation

Earlier phases reasonably deferred source-level stdlib `Monad` while constructor-kinded interfaces, HKT evidence, partial constructor application, selected evidence lowering, and full bind lowering were missing. Those prerequisites now exist in implemented MVP form. Keeping old hidden dictionary restrictions after the substrate landed creates a loophole: `do:K` can appear generalized while the standard library still lacks the public algebra it is supposed to use.

This spec retires that obsolete deferral and makes the final surface testable: users and libraries should import `std::algebra` interfaces, instances should resolve through standard evidence, and do/comprehension lowering should use the same evidence path rather than hidden unrelated bridge authority.

## Namespace Decision

Algebraic interfaces live under `std::algebra`, not as direct root-level `std` exports.

Required source layout:

```text
std/src/algebra/mod.ash
std/src/algebra/semigroup.ash
std/src/algebra/monoid.ash
std/src/algebra/functor.ash
std/src/algebra/applicative.ash
std/src/algebra/monad.ash
```

`std/src/lib.ash` must expose the namespace:

```ash
pub mod algebra;
```

The implementation may optionally add carefully named prelude re-exports later, but this phase's canonical import path is explicit:

```ash
use algebra::monad::{Monad};
use algebra::functor::{Functor};
```

`std::category` is intentionally not introduced in this phase. Category-level abstractions such as category, bifunctor, profunctor, comonad, or arrows remain future library work once the algebra namespace is stable.

## Deferral and Planned-Feature Reconciliation

| Prior item | Source | Original reason | Current status | Decision | Gate |
|---|---|---|---|---|---|
| Public stdlib `Monad<M>` deferred | SPEC-054 / SPEC-067 | HKT/evidence/method lowering not ready | SPEC-067 and SPEC-069 now provide implemented MVP substrate | Retire deferral; add `std::algebra::monad` | Importable stdlib interface test |
| Hidden Act/Proc/Workflow do dictionaries | SPEC-054 / SPEC-069 | Tower carriers needed bridge dictionaries before public algebra | Public tower operations exist; evidence lowering exists | Replace or quarantine as named prelude evidence tied to public symbols | Negative leakage test for anonymous hidden authority |
| `Option`/`Result`/`List` dictionaries deferred | SPEC-055 / SPEC-067 | Pure container evidence and partial constructor targets incomplete | `Option`, `Result`, `List`, and `Result<_, E>` do-target substrate exist | Implement ordinary instances where source syntax supports them | `do:Option`, `do:Result<_, E>` stdlib evidence tests |
| Law proving/checking | SPEC-054 / SPEC-067 | No law syntax/proof/test-generation substrate | Synthesized/generated test runner now exists but law generation is not specified | Split to follow-on law-test/proof phase | Follow-up packet links law profiles to SPEC-077-style generated tests |
| Fully self-hosted Act/Proc/Workflow runtime representation | SPEC-047..SPEC-051 / SPEC-069 | Opaque runtime carriers require Rust runtime state | Still true | Keep deferred | Opaque carrier tests continue to reject denotation of hidden runtime envs |

## Standard Interfaces

The audit gate must freeze exact syntax against the live parser, lowering, and typechecker before implementation. The following block is a logical target, not permission to write syntax the live parser/lowerer rejects. If current Ash interface methods require positional-only argument types, no method-level generics, or a different kinded-binder spelling, TASK-1020 must translate this logical target into exact accepted source syntax before TASK-1021 starts:

```ash
pub interface Semigroup<A> {
    append(a: A, b: A) -> A;
}

pub interface Monoid<A> {
    empty() -> A;
    append(a: A, b: A) -> A;
}

pub interface Functor<F : * -> *> {
    map<A, B>(fa: F<A>, f: (A) -> B) -> F<B>;
}

pub interface Applicative<F : * -> *> {
    pure<A>(value: A) -> F<A>;
    apply<A, B>(ff: F<(A) -> B>, fa: F<A>) -> F<B>;
}

pub interface Monad<M : * -> *> {
    unit<A>(value: A) -> M<A>;
    bind<A, B>(ma: M<A>, f: (A) -> M<B>) -> M<B>;
}
```

If live Ash syntax requires `return` rather than `unit` for selected Monad evidence, the audit task must choose one canonical interface spelling and patch `do` lowering accordingly. The preferred public method name is `unit`; `return` remains block syntax, not a root-level ordinary function name, unless the live evidence path makes `return` unavoidable.

## Required Instances

### Pure data instances

The phase should implement source-level instances for pure carriers wherever current Ash supports the bodies. A prelude-backed fallback for pure `Option`/`Result`/`List`/`String` evidence is allowed only when TASK-1020 records a concrete live-syntax or module-loading blocker, ties the fallback to importable stdlib symbols, and creates a named follow-up to replace it:

```text
Semigroup<String>
Monoid<String>
Semigroup<List<A>>
Monoid<List<A>>
Functor<Option>
Applicative<Option>
Monad<Option>
Functor<Result<_, E>>
Applicative<Result<_, E>>
Monad<Result<_, E>>
Functor<List>
Applicative<List> if list product/application semantics are chosen explicitly
Monad<List> if concat-map/bind support is available or added honestly
```

`Int` additive/product monoids are not required in this first slice unless the audit chooses explicit named wrappers or modules to avoid ambiguous multiple `Monoid<Int>` instances.

### Tower carrier instances

`Act`, `Proc`, and `Workflow` remain opaque runtime carriers. Their standard instances must delegate to public tower operations:

```text
Monad<Act>      -> act::unit / act::bind
Monad<Proc>     -> proc::unit / proc::bind
Monad<Workflow> -> workflow::unit / workflow::bind
```

Functor and Applicative instances may be direct implementations or derived from `unit`/`bind` if the implementation can express the bodies in current Ash. If source-level impl bodies cannot yet express the required generic functions, the compiler prelude may install named evidence that points to public stdlib symbols. Anonymous hidden dictionary operations must not remain independent semantic authority.

## Do and Comprehension Reconciliation

`do:K` and explicit-target comprehensions must resolve through selected `Monad<K>` evidence. Acceptance tests must include final-surface imports/evidence, not only inline fixtures.

Required positive cases:

```text
do:Option
do:Result<_, E>
do:Act
do:Proc
do:Workflow
explicit-target comprehension over at least Option and one tower carrier
```

Required negative cases:

```text
missing Monad<K> still fails closed
ambiguous Monad<K> still fails before lowering
Act/Proc/Workflow do lowering does not select anonymous unrelated hidden dictionaries
```

## Usable Library Functions

The phase should add ordinary algebra helpers under `std::algebra`, implemented in Ash source when possible:

```text
algebra::functor::void
algebra::functor::replace
algebra::applicative::lift2
algebra::applicative::then
algebra::monad::then
algebra::monad::join
algebra::monad::compose
algebra::monoid::concat
```

The audit gate may trim this list if the current interface method-call substrate cannot express a helper honestly. Any trimmed helper must become an explicit follow-up row, not an implicit omission.

## Law Profiles and Generated Tests

Algebra laws are normative contracts for these interfaces. This phase does not implement law proof checking or automatic law-test generation.

Instead, this phase must preserve enough structure for a later law-test phase:

```text
interface name
method names and signatures
instance identity
type/generator requirements
equivalence relation requirements
side-effect trace/equivalence requirements for Act/Proc/Workflow
```

A follow-on generated-test phase must integrate with the SPEC-077 synthesized/generated test-runner framework to derive law tests for supported instances. TASK-1026 must create a concrete follow-up task/phase seed with acceptance rows for generated law tests; a prose-only audit note is not sufficient. Pure instances such as `Option`, `Result`, `List`, and `String` can use direct generated values. `Act`, `Proc`, and `Workflow` law tests require deterministic capability/runtime/scheduler fixtures and may defer until safe equivalence relations exist.

## Non-Goals

- No new Ash syntax.
- No typeclass deriving.
- No law proof checker in this phase.
- No automatic law-test generation in this phase, beyond the follow-up handoff packet.
- No open-world typeclass/coherence redesign beyond the current implemented MVP evidence model.
- No unrestricted type lambdas, higher-rank polymorphism, or broad multi-parameter constructor classes.
- No self-hosting of the internal `ActEnv`, process scheduler, workflow admission kernel, or other opaque runtime carrier internals.

## Acceptance Matrix

Filtered focused tests must prove non-zero execution or be replaced by explicit artifact assertions. A cargo filter that can pass with zero matching tests is not acceptable closeout evidence.


| ID | Requirement | Evidence |
|---|---|---|
| A78-1 | `std/src/algebra/*` modules exist and are exported by `std/src/lib.ash` | File-existence and importability tests |
| A78-2 | `Semigroup`, `Monoid`, `Functor`, `Applicative`, and `Monad` interfaces parse/check through the real stdlib path | Engine/std module check tests |
| A78-3 | Pure data instances resolve from stdlib, not local test-only fixtures | `Option`/`Result`/`List`/`String` tests |
| A78-4 | Tower carrier Monad evidence is tied to public stdlib operations or a named prelude shim | `do:Act`/`do:Proc`/`do:Workflow` evidence tests |
| A78-5 | `do:Option` and `do:Result<_, E>` lower through selected stdlib Monad evidence | Typeck/engine tests |
| A78-6 | Comprehensions use the same selected evidence path as `do:K` | Comprehension tests |
| A78-7 | Missing or ambiguous evidence still fails closed | Negative diagnostics tests |
| A78-8 | Old hidden bridge authority is removed or quarantined with negative leakage coverage | Do-target audit/test |
| A78-9 | Usable algebra helper functions compile and run in examples | Engine examples/tests |
| A78-10 | Law proof/test derivation is split into an explicit follow-up packet tied to generated tests | Follow-up task/spec seed |
| A78-11 | Reference docs and stale deferral language are reconciled across Monad/algebra/dictionary/evidence/Option/Result/List/tower bridge wording, with historical wording explicitly labeled | Broad stale-deferral sweep |
| A78-12 | Broad affected-crate and workspace verification passes before closeout | Closeout gate |

## Implementation Tasks

- TASK-1020: Audit gate and exact syntax/evidence freeze.
- TASK-1021: `std::algebra` namespace and interface modules.
- TASK-1022: Pure data instances for `Option`, `Result`, `List`, and string/list monoids.
- TASK-1023: Tower carrier instances and hidden-bridge reconciliation.
- TASK-1024: `do:K` and comprehension evidence rewiring to stdlib/prelude evidence.
- TASK-1025: Usable algebra helper functions and examples.
- TASK-1026: Law-profile generated-test follow-up packet.
- TASK-1027: Reference documentation and corpus migration.
- TASK-1028: Closeout, broad verification, and stale-deferral cleanup.

## Changelog

### 2026-06-06

- Initial draft created to remediate the gap between implemented generalized Monad evidence lowering and missing source-visible standard algebra library surfaces.
