# DESIGN-NOTE: QuickCheck v1 Ordinary Strategy Semantics

**Status:** Frozen design decisions for next implementation phase
**Date:** 2026-06-15
**Supersedes / hardens:** [DESIGN-NOTE: QuickCheck-Style Property Testing and Future Evidence Families](DESIGN-NOTE-QUICKCHECK-PROPERTY-TESTING.md)
**Spec:** [SPEC-087: QuickCheck v1 Ordinary Strategy Semantics](../spec/SPEC-087-QUICKCHECK-V1-ORDINARY-STRATEGY-SEMANTICS.md)
**Plan:** [PLAN-151: QuickCheck v1 Ordinary Strategy Semantics](../plan/PLAN-151-QUICKCHECK-V1-ORDINARY-STRATEGY-SEMANTICS.md)

## Summary

This note records the frozen design decisions from the Phase 150 follow-up design session. Phase 150 delivered a useful QuickCheck-like MVP through metadata/runner bridges. The next phase removes those semantic bridges and specifies the target Ash-facing model:

- `Strategy<A>` is an ordinary pure Ash value with callable `gen` and `shrink` fields.
- `GenContext` is the generator-visible configuration object and trace/cache/replay anchor.
- QuickCheck generation samples one value per context: `gen : GenContext -> A`.
- Small/enumerated domains are future `SmallCheck` / `Series` work, not QuickCheck batch generation.
- `Arbitrary<A>` is minimal ordinary evidence: `arbitrary() -> Strategy<A>`.
- Strategy overrides are pure Ash expressions of type `Strategy<A>`.
- QuickCheck uses stable, versioned RNG/split semantics, random seeds by default, and externally supplied replay seeds.
- Positive empirical evidence accumulates as run history; counterexamples/errors remain active findings while identity-compatible.

## Frozen decisions

### D1. Strategy is an ordinary pure value

Target shape:

```ash
pub type Strategy<A> = Strategy {
    gen: GenContext -> A,
    shrink: A -> List<A>,
}
```

`Strategy<A>` is not an interface-only concept, special declaration form, closed DSL AST, or trusted string ID. Runner hooks may optimize execution, but the semantic authority is the ordinary Ash value.

Hard validity requirements: `gen` and `shrink` are pure, deterministic, type-preserving, and `shrink` returns a finite ordered list. Quality obligations for public strategies are domain-preserving shrink, progress toward simpler values, documented size interpretation, useful distribution, and stable behavior under the declared RNG/split version. No completeness or globally minimal shrinking guarantee is implied.

### D2. GenContext is helper-first and generator-visible only

`GenContext` is the generator-visible configuration value for one candidate and the trace/cache/replay anchor. Public examples use helpers rather than field arithmetic:

```ash
qc::context::size(ctx)
qc::context::seed(ctx)
qc::context::split(ctx, 0)
qc::context::variant(ctx, "left")
qc::context::indexed(ctx, "elem", i)
qc::context::resize(ctx, new_size)
```

`seed` is semantically deterministic RNG state even if represented as `Int` in the first implementation. `case_index` is runner trace metadata, not part of generator-visible `GenContext`.

### D3. QuickCheck gen returns one value

QuickCheck is sampled property testing:

```ash
gen: GenContext -> A
```

`GenContext -> List<A>` belongs to a future SmallCheck/Series-style backend for bounded/enumerated domains.

### D4. Size is strategy-specific

`size` is not a universal hard bound. Each public strategy documents how it interprets size. `qc::int::bounded(min, max)` may ignore size; `qc::list::of(elem)` may use size for length and document element resizing; recursive combinators use size to bound recursive self calls. `size_policy` is documentation-only in v1, not a trusted `Strategy` field.

### D5. Arbitrary is minimal ordinary evidence

```ash
pub interface Arbitrary<A> {
    arbitrary() -> Strategy<A>
}
```

`Arbitrary<A>` supplies the canonical/default strategy for `A`. Alternate domains are ordinary functions/values returning `Strategy<A>`. Resolution uses ordinary in-scope evidence only; there is no hidden runner-global primitive/container fallback registry.

Stdlib evidence is imported explicitly, normally via:

```ash
use test::quickcheck::prelude
use test::quickcheck as qc
```

### D6. Overrides are pure strategy expressions

QuickCheck `with { ... }` blocks are parameter/domain override maps only:

```ash
proof division_safe(x: Int, y: Int) {
    by test quickcheck cases 100 with {
        y <- strategy qc::int::nonzero()
    }
}
```

Both `x <- strategy expr` and `x <- expr` are accepted; both require a pure `Strategy<T>` expression for parameter `x: T`. Partial overrides fall back to ordinary `Arbitrary<T>` evidence. Missing evidence fails closed.

### D7. Stable RNG/split and seed policy

QuickCheck v1 must publish a versioned deterministic RNG/split contract such as `ash-quickcheck-rng-v1`, with golden vectors for root seeds, split paths, and choices. Helpers include `split`, `variant`, `indexed`, `resize`, `choose_int`, and `choose_bool`.

Seeds are random by default and always recorded. External CLI/replay seed overrides source seed. Source `seed` is allowed but discouraged/linted. Source `cases N` is exact evidence budget and is not silently overridden.

### D8. Namespace is submodule-first with alpha root aliases

Canonical organization:

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

Root aliases such as `qc::positive_ints()` and `qc::list_of(...)` are alpha convenience aliases over canonical submodule APIs.

### D9. Combinators are namespaced functions

Canonical APIs are functions such as `qc::combinator::map`, `map2`, `one_of`, `one_of_weighted`, `recursive`, and `recursive_with`. Method syntax may become sugar later, but v1 does not depend on it.

### D10. Weighted choice and bounded recursion are v1 features

`Weighted<A>` is generic but QuickCheck-local. `one_of` / `one_of_weighted` accept ordinary lists and fail closed on empty lists or non-positive weights. Validation happens at construction where possible and at consumption.

Recursive generation includes default and configurable APIs:

```ash
qc::combinator::recursive(base, expand)
qc::combinator::recursive_with(base, expand, config)
```

Config has `base_weight > 0`, `expand_weight >= 0`, and `size_step > 0`. At `size <= 0`, use `base`. At positive sizes, choose between `base` and `expand(smaller_self)` using constant weights; the self strategy passed to `expand` is guarded by size descent.

### D11. Shrinking is simple and explicit

V1 does not use hidden generation provenance for automatic structural shrinking. Plain `map`/`map2` have empty/conservative shrinkers. Explicit helpers include:

```ash
map_with_shrink
map_project       -- B -> Option<A>
map2_with_shrink
map2_project      -- C -> Option<(A, B)>
with_shrink       -- replacement
append_shrink     -- existing ++ extra
prepend_shrink    -- extra ++ existing
```

`map2_project` shrinks fields through the constructor only. Recursive child replacement such as `Add(a,b) -> a | b` is written explicitly in final-domain shrinkers. The runner preserves shrink candidate order exactly and does not deduplicate.

### D12. Runner failures and evidence history

Execution failure classes for shrink preservation are `property_false`, `runtime_error`, and `timeout`. A shrink candidate is accepted only if it reproduces the original class. `gen(ctx)` errors are generator/setup errors with repro context and no v1 shrinking. `shrink(value)` errors preserve the original property failure by default while recording shrink error; strict law policy may escalate.

Every QuickCheck run records seed, seed source, cases, source/evidence identity, RNG/backend versions, and outcome. Compatible positive runs aggregate into passed runs/cases. Counterexamples and errors are active findings while identity-compatible and do not erase each other. A later pass does not clear a compatible error. Same-seed divergent outcomes are nondeterminism errors. Active aggregate compatibility uses broad conservative identity in v1; narrow structural/function identity is future work.
