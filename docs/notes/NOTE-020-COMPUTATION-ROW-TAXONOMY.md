# NOTE-020: Computation Row Taxonomy and Pure Computation

**Date:** 2026-06-24
**Status:** Draft note -- taxonomy and terminology refinement
**Purpose:** Replace the narrower "effect row" framing with "computation row" as the
target Ash annotation space. The note records why the change is needed, what kinds of facts
can live in a computation row, and how this changes the definition of pure computation.

Companion to NOTE-013 through NOTE-019, NOTE-021, and SPEC-096b, SPEC-097b, SPEC-101, and
SPEC-102.

## 0. Motivation

The recent target-language notes use "effect row" for the row attached to computations.
That was useful while the central question was operation requirements and handler/provider
discharge. It is now too narrow.

Rows need to account for facts that are not ordinary algebraic effects:

1. lazy, memo, and eager evaluation modes;
2. contracts, laws, proof obligations, and evidence;
3. recoverable pure failures such as division by zero;
4. bottom, divergence, traps, and totality information;
5. authority, provider, resource, process, and admission requirements;
6. diagnostic, trace, provenance, and runtime lifecycle obligations.

Calling all of these "effects" blurs distinctions that Ash needs to preserve. A lazy pure
computation is not an IO effect. A checked postcondition is not an authority requirement. A
recoverable arithmetic failure is not a provider call. They can all be computation facts,
but they do not compose or discharge in the same way.

The proposed terminology is:

```text
computation row
```

An effect row becomes one family inside the larger computation-row model.

## 1. Central Model

Target Ash still has one ambient computation constructor:

```text
Ash<rho, A>
```

where:

- `A` is the produced value type;
- `rho` is a computation row;
- ordinary surface functions elaborate to functions returning `Ash<rho, A>`.

For example:

```ash
fn read(path: Path) -> {fs.read} String
```

means, at the semantic boundary:

```text
Path -> Ash<{fs.read}, String>
```

But the row is not only an unordered set of external effects. It is a typed collection of
computation facts:

```text
rho = eval facts
    + failure and partiality facts
    + operational requirements
    + authority and admission requirements
    + resource and region facts
    + contract and evidence facts
    + runtime lifecycle facts
    + diagnostic and trace facts
```

Some facts are set-like. Some are mode-like and require normalization. Some are linear or
ownership-sensitive. Some are evidence obligations. The row language must know which kind
of fact it is manipulating.

NOTE-021 records the current surface-syntax direction for this model: compact inline rows
remain available for small callable types, while large rows move to a callable `where`
section with `row { ... }` as an alternate layout for the same callable type row; predicate-like
artifacts are named facts, and source rows contain evidence requirements that denote the fact
plus its proof/check/record rather than embedding predicate bodies directly.

See NOTE-021's "Pre-Spec Delta" section before updating SPEC-095b, SPEC-096b, or SPEC-097b:
the living-note terminology and row/evidence syntax intentionally differ from the current
target specs.

## 2. Why Evaluation Modes Belong In the Row

Lazy and memo computations are not separate monads in target Ash. They are computation
modes of the same ambient Ash computation.

This argues against:

```text
Ash<mode, effects, A>
```

as the primary conceptual model. It splits the computation annotation space into two axes
even though mode participates in the same questions as effects:

- what must be preserved in public summaries;
- what `bind` and `do` sequencing must account for;
- what can be forced, shared, repeated, or memoized;
- what is equivalent to a plain value;
- what is legal to duplicate in a continuation.

The better model is:

```text
Ash<rho, A>
```

with evaluation facts inside `rho`:

```text
Ash<{eval eager}, A>
Ash<{eval lazy}, A>
Ash<{eval memo}, A>
Ash<{eval memo, fail ParseError, ensures valid_ast}, Ast>
```

Evaluation facts are not ordinary unordered effects. They are computation-row facts with a
small normalization algebra.

## 3. Taxonomy of Computation Row Facts

This taxonomy is intentionally a first pass. It names families that future specs can refine
into exact grammar, Core carriers, normalization rules, and diagnostics.

| Family | Examples | Meaning | Composition shape |
|---|---|---|---|
| Evaluation mode | `eval eager`, `eval lazy`, `eval memo` | When/how a computation is evaluated, forced, shared, or cached. | Mode algebra, not plain union. |
| Recoverable failure | `fail DivideByZero`, `fail ParseError` | A typed failure path that callers or handlers may recover from. | Set-like, plus payload typing and handler discharge. |
| Partiality/bottom | `bottom`, `diverges`, `trap ContractViolation` | Computation may not produce a value normally. | Classification and diagnostic rules; not authority. |
| Operational effects | `fs.read`, `db.write`, `net.request`, `time.now`, `random` | External operation requirements interpreted by handlers/providers/runtime. | Set-like requirements with operation identity and discharge. |
| Authority/admission | `requires role R`, `requires provider P`, `requires policy D`, authority for operation `fs.read` | Permission or admission facts needed before execution. Exact surface spelling remains unresolved. | Requirement/discharge, never grants by row inclusion alone. |
| Resources/regions | `owns region R`, `borrows resource X`, `sendable A`, `moves process` | Ownership, lifetime, region, and transfer constraints. | Linear/affine or ownership-sensitive rules. |
| Contracts/evidence | `requires pre P`, `ensures post Q`, `law L`, `proof obligation O`, `evidence E` | Logical claims, obligations, and discharge records attached to computation. | Subsumption, discharge, lifecycle, and blame rules. |
| Runtime lifecycle | `supervised`, `restartable`, `cancellable`, `streaming`, `long_lived` | Runtime organization and lifecycle requirements. | Contextual; often admitted by app/supervisor boundaries. |
| Diagnostics/trace | `records provenance`, `emits report`, `audit event K` | Required observability, report, or trace production. | Often accumulative, but tied to runtime/reporting boundaries. |

The row type checker should not treat these as interchangeable atoms. It should preserve
the fact family because each family answers different questions:

```text
Can it be duplicated?
Can it be discharged statically?
Does it require runtime authority?
Does it affect totality?
Does it affect value equivalence?
Does handler order matter?
Does it cross process or app boundaries?
```

Current capability terminology is not a target-language computation-row category or prefix.
It is subsumed by effect operations interpreted through admitted handlers/providers and
checked against admission/evidence rules. Ordinary operation requirements should be spelled
directly, such as `{fs.read}` or `{net.request}`. Rows may also need to record authority or
admission requirements, but the syntax for introducing those facts is not yet settled.

## 4. Pure Is Not Empty Row

The previous shorthand:

```text
pure = empty row
```

is too strong and too weak.

It is too strong because pure computations may still carry meaningful computation facts:

```text
Ash<{eval lazy}, A>
Ash<{eval memo}, A>
Ash<{fail DivideByZero}, Int>
Ash<{bottom}, A>
Ash<{ensures nonnegative}, Int>
Ash<{eval memo, fail ParseError, ensures valid_ast}, Ast>
```

These computations can be pure in the sense that they do not interact with the outside
world or require runtime authority.

It is too weak because an empty or small row does not automatically tell us the computation
is already a plain value. Lazy and memo modes make a computation operationally distinct
from its result even when the computation is pure.

The refined definitions are predicates over computation rows:

```text
is_pure(rho)
  no external operation, authority, admission, provider, process, resource, time,
  randomness, or other outside-world requirement is present.

is_total(rho)
  no recoverable failure, bottom, divergence, unchecked trap, or partiality fact is present.

is_value_like(rho)
  pure + total + eager/default evaluation + no runtime-significant suspension, memo,
  resource, trace, or unresolved evidence fact.

needs_runtime(rho)
  execution depends on runtime/provider/admission/process/app/host machinery.

needs_evidence(rho)
  contracts, laws, proof obligations, or evidence facts remain to be discharged or recorded.
```

This gives a clearer lattice of concepts:

```text
value-like  => pure and total
total       does not imply pure
pure        does not imply total
effectful   => not pure
partial     may still be pure
lazy/memo   may still be pure, but usually not value-like
```

Examples:

| Row | Pure? | Total? | Value-like? | Notes |
|---|---:|---:|---:|---|
| `{}` or `{eval eager}` | yes | yes | yes | Trivial pure computation, assuming eager/default has no runtime significance. |
| `{eval lazy}` | yes | yes if body total | no | Deferred computation is not the same operational value as `A`. |
| `{eval memo}` | yes | yes if body total | no | Sharing/cache behavior is meaningful. |
| `{fail DivideByZero}` | yes | no | no | Pure arithmetic may fail recoverably. |
| `{bottom}` | yes | no | no | No outside interaction, but no normal value guarantee. |
| `{ensures nonnegative}` | yes | yes if proof/discharge succeeds | usually no | Contract/evidence may matter even without effects. |
| `{fs.read}` | no | depends | no | Requires external operation/provider. |
| `{time.now}` | no | depends | no | Reads environment; not pure. |

## 5. Consequences for Function Types

The ordinary surface form remains convenient sugar:

```ash
fn f(x: A) -> {rho} B
```

Semantic boundary:

```text
A -> Ash<rho, B>
```

Pure ordinary functions are not defined by `rho = {}`. They are defined by the `is_pure`
predicate over `rho`:

```ash
fn div(x: Int, y: Int) -> {fail DivideByZero} Int
```

This is pure but not total.

```ash
fn delayed_score(input: Input) -> {eval lazy, ensures nonnegative} Score
```

This is pure if its body has no outside-world requirements, but it is not value-like.

By contrast:

```ash
fn read(path: Path) -> {fs.read} String
```

is not pure because the row contains an operational requirement.

## 6. Consequences for `do`, `bind`, and Row Combination

`do` remains sequencing sugar for the ambient computation model:

```text
bind : Ash<rho1, A> -> (A -> Ash<rho2, B>) -> Ash<combine(rho1, rho2), B>
```

The important refinement is that `combine` is not a blind set union over all facts.

Examples:

- operational effects union as requirements;
- recoverable failures accumulate unless handled;
- contracts compose through pre/post and discharge rules;
- authority requirements accumulate but do not grant authority;
- resource facts obey ownership and lifetime constraints;
- evaluation modes normalize through a mode algebra;
- lifecycle facts may require an app/supervisor context;
- trace/provenance facts accumulate but may be discharged by a reporting boundary.

Future specs should therefore define computation-row normalization by fact family, not by
one global unordered-row rule.

## 7. Syntax Implications

The computation-row terminology keeps the daily syntax small:

```ash
fn add(x: Int, y: Int) -> Int
fn div(x: Int, y: Int) -> {fail DivideByZero} Int
fn read(path: Path) -> {fs.read} String
```

Advanced libraries can still name computation values explicitly:

```ash
type Susp<rho, A> = Ash<rho, A>
type Thunk<rho, A> = Unit -> Ash<rho, A>
```

Whether type-level sugar such as `Unit -{rho}-> A` exists should be decided separately.
The main grammar constraint is that sugar and sugarless computation types should remain
visibly distinct enough to avoid parser ambiguity and reader confusion.

## 8. Open Design Questions

1. What is the final surface spelling for computation rows now that they include more than
   effects?
2. Which row fact families are admitted in ordinary function signatures versus Core-only
   summaries?
3. What exact mode algebra combines eager, lazy, and memo facts through `bind`, `force`,
   handlers, and continuations?
4. Which failures are recoverable `fail` facts, which are traps, and which are bottom or
   divergence facts?
5. Which contract/evidence facts affect value-like equivalence?
6. Which row entries can be hidden, summarized, or discharged at module boundaries?
7. What is the static criterion for multi-shot-pure continuation legality once "pure" is no
   longer equivalent to "empty row"?

## 9. Working Principle

The target rule:

```text
A computation row records all semantically relevant facts about evaluating an Ash
computation. Effects are one family of those facts, not the whole row.
```

The design payoff is that Ash can say:

```text
pure but partial
pure but lazy
pure but memoized
pure but contract-bearing
effectful and authority-requiring
runtime-admitted and supervised
```

without pretending all of those are the same kind of effect.

## 10. References

Internal references:

- [NOTE-013: Ambient Monad and Handler Composition Algebra](NOTE-013-AMBIENT-MONAD-AND-HANDLER-COMPOSITION-ALGEBRA.md)
- [NOTE-014: Contract Systems Unification](NOTE-014-CONTRACT-SYSTEMS-UNIFICATION.md)
- [NOTE-015: Current-to-Target Language Forms](NOTE-015-CURRENT-TO-TARGET-LANGUAGE-FORMS.md)
- [NOTE-016: Runtime Organization, Behaviours, and Reactive Modes](NOTE-016-RUNTIME-ORGANIZATION-BEHAVIOURS-REACTIVE-MODES.md)
- [NOTE-017: Memory Regions, Ownership, and Utilization](NOTE-017-MEMORY-REGIONS-OWNERSHIP-AND-UTILIZATION.md)
- [NOTE-018: Boundary Discipline for Target Ash](NOTE-018-BOUNDARY-DISCIPLINE.md)
- [NOTE-019: Target Ash Convergence Plan](NOTE-019-TARGET-ASH-CONVERGENCE-PLAN.md)
- [NOTE-021: Row, Callable, Where, and Fact Syntax](NOTE-021-ROW-CALLABLE-WHERE-AND-FACT-SYNTAX.md)
- [SPEC-096b: Target Effect System](../spec/SPEC-096b-TARGET-EFFECT-SYSTEM.md)
- [SPEC-097b: Target Type System](../spec/SPEC-097b-TARGET-TYPE-SYSTEM.md)
- [SPEC-101: Lazy and Memo Computation Modes](../spec/SPEC-101-LAZY-AND-MEMO-COMPUTATION-MODES.md)
- [SPEC-102: CPS Continuation Multiplicity](../spec/SPEC-102-CPS-CONTINUATION-MULTIPLICITY.md)

## 11. Changelog

- 2026-06-27: Linked NOTE-021 as the surface-syntax companion for compact inline rows,
  expanded `where row { ... }` rows, named fact declarations, and evidence row entries.
  NOTE-021 now carries the pre-spec delta checklist for later target-spec alignment.
- 2026-06-27: Normalized open-question wording from row facts to row entries to align with
  NOTE-021's source-row/evidence terminology.
- 2026-06-24: Clarified that ordinary operation row items are spelled directly, such as
  `{fs.read}`, and authority-bearing status is handled through unresolved
  admission/provider/handler facts rather than a `cap` row prefix.
- 2026-06-24: Initial draft. Introduces computation rows as the broader target Ash row
  terminology, records a first taxonomy of row fact families, and refines pure computation
  away from the empty-row definition.
