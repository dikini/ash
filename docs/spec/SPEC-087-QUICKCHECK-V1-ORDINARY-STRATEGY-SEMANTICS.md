# SPEC-087: QuickCheck v1 Ordinary Strategy Semantics

**Status:** Implemented MVP (Phase 151; Phase 176 recursive-combinator cleanup); bounded recursive generation remains fail-closed/deferred
**Date:** 2026-06-15
**Builds on:** [SPEC-081](SPEC-081-LAW-TEST-EVIDENCE-SUBSTRATE.md), [SPEC-082](SPEC-082-PROPERTY-GENERATION-AND-SHRINKING-SUBSTRATE.md), [SPEC-086](SPEC-086-QUICKCHECK-ARBITRARY-STRATEGY.md)
**Design note:** [DESIGN-NOTE: QuickCheck v1 Ordinary Strategy Semantics](../design/DESIGN-NOTE-QUICKCHECK-V1-ORDINARY-STRATEGY-SEMANTICS.md)
**Plan:** [PLAN-151: QuickCheck v1 Ordinary Strategy Semantics](../plan/PLAN-151-QUICKCHECK-V1-ORDINARY-STRATEGY-SEMANTICS.md)

## Summary

This specification hardens QuickCheck from the Phase 150 metadata/runner-bridge MVP into the target ordinary-Ash model. QuickCheck v1 uses ordinary pure `Strategy<A>` values, minimal in-scope `Arbitrary<A>` evidence, pure per-parameter strategy overrides, deterministic versioned RNG/split semantics, bounded recursive combinators, explicit/simple shrinkers, and aggregate empirical evidence history.

## Scope

### In scope

- Ordinary `Strategy<A>` values with callable `gen` and `shrink` fields.
- `GenContext` helper-first API and versioned deterministic RNG/split helpers.
- Minimal `Arbitrary<A>` evidence and explicit prelude imports.
- Parser/typechecker support for `by test quickcheck with { ... }` strategy overrides.
- Strategy resolution without hidden runner-global primitive/container fallback.
- Submodule-first `test::quickcheck` namespace layout and alpha root aliases.
- Core strategy combinators, including weighted choice and bounded recursion.
- Simple shrink semantics, explicit shrink wrappers, projection helpers, and failure-class-preserving shrink acceptance.
- Random-by-default seed policy, external replay override, and source-seed linting.
- Run records and compatible aggregate empirical evidence history.

### Out of scope

- SmallCheck/series enumeration implementation.
- Solver/proof-producing evidence.
- Automatic `Arbitrary` derivation.
- Effectful/capability-bearing generators.
- Hidden generation provenance for automatic structural shrinking.
- Global runner registry/fallback for default strategies.
- Narrow compiler structural/function identity for evidence aggregation.

## Normative model

### Strategy

`Strategy<A>` is an ordinary Ash value:

```ash
pub type Strategy<A> = Strategy {
    gen: GenContext -> A,
    shrink: A -> List<A>,
}
```

A valid strategy requires pure, deterministic, type-preserving generation and shrinking. `shrink(a)` returns a finite ordered list. Domain preservation, progress, useful distribution, and size-policy documentation are public strategy quality obligations rather than generic typechecker proofs.

### GenContext

`GenContext` is the generator-visible configuration value for one candidate and the trace/cache/replay anchor. Public examples use helper-first access:

```ash
qc::context::size(ctx)
qc::context::seed(ctx)
qc::context::split(ctx, 0)
qc::context::variant(ctx, "left")
qc::context::indexed(ctx, "elem", i)
qc::context::resize(ctx, new_size)
qc::context::choose_int(ctx, min, max)
qc::context::choose_bool(ctx)
```

`seed(ctx)` is debug/repro visibility, not permission to define stable generation through arbitrary seed arithmetic. `case_index` is runner trace metadata, not generator-visible state.

### Generation cardinality and size

QuickCheck generation samples one value per context:

```ash
gen: GenContext -> A
```

`GenContext -> List<A>` belongs to a future SmallCheck/Series-style backend. `size` is strategy-specific. Public stdlib strategies must document size interpretation. The core `Strategy<A>` type has no `size_policy` metadata field in v1.

### Arbitrary

`Arbitrary<A>` provides the canonical/default strategy for `A`:

```ash
pub interface Arbitrary<A> {
    arbitrary() -> Strategy<A>
}
```

Default parameter resolution is: explicit `with` override, otherwise ordinary in-scope `Arbitrary<T>` evidence, otherwise fail closed. `by test quickcheck` does not inject hidden primitive/container generators. `property` and `quickcheck` are synonymous surface vocabulary for the same proof evidence mode; only one AST representation (`ProofBody::ByTestProperty`) should exist, extended with an optional `strategies` payload.

Stdlib defaults are imported explicitly:

```ash
use test::quickcheck::prelude
use test::quickcheck as qc
```

## Override syntax

QuickCheck `with { ... }` blocks are parameter-strategy override maps only. Run configuration remains outside the block.

Accepted forms:

```ash
x <- strategy expr
x <- expr
```

Both require a generated parameter `x: T`, no duplicate binding, pure RHS, and `expr : Strategy<T>`.

`by test property` and `by test quickcheck` are synonymous surface spellings for the same proof evidence mode. The parser accepts both; the AST and runner schema have one representation (`ProofBody::ByTestProperty` with optional `strategies`). Source spelling is preserved for diagnostics only.

Example:

```ash
proof division_safe(x: Int, y: Int) {
    by test property with {
        y <- qc::int::nonzero()
    }
}
```

Partial overrides are allowed; unspecified parameters use ordinary `Arbitrary<T>` evidence.

## Namespace and prelude

Canonical modules:

```text
test::quickcheck::context
test::quickcheck::strategy
test::quickcheck::arbitrary
test::quickcheck::int
test::quickcheck::bool
test::quickcheck::string
test::quickcheck::list
test::quickcheck::combinator
test::quickcheck::prelude
```

Root aliases are alpha convenience API over canonical submodule paths.

## Combinators

Canonical combinators are namespaced functions.

Choice/weights:

```ash
qc::combinator::one_of(strategies: List<Strategy<A>>) -> Strategy<A>
qc::combinator::weighted(weight: Int, value: A) -> Weighted<A>
qc::combinator::one_of_weighted(choices: List<Weighted<Strategy<A>>>) -> Strategy<A>
```

`one_of` / `one_of_weighted` lists must be non-empty and weights positive. Invalid constants are rejected at check/admission when detectable; dynamic invalid values fail closed. `Weighted<A>` is QuickCheck-local in v1.

Map/project helpers:

```ash
qc::combinator::map(source, f)
qc::combinator::map_with_shrink(source, f, shrink_b)
qc::combinator::map_project(source, f, project: B -> Option<A>)
qc::combinator::map2(left, right, f)
qc::combinator::map2_with_shrink(left, right, f, shrink_c)
qc::combinator::map2_project(left, right, f, project: C -> Option<(A, B)>)
```

Plain map-like combinators use empty/conservative shrinkers. Projection helpers reuse source shrinkers through supplied `Option` projectors. `map2_project` shrinks fields through the constructor only.

Recursive generation:

```ash
qc::combinator::recursive(base, expand)
qc::combinator::recursive_with(base, expand, config)
qc::combinator::recursive_config(base_weight, expand_weight, size_step)
qc::combinator::default_recursive_config()
```

Constraints: `base_weight > 0`, `expand_weight >= 0`, `size_step > 0`. At `size <= 0`, use `base`. At positive sizes, choose between `base` and `expand(smaller_self)` using constant weights. The self strategy passed to `expand` is guarded by size descent.

Shrink wrappers:

```ash
qc::combinator::with_shrink(strategy, shrink)      -- replacement
qc::combinator::append_shrink(strategy, extra)     -- existing ++ extra
qc::combinator::prepend_shrink(strategy, extra)    -- extra ++ existing
```

The runner preserves shrink candidate order exactly and does not deduplicate.

## RNG, seeds, cases, and replay

QuickCheck must define a versioned RNG/split contract. The implementation task must select `ash-quickcheck-rng-v1` and add golden vectors for root seeds, split paths, and chosen values.

Case count: default `cases = 100`; source `cases N` is exact; precedence is source cases > CLI/project cases > default 100.

Seed: random by default; effective seed always recorded; source `seed N` is allowed but discouraged/linted; explicit CLI/replay seed overrides source seed; explicit random-seed run policy may override source seed.

## Runner semantics

QuickCheck stops at the first failing generated case by default, then shrinks that failure.

Execution failure classes for shrink preservation:

```text
property_false
runtime_error
timeout
```

A shrink candidate is accepted only if it reproduces the original failure class. Mismatched candidates are recorded and skipped.

Setup/evidence errors do not enter ordinary shrink:

```text
missing_evidence
invalid_config
generator_error
shrink_error
strategy_type_error
impure_strategy
invalid_override
unknown_binding
duplicate_binding
wrong_strategy_type
```

If `gen(ctx)` errors, report `generator_error` with repro context and no v1 shrink. If `shrink(value)` errors after a valid counterexample exists, preserve the original property failure by default and record `shrink_status = error`; strict law/evidence policy may escalate.

## Evidence history and active findings

Every QuickCheck run records exact run data: seed, seed source, case count, source/evidence identity, RNG/backend versions, outcome, and replay data for failures.

Aggregate evidence rolls up compatible runs with passed run/case counts, case buckets, latest run cases, and active finding flags/counters. Positive aggregate condition requires no compatible counterexample, no compatible error, and no nondeterminism.

Counterexamples and errors are both active findings while identity-compatible. They do not erase each other. A later pass does not dilute a compatible error or counterexample. Same-seed divergent outcomes under the same compatible identity are nondeterminism and must be reported as an error.

Active aggregate compatibility uses broad conservative identity in v1. Case counts are run metadata, not a hard aggregate separator; different case budgets roll up into compatible aggregate history while preserving per-run exact counts.

## Acceptance criteria

- Final-surface `.ash` examples use ordinary `Strategy<A>` values and in-scope `Arbitrary<A>` evidence, not runner-global fallback registries.
- `with { ... }` supports explicit and inferred strategy overrides and rejects wrong/missing/duplicate/impure overrides fail-closed.
- RNG/split golden vectors prove deterministic stable replay.
- Recursive combinator tests prove size-bounded generation and invalid config rejection.
- Shrink tests prove ordered candidate preservation, no automatic dedup, and failure-class-preserving acceptance.
- Evidence-history tests prove random seed recording, exact source cases, aggregate pass rollup, sticky errors, active counterexample/error coexistence, and invalidation on broad identity change.
- No-Cargo `$ASH_UNDER_TEST test ...` fixtures demonstrate final user-facing behavior.
