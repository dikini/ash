# Review Report: DESIGN-NOTE-INTERFACE-LAWS.md

**Reviewer:** Hermes Agent  
**Date:** 2026-06-08  
**File:** `/home/dikini/Projects/ash/docs/design/DESIGN-NOTE-INTERFACE-LAWS.md`  
**Status:** Draft  
**Scope:** Syntax validity, semantic correctness, clarity, usability, safety, versioning

---

## Executive Summary

This design note proposes `law` and `proof` syntax for Ash interfaces and modules. The design is theoretically sound and well-motivated, but contains **critical syntax inconsistencies** with live Ash syntax, **semantic gaps** around effect levels and proof totality, and **internal contradictions** that must be resolved before implementation. The document also overreaches in several areas (effect laws, proof traces) while under-specifying critical areas (law caching in `ash.lock`, incremental checking invalidation).

---

## Critical Issues (Must Fix)

### C1: Interface method signatures use wrong syntax (§3.1, §3.2, §3.3)
**Lines:** 47–48, 72–75, 96–98

The design note writes:
```ash
pub interface Semigroup<A> {
    append(a: A, b: A) -> A
```

But live Ash stdlib interfaces use **no parameter names in interface declarations**, only types:
```ash
pub interface Semigroup<A> {
    append(A, A) -> A
}
```
(See `std/src/algebra/semigroup.ash`, `std/src/algebra/monad.ash`.)

Similarly, the Monad law examples write:
```ash
    unit(A) -> M<A>
    bind(M<A>, A -> M<B>) -> M<B>
```

But live syntax is:
```ash
    unit(A) -> M<A>
    bind(M<A>, A -> M<B>) -> M<B>
```

Actually, checking `std/src/algebra/monad.ash`:
```ash
pub interface Monad<M : * -> *> where M: Applicative {
    unit(A) -> M<A>
    bind(M<A>, A -> M<B>) -> M<B>
}
```

The design note's `unit(A) -> M<A>` matches, but `bind(M<A>, A -> M<B>) -> M<B>` uses `A -> M<B>` (bare arrow) where live Ash callable syntax is parenthesized `(A) -> M<B>` per SPEC-072/TASK-957. This is a **syntax mismatch**.

**Fix:** All interface method signatures and law parameter types must use live Ash syntax: parenthesized callable types `(A) -> M<B>`, not bare `A -> M<B>`.

---

### C2: `fn(x) { ... }` lambda syntax is inconsistent with preferred closure syntax
**Lines:** 87, 103

The design note uses `fn(x) { bind(f(x), g) }` inside law propositions. While `fn(x) { ... }` parses (per TASK-556), the **preferred** pure closure syntax per SPEC-072/TASK-959 is `|x| -> body`. The design note should use preferred syntax in normative examples, or at minimum acknowledge the two forms.

**Fix:** Use `|x| -> bind(f(x), g)` in law examples, or add a note that `fn(x) { ... }` and `|x| -> ...` are equivalent.

---

### C3: `impl` block syntax mismatch — missing `pub` and wrong method syntax
**Lines:** 112–120, 124–133

The design note writes:
```ash
impl Semigroup<String> {
    append(a, b) = string::concat(a, b)
```

Live Ash `impl` blocks use `pub impl` and method syntax matches interface declarations:
```ash
pub impl Semigroup<String> {
    append(a, b) = string::concat(a, b)
}
```
(See `std/src/string.ash` line 8: `pub impl Semigroup<String> {`.)

Also, the design note's `append(a, b) = ...` uses parameter names without types. Live impls also omit types (they're inherited from the interface), so this part is acceptable, but the missing `pub` is inconsistent with stdlib conventions.

**Fix:** Add `pub` to all `impl` examples, or explicitly state that `impl` visibility follows interface visibility rules.

---

### C4: `proof` block placement is ambiguous for interface laws
**Lines:** 112–120, 124–133

The design note places `proof` blocks **inside** `impl` blocks for interface laws. This is clear. But for module laws (§7.3), it places them at module scope. The distinction is correct per §7.5, but the syntax for module-scope proofs is never formally specified — only shown in examples.

**Fix:** Add a formal BNF-style syntax rule for module-scope `proof` declarations, matching the interface `proof` syntax in §3.4.

---

### C5: Contradiction on law proposition purity (§8 vs §3.5)
**Lines:** 407, 151–158

§8 (Open Questions) resolves: "Must law propositions reference only Pure functions? **Resolved: Yes.** Law propositions are `Prop`-typed and must reference only `Pure` functions."

But §3.5 shows:
```ash
impl Monad<Act<A>> {
    proof act_associativity(ma, f, g, equiv) by test {
        generator: bounded_act_generator(),
        ...
    }
}
```

This `proof` references `Act<A>` values and `bind` — which are **not** `Pure`. If law propositions must be `Pure`, then a proof about `Act` monad associativity cannot be written as a pure expression. The `by test` delegation is a synthetic test, not a pure proof, but the design note says proofs must be "pure, total, terminating expressions" (§3.4, line 143).

This is a **fundamental contradiction**: either (a) tower carrier proofs are impossible as pure expressions and must always use `by test`, or (b) the purity restriction is too strong. The design note must resolve this.

**Fix:** Explicitly state that tower carrier laws require `by test` (synthetic testing) because their propositions reference effectful computations. Only `Pure` carrier laws may use `by_definition` or handwritten pure proofs. This aligns with D1 (different equivalence relations stay different).

---

### C6: `BoundedEquiv` return type `EquivResult` is not a `Bool`
**Lines:** 172–181

The design note defines:
```ash
pub interface BoundedEquiv<C<A>> {
    equiv(ca: C<A>, cb: C<A>) -> EquivResult
}
```

Where `EquivResult` is an enum with `Equal`, `NotEqual { counterexample: Trace }`, `Timeout`.

But law propositions are typed as `Prop` — uninhabited types with no runtime value. A law body like:
```ash
equiv.equiv(bind(bind(ma, f), g), bind(ma, fn(x) { bind(f(x), g) }))
```

would have type `EquivResult`, not `Prop`. The design note never explains how `EquivResult` converts to `Prop`. Is there an implicit coercion? A wrapper? This is a **type system gap**.

**Fix:** Define the conversion explicitly. Options:
1. `EquivResult` implements a method `is_equal() -> Bool`, and the law is `equiv(...).is_equal()`.
2. Laws accept any expression that reduces to a truth value, with `EquivResult` having special handling.
3. Introduce a `Prop`-producing equivalence wrapper.

---

## Warnings (Should Fix)

### W1: `#[trace]` and `#[no_test]` are hypothetical with no parser support
**Lines:** 204, 496

The design note introduces `#[trace]` and `#[no_test]` attributes. The context explicitly states these are "hypothetical attribute syntax (not yet in language)." This is acceptable for a design note, but the document should more clearly mark these as **proposed** rather than accepted syntax. §11 uses `#[trace]` extensively without any "proposed" marker.

**Fix:** Add a "Proposed" or "Stage 2+" marker to all attribute syntax, or move attributes to an "Open Questions" section.

---

### W2: `ash.lock` evidence caching is under-specified
**Lines:** 204–205, 410

The design note says: "Test results may be cached in a manifest (`ash.lock`) with seed and timestamp." But `ash.lock` per SPEC-073 is a **dependency lockfile** recording exact git commit resolution and package metadata. It has no schema for law test results.

Caching test results in `ash.lock` would:
1. Pollute the dependency lockfile with test metadata.
2. Create merge conflicts on every test run.
3. Violate SPEC-073's stability requirement ("MUST be stable enough for review diffs").

**Fix:** Use a separate file (e.g., `.ash/law-cache.toml` or `ash-law.lock`) for cached test results. Document the schema and invalidation rules explicitly.

---

### W3: Source hash for cache invalidation is impractical
**Lines:** 410

"`ash.lock` entries include a hash of the law's source text and the source text of all functions it references."

This requires tracking the **transitive closure** of all functions referenced by a law. In a module with deep call graphs, this is expensive and fragile. A change to a private helper's implementation (not its contract) would invalidate all downstream law caches.

**Fix:** Use a coarser-grained invalidation strategy:
- Module-level source hash (cheaper, slightly over-invalidates).
- Or track only **public API signatures** for invalidation, not implementations.
- Or make cache invalidation explicit via `ash check --invalidate-laws`.

---

### W4: Semantic versioning claim is too strong
**Lines:** 411

"Adding, removing, or changing a law is a breaking change for the module's public API."

This is **overly conservative**. Adding a law is a **backward-compatible** change for consumers — it imposes new obligations on the *implementor*, not the *caller*. Rust traits can add default methods without breaking semver. Similarly, adding a law with a default synthetic-test proof should not break consumers.

Removing or changing a law *is* breaking. But adding a law should be **minor version** bump, not major.

**Fix:** Distinguish:
- Adding a law: minor version bump (new obligation, backward-compatible for callers).
- Removing/changing a law: major version bump (breaking change).

---

### W5: Module law proof visibility is confusing
**Lines:** 323–335, 347–358

The design note says proof terms are **not exported** (D8), but law declarations **are** exported (D6). The table in §7.5 says "Proof export: N/A — proofs live in `impl`" for interface laws, but "Proof terms are not exported; only results cross boundaries" for module laws.

This creates an asymmetry: interface law proofs live in `impl` blocks (which are already scoped to a type), while module law proofs live at module scope. But what if a module law is proven in a **test module** (line 304: "In the same module, or in a test module")? Can test modules prove module laws? If so, the proof result must somehow be published back to the original module.

**Fix:** Clarify the test module → original module proof result propagation mechanism. Or restrict module law proofs to the declaring module only.

---

### W6: `ActMonad` interface example is malformed
**Lines:** 96–104

```ash
pub interface ActMonad {
    law act_associativity<A, B, C>(
        ma: Act<A>,
        f: A -> Act<B>,
        g: B -> Act<C>,
        equiv: BoundedEquiv<Act<C>>
    ) : equiv.equiv(...)
}
```

This declares `ActMonad` as a standalone interface with no methods, only a law. But laws are supposed to reference **interface methods** (line 44: "It references interface methods (bound in scope)"). There are no methods in `ActMonad` for the law to reference.

Also, `ActMonad` should presumably require `Monad<Act>` evidence, but it doesn't declare that constraint.

**Fix:** Either:
1. Make `ActMonad` an `impl` block law (not an interface law), or
2. Add `where Self: Monad<Act>` constraint and reference `bind` from the `Monad` interface, or
3. Remove the `ActMonad` example and use `impl Monad<Act>` with a `proof` block instead.

---

### W7: `from_string_round_trip` law is trivially true
**Lines:** 286–288

```ash
law from_string_round_trip(s: String, eq: Eq<PathBuf>)
  : eq.equiv(from_string(s), from_string(s))
```

This is `eq.equiv(x, x)` — reflexivity, always true. The intended law was probably something about `to_string(from_string(s)) == s` or `from_string(to_string(p)) == p`, but `from_string` alone has no round-trip property.

**Fix:** Rewrite to a meaningful law, or remove if no inverse function exists.

---

### W8: `join_preserves_absolute` law uses `if/then/else` without specifying syntax
**Lines:** 276–279

```ash
law join_preserves_absolute(base: PathBuf, child: String, eq: Eq<PathBuf>)
  : if is_absolute(base)
    then is_absolute(join(base, child))
    else true
```

Ash has no documented `if/then/else` expression syntax at the value level (only `if ... { ... } else { ... }` blocks). The design note invents `if/then/else` inline expression syntax without referencing any spec.

**Fix:** Use `match` or a block-style `if` expression, or explicitly propose `if/then/else` as new syntax.

---

## Suggestions (Nice to Have)

### S1: Add syntax for default law proofs
**Lines:** 109–133

The design note says "If omitted, the law remains unproven and synthetic tests apply." But there's no syntax for a default `by_definition` or `by test` proof at the interface level. Rust traits support default method implementations; similarly, laws could have default proofs:

```ash
pub interface Semigroup<A> {
    append(a: A, b: A) -> A

    law associativity(a, b, c, eq) : ...
        default by test
}
```

This would reduce boilerplate for common cases.

---

### S2: Distinguish `law` as declaration vs `law` as proposition more clearly
**Lines:** 46–68

The word "law" is overloaded: it's the keyword for the declaration, the named proposition, and the conceptual algebraic law. Consider using `prop` or `theorem` for the declaration keyword to avoid confusion, or consistently use "law declaration" vs "law proposition" vs "algebraic law."

---

### S3: Proof trace depth limit should be configurable per-law, not just global
**Lines:** 510–528

The design note allows `#[trace(max_depth = 100)]` per proof. This is good. But it should also allow per-law default limits in the interface declaration:

```ash
law associativity(a, b, c, eq)
  : ...
  trace max_depth 50
```

---

### S4: Add a section on law documentation generation
**Lines:** 245–260

Module laws are motivated partly by documentation (§7.1: "Documentation: what guarantees does the module provide as a whole?"). But the design note doesn't specify how laws appear in generated docs. Do they render as equations? As prose? With proof status badges?

---

### S5: Effect laws section (§12) is too speculative
**Lines:** 575–626

Section 12 on "Effect-Related Laws and Capability Integration" introduces hypothetical syntax (`requires`, `ensures`, `effect_level`) with no grounding in existing Ash effect syntax. This section should be marked "Highly Speculative — Future Design Note" or moved to a separate document. It risks confusing readers about what is planned vs. what exists.

---

## Approval (What's Good)

### A1: Strong theoretical foundation
The Curry-Howard correspondence framework (§6) is well-explained and provides a coherent long-term vision without overcommitting to dependent types.

### A2: Explicit equivalence relations (D1)
Requiring explicit `eq` or `equiv` parameters in laws is the right design. It prevents silent semantic drift when types move between effect levels.

### A3: Proof locality (D2, D7, D8)
Keeping proofs local to `impl` blocks, allowing private function references in proofs, and not exporting proof terms are all sound decisions that protect implementation details.

### A4: Clear staging
The Stage 1–4 roadmap (§5) is realistic and avoids overcommitting. Stage 1 (parse and store) is immediately implementable.

### A5: Module laws are well-motivated
§7.1's motivation for module-scoped laws (API consistency, refactoring safety) is compelling and fills a real gap.

### A6: Synthetic test integration
Delegating unproven laws to synthetic tests (§5.2) is pragmatic. The explicit `by test` syntax makes the delegation visible.

---

## Internal Inconsistencies

| # | Inconsistency | Location | Severity |
|---|---------------|----------|----------|
| I1 | Law propositions must be Pure (§8) but tower carrier proofs reference Act/Proc (§3.5) | Lines 151–158, 407 | Critical |
| I2 | `ash.lock` is a dependency lockfile (SPEC-073) but used for test result caching (§5.2) | Lines 204–205 | Critical |
| I3 | `ActMonad` interface has laws but no methods to reference | Lines 96–104 | Warning |
| I4 | `from_string_round_trip` is trivially reflexive, not a round-trip | Lines 286–288 | Warning |
| I5 | `if/then/else` expression syntax is used but not specified anywhere in Ash | Lines 276–279 | Warning |
| I6 | §8 says "Resolved: Yes" to proof names being statically checked, but §5.1 says "no totality checking" in Stage 1 | Lines 189–193, 405 | Minor |
| I7 | `BoundedEquiv` returns `EquivResult`, but law propositions are `Prop` — no conversion rule given | Lines 172–181 | Critical |

---

## Recommendations

1. **Freeze syntax audit:** Before any Stage 1 implementation, run a full syntax audit against `std/src/algebra/*.ash` and the live parser to ensure all examples use accepted Ash syntax.

2. **Resolve C5 (purity contradiction):** Explicitly carve out tower carrier laws as `by test`-only, with `Pure` carrier laws as the only ones eligible for `by_definition` or handwritten proofs.

3. **Separate cache file:** Move law test result caching out of `ash.lock` into a dedicated `.ash/law-cache.toml` or similar.

4. **Fix `ActMonad` example:** Either make it an `impl` block or add the required `Monad` constraint and methods.

5. **Clarify `EquivResult → Prop` conversion:** Add a typing rule or wrapper function.

6. **Soften semver claim:** Distinguish adding laws (minor) from removing/changing (major).

7. **Mark §12 as speculative:** Add a prominent disclaimer that effect laws are not planned for Stage 1–3.

8. **Add formal syntax for module-scope proofs:** Match the rigor of §3.4's proof syntax.

---

*End of review.*
