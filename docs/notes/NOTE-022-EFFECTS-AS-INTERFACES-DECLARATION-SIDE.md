# NOTE-022: Effects as Interfaces — Declaration Side Unification

**Date:** 2026-06-27
**Status:** Living document — declaration-side decision captured; dispatch-side open
**Purpose:** Record the decision to unify effect operation declarations with the existing
interface/impl machinery, eliminating the separate `effect` keyword. This note owns the
declaration-side decision and its consequences. Dispatch-side concerns (handler surface,
resume, admission, installation) are explicitly out of scope and tracked separately.

Companion to NOTE-013 (handler composition algebra), NOTE-015 (language form taxonomy),
NOTE-018 (boundary discipline), NOTE-019 (convergence plan), NOTE-020 (computation row
taxonomy), and NOTE-021 (row/callable/where/fact syntax).

## Pre-Spec Delta

This note supersedes prior "settled" directions:

- **NOTE-015 §7.1** says: "make `effect` the canonical target vocabulary." NOTE-022 says:
  interfaces are the canonical vocabulary; no `effect` keyword.
- **NOTE-019 changelog (2026-06-25)** says: "`effect` blocks contain `fn` operation
  signatures." NOTE-022 says: interfaces contain `fn` operation signatures.
- **NOTE-018 §3.2** shows `effect Fs { fn read ... }` as the canonical declaration shape.
  NOTE-022 says: `interface Fs { fn read ... }` is the canonical shape.
- **NOTE-013 §11.1** uses `effect Fs { ... }` blocks with inline externs and handler
  clauses. NOTE-022 says: the interface declares operation signatures; externs and handler
  clauses are separate dispatch-side concerns.

The conceptual content of these notes — handler composition algebra (NOTE-013), boundary
discipline (NOTE-018), convergence tracks (NOTE-019), row taxonomy (NOTE-020) — is
unaffected. Only the declaration surface changes.

## 0. Motivation

Target Ash already needs interface/impl machinery for ad-hoc polymorphism: associated
types, generics, where clauses, super-constraints, default methods. A separate `effect`
declaration form duplicates this machinery for operation signatures. The question was
whether the duplication is justified.

It is not. The operation signature set is a type contract. Interfaces declare type
contracts. The interface IS the single declaration site for operation signatures. What
makes operations *behave* as effects — Raise, Handle, resume, row discharge — lives at the
dispatch side and at the CPS substrate, not at the declaration site.

## 1. The Decision

**Operation signatures are declared as interface methods. There is no `effect` keyword.**

```ash
interface Fs {
    fn read(path: Path) -> String
    fn write(path: Path, contents: String) -> Unit
}

interface Choice {
    fn choose<A>(options: List<A>) -> A
}

interface Store<K, V> {
    type Key
    fn get(key: Self::Key) -> Option<V>
    fn put(key: Self::Key, value: V) -> Unit
}
```

These are ordinary interface declarations. They participate in the existing interface/impl
type system. They carry generics, associated types, where clauses, and all other
interface-level machinery.

The row item for an operation is its resolved operation identity — the fully-qualified
interface method:

```ash
fn load(path: Path) -> {Fs.read} String {
    Fs.read(path)
}
```

or equivalently through name resolution:

```ash
fn load(path: Path) -> {fs.read} String {
    fs.read(path)
}
```

After name resolution, Core/CPS sees one canonical operation identity and one row item.
This is unchanged from NOTE-018 §3.2 — only the declaration keyword changes.

## 2. Why This Works — Declaration vs Dispatch Separation

The key insight is that **the interface participates in type-checking, not in dispatch.**

| Concern | Mechanism | Changed by this decision? |
|---|---|---|
| Operation signature | Interface method declaration | Was `effect`, now `interface` |
| Operation identity | Fully-qualified module path | No |
| Type-checking handler clauses | Check against interface method signature | No (was already the intent) |
| Dispatch (which handler catches) | Handle frame nesting in CPS IR | No |
| Row discharge | Handle frame removes operation from body row | No |
| Resume continuation | `Value::Cont` with multiplicity (SPEC-102) | No |
| Handler composition | Handle frame stack semantics (NOTE-013 §7) | No |

The CPS substrate — Raise, Handle, HandlerClause, Value::Cont, LetCont, LetContCall — is
completely unchanged. Surface lowering produces the same Core terms and CPS nodes regardless
of whether the operation was declared via `effect` or `interface`.

## 3. What the Interface Does

The interface is a **type contract** for operations:

1. **Signature authority.** There is one canonical place that says "Fs.read takes a Path
   and returns a String." Every handler clause is type-checked against this signature.
2. **Generics and associated types.** Parameterized operations (`Store<K, V>`) and
   operations with associated types use ordinary interface generics. No special effect
   grammar is needed.
3. **Where clauses.** Operations may carry constraints: `fn get(key: Key) -> Option<V>
   where Key: Hash`. These are ordinary interface-level constraints.
4. **Module identity.** The fully-qualified interface method name (`Fs.read`,
   `choice.choose`) is the canonical operation identity used in rows and Raise nodes.

## 4. What the Interface Does NOT Do

The interface does NOT participate in dispatch:

1. **No coherence obligation.** Type-class coherence (at most one global impl per type)
   does not apply. Multiple handlers for the same interface coexist at different Handle
   frame nesting levels. This is inherent to algebraic effects — handler composition
   requires simultaneous interpretations.
2. **No impl resolution for dispatch.** When `Fs.read(path)` is called, the compiler does
   not search for an impl of `Fs`. It emits a `Raise` node. The runtime searches the Handle
   frame stack. The interface constrains the handler clause's type, not the call's
   resolution path.
3. **No authority.** The interface declares what the operation looks like. It does not
   grant permission to call or handle it. Authority is an admission concern (dispatch
   side).
4. **No handler definition.** The interface declares the operation signature. Handler
   clauses — which receive the resume continuation and produce the handler's answer — are
   a separate construct with a different signature shape. This is the dispatch side.

The separation:

```text
DECLARATION SIDE (this note):
  interface declares operation signatures
  interface carries generics, associated types, where clauses
  row item = resolved interface method identity

DISPATCH SIDE (separate track):
  handler clauses pattern-match on raised operations
  resume continuation threading and multiplicity
  handler installation and admission
  authority/provenance evidence
```

## 5. Correspondence to Prior Art

The "interface declares, handler interprets" model is consistent with both Koka and Frank:

| Language | Declaration form | Dispatch form |
|---|---|---|
| Koka | `effect` with operation signatures | `handle ... with` blocks |
| Frank | implicit (operations are type-level row items) | handler definitions with `on` clauses |
| Ash (target) | `interface` with method signatures | TBD (dispatch side, open) |

Ash's contribution is that the declaration form is not specialized — it reuses the full
interface/impl machinery. Koka's `effect` is structurally similar to an interface but lacks
generics, associated types, and where clauses on operations. Frank avoids the problem by
not having a separate declaration form at all. Ash gets both the declaration-site contract
and the full type-system power.

Key prior art:

- **Leijen, "Koka: Programming with Row-Polymorphic Effects" (2014).**
  Row-polymorphic effect system; `effect` declarations are structurally trimmed interfaces.
  https://www.microsoft.com/en-us/research/wp-content/uploads/2016/08/koka-technical.pdf
- **Lindley, McBride & McLaughlin, "Do Be Do Be Do" (2017).**
  Frank: handlers as functions; operations are row items without separate declarations.
  https://doi.org/10.1145/3064898
- **Plotkin & Power, "Computational Effects as Operations" (2002).**
  Effects as operations of an algebraic theory; the signature is the theory's signature.
  https://www.sciencedirect.com/science/article/pii/S0304397502004449

## 6. What the Erlang-Style Arity Discrimination Resolves

The discussion considered whether handler clauses could be distinguished from operation
calls by arity (Erlang-style: `read/1` is the call, `read/2` is the handler clause with
resume). This insight is partially absorbed:

- The **operation call** signature is the interface method signature: `fn read(path: Path)
  -> String`.
- The **handler clause** signature is different: it includes a resume continuation
  parameter. The arity/signature difference is real.
- The interface declares the operation signature. The handler clause signature is derived
  from it (or declared separately at the dispatch side).

The arity insight confirms that operation and handler clause are genuinely different
signatures. The interface declares the operation; the handler clause's full signature —
including resume and answer type — belongs to the dispatch side. This is why
declaration/dispatch separation is not just convenient but semantically grounded.

## 7. Impact on Existing Notes

| Note | Impact | Action |
|---|---|---|
| NOTE-013 | Earlier §11.1 versions used `effect Fs { ... }` in extern placement examples. | Reconciled: examples now use `interface`; Open Q1 is resolved by this note. Dispatch-side questions remain with NOTE-023/024. |
| NOTE-014 | Lines 660-720 use `effect Fs { ... }` in contract examples. | Replace `effect` with `interface` in examples. |
| NOTE-015 | §7.1 previously said "make `effect` the canonical target vocabulary." | Reconciled: interfaces are canonical; `effect` keyword is retired; NOTE-025 supplies impl/type-qualified identity. |
| NOTE-018 | §3.2 previously showed `effect Fs { ... }` as canonical declaration. | Reconciled: main decision text now uses interface operation declarations and provider/handler admission. |
| NOTE-019 | §3.4, §4.4, gap register, and §6.1 previously referenced "effect declarations." | Reconciled: target convergence text now uses interface operation declarations plus impl/type-qualified identities. |
| NOTE-020 | References effect operations as row generators. | No change needed — NOTE-020 discusses operations at the semantic level, not the declaration keyword. |
| NOTE-021 | References operations in rows (`fs.read`). | No change needed — NOTE-021 is about row syntax, not declaration form. |

## 8. Open Questions (Dispatch Side)

These are explicitly deferred to a separate dispatch-side design track:

1. **Handler clause surface grammar.** How are handler clauses declared? Koka-style
   `handle ... with` blocks? Frank-style named handlers with `on` clauses? Something else?
2. **Resume continuation access.** How do handler clauses name and use the resume
   continuation? Is it an explicit parameter, an implicit like `self`, or derived from the
   handler clause signature?
3. **Resume multiplicity at the surface.** SPEC-102 gives Core/CPS multiplicity
   (`Affine`, `MultiShotPure`). How is this declared at the surface — on the handler, on
   the clause, or inferred?
4. **Answer type.** Handler clauses return the handler's answer type, not the operation's
   result type. How is the answer type threaded through the handler declaration?
5. **Handler installation and admission.** How are handlers installed at runtime? How is
   authority/provenance evidence attached? This is the admission gate (NOTE-019 §3.2:
   "rows are requirements, not authority").
6. **Extern placement.** NOTE-013 §11.1 documented two placements (effect-level canonical
   hook vs provider-level adapter). With interfaces replacing `effect`, externs attach to
   handler/provider declarations. The placement choice remains open.

## 9. Working Principle

```text
The interface is an effect sort: a type contract for operations.
It declares signatures, generics, associated types, laws, and constraints.
It does not dispatch, grant authority, or define handler behavior.

The impl type is the operation identity carrier.
Rows name impl-qualified operation identities (e.g., PosixFs::read), not
interface-qualified ones (e.g., Fs.read). See NOTE-025 for the full model.

Dispatch is Handle frame nesting in the CPS substrate.
Authority is admission evidence at installation time.
Handler clauses are a separate construct on the dispatch side.
```

**Revision (NOTE-025):** This note originally stated "rows name resolved interface method
identities." NOTE-025 revises this: the interface is the sort, the impl type is the identity
qualifier. After monomorphization, the row item is `ImplType::operation`, not
`Interface.operation`. This enables multiple simultaneous handlers for the same interface.

## 10. References

Internal references:

- [NOTE-013: Ambient Monad and Handler Composition Algebra](NOTE-013-AMBIENT-MONAD-AND-HANDLER-COMPOSITION-ALGEBRA.md)
- [NOTE-014: Contract Systems Unification](NOTE-014-CONTRACT-SYSTEMS-UNIFICATION.md)
- [NOTE-015: Current-to-Target Language Forms](NOTE-015-CURRENT-TO-TARGET-LANGUAGE-FORMS.md)
- [NOTE-018: Boundary Discipline for Target Ash](NOTE-018-BOUNDARY-DISCIPLINE.md)
- [NOTE-019: Target Ash Convergence Plan](NOTE-019-TARGET-ASH-CONVERGENCE-PLAN.md)
- [NOTE-020: Computation Row Taxonomy and Pure Computation](NOTE-020-COMPUTATION-ROW-TAXONOMY.md)
- [NOTE-021: Row, Callable, Where, and Fact Syntax](NOTE-021-ROW-CALLABLE-WHERE-AND-FACT-SYNTAX.md)
- [NOTE-025: Effect Identity via Sorts and Impls](NOTE-025-EFFECT-IDENTITY-VIA-SORTS-AND-IMPLS.md) — revises the identity model
- [SPEC-098b: Target CPS IR](../spec/SPEC-098b-TARGET-IR.md)
- [SPEC-100: Core Type Checking](../spec/SPEC-100-CORE-TYPE-CHECKING.md)
- [SPEC-102: CPS Continuation Multiplicity](../spec/SPEC-102-CPS-CONTINUATION-MULTIPLICITY.md)

External references:

- Leijen, "Koka: Programming with Row-Polymorphic Effects" (2014).
  https://www.microsoft.com/en-us/research/wp-content/uploads/2016/08/koka-technical.pdf
- Lindley, McBride & McLaughlin, "Do Be Do Be Do" (2017).
  https://doi.org/10.1145/3064898
- Plotkin & Power, "Computational Effects as Operations" (2002).
  https://www.sciencedirect.com/science/article/pii/S0304397502004449

## 11. Changelog

- 2026-06-27: Initial version. Captures the decision to unify effect operation declarations
  with interfaces, eliminating the `effect` keyword. Documents declaration/dispatch
  separation, impact on existing notes, and open dispatch-side questions.
- 2026-06-27: Revised by NOTE-025. The identity model changes: the interface is now an effect
  sort (abstract family), and the impl type is the operation identity qualifier. Rows name
  `ImplType::operation`, not `Interface.operation`. Updated working principle and references.
