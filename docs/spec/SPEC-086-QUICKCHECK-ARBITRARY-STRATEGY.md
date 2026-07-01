# SPEC-086: QuickCheck Arbitrary and Strategy Property Testing

**Status:** Implemented MVP (Phase 150); hardened by SPEC-087 / Phase 151
**Date:** 2026-06-15
**Builds on:** [SPEC-081](SPEC-081-LAW-TEST-EVIDENCE-SUBSTRATE.md), [SPEC-082](SPEC-082-PROPERTY-GENERATION-AND-SHRINKING-SUBSTRATE.md)
**Design note:** [DESIGN-NOTE: QuickCheck-Style Property Testing and Future Evidence Families](../design/DESIGN-NOTE-QUICKCHECK-PROPERTY-TESTING.md)
**Plan:** [PLAN-150: QuickCheck Arbitrary and Strategy Property Testing](../plan/PLAN-150-QUICKCHECK-ARBITRARY-STRATEGY.md)
**Superseded / hardened by:** [SPEC-087: QuickCheck v1 Ordinary Strategy Semantics](SPEC-087-QUICKCHECK-V1-ORDINARY-STRATEGY-SEMANTICS.md)

## Summary

Add a standard-library QuickCheck-like property-testing substrate under the Ash `test` subspace. The phase introduces `test::quickcheck::Strategy<T>` as the explicit generated-domain/shrinker carrier and `test::quickcheck::Arbitrary<T>` as default evidence for a type's canonical strategy. Laws and properties share proposition structure but retain distinct enforcement semantics: property failure is a test failure, while law refutation can invalidate law evidence under the active compile/admission policy.

## Motivation

Phase 146 implements bounded generated bindings in the runner. That is useful but still runner-owned. The next step is to move the author-facing abstraction into Ash library code so users can define reusable generators and shrinkers in Ash, override domains per property, and eventually compose them with other test backends such as SmallCheck.

## Required Agent Skills

Implementation agents must load and follow:

- `rust-skills` for Rust code, public APIs, error handling, and clippy-clean implementation.
- `ash-language-feature-spec-writing` for live Ash syntax, interface/impl constraints, parser/typechecker boundaries, and final-surface examples.
- `test-driven-development` for RED-GREEN-REFACTOR implementation slices.
- `verification-before-completion` before marking tasks complete.
- `systematic-debugging` for runner, stdlib, evidence, cache, and property failures.

## Scope

### In Scope

- `test::quickcheck` stdlib subspace design and initial implementation.
- `Strategy<T>` as a runner-supported, Ash-visible, opaque or library-carried strategy value.
- `Arbitrary<T>` as default evidence providing `arbitrary() -> Strategy<T>`, `gen(seed, size) -> List<T>`, and `shrink(value) -> List<T>`.
- Strategy laws and Arbitrary coherence laws documented in the library and tested where executable.
- Built-in primitive/container default strategies for Phase 146-supported domains.
- Explicit strategy overrides for law/property parameters.
- Law vs property result semantics and JSON evidence labels.
- Versioned law-evidence cache schema design and first implementation slice if the audit proves the storage seam is ready.
- No-Cargo `$ASH_UNDER_TEST test ...` fixtures and documentation examples.

### Non-Goals

- automatic deriving of `Arbitrary<T>`,
- unrestricted source-world generation,
- full `test::smallcheck` implementation,
- proof-producing synthesis,
- solver proof checking,
- coverage/mutation/flake/distributed orchestration,
- effectful or capability-bearing generators,
- multiple overlapping default `Arbitrary<T>` instances for the same type.

## Normative Model

### Strategy

`Strategy<T>` is the value-level representation of a particular generated test domain for `T` plus its shrink relation.

A strategy must support, conceptually:

```ash
strategy::gen(Strategy<T>, seed: Int, size: Int) -> List<T>
strategy::shrink(Strategy<T>, value: T) -> List<T>
```

The first implementation may represent `Strategy<T>` as a runner-owned opaque carrier rather than a fully ordinary Ash ADT. The public API still lives under `test::quickcheck`.

### Arbitrary

`Arbitrary<T>` is default evidence for a canonical strategy:

```ash
pub interface Arbitrary<T> {
    arbitrary() -> Strategy<T>
    gen(Int, Int) -> List<T>
    shrink(T) -> List<T>
}
```

Because current interface method syntax uses positional parameter types, the source form should avoid named method parameters until a future syntax phase supports them.

### Coherence Laws

The following are normative library laws. They may initially be documented or tested by runner fixtures; later phases can promote them to executable stdlib law declarations when the necessary equality/proof substrate is ready.

```ash
law arbitrary_gen_coherent(seed: Int, size: Int, eq: Eq<List<T>>):
    eq.equiv(gen(seed, size), strategy::gen(arbitrary(), seed, size))

law arbitrary_shrink_coherent(value: T, eq: Eq<List<T>>):
    eq.equiv(shrink(value), strategy::shrink(arbitrary(), value))
```

Strategy constructors also carry laws:

- generated values have type `T`,
- shrink candidates have type `T`,
- shrink candidates are intended to remain inside the same semantic subdomain,
- shrink candidates should be no more complex than the original value under the strategy's measure,
- repeated shrinking must be bounded by runner caps,
- the runner accepts a shrink candidate into the trace only if the property still fails.

### Default and Override Resolution

For each law/property parameter `x: T`, resolution order is:

1. explicit strategy override for `x`,
2. explicit generator/shrinker metadata bridge for `x` if still supported,
3. fixture/project override if a future manifest exists,
4. `Arbitrary<T>::arbitrary()` evidence,
5. built-in primitive/container fallback only where explicitly specified,
6. deferred/invalid evidence; never count missing evidence as pass.

### Law vs Property

`property` and `law` are near-synonyms at the proposition level but differ in enforcement.

| Construct | Execution | Failure consequence |
|---|---|---|
| property | `ash test` | test failure |
| law by quickcheck | `ash test`, evidence cache/policy | refutation invalidates law evidence and can fail compilation/admission under strict policy |
| law by solver/proof | future proof subsystem | invalid proof/refutation fails the relevant proof boundary |

Stale/missing law evidence is not the same as refutation. The active policy decides whether to warn, defer, rerun, or fail closed.

## Examples

### Default Arbitrary

```ash
use test::quickcheck::{Arbitrary, Strategy}

pub type Expr =
    Lit { value: Int }
  | Add { left: Expr, right: Expr }

impl Arbitrary<Expr> {
    arbitrary() = test::gens::any_exprs()
    gen(seed, size) = strategy::gen(arbitrary(), seed, size)
    shrink(value) = strategy::shrink(arbitrary(), value)
}

law normalize_idempotent(e: Expr): normalize(normalize(e)) == normalize(e)
proof normalize_idempotent(e: Expr) {
    by test quickcheck
}
```

The example uses named values in bodies illustratively; implementation tasks must validate final syntax against the live parser and may use metadata bridge fixtures where needed.

### Strategy Override

```ash
proof eval_total_on_safe_exprs(e: Expr) {
    by test quickcheck with {
        e <- strategy test::gens::nonzero_denominator_exprs
    }
}
```

If the `with` syntax is not in the first implementation slice, use the metadata bridge:

```ash
-- @test strategy e: test::gens::nonzero_denominator_exprs
```

### Sorted List Property

```ash
property binary_search_finds_member(xs: List<Int>, x: Int):
    contains(xs, x) == option::is_some(binary_search(xs, x))

proof binary_search_finds_member(xs: List<Int>, x: Int) {
    by test quickcheck with {
        xs <- strategy test::gens::sorted_int_lists
        x <- strategy test::quickcheck::ints
    }
}
```

## Result Reporting

QuickCheck evidence rows must preserve the Phase 145/146 JSON guarantees and add enough fields for library-backed strategies:

- evidence family: `test`,
- test mode/backend: `quickcheck`,
- law/property classification,
- strategy source for each binding,
- Arbitrary evidence source when defaulted,
- seed/size schedule,
- original generated bindings,
- shrunk counterexample and shrink trace when present,
- cache key or cache status for law evidence where applicable.

## Evidence Cache Schema

The phase must define a versioned law-evidence cache key even if the first implementation only writes/reads a minimal local cache.

Required key material:

- Ash compiler/test-runner version,
- stdlib/test backend version,
- source artifact hash,
- module/law/proof identity,
- proof body hash,
- `Arbitrary<T>` impl identities/hashes,
- explicit `Strategy<T>` identities/hashes,
- seed/size/max-case policy,
- shrink policy/caps,
- backend schema version.

## Implementation Tasks

- [TASK-1485](../plan/tasks/TASK-1485-quickcheck-design-and-live-syntax-audit.md): Audit live syntax, stdlib module surfaces, interface evidence, runner seams, and cache seams.
- [TASK-1486](../plan/tasks/TASK-1486-quickcheck-stdlib-namespace.md): Add `test::quickcheck` namespace skeleton and docs.
- [TASK-1487](../plan/tasks/TASK-1487-strategy-carrier-and-combinator-api.md): Define `Strategy<T>` carrier and core combinator API.
- [TASK-1488](../plan/tasks/TASK-1488-arbitrary-interface-and-laws.md): Define `Arbitrary<T>` interface and library law docs/tests.
- [TASK-1489](../plan/tasks/TASK-1489-primitive-container-arbitrary-impls.md): Add primitive/container default strategies.
- [TASK-1490](../plan/tasks/TASK-1490-runner-strategy-resolution.md): Resolve explicit strategies and `Arbitrary<T>` evidence in the runner.
- [TASK-1491](../plan/tasks/TASK-1491-quickcheck-generation-and-shrinking-execution.md): Execute strategy generation/shrinking and record repro artifacts.
- [TASK-1492](../plan/tasks/TASK-1492-law-property-enforcement-and-cache-schema.md): Split law/property outcomes and add evidence cache schema.
- [TASK-1493](../plan/tasks/TASK-1493-quickcheck-final-surface-fixtures.md): Add no-Cargo fixtures for defaults, overrides, and failing shrink cases.
- [TASK-1494](../plan/tasks/TASK-1494-quickcheck-documentation-cookbook.md): Write documentation/cookbook examples for Arbitrary, Strategy, overrides, composition, shrinking, and law/property behavior.
- [TASK-1495](../plan/tasks/TASK-1495-quickcheck-future-backends-design-note.md): Validate and link the future-backend design note.
- [TASK-1496](../plan/tasks/TASK-1496-quickcheck-closeout.md): Close out the phase, reconcile statuses, and run broad verification.

## Changelog

### 2026-06-15

- Created this implementation-grade planning specification for QuickCheck-like `test::quickcheck` property testing.
