# NOTE-001: Workflow Computation Type (`comp T`)

## Status: Future Opportunity

## Summary

This note records the principled long-term design for a first-class **workflow computation type** (`comp T`) in Ash. It corresponds to **Option C3** in `DESIGN-028-STATEMENT-LIFTING.md`. The type is intentionally deferred from the MVP but is the natural target for a post-stabilization type-system overhaul aimed at JIT compilation and first-class monadic combinators.

## Motivation

Ash workflows are already semantically monadic:
- `return(v)` lifts a pure value into the neutral workflow context.
- `let x = stmt in ...` is sequential composition (`bind`).
- Metadata (effects, trace, provenance, obligations) accumulates monotonically across steps.

However, the current type system does not *represent* this monad explicitly. Workflows are distinguished syntactically (`workflow` vs `fn`), not typologically. A first-class `comp T` type would:

1. Make the monad explicit in the type system.
2. Enable generic combinators (`map`, `sequence`, `traverse`) over effectful computations.
3. Provide a rigorous compilation target for JIT backends.
4. Allow future modalities (`async T`, `par T`) to reuse the same infrastructure.

## Type Definition

```haskell
comp T   -- a computation that, when executed, produces a T
         -- and may accumulate effects, trace, provenance, and obligations
```

## Core Operations

### `return : T -> comp T`

Lifts a pure value into the computation monad with neutral metadata:
- effect: `Epistemic`
- trace: `[]`
- provenance: `empty`
- obligations: `pending=∅, discharged=∅`

### `bind : comp T -> (T -> comp U) -> comp U`

Sequences two computations, composing their metadata:
- value: `f(extract_value(wf))`
- effect: `join(ε₁, ε₂)` (least upper bound in the effect lattice)
- trace: `append(T₁, T₂)`
- provenance: `extend(π₁, π₂)`
- obligations: union of pending and discharged sets

### `fmap : (T -> U) -> comp T -> comp U`

Maps a pure function over a computation:

```haskell
fmap f c = bind(c, \v -> return(f(v)))
```

This is the typed equivalent of the pipe operator `|>` in `DESIGN-028`.

## Metadata Composition Laws

The workflow monad satisfies the standard monad laws because each metadata dimension is associative and has a neutral element:

| Law | Holds because |
|---|---|
| Left identity | Neutral metadata is the identity for join/append/extend/union. |
| Right identity | Same as above. |
| Associativity | `join` (semilattice), `append` (monoid), `extend` (tree monoid), and `union` (set monoid) are all associative. |

## Why `comp T` is Not in the MVP

1. **Type-system surface area.** Adding `comp T` requires changes to:
   - `Type` representation
   - Unification algorithm
   - Generic bounds (`T: Interface` inside `comp T` contexts)
   - Variance inference
   - Error message infrastructure

2. **User-facing complexity.** Every workflow variable would carry an implicit `comp` wrapper. Users must track `T` vs `comp T`, even for simple scripts.

3. **Blocking cost.** `comp T` is not an incremental refactor; it gates most other type-system features until complete.

## Trigger Conditions for Revisiting

Consider promoting this note to a formal design/spec when:
- The Ash surface syntax and big-step semantics (`SPEC-004`) are frozen.
- A JIT backend project begins and requires an explicit computation type as its IR boundary.
- Generic libraries over workflows (e.g., a standard `workflow-prelude`) become a priority.
- The current syntactic separation (C1) proves insufficient for advanced composition patterns.

## Related Documents

- `DESIGN-028-STATEMENT-LIFTING.md` — immediate MVP path using pipe operator + ANF lifting (C1)
- `DESIGN-027-SMALL-STEP-IR-COMPRESSION.md` — runtime alignment with an explicit state machine
- `SPEC-001-IR.md` — canonical IR contract
- `SPEC-004-SEMANTICS.md` — big-step semantics and metadata composition
