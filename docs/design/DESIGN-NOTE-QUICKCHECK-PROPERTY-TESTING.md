# DESIGN-NOTE: QuickCheck-Style Property Testing and Future Evidence Families

**Status:** Design note / future reference
**Date:** 2026-06-15
**Related:** [SPEC-082](../spec/SPEC-082-PROPERTY-GENERATION-AND-SHRINKING-SUBSTRATE.md), [SPEC-086](../spec/SPEC-086-QUICKCHECK-ARBITRARY-STRATEGY.md), [SPEC-085](../spec/SPEC-085-PROOF-PRODUCING-SYNTHESIS-TODO-SPEC.md)
**Superseded / hardened by:** [DESIGN-NOTE: QuickCheck v1 Ordinary Strategy Semantics](DESIGN-NOTE-QUICKCHECK-V1-ORDINARY-STRATEGY-SEMANTICS.md) and [SPEC-087](../spec/SPEC-087-QUICKCHECK-V1-ORDINARY-STRATEGY-SEMANTICS.md) for the ordinary `Strategy<A>` v1 target.

## Summary

Ash should grow a standard-library test subspace for property testing, starting with a QuickCheck-like backend under `test::quickcheck`. The end-state separates the logical proposition from the evidence backend:

- `property` is a test-runner proposition. Failing a property fails `ash test`; it does not make the program fail compilation.
- `law` is a required invariant. Refuting a law under an accepted evidence policy can invalidate the enclosing impl/module/admission boundary.
- `Strategy<T>` is a value-level generator/shrinker object for a particular test domain of `T`.
- `Arbitrary<T>` is interface evidence for the canonical/default `Strategy<T>` for `T`.

This note records the broader design space so the implementation spec can stay narrow while future SmallCheck, solver, proof-producing, and coverage/mutation phases remain coherent.

## Terminology

| Term | Meaning |
|---|---|
| `property` | Test-runner proposition. It is checked by `ash test`; failure is test failure. |
| `law` | Required semantic invariant. Accepted evidence may be cached; refutation can fail compilation/admission under policy. |
| `Strategy<T>` | Explicit generated subdomain plus shrink relation for `T`. Useful for overrides and composition. |
| `Arbitrary<T>` | Canonical/default QuickCheck-style evidence that supplies a default `Strategy<T>`. |
| `quickcheck` | Sampled property-testing backend using generated cases and shrinkers. |
| `smallcheck` | Future bounded exhaustive backend using finite/depth-indexed enumeration. |
| solver/proof evidence | Future non-test evidence families with proof artifacts, replay/checking, and stricter trust boundaries. |

## Target Namespace

The public surface should live under a standard-library `test` subspace, with backend-specific modules:

```ash
use test::quickcheck::{Arbitrary, Strategy}
```

Future modules:

```ash
use test::smallcheck::{Enumerable, Series}
use test::solver::{SolverProof}
```

This keeps test-generation library code in Ash rather than making the runner a bag of ad-hoc magic. The runner may still have compiler-known support for opaque carriers and execution hooks, but the author-facing surface is a library API.

## Target QuickCheck Shape

Illustrative final model:

```ash
pub opaque type Strategy<T>

pub interface Arbitrary<T> {
    arbitrary() -> Strategy<T>
    gen(Int, Int) -> List<T>
    shrink(T) -> List<T>

    law gen_matches_strategy(seed: Int, size: Int, eq: Eq<List<T>>):
        eq.equiv(gen(seed, size), strategy::gen(arbitrary(), seed, size))

    law shrink_matches_strategy(value: T, eq: Eq<List<T>>):
        eq.equiv(shrink(value), strategy::shrink(arbitrary(), value))
}
```

The exact syntax may change as Ash default methods, associated types, and qualified interface methods mature. Normatively, `gen` and `shrink` must be coherent projections of `arbitrary()`. Implementations must not provide unrelated generation, shrinking, and strategy behavior.

## Why Strategy Exists If Arbitrary Exists

Plain `Arbitrary<T>` is iffy because many properties need a semantic subdomain of the same type.

Example: sorted lists.

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

`Arbitrary<List<Int>>` may generate unsorted lists. The property needs the sorted-list subspace. A `Strategy<List<Int>>` expresses that local domain without replacing the default `Arbitrary<List<Int>>` instance.

Example: expressions.

```ash
pub type Expr =
    Lit { value: Int }
  | Var { name: String }
  | Add { left: Expr, right: Expr }
  | Div { left: Expr, right: Expr }

pub fn any_exprs() -> Strategy<Expr> { ... }
pub fn nonzero_denominator_exprs() -> Strategy<Expr> { ... }
```

Different properties choose different domains:

```ash
proof print_parse_roundtrip(e: Expr) {
    by test quickcheck
}

proof evaluator_total_on_safe_division(e: Expr) {
    by test quickcheck with {
        e <- strategy test::gens::nonzero_denominator_exprs
    }
}
```

`Arbitrary<Expr>` supplies the default. Explicit strategies override for domain-specific laws/properties.

## Strategy Composition Examples

Strategies compose by mapping, products, alternatives, and recursion. The examples below are illustrative API sketches, not a parser commitment.

```ash
pub fn lit_exprs() -> Strategy<Expr> {
    quickcheck::map(quickcheck::ints(), |n| -> Lit { value: n })
}

pub fn var_exprs() -> Strategy<Expr> {
    quickcheck::map(quickcheck::identifiers(), |name| -> Var { name })
}

pub fn add_exprs(child: Strategy<Expr>) -> Strategy<Expr> {
    quickcheck::map2(child, child, |left, right| -> Add { left, right })
}

pub fn any_exprs() -> Strategy<Expr> {
    quickcheck::recursive(
        quickcheck::one_of([lit_exprs(), var_exprs()]),
        |self| -> quickcheck::one_of([add_exprs(self)])
    )
}
```

Composition must preserve shrink structure. If `map(ints(), Lit)` generates `Lit { value: n }`, shrinking the expression maps integer shrinks back through `Lit`. For `map2`, shrinking can shrink either child and may also include constructor-specific simplifications such as replacing `Add(left, right)` with `left` or `right` when the strategy declares such a simplification.

## Law and Property Enforcement

Ash should maintain two near-synonyms:

```text
property = proposition used as a test
law      = proposition required as semantic evidence
```

Both may be written over the same expression language. The distinction is enforcement:

| Construct | Checked by | Failure consequence |
|---|---|---|
| `property` | `ash test` | test failure only |
| `law` with `by test quickcheck` | `ash test`, compile/admission evidence policy | refuted law evidence can invalidate impl/module/admission; stale/missing evidence depends on policy |
| `law` with solver/proof evidence | compiler/admission proof subsystem | refutation or invalid proof can fail compilation/admission |

A law need not rerun every empirical proof every compile. It needs acceptable non-stale evidence under the active policy.

## Evidence Cache Direction

Law evidence needs version-moderated caching to avoid repeating redundant proofs/tests.

Cache key material should include:

- compiler/test-runner version,
- stdlib/test backend version,
- source artifact hash,
- module/law/proof identity,
- proof body hash,
- `Arbitrary<T>` impl identities/hashes,
- explicit `Strategy<T>` identities/hashes,
- seed and size schedule,
- max cases / max shrink steps,
- evidence backend schema version.

A stale cache is not a refutation. Policy decides whether stale/missing evidence warns, defers, reruns, or fails closed.

## Future Backends

### SmallCheck

SmallCheck should live under `test::smallcheck` and use bounded exhaustive domains instead of sampled strategies.

Illustrative surface:

```ash
pub opaque type Series<T>

pub interface Enumerable<T> {
    series(Int) -> Series<T>
}
```

Evidence mode:

```ash
proof monoid_identity(x: T) {
    by test smallcheck depth 4
}
```

### Solver and Proof-Producing Synthesis

Solver/proof evidence is not a test backend. It should produce proof artifacts with replay/checking and a separate trust boundary.

Examples of future families:

```ash
proof associativity(x: T, y: T, z: T) {
    by solver z3
}

proof rewrite_preserves_semantics(e: Expr) {
    by synthesis bounded_depth 5
}
```

These are deliberately separate from `test::quickcheck` so empirical evidence and proof evidence cannot be confused.

### Coverage, Mutation, Flake Quarantine, Distributed Orchestration

These remain operational test-runner layers:

- coverage tells which properties/laws/branches/evidence families were exercised,
- mutation testing checks test/law sensitivity,
- flaky quarantine classifies unstable properties/tests,
- distributed orchestration shards and merges results.

They should consume the stable JSON evidence model rather than redefine generation semantics.

## Documentation Requirements

The eventual user documentation should include:

1. Quick start: define `Arbitrary<T>` and run a property.
2. Strategy override cookbook: sorted lists, nonzero denominators, valid vs invalid parser input.
3. Strategy composition guide: `map`, `map2`, `one_of`, `recursive`, `list_of`, `option_of`.
4. Shrinking guide: domain preservation, failure-preserving shrink acceptance, shrink traces.
5. Law vs property guide: when failure is test failure vs semantic invalidation.
6. Evidence cache guide: why laws do not rerun every proof every compile, and what invalidates cached evidence.
7. Backend roadmap: QuickCheck now; SmallCheck, solver/proofs, mutation/coverage/distribution later.

## Non-Goals for the First QuickCheck Phase

- no automatic derivation of `Arbitrary<T>`,
- no unrestricted source-world generation,
- no proof-producing synthesis,
- no full SmallCheck implementation,
- no global manifest registry as the primary model,
- no effectful generators in the MVP,
- no implicit generator success when evidence is missing.
