# NOTE-014: Contract Systems Unification — Hoare, Laws, Properties, and the Computation Row

**Date:** 2026-06-23
**Status:** Living document — exploration in progress
**Purpose:** Capture the precise semantics of Ash's contract systems and how they unify
through the computation row. Companion to NOTE-013 (ambient monad and handler composition).
Updated as new insights emerge; restructured for flow and readability later.

## 0. Motivation

Ash has two contract systems that risk conceptual schizophrenia:

1. **Hoare-style contracts** (named, site-specific): `requires`, `ensures`, `invariant`.
   These attach to computation sites (function entry/exit, loops, data structures). Logical
   shape: Hoare triples `{P} C {Q}`.

2. **Universal contracts** (no-name, declaration-specific): `law` (must be proven) and
   `property` (tested only). These attach to interface/module declarations. Logical shape:
   universal theorems `∀x⃗. P(x⃗)`.

Both end up in the same `ContractEffect` enum in the CPS IR computation row. The danger is
treating them as the same thing when they have different logical shapes, different discharge
mechanisms, and different lifecycles.

This note develops the precise story: they are NOT two arbitrary systems. They are two
instances of the same underlying mechanism — contract requirements in the computation row —
differing only in attachment site, discharge mode, and logical shape. And they compose: laws
produce evidence that can discharge Hoare preconditions.

## 1. The Two Systems, Precisely

### 1.1 System A: Hoare Contracts (Named, Site-Specific)

Declarations:

```ash
fn safe_div(a: Int, b: Int) -> Int
    requires: b != 0
    ensures: result * b == a
{
    a / b
}
```

- `requires: P` — precondition (Hoare `P`), checked at function entry.
- `ensures: Q` — postcondition (Hoare `Q`), checked at function exit.
- `invariant: I` / `where { I }` — invariant, checked at loop/data-structure boundaries.

These are **compositional**: the triple `{P} C {Q}` composes sequentially. A function
satisfying `{P} C {Q}` can be called from a context where `P` holds, and the caller can
assume `Q` after the call.

### 1.2 System B: Universal Contracts (No-Name, Declaration-Specific)

Declarations:

```ash
interface Semigroup<A> {
    append(a: A, b: A) -> A

    law associativity(a: A, b: A, c: A, eq: Eq<A>)
      : eq.equiv(append(append(a, b), c), append(a, append(b, c)))
}

property commutativity<A: Commutative> {
    forall x, y: A.
    x <> y == y <> x
}
```

- `law` — universal theorem (`∀x⃗. P(x⃗)`), must be proven by some means.
- `property` — statistical hypothesis, tested by QuickCheck/smallcheck, advisory only.

These are **not compositional** in the Hoare sense. A law holds for all values universally.
It does not compose with computation sites — it is a global property of the interface
implementation.

### 1.3 The Key Distinction

| Dimension | Hoare contracts | Laws | Properties |
|-----------|----------------|------|------------|
| Logical shape | `{P} C {Q}` (triple) | `∀x⃗. P(x⃗)` (universal) | `∀x⃗. P(x⃗)` (universal) |
| Attached to | Computation site (call, loop) | Interface/module declaration | Interface/module declaration |
| Discharge obligation | Per-invocation | Once per implementation | None (advisory) |
| Blocking? | Yes (static or runtime) | Yes (must be proven) | No (test harness only) |
| Compositional? | Yes (Hoare sequencing) | No (global) | No (global) |

The schizophrenia risk is treating laws as if they were Hoare contracts (site-specific) or
vice versa. The hoare-clauses.md design note already identified: "Hoare clauses are
sufficient for the core contract system (requires, ensures, where). They are not sufficient
for laws and properties."

## 2. The Unification: Both Are Computation-Row Items with Different Discharge Modes

Both systems appear in the CPS IR computation row as `ContractEffect` variants (SPEC-097b §3.6,
SPEC-098b §4):

```rust
pub enum ContractEffect {
    Requires(PredicateRef),
    Ensures(PredicateRef),
    Invariant(PredicateRef),
    Law { name: Name, predicate: PredicateRef },
    Obligation(NamePath),
    Guard(PredicateRef),
}
```

But they discharge differently. The `DischargeMode` enum (SPEC-098b §4.1) captures the
mechanism:

```rust
pub enum DischargeMode {
    Static,      // discharged by type checker / SMT
    Evidence,    // discharged by proof / test / law evidence
    Dynamic,     // discharged by runtime contract handler
}
```

The full matrix:

| Contract kind | Row item | Discharge category | Mechanism | IR representation |
|---|---|---|---|---|
| `requires` (static) | `Contract::Requires(P)` | Ambient discharge | SMT / refinement type | Erased from runtime row; refinement on parameter types |
| `requires` (dynamic) | `Contract::Requires(P)` | Raised operation | `Raise ContractViolation` | `Raise` node; `Failure` effect in row |
| `ensures` (static) | `Contract::Ensures(Q)` | Ambient discharge | SMT / refinement type | Erased; refinement on return type |
| `ensures` (dynamic) | `Contract::Ensures(Q)` | Raised operation | `Raise ContractViolation` | `Raise` node at exit; `Failure` effect in row |
| `invariant` (static) | `Contract::Invariant(I)` | Ambient discharge | SMT | Erased; refinement predicate |
| `invariant` (dynamic) | `Contract::Invariant(I)` | Raised operation | `Raise ContractViolation` | `Raise` node at boundary |
| `law` (proven) | `Contract::Law{name,pred}` | Ambient discharge | Proof (SMT/Lean/by_definition) | Erased; trusted evidence in `ContractDischarge.evidence` |
| `law` (survived testing) | `Contract::Law{name,pred}` | Falsification evidence | QuickCheck/smallcheck/fuzzing | Erased; test evidence ref (confidence, not proof) |
| `law` (refuted) | `Contract::Law{name,pred}` | Blocking | Counterexample found | Compile error or deferred-status |
| `property` | — (not a row item) | Falsification only | Test harness / fuzzing | Not lowered to IR |

### 2.1 Properties Are NOT Row Items

This is a critical design decision. Properties are advisory — they belong to the test
harness, not to the computation row. A failing property test does not block execution; it is
reported as a test failure, separate from the compilation/runtime pipeline.

Properties are compile-time metadata that generate test cases. They never discharge a row
item. They never produce runtime code. They are never checked at a computation site.

### 2.2 Laws ARE Row Items — but Discharge Once, Not Per-Invocation

Laws are row items attached to interface implementations, not to individual call sites. A
law's discharge obligation is discharged **once per implementation** — when the proof or
test evidence is established. After discharge, the evidence is reused everywhere the
interface is used.

This means the law's row contribution is erased after discharge. The `ContractDischarge`
struct records the evidence:

```rust
pub struct ContractDischarge {
    pub contract: ContractEffect,
    pub mode: DischargeMode,
    pub evidence: Option<EvidenceRef>,   // proof, test result, or SMT certificate
    pub source_span: Span,
}
```

"A contract effect cannot be silently erased from a row without recording its discharge
mode." (SPEC-098b §4.1). This is the audit trail: even when a law is discharged, the fact of
discharge and the evidence used are preserved.

## 3. How Static and Dynamic Hoare Contracts Lower to the IR

This is the bridge between the surface and the IR, made precise.

### 3.1 Static Hoare → Refinement Types (No Runtime Code)

```ash
-- Surface
fn safe_div(a: Int, b: Int) -> Int
    requires: b != 0
    ensures: result * b == a
{
    a / b
}
```

Lowers to Core as a refined type:

```
fn safe_div(a: Int | b != 0, b: Int | b != 0) -> Int | result * b == a
```

The `requires` becomes a refinement constraint on parameter types. The `ensures` becomes a
refinement constraint on the return type. The type checker collects these constraints and
passes them to the SMT solver.

SMT result handling:

| SMT Result | Action |
|---|---|
| `unsat` (proven — no counterexample exists) | Compile with refinement; erased from runtime row |
| `sat` (disproven — counterexample exists) | Compilation error with counterexample |
| `unknown` | **Demote to dynamic** — runtime check inserted |

This is **gradual verification**: what can be proven statically is checked at compile time;
what cannot is checked at runtime.

### 3.2 Dynamic Hoare → Raise/Handle (Runtime Code)

```ash
-- Surface
fn safe_div(a: Int, b: Int) -> {ContractViolation} Int
    dynamic requires: b != 0
    dynamic ensures: result * b == a
{
    a / b
}
```

Lowers to CPS IR:

```
fn safe_div(a: Int, b: Int) -> {Failure} Int {
    if b == 0 {
        Raise { op: ContractViolation("requires: b != 0"), resume: k }
    };
    let result = a / b;
    if result * b != a {
        Raise { op: ContractViolation("ensures: result * b == a"), resume: k }
    };
    Jump(k, result)
}
```

The `ContractViolation` is a `Raise` node — it is an algebraic operation request, in the
"raised operations" category (SPEC-098b §5.6). It is matched by a `Handle` frame:

```ash
fn with_contract_check<A>(action: {Failure} A) -> Result<A, ContractError> {
    handle action with {
        ContractViolation(msg) => Err(ContractError(msg))
    }
}
```

This connects directly to NOTE-013: dynamic contracts are handler-dischargeable effects. The
contract handler is a handler for the `Failure`/`Contract` effect category, with its own
resume strategy (typically: do not resume on violation, like Either's early exit).

## 4. How Laws and Properties Lower

### 4.1 Laws: The Proven Subset

A law declaration declares a theorem. The ideal outcome is a proof:

```
law associativity<A: Semigroup>
  → SMT / Lean / Coq / by_definition
  → Result:
      Proved      → Evidence: associative(op) → Usable in refinements
      Disproved   → Compile error (counterexample) — the theory is false
      Unknown     → No proof, no disproof — falls into the evidence regime (§4.3)
```

When a law is proved, it generates a **refinement predicate** usable in Hoare contracts:

```ash
fn fold<A>(xs: List<A>, init: A, op: (A, A) -> A) -> A
    requires: associative(op)    -- discharged by proven law evidence
{
    ...
}
```

The `requires: associative(op)` is discharged by the law's evidence, not by an SMT query at
the call site. The evidence flows: law → proof → refinement predicate → precondition
discharge.

**In reality, we will not be able to provide proofs for everything — only a critical
subset.** The proven subset is small but trusted. The rest lives in the evidence regime.

### 4.2 Properties: The Test Harness (Advisory)

Properties produce test evidence — they are never proofs:

```
property commutativity<A: Commutative>
  → QuickCheck (100 cases) + SmallCheck (depth 3)
  → Advisory: "tested: 100/100 passed"
  → Not usable in types. Not a row item. Not lowered to IR.
```

Properties are never lowered to refinements. They are advisory only. Failing property tests
are reported as test failures, not compilation errors.

### 4.3 The Epistemology: Falsification, Not Verification

This is the conceptual core. Ash's contract system is fundamentally **falsificationist**
(Popperian), not verificationist:

1. **We declare a theory of correct operation** — the laws and properties that should hold.
2. **We try to gather counter-evidence that disproves the theory.** This is the domain of
   property tests and fuzzing.
3. **Proofs cover only the critical subset** — the parts where absolute certainty matters
   (security invariants, financial calculations, core algebra). For everything else,
   repeated independent QuickCheck runs increase our confidence that the law *may* hold,
   without ever constituting proof.

The three evidence states are therefore:

| State | Meaning | Can disprove? | Can prove? |
|-------|---------|---------------|------------|
| **Proven** | A solver/theorem prover established the theorem | No | Yes |
| **Survived testing** | No counterexample found after N independent runs | Yes (any single run can) | No |
| **Refuted** | A counterexample was found | Yes (the counterexample is the proof of ¬theorem) | N/A |

The asymmetry is the point: a single counterexample refutes; a million passing tests do not
prove. This is why:

- **Proven laws** can discharge Hoare preconditions (they are trusted facts).
- **Tested laws/properties** cannot discharge anything in the type system — they are
  confidence signals, not facts. But they serve a different purpose: they actively hunt for
  refutation.

This reframes the relationship between laws and properties:

- **Laws** are the declarations of the theory (`∀x⃗. P(x⃗)`). They may have proof evidence
  (trusted) or test evidence (untrusted but useful).
- **Properties** are pure falsification instruments. They exist solely to hunt for
  counterexamples. They make no truth claim — only a "no counterexample found yet" claim.

### 4.4 Evidence Lifecycle

Because we cannot prove everything, the system must document, for each declared law/property:

1. **The proposition** — what the theory says should hold.
2. **Proof status** — proven (by what mechanism), unproven, or refuted (with counterexample).
3. **Test evidence** — what testing has been done and what it found (runs, seeds, results).
4. **Refutation status** — has counter-evidence been found? If so, the theory is false and
   must be fixed or the declaration is dishonest.

The `ContractDischarge` struct in the IR (§2.2) records the discharge mode and evidence
reference. For test-based evidence, the `evidence` field points to test metadata (seed, run
count, result). This is the audit trail: not "this is true," but "here is what we know about
whether this is true."

A law or property that has neither proof nor test evidence is an **untested assertion** — a
statement of intent with no backing. The system should flag these honestly rather than
silently treating absence-of-evidence as evidence-of-correctness.

## 5. The Composition: Laws Feed Hoare Contracts

The two systems compose. This is the unifying insight that prevents schizophrenia.

A law establishes a universal fact about an interface implementation. That fact can be used
to discharge a Hoare precondition at a computation site. The flow:

```
interface Semigroup<A> {
    append(a: A, b: A) -> A
    law associativity(a, b, c): append(append(a, b), c) == append(a, append(b, c))
}

impl Semigroup<Int> {
    append(a, b) = a + b
    proof associativity(a, b, c) { by_definition }    -- evidence established
}

-- The law evidence discharges the requires clause:
fn fold<A>(xs: List<A>, init: A, op: (A, A) -> A | associative(op)) -> A { ... }
```

At the `fold` call site with `op = +`, the type checker:

1. Sees `requires: associative(op)`.
2. Looks up law evidence for `associative(+)`.
3. Finds the discharged `ContractDischarge { mode: Evidence, evidence: ... }`.
4. Discharges the precondition from the row.

No SMT query at the call site. The law's evidence propagates.

## 6. The Handler Soundness Connection (from NOTE-013)

The contract systems connect to the handler composition algebra from NOTE-013:

### 6.1 Dynamic Contracts Are Handler-Dischargeable

Dynamic `requires`/`ensures` lower to `Raise { op: ContractViolation }`. This is a raised
operation in the `Failure` effect category. It is handled by a `Handle` frame — a contract
handler.

The contract handler's resume strategy is typically **shallow** (no resume on violation),
analogous to Either's early exit. This means:

- A contract handler inside a deep state handler preserves state on contract violation
  (analogous to `StateT(EitherT)` — ORDER A in NOTE-013 §7.2).
- A contract handler outside a deep state handler discards state on contract violation
  (analogous to `EitherT(StateT)` — ORDER B).

The handler nesting order determines which semantics you get. This is derivable from the
NOTE-013 algebra.

### 6.2 Static Contracts and Law Evidence Are Ambient Discharge

Static Hoare contracts (refinement types) and law evidence are **ambient discharge** items
(SPEC-098b §5.6). They are not handled by `Handle` frames — they are discharged by:

- The type checker (static Hoare via SMT)
- Evidence proofs (laws via Lean/Coq/SMT)
- Cached evidence references (`ContractDischarge.evidence`)

They are erased from the runtime row after discharge. They never become `Raise` nodes.

### 6.3 The Clean Separation

| Discharge category | Mechanism | IR representation | Handler involved? |
|---|---|---|---|
| Ambient (static Hoare) | SMT / refinement | Erased; type refinement | No |
| Ambient (proven law) | Proof certificate | Erased; trusted evidence cached | No |
| Raised (dynamic Hoare) | Runtime handler | `Raise ContractViolation` | Yes |
| Falsification (tested law / property) | QuickCheck / smallcheck / fuzzing | Not in runtime IR (test harness) | No |

This is the precise semantics. There is no ambiguity about whether a contract is checked
statically or dynamically, or whether it involves a handler: the discharge mode determines
it, and the IR representation follows mechanically.

## 7. Surface Syntax Design Implications

### 7.1 The Three Declaration Forms Must Stay Distinct

```ash
-- Hoare contracts (site-specific, System A)
fn safe_div(a: Int, b: Int) -> Int
    requires: b != 0
    ensures: result * b == a
{ ... }

-- Laws (universal, must be proven, System B)
law associativity<A: Semigroup>(a, b, c, eq)
  : eq.equiv(append(append(a, b), c), append(a, append(b, c)))

-- Properties (universal, tested only, advisory)
property commutativity<A: Commutative>(x, y)
  : x <> y == y <> x
```

These must not be conflated in the surface. `requires`/`ensures`/`invariant` are Hoare.
`law` is a theorem. `property` is a hypothesis. Different keywords, different semantics,
different discharge.

### 7.2 The Static/Dynamic Distinction on Hoare Contracts

The hoare-clauses.md note proposes `static`/`dynamic` qualifiers. The design decision:

- **Default is static** (SMT checks at compile time). If SMT returns `unknown`, demote to
  dynamic automatically (gradual verification).
- **Explicit `dynamic`** forces runtime checking via the effect system.
- **Explicit `static`** is redundant but allowed for clarity.

```ash
-- Static (default): SMT at compile time
fn f(x: Int) -> Int requires: x > 0 { ... }

-- Dynamic: runtime check via Raise ContractViolation
fn f(x: Int) -> {Failure} Int dynamic requires: x > 0 { ... }

-- Both: static check + runtime assertion (belt and suspenders)
fn f(x: Int) -> {Failure} Int
    requires: x > 0           -- static
    dynamic requires: x > 0   -- runtime
{ ... }
```

### 7.3 Law Evidence Syntax

The proof body declares how a law is discharged:

```ash
impl Semigroup<Int> {
    append(a, b) = a + b

    proof associativity(a, b, c, eq) {
        by_definition                    -- definitional equality
    }
}

impl Monad<Option> {
    -- No proof block → synthetic test obligation (QuickCheck/smallcheck)
}

impl Monad<Act<A>> {
    proof act_associativity(ma, f, g, equiv) by test {
        generator: bounded_act_generator(),
        equivalence: equiv,
        schedule: deterministic_schedule(),
        max_steps: 100,
    }
}
```

Future evidence modes (deferred, not in surface yet):

- `by z3 { ... }` — SMT solver certificate
- `by lean { ... }` — Lean proof term
- `by coq { ... }` — Coq proof term

## 8. Attachment Sites: What Contracts Attach to What

The surface language has several declaration forms. Each form admits specific contract types.
The governing principle:

- **Hoare contracts** (`requires`/`ensures`/`invariant`) attach to **computation sites** —
  points where control flow enters, exits, or crosses a boundary. Their logical shape is the
  triple `{P} C {Q}`.
- **Universal contracts** (`law`/`property`) attach to **named declarations** — types,
  interfaces, effects, modules. Their logical shape is `∀x⃗. P(x⃗)`.
- **Proofs** (`proof`) attach only to `impl` and `handler` — they discharge law obligations
  declared elsewhere.

Some sites support both: interface methods and effect operations are declarations that carry
inherited Hoare proof obligations for implementors.

### 8.1 The Attachment Matrix

```
                    requires  ensures  invariant    law      property   proof
                    (Hoare P) (Hoare Q) (Hoare I)  (∀,prove) (∀,falsify) (discharge)
─────────────────────────────────────────────────────────────────────────────────────
fn                  YES       YES      YES(loops)   no*       no*        no
type decl           no        no       YES          YES       YES        no
interface (trait)   YES**     YES**    YES          YES       YES        no
impl                YES***    YES***   YES          no        no         YES
effect decl         YES†      YES†     no           YES††     YES††      no
handler             YES‡      YES‡     YES          YES‡‡     YES‡‡      YES
capability          YES       YES      YES          YES§      YES§       no
module (top-level)  no        no       no           YES       YES        no
```

### 8.2 Explanations

**\* fn and laws/properties.** An individual `fn` cannot directly carry a `law`. Laws are
universal theorems about combinations of operations (e.g., associativity of `append`). A
single function is a computation site — it gets Hoare contracts, not universal theorems.
However, a module-level law CAN reference individual functions:

```ash
-- In module scope, not on the fn itself:
law sort_idempotent<A: Ord>(xs: List<A>)
  : sort(sort(xs)) == sort(xs)
```

The attachment is at the module; the reference is to the fn. This is deliberate: equational
theories live at the declaration level, not at individual call sites.

**\*\* Interface methods and Hoare contracts.** Interface method signatures CAN carry
`requires`/`ensures`. This makes the interface a contract specification, not just a method
list:

```ash
interface Stack<A> {
    push(a: A) -> Unit
    pop() -> A requires: not_empty ensures: result == old(peek())
    peek() -> Option<A>
}
```

If `pop` carries `requires: not_empty` in the interface, then:

- Every `impl Stack` must satisfy this contract on its `pop` body.
- The type checker verifies the impl's `pop` satisfies `{not_empty} body {result is top}`.
- Callers of `pop` must establish `not_empty` as a precondition.

The contract's *shape* is Hoare (about method entry/exit), but its *scope* is universal (it
applies to all implementations). The interface declares the contract; the impl discharges
it. This is exactly parallel to how interface method signatures work — the signature is
universal (all impls must match), but each call is site-specific.

**\*\*\* impl methods and Hoare contracts.** Method bodies in an `impl` are computation
sites. They can carry their own `requires`/`ensures`, which must be consistent with (no
weaker than) whatever the interface method signature declared. If the interface says
`requires: P`, the impl can strengthen to `requires: P && Q` but not weaken to
`requires: true`.

**† Effect operations and Hoare contracts.** Effect operation signatures can carry
preconditions/postconditions that constrain what any handler implementing this effect must
provide:

```ash
effect State<S> {
    get() -> S ensures: result == current_cell
    put(s: S) -> Unit ensures: current_cell == s
}
```

This constrains the operation protocol: any handler for `State` must guarantee `get` returns
the current cell value and `put` sets it.

**†† Effect declarations and laws.** This is the equational theory of the effect's
operations — the equations the handler must satisfy for soundness (NOTE-013 §9 Level 2):

```ash
effect State<S> {
    get() -> S
    put(s: S) -> Unit

    law get_put_identity(k: () -> {State<S>} Unit)
      : bind(get(), λs. bind(put(s), k)) ≡ k
}
```

This is where the effect system meets the proof system: the effect declares the theory, the
handler proves it satisfies the theory.

**‡ Handler clauses and Hoare contracts.** Handler clause bodies are computation sites. A
handler clause body can carry `requires`/`ensures`:

```ash
handle computation with {
    get(k) ensures: k receives current cell value { ... }
    put(s, k) ensures: cell updated to s { ... }
}
```

**‡‡ Handlers and laws.** Handler soundness. The handler must satisfy the equational theory
declared by the effect (††). This is a `law` on the handler — or more precisely, the handler
carries a `proof` discharging the effect's laws. The proof obligation lands here:

```ash
handler state_handler<S>(initial: S) for State<S> {
    get(k) => resume(k(current_cell))
    put(s, k) => resume(k(()))

    -- Proof that this handler satisfies State's equational theory:
    proof get_put_identity(k) { by_definition }
}
```

**§ Capabilities and laws.** Capabilities have an authority algebra — laws about how
authority composes, delegates, and revokes:

```ash
capability DbAccess {
    read(table: String, key: String) -> Option<Row>
}

law delegation_revocation<A>(
    cap: A,
    delegatee: A,
    eq: Eq<Authority<A>>
) : eq.equiv(
    revoke(delegate(cap, delegatee), delegatee),
    authority(cap)
)
```

**Capabilities as effect-owned extern boundaries.** If capabilities collapse into effects
and handlers, the host boundary must still remain explicit. A capability operation such as
`fs.read`, `net.listen`, or `llm.chat` is not an ordinary pure function. It is a typed effect
operation with authority, contract, failure, and evidence obligations.

The safe Ash-facing declaration is the operation interface (per NOTE-022):

```ash
interface Fs {
    fn read(path: String) -> String
        requires: allowed_path(path)
        raises: FsError
        evidence: fs_trace
}
```

**Host/FFI and extern placement have been consolidated in [NOTE-024](NOTE-024-HOST-FFI-AND-EXTERN.md).**

The current target position (per NOTE-024): `extern` is a reserved keyword with no grammar
production; `builtin(...)` is the only host-reaching mechanism, callable only inside trusted
stdlib handler/provider method bodies. The prior `extern unsafe fn` two-placement proposals
(Placement A: interface-attached, Placement B: handler-local) and the four obligation layers
are archived in NOTE-024 §3 as the design space for a future host/FFI spec. NOTE-022
invalidated Placement A.

The contract layer separation remains relevant regardless of the host-reaching mechanism:

| Layer | What it states | Mechanism |
|-------|----------------|-----------|
| Operation Hoare contract | caller/callee obligations such as `allowed_path(path)` | `requires` / `ensures` on the effect operation and handler clause |
| Handler law | semantic theory, e.g. read-after-write, replay equivalence, mock equivalence | `law` on the effect, `proof` on the handler |
| ABI safety claim | host string/bytes ownership, raw error shape, async/blocking convention | `builtin(...)` in trusted stdlib, or future `extern unsafe` adapter (see NOTE-024) |
| Authority claim | whether this execution may install/use the handler | row discharge/admission evidence |

The key point: laws can express handler algebraic theories, but they should not pretend to
prove the entire host ABI. The safe effect operation is where Ash's Hoare contracts, laws,
failure effects, and provenance attach.

### 8.3 Type Invariants: The Bridge

Type declarations carry `invariant` — a Hoare-shaped contract checked at construction and
preserved by every operation:

```ash
type SortedList<A: Ord> = Cons { head: A, tail: SortedList<A> }
    invariant: tail.is_empty() || head <= tail.head
```

This is the bridge between computation-site and declaration-level contracts:

- It is **declared on the type** (declaration scope).
- It is **checked at every computation site** that constructs or mutates the type
  (computation-site discharge).
- The type checker must prove the invariant holds at construction and is preserved by any
  operation that modifies the value.

This connects directly to refinement types (§3.1): the invariant becomes a refinement
predicate on the type, and every constructor/modifier must be verified against it.

### 8.4 The Clean Principle

Contracts attach at two levels:

```
LEVEL 1: Computation sites (fn bodies, handler clauses, impl methods, capability ops)
   → Hoare contracts: requires, ensures, invariant
   → checked at that site (static via SMT, or dynamic via Raise)
   → logical shape: {P} C {Q}

LEVEL 2: Declarations (types, interfaces, effects, modules)
   → laws and properties: universal theorems ∀x⃗. P(x⃗)
   → proofs attached to impl/handler discharge laws declared at level 2
   → invariant on types is the bridge: declared at level 2, checked at level 1
```

Interface methods and effect operations are the overlap: they are declarations (universal
scope) that carry Hoare contracts (computation-site shape) inherited as proof obligations
for implementors. This is not a contradiction — it is the same universal/site distinction
that already exists for method signatures.

## 9. The Precise Semantics for Core/IR Lowering

### 9.1 Lowering Rules

| Surface construct | Core/IR lowering | Row contribution |
|---|---|---|
| `requires: P` (static, proved) | Refinement on param types | Erased (`Static` discharge) |
| `requires: P` (static, unknown) | Demote to dynamic | `Failure` effect |
| `requires: P` (dynamic) | `Raise ContractViolation` at entry | `Failure` effect |
| `ensures: Q` (static, proved) | Refinement on return type | Erased (`Static` discharge) |
| `ensures: Q` (dynamic) | `Raise ContractViolation` at exit | `Failure` effect |
| `invariant: I` (static) | Refinement predicate | Erased (`Static` discharge) |
| `invariant: I` (dynamic) | `Raise ContractViolation` at boundary | `Failure` effect |
| `law { ∀x⃗. P(x⃗) }` (proved) | Evidence ref in `ContractDischarge` | Erased (`Evidence` discharge) |
| `law { ∀x⃗. P(x⃗) }` (tested) | Advisory evidence ref | Erased (advisory) |
| `law { ∀x⃗. P(x⃗) }` (unproven) | Compile error or deferred | Blocking |
| `property { ∀x⃗. P(x⃗) }` | Test harness metadata | None (not in IR) |

### 9.2 The RecordDischarge Administrative Term

SPEC-098b §2.3 defines `RecordDischarge` as an administrative term that records contract
discharge status:

```rust
RecordDischarge { discharge: ContractDischarge, body: Term }
```

This is a no-op at runtime but preserves metadata for audit and evidence caching. Every time
a contract effect is discharged (statically or via evidence), a `RecordDischarge` node
records the discharge. This ensures the audit trail is complete even when the contract is
erased from the runtime row.

### 9.3 Gradual Verification Flow

```
requires: P
  │
  ├─ SMT proves P at call site → erase, RecordDischarge(Static)
  │
  ├─ SMT disproves P → compile error (counterexample)
  │
  └─ SMT unknown / explicit dynamic
       │
       ├─ Insert Raise ContractViolation at call site
       ├─ Failure effect in row
       └─ Handler at runtime boundary discharges or propagates
```

## 10. The Two Systems Are Not Schizophrenic — They Are Layered

The apparent schizophrenia dissolves when you see the layering:

```
LAYER 1: Properties + tested laws (falsification regime)
   ↓ hunt for counter-evidence; repeated independent runs increase confidence
   ↓ a single counterexample refutes; a million passes do not prove
   ↓ never discharges anything in the type system

LAYER 2: Proven laws (the trusted subset)
   ↓ proof certificates (SMT / Lean / by_definition)
   ↓ trusted facts that discharge Hoare preconditions

LAYER 3: Hoare contracts (site-specific triples)
   ↓ static: refinement types, SMT-checked (may use proven law evidence)
   ↓ dynamic: Raise ContractViolation, handler-discharged

LAYER 4: Computation row (the IR accounting)
   ↓ carries all contract items until discharged
   ↓ discharge mode determines runtime representation
   ↓ RecordDischarge preserves audit trail
```

Each layer feeds the one below. Properties and tested laws hunt for refutation. Proven laws
produce trusted facts. Trusted facts discharge Hoare preconditions. Hoare contracts appear in
the computation row. The row is discharged by the appropriate mechanism.

The falsificationist framing resolves the apparent schizophrenia: there is one pipeline, but
two epistemic regimes within it.

- **Proven** (Layers 2-3): the critical subset where absolute certainty matters. Proofs
  discharge types. Small, trusted.
- **Falsified/Testing** (Layers 1, 3): everything else. We declare a theory, hunt for
  counter-evidence, and honestly record what we found. Testing cannot prove, but repeated
  independent runs increase confidence the law *may* hold.

The two "systems" are different stages of the same pipeline:

- **System B (laws/properties)** operates at the declaration level. It declares the theory
  and gathers evidence (proof or counter-evidence).
- **System A (Hoare)** operates at the computation level. It consumes evidence (for static
  discharge) and/or checks at runtime (for dynamic discharge).

## 11. Contract Lifetimes and Operational Semantics

The previous sections described WHAT contracts are and WHERE they attach. This section
describes WHEN they are checked, discharged, and used, across the full compilation pipeline.

### 11.1 The Pipeline and Its Lifetimes

Ash compilation proceeds through several stages, each with a distinct scope of visibility:

```
SURFACE (.ash)      CORE (.core)      CPS IR (.core.cps)      BACKEND (native/WASM)
  │                    │                    │                       │
  ├─ parse ───────────┤                    │                       │
  ├─ name resolution ─┤                    │                       │
  ├─ type infer/check ┤                    │                       │
  │  (SMT queries)    │                    │                       │
  ├─ law prop check ──┤                    │                       │
  │  (Prop totality)  │                    │                       │
  ├─ lowering ────────┼─→                  │                       │
  │                    ├─ IR lowering ─────┤                       │
  │                    │                    ├─ module load ────────┤
  │                    │                    │  (cross-module        │
  │                    │                    │   evidence resolution)│
  │                    │                    ├─ link time ──────────┤
  │                    │                    │  (whole-program SMT,  │
  │                    │                    │   LTO, law gathering) │
  │                    │                    ├─ optimization ───────┤
  │                    │                    │  (TCO, inline, DCE,   │
  │                    │                    │   spec, dead handler) │
  │                    │                    ├─ codegen ────────────┤
  │                    │                    │                       ├─ run time
  │                    │                    │                       │  (dynamic checks,
  │                    │                    │                       │   handlers)
  └─ test time (SEPARATE pipeline: ash test)
     (QuickCheck, smallcheck, fuzzing — does not run during compilation)
```

### 11.2 The Monotonicity Principle

Contract discharge is **monotonic across the pipeline**: once a contract obligation is
discharged, it stays discharged. More importantly, a contract that was *undischargeable* at
an early stage (compile time) may *become* dischargeable at a later stage (module load, link
time) when more context is available.

```
Stage                  Visibility                     Can discharge
─────────────────────────────────────────────────────────────────────
Type check             one module (no bodies)         SMT on local refinements
Lowering               one module (with bodies)        SMT with body reasoning
Module load            imported modules' Core           cross-module law evidence
Link time              whole program                    whole-program SMT, all laws
Run time               live execution state             dynamic checks, handlers
```

Each stage can only *add* evidence, never remove it. This means the compiler pipeline should
be designed to **defer** discharge decisions when possible: a contract that is "unknown" at
type-check time should be carried forward (as a pending obligation, possibly with a dynamic
fallback) rather than rejected.

### 11.3 Per-Contract-Type Lifecycle

#### 11.3.1 Static Hoare (requires/ensures/invariant)

```
Type check
  │
  ├─ SMT proves P → refinement type, ERASED from runtime row permanently
  │                  (RecordDischarge{mode: Static})
  │
  ├─ SMT disproves P → COMPILE ERROR (counterexample). Pipeline stops.
  │
  └─ SMT unknown → CARRIED FORWARD as pending obligation
       │
       Lowering: demoted to Raise ContractViolation (dynamic fallback)
                  Failure effect enters row
       │
       Module load: SMT gains imported module bodies
         │
         ├─ now provable → ELIDE the Raise node, remove Failure effect
         │                  (RecordDischarge{mode: Static})
         │
         └─ still unknown → carried forward to link time
              │
              Link time: whole-program SMT
                │
                ├─ now provable → ELIDE (as above)
                └─ still unknown → reaches run time as dynamic check
                     │
                     Run time: Raise fires on violation
                                Handler catches (if installed) or trap (bottom)
```

Key point: the "unknown" branch is NOT a failure. It is a gradual-verification decision —
the contract carries forward with a dynamic fallback until (possibly) more context makes it
statically dischargeable.

#### 11.3.2 Dynamic Hoare (explicit dynamic or demoted)

```
Lowering → Raise ContractViolation node inserted at boundary
           Failure effect in row

Module load / Link time: may become statically provable
  → Raise node ELIDED (dead-code elimination, §11.4.2)
  → Failure effect removed from row

Run time: if not elided, Raise fires on violation
  → Handler (if installed) discharges or propagates
  → If no handler: trap (bottom), TrapReason::ContractViolation
```

#### 11.3.3 Laws

```
Type check
  │
  ├─ proof present (by_definition / by z3 / by lean)
  │    → totality check on proof body
  │    → ContractDischarge{mode: Evidence, evidence: ProofCert}
  │    → ERASED from runtime row
  │
  ├─ no proof, but test evidence available
  │    → ContractDischarge{mode: Evidence, evidence: TestResult}
  │    → ADVISORY: does not discharge type obligations
  │
  └─ no proof, no test evidence
       → pending obligation (untested assertion)

Lowering
  │
  ├─ proved law → evidence ref recorded, erased from row
  ├─ tested law → advisory evidence ref recorded, erased from row
  └─ untested → if law is required for a precondition discharge:
                  → COMPILE ERROR or DEFERRED status
                if law is standalone:
                  → carried forward as pending (warning)

Module load: cross-module law evidence
  │
  └─ Module A requires associative(op).
     Module B exports proof evidence for associative(+).
     → Module A can use B's evidence to discharge the precondition.
     → This is cross-module law evidence resolution.

Link time: ALL law evidence from ALL modules gathered
  │
  └─ Laws that had no proof at compile time may gain:
       ├─ proof evidence from another module
       └─ test evidence (if test harness ran and cached results)

Run time: laws are ERASED. No runtime overhead.
  Only RecordDischarge audit nodes preserve the discharge metadata.
```

#### 11.3.4 Properties

```
Test time ONLY (separate pipeline: ash test)
  │
  ├─ QuickCheck generates random cases
  ├─ Smallcheck enumerates finite domains
  ├─ Fuzzing generates adversarial inputs
  │
  └─ Result:
       ├─ all pass → ADVISORY: "survived N independent runs"
       │              Not proof. Not usable in types. Not in the binary.
       │
       └─ counterexample found → REPORTED as test failure
                                   Does NOT block compilation.
                                   Reported to the developer separately.

Module load / Link / Run time: properties do not exist.
  They are not lowered to IR. They are not in the binary.
```

### 11.4 Contract-Guided Optimizations

This is where contract evidence pays for itself. Proved contracts and discharged laws are not
just audit metadata — they are **enabling facts** for the optimizer.

#### 11.4.1 Tail-Call Optimization Through Handlers

If law evidence proves a handler is **deep** (always resumes — NOTE-013 §6), and its clause
body has no additional effects beyond the resume continuation, the compiler can deallocate the
handler frame after resume and emit a direct tail call.

Without this evidence, the compiler must be conservative: the handler might not resume
(shallow strategy, like Option/Either), so the frame must survive.

```
-- With evidence: state_handler is deep + clause body is pure
handle computation with { get(k) => resume(k(cell)) }
  → TCO: handler frame deallocated, direct jump to resume target

-- Without evidence: handler might not resume
handle computation with { op(k) => ... }
  → conservative: frame retained until clause body completes
```

This connects directly to NOTE-013 §6: the resume strategy table. A handler whose resume
strategy is provably deep and whose clause effects are a subset of the body row can be
optimized to a tail call.

#### 11.4.2 Dynamic Check Elision (Contract LTO)

A contract demoted to `Raise ContractViolation` at compile time might be provable at module-
load or link time (more context available). The `Raise` node is removed; the `Failure` effect
leaves the row. This is **LTO for contracts**:

```
Compile time: requires P unknown → Raise ContractViolation inserted
Module load:  caller body now visible → SMT proves P at this call site
              → Raise node ELIDED → Failure effect removed
              → optimization barrier lifted
```

This is why the monotonicity principle matters: deferring discharge to module load or link
time enables optimization that was impossible at compile time.

#### 11.4.3 Inlining Without Runtime Checks

If the caller can prove (via law evidence + SMT) that `requires: P` is satisfied at the call
site, the function can be inlined without inserting a dynamic check. Without evidence, the
dynamic check must be preserved even after inlining:

```
-- With evidence: associative(+) is proven
fn fold<A>(xs: List<A>, init: A, op: (A,A) -> A | associative(op)) -> A
→ fold can be inlined at call site without runtime check for associativity

-- Without evidence: associativity is only tested (not proven)
→ dynamic check (or contract handler) must remain even after inlining
```

#### 11.4.4 Effect Row Minimization

If law evidence proves a handler is sound (satisfies its equational theory — NOTE-013 §9
Level 2), the handler's effects can be removed from the residual row. This cascades: smaller
rows enable more optimization (fewer constraints on what the computation can do):

```
handle computation with { state_handler }
  → State effect removed from row (handler discharges it)
  → If the only consumer of State in the row was this scope:
     → row shrinks
     → downstream optimizations unblocked (fewer handler frames to maintain)
```

#### 11.4.5 Algebraic-Law-Driven Specialization

Proven laws enable algebraic transformations that would be unsound otherwise:

| Proven law | Optimization enabled |
|---|---|
| associativity | `fold` tree-restructuring, parallel reduction |
| commutativity | operation reordering for cache/schedule optimization |
| idempotence | deduplication: `f(f(x)) == f(x)` |
| distributivity | factoring: `a*(b+c) == a*b + a*c` |
| identity | elimination: `x <> mempty == x` |
| null-homotopy | `f(g(x)) == x` (inverse elimination) |

The compiler does not guess. It uses proven law evidence to justify transformations. Tested-
only laws (survived-testing) do NOT enable these optimizations — they are advisory only.

#### 11.4.6 Dead Handler Elimination

If computation-row analysis (from NOTE-013) shows that an effect operation is never raised in
a given scope (provable from the rows of all enclosed computations), the handler for
that operation is dead code:

```
handle computation with {
    get(k) => ...      -- dead: no Raise{get} in computation's row
    put(s,k) => ...    -- dead: no Raise{put} in computation's row
}
→ entire handler frame eliminated
```

This is row-precision analysis: the computation row tells you exactly which operations *could*
be raised, so handlers for absent operations are dead.

### 11.5 The Language Boundary and Contract Checking

Different languages in the pipeline carry contracts differently:

```
SURFACE: contracts are DECLARED (requires/ensures/law/property/proof)
         → human-readable, attached to declarations
         → all contract forms are syntactically present

CORE:    contracts are REFINEMENT TYPES + PENDING OBLIGATIONS
         → static Hoare → refinement predicates on types
         → laws → ContractEffect in the type, with discharge status
         → dynamic Hoare → Raise ContractViolation in the term
         → properties → metadata only, not in the term language

IR (CPS): contracts are ROW ITEMS + DISCHARGE RECORDS
         → ContractEffect variants in the computation row
         → DischargeMode determines runtime representation
         → RecordDischarge preserves audit trail
         → properties do not exist at this level

BACKEND: contracts are either ERASED or EMBEDDED
         → proven/static → erased (no code generated)
         → dynamic → Raise ContractViolation compiled to assertion + handler dispatch
         → advisory → erased (not in binary)
         → RecordDischarge metadata optionally preserved in debug info
```

The key transition: at the Surface→Core boundary, contracts are *structuralized* (declarations
become refinement types, pending obligations, or Raise nodes). At the Core→IR boundary, they
are *accounted* (row items with discharge status). At the IR→Backend boundary, they are
*realized or erased* (either compiled to runtime code or removed).

### 11.6 Module Load Time: The Cross-Module Evidence Boundary

Module load is the critical junction between compile time and link time. It deserves special
attention because it is where **cross-module law evidence resolution** happens:

```
Module A:  fn fold<A>(...) requires: associative(op) { ... }
Module B:  impl Semigroup<Int> { proof associativity { by_definition } }

When A imports B:
  1. B's Core is loaded (or B's evidence summary is read).
  2. B's ContractDischarge records for associative(+) become visible.
  3. A's pending obligation "requires associative(+)" is matched against B's evidence.
  4. If matched: A's obligation is DISCHARGED at module load time.
  5. If not matched: A's obligation carries forward to link time.
```

This means module load time is a real compilation phase with its own SMT queries and evidence
matching, not just file loading. It is the first point where **whole-program reasoning across
module boundaries** becomes possible.

The evidence summary that modules export must be sufficient for this resolution without
re-running the solver:

```
Module evidence export (per module):
  - ContractDischarge records (contract, mode, evidence ref)
  - Law proof certificates (for proven laws)
  - Law test results (for tested laws — advisory)
  - Refinement type summaries (for static Hoare contracts)

Module evidence import (at load time):
  - Match required obligations against exported evidence
  - Resolve SMT queries using imported refinement summaries
  - Record resolved obligations as RecordDischarge nodes
```

## 12. Gaps and Missing Angles

The preceding sections define WHAT contracts are, WHERE they attach, and WHEN they are
checked. This section catalogs perspectives and angles NOT yet addressed — gaps that must
be worked through before the contract system is well-defined. Each is tagged with priority
and the work it blocks.

### GAP 1: Blame and Accountability [CRITICAL — blocks diagnostics, blocks impl contract checking]
**Status: Resolved in NOTE-027.** Blame labels (`BlameLabel { party, polarity, module_path,
function_name, contract_text, source_span }`) carry diagnostic state through the IR.
Polarity: `requires` violated → caller (negative); `ensures` violated → callee/impl
(positive). Blame is immutable through handler composition — handler decisions (resume,
propagate, escape) are recorded separately, never as blame. See NOTE-027 §2 (Blame
Assignment), §3 (Blame Through Handler Composition), §4 (Diagnostic State).

When a dynamic contract fires, the system needs to know WHO violated it. This is the
Findler-Felleisen higher-order contract blame theory:

- `requires: P` violated → blame the **caller** (caller didn't establish the precondition)
- `ensures: Q` violated → blame the **callee** (function body failed to deliver)

Without blame labels, a violation just says "something broke." With blame, it says "module A
called safe_div without establishing b != 0" or "module B's sort impl failed its sortedness
postcondition." This is essential for diagnostics and maps to the obligation/guarantee model:
the caller has an obligation (establish P), the callee has a guarantee (deliver Q).

Blame also propagates through handler composition (NOTE-013). If a contract handler catches a
violation and resumes with a default, who carries the blame? The original caller, the handler
that chose to resume, or both? The handler composition algebra from NOTE-013 §7 applies here:
the innermost handler catches first, and the blame depends on nesting order.

**Blocks:** impl contract checking (§8.2 \*\*\*), runtime diagnostics, contract handler
semantics.

### GAP 2: Monadic Hoare Logic (Contract Composition Through bind) [ARCHITECTURAL — blocks verification of composed computations]

**Status: Resolved in NOTE-030.** Rows compose through `ρm ∪ ρk`, but contracts compose
through predicate transformers. For `m` ensuring `Q(a)` and continuation `k(a)` requiring
`R(a)`, the bind boundary creates the proof obligation `∀a. Q(a) ⇒ R(a)`. The generic
composed postcondition existentially threads the intermediate value: `∃a. Q(a) ∧ S(a, b)`.
Static proof records `ContractDischarge` metadata; failed dynamic fallback follows NOTE-029's
structured-bottom rule. See NOTE-030 §1 (Core decision), §3 (Types and contract summaries),
§4 (Semantics), and §5 (Worked examples).

The original gap was that NOTE-013 formalized row-polymorphic `bind`, and §1.1 sketched
ordinary sequential Hoare composition, but Ash did not yet have a rule for data-dependent
contract composition through `bind`:

```
f : Comp<{requires P}, A>         (precondition P, delivers A)
g : A -> Comp<{requires Q}, B>    (precondition Q on f's result, delivers B)

bind(f, g) : Comp<{requires ??, ensures ??}, B>
```

NOTE-030 refines the earlier sketch. The combined row is `ρf ∪ ρg`, but the continuation
precondition is discharged by the producer postcondition: for producer postcondition `Q(a)`
and continuation precondition `R(a)`, the central proof obligation is `∀a. Q(a) ⇒ R(a)`.
This is **monadic Hoare logic** / weakest-precondition reasoning, not simple sequential
predicate concatenation.

The computation row already accounts for effects by union, but contract predicates compose
through predicate transformers. NOTE-030 makes this modular, so composed computations do not
need to be inlined and re-proved from scratch every time.

**Blocks:** modular verification, contract-aware optimization of composed computations.

### GAP 3: Contract Subsumption / Variance [CRITICAL — blocks interface→impl contract inheritance]
**Status: Resolved in NOTE-027.** The behavioral subtyping rule is formalized as:
`{P} C {Q} ⊑ {P'} C {Q'} iff P ⇒ P' (precondition contravariant — weakens) and Q' ⇒ Q
(postcondition covariant — strengthens)`. Checked eagerly at impl definition time. Impl
with no explicit contracts inherits the interface's contracts exactly. See NOTE-027 §1
(Contract Subsumption), §1.5 (check timing), §5 (verification algorithm).

We stated that impl contracts must be "stronger than" interface contracts (§8.2), but never
formalized the subsumption rule. The standard behavioral subtyping rule is:

```
{P} C {Q} ⊑ {P'} C {Q'}  iff  P' ⇒ P   (contravariant precondition)
                                 and  Q ⇒ Q'  (covariant postcondition)
```

An impl can **weaken** the precondition (accept more inputs) and **strengthen** the
postcondition (guarantee more). Without this rule formalized, the interface→impl contract
inheritance from §8.2 is undefined — the type checker cannot verify that an impl satisfies its
interface's contracts.

This also interacts with the computation row: if an impl strengthens a postcondition, does the
row change? Usually not — the postcondition is about values, not effects. But an impl that adds
effects would violate row subtyping independently.

**Blocks:** interface method contracts (§8.2 \*\*), impl contract verification (§8.2 \*\*\*).

### GAP 4: Interaction with Evaluation Modes [DESIGN DECISION — connects to SPEC-101 lazy/memo]
**Status: Resolved in NOTE-028.** Purity is denotational: referential transparency is the
language-level test. `strict`/`lazy`/`memo` and the handler marker are purity-preserving
type attributes; impurity comes from residual/latent rows, not attributes. Contract timing:
strict checks at call/return boundaries, lazy checks on every force, memo checks on first
force and replays cached terminal outcomes. See NOTE-028 §1 (Purity model), §3 (Contract
timing principle), and §4 (Contract timing by mode).

SPEC-101 introduces lazy and memo computation modes. Contracts on lazy/memoized computations
have a temporal dimension we have not addressed:

- `requires` on a lazily-evaluated argument: checked when? At call site (eager) or at force
  (lazy)?
- `ensures` on a lazily-evaluated result: checked at construction or at force?
- `invariant` on a lazy data structure: checked at construction or at every force?
- Memoized computation: does the contract check fire once (at first evaluation, cached) or
  every access?

This connects to the temporal variance concern (NOTE-013 §9.4). The answer determines whether
contract checking is part of the computation's denotational semantics or part of its
operational semantics.

**Blocks:** contracts on lazy/memo computations, connection between contract system and
evaluation mode system.

### GAP 5: Concurrent / Distributed / Temporal Contracts [ARCHITECTURAL — needed for Proc/Workflow levels]

Everything in NOTE-014 assumes sequential, single-threaded computation. But the tower's Proc
and Workflow levels span multiple processes and agents. There, contracts look fundamentally
different:

- **Temporal contracts**: "this process must respond within 5s" — requires runtime monitoring,
  not SMT.
- **Supervision contracts**: "if child process fails, supervisor restarts it" — involves
  process lifecycle, not just values.
- **Obligation contracts**: "this workflow must eventually discharge obligation X" —
  **liveness** properties, not safety properties.
- **Policy contracts**: "only authorized roles can invoke this" — governance, not correctness.

These are not Hoare triples and not universal theorems. They are **temporal logic** properties
(LTL/CTL). The discharge mechanism is runtime monitoring, not SMT or QuickCheck. This is a
third contract paradigm that does not fit neatly into either System A or System B as currently
defined.

This may need its own treatment (possibly NOTE-015).

**Blocks:** Proc-level and Workflow-level contracts, the full tower.

### GAP 6: Contract Failure Observability and Bottom Behavior [CRITICAL — connects to user's first-class bottom concern]
**Status: Resolved in NOTE-029.** Default dynamic contract failure is structured bottom:
`Trap { reason: ContractViolation(ContractDiagnostic) }`. `ContractViolation` is not a row
item and is not implicitly resumable. Recoverable contract behavior must lower to an explicit
`fail` effect and expose `{fail ...}` in the row. Diagnostics preserve predicate, source span,
blame, observed values, call chain, discharge history, handler decisions, and replay status.
Lazy failures produce fresh diagnostics on each force; memo failures replay the first terminal
diagnostic. See NOTE-029 §1 (Core decision), §2 (Diagnostic payload), §3 (Lowering semantics),
and §5 (Memo and lazy replay semantics).

The previous draft treated `ContractViolation` as either a trap (bottom) or a raised effect,
which left the row/type boundary unclear. NOTE-029 resolves this: default contract failure is
structured bottom (`Trap`), while recoverable behavior must use an explicit `fail` effect with
row accounting. The diagnostic-survival question is answered by `ContractDiagnostic`.

Contract failure IS a structured bottom — it should carry rich diagnostic state:

- Source location of the violated contract
- Values involved in the violation (actual arguments, not just the predicate text)
- The call stack / continuation chain at the point of violation
- The discharge history — was this contract demoted from static? Was it always dynamic?
- Whether a handler was installed and chose not to resume
- Blame label (GAP 1)

NOTE-029 resolves the boundary question: `ContractViolation` is a built-in trap reason for
default dynamic contract failure, not a user-defined resumable effect. Recoverable contract
behavior is explicit and lowers to `Raise { op: fail ..., ... }`; the corresponding `fail`
item appears in the row. Thus terminal failure and recoverable failure are distinguished by
lowering and row accounting, not by overloading `ContractViolation` itself.

**Blocks:** runtime diagnostics, handler design, failure row accounting.

### GAP 7: Meta-Level Soundness [THEORETICAL — needed for trust in optimizer and gradual verification]

There are soundness theorems ABOUT the contract system that we assume but have not stated:

- **Gradual verification soundness**: if SMT proves P statically, then no runtime check for P
  will ever fire. (If this fails, the SMT result was wrong.)
- **Optimizer soundness**: if law evidence justifies an optimization, the optimized code is
  semantics-preserving. (If this fails, the law proof was wrong, or the optimizer misapplied
  it.)
- **Blame soundness**: if blame is assigned to party X, then party X actually violated the
  contract. (Not party Y.)

These are meta-theorems — proofs about the proof system. They matter because the whole
optimization story (§11.4) and gradual verification flow (§9.3) rest on their validity. They
should be stated (even if proofs are deferred) before we trust the system.

**Blocks:** trust in contract-guided optimization, trust in gradual verification.

### GAP 8: The Contract ↔ Capability Boundary [DESIGN DECISION — needed for Failure effect clarity]

The attachment matrix (§8) puts capabilities and contracts as separate `EffectItem` kinds, but
their interaction needs discussion. A capability check ("does caller have db.read authority?")
is an authority question. A contract check ("is key != null?") is a correctness question. Both
appear in the row, both can raise. But:

- Capability violations are about PERMISSION (you are not allowed)
- Contract violations are about CORRECTNESS (your inputs/outputs are wrong)

These are morally different failures. Conflating them in a single `Failure` effect might lose
information. Or it might be fine — both are handler-dischargeable, and the handler can
distinguish by inspecting the effect's payload. But this is a design decision that should be
explicit, especially given the capability authority algebra in §8.2.

The effect-owned extern model sharpens the boundary:

- Authority denial belongs to the admission/row-discharge path: no handler/binding may be
  installed for the requested operation effect.
- Contract violation belongs to the operation protocol: a handler exists, but the operation's
  Hoare precondition/postcondition failed.
- ABI failure belongs to the unsafe adapter path: the host call failed or returned data that
  could not be decoded into the Ash operation's declared type.

These may all lower through `Failure`-like runtime reporting, but their causes should remain
distinct in diagnostics and evidence. This gap is therefore narrowed, not closed: the failure
taxonomy still needs concrete row/IR spelling.

**Blocks:** Failure effect taxonomy, capability-contract interaction semantics.

### GAP 9: Contract Lowering from Surface to Core [CRITICAL — known implementation gap]

**Status: Partially resolved in NOTE-031.** NOTE-031 defines the predicate well-formedness
boundary that must run before Surface predicates become Core constraints: expression-like
surface predicates are classified as `StaticPredicate`, `DynamicPredicate`, or rejected;
`old(...)` becomes a boundary-local `SnapshotRef`; predicate faults are distinct from false
predicates; and unsupported-but-pure forms remain dynamic rather than being silently erased.
The remaining implementation work is the concrete Core AST/schema and lowering algorithm.

The user previously identified (2026-04-10) that contract lowering from surface contracts
(`Requirement::Arithmetic { expr: Expr }`) to core contracts (structured `{ var, constraint }`)
is a major dependency that needs to be made explicit. This is the Surface→Core boundary
described abstractly in §11.5 ("structuralized") but not concretely specified.

The surface has expression-based contract predicates. The Core needs structured
constraint/predicate representations suitable for SMT queries. Before NOTE-031, the mapping
between these two representations — what expression forms are supported, how they translate to
SMT-consumable constraints, what happens to unsupported forms — was undefined. NOTE-031 now
defines the classification boundary; the remaining gap is the concrete Core predicate schema
and lowering algorithm.

**Blocks:** concrete implementation of Surface-to-Core predicate lowering and static contract
checking over the Core predicate schema.

## 13. Open Questions

1. **`old(x)` in postconditions.** How do we snapshot pre-state for `ensures` clauses that
   reference the pre-call value? **Resolved in NOTE-031.** `old(...)` is boundary-local and
   lowers to a `SnapshotRef` captured before the body governed by the postcondition runs.
   Snapshot expressions are initially limited to boundary paths such as `old(x)` and
   `old(x.field)`, not arbitrary computations.

2. **Refinement types + row polymorphism interaction.** How do refinements interact with
   open rows? E.g., `fn f<A | P>(x: A) -> {r} B` — does the refinement `P` constrain the
   row variable `r`?

3. **Law inheritance across interface constraints.** SPEC-080 establishes that `Monad`
   requires `Applicative` evidence. Do `Monad` laws inherit `Applicative` law obligations?
   The current design says no (D3 in DESIGN-NOTE-INTERFACE-LAWS), but this may need
   revisiting.

4. **Cross-handler contract equations.** When a dynamic contract handler is composed with
   other handlers (per NOTE-013), what cross-handler equations hold? E.g., does state
   preservation on contract violation depend on handler nesting order? (Yes, per NOTE-013
   §7.2 — this is derivable.)

5. **ContractViolation as built-in vs. user-defined effect.** Should `ContractViolation` be
   a built-in `Failure` effect, or a user-defined algebraic effect? The current IR models
   it as `TrapReason::ContractViolation` (a trap, not a resumable operation) for the
   unrecoverable case, and as `Raise { op: Failure(...) }` for the recoverable case. The
   boundary needs clarification.

6. **Evidence serialization and cross-module caching.** Law evidence (proof certificates,
   test results) needs to be serializable and cacheable across module boundaries. The
   `.ash/law-cache.toml` mechanism is a start, but cross-package evidence semantics remain
   future work.

7. **Property vs. law boundary.** When does a property "graduate" to a law? Should there be
   a syntax for promoting a tested property to a proven law? Or are they always separate
   declaration forms?

8. **Effect-local extern placement.** **Consolidated in NOTE-024.** `extern` is a reserved
   keyword with no grammar production in the current target language. `builtin(...)` is the
   only host-reaching mechanism. The prior two-placement model (interface-level vs
   handler-level) is archived in NOTE-024 §3. NOTE-022 invalidated Placement A.

## 14. References

### Internal references

- **NOTE-013** — Ambient monad and handler composition algebra. This note's companion.
  `docs/notes/NOTE-013-AMBIENT-MONAD-AND-HANDLER-COMPOSITION-ALGEBRA.md`
- **hoare-clauses.md** — Earlier design note exploring Hoare clauses in Ash. Established
  that Hoare is sufficient for dynamic contracts but not for laws/properties.
  `docs/design/hoare-clauses.md`
- **DESIGN-NOTE-INTERFACE-LAWS.md** — Interface laws syntax, semantics, and Curry-Howard
  roadmap. The implemented MVP for `law`/`proof` declarations.
  `docs/design/DESIGN-NOTE-INTERFACE-LAWS.md`
- **SPEC-027** — Pure functions with `requires`/`ensures`.
  `docs/spec/SPEC-027-PURE-FUNCTIONS.md`
- **SPEC-079** — Standard algebra, comonad, and Kleisli helpers (laws on algebraic
  interfaces).
  `docs/spec/SPEC-079-STANDARD-ALGEBRA-COMONAD-AND-KLEISLI-HELPERS.md`
- **SPEC-080** — Interface evidence constraints (`Monad` requires `Applicative`).
  `docs/spec/SPEC-080-INTERFACE-EVIDENCE-CONSTRAINTS.md`
- **SPEC-081** — Law test evidence substrate (`by test` evidence modes: authored, property,
  small-world).
  `docs/spec/SPEC-081-LAW-TEST-EVIDENCE-SUBSTRATE.md`
- **SPEC-082** — Property generation and shrinking substrate (QuickCheck infrastructure).
  `docs/spec/SPEC-082-PROPERTY-GENERATION-AND-SHRINKING-SUBSTRATE.md`
- **SPEC-097b** — Target type system: `ContractEffect` enum, row syntax, discharge.
  `docs/spec/SPEC-097b-TARGET-TYPE-SYSTEM.md`
- **SPEC-098b** — Target CPS IR: `ContractDischarge`, `DischargeMode`,
  `RecordDischarge`, `Raise`/`Handle` for contracts.
  `docs/spec/SPEC-098b-TARGET-IR.md`
- **SPEC-006** — Policy definitions with `where` invariants.
  `docs/spec/SPEC-006-POLICY-DEFINITIONS.md`

### External references

- **Hoare, "An Axiomatic Basis for Computer Programming"** (1969).
  Original Hoare logic. The `{P} C {Q}` triple.
  <https://doi.org/10.1145/363235.363259>

- **Floyd, "Assigning Meanings to Programs"** (1967).
  Preconditions and postconditions via flowchart annotation. Precursor to Hoare logic.
  <https://doi.org/10.1007/978-94-010-9750-6_6>

- **Claessen & Hughes, "QuickCheck: A Lightweight Tool for Random Testing of Haskell
  Programs"** (2000).
  Property-based testing. The foundation for Ash's property/QuickCheck evidence.
  <https://doi.org/10.1145/357766.351266>

- **Runciman, Chepalyk, & Chitil, "SmallCheck: A New Tool for Testing Haskell Programs"
  (2008)**.
  Exhaustive testing for finite domains. The foundation for small-world evidence.
  <https://doi.org/10.1145/1411286.1311290>

- **Freeman & Pfenning, "Refinement Types for ML"** (1991).
  Refinement types as the static-checking mechanism for Hoare contracts.
  <https://doi.org/10.1145/115374.115380>

- **Swamy et al., "Dependent Types and Multi-Monadic Effects in F*"** (2016).
  Dependent types + effect system + SMT for proving program properties. The closest
  industrial system to Ash's target (refinement types + computation rows + proof evidence).
  <https://doi.org/10.1145/2946614>

- **de Moura & Bjørner, "Z3: An Efficient SMT Solver"** (2008).
  The SMT solver used for static contract discharge.
  <https://doi.org/10.1007/978-3-540-78800-3_24>

- **Findler & Felleisen, "Contract Systems for Higher-Order Functions"** (2002).
  Higher-order contracts with dynamic checking. Relevant to how `ContractViolation` should
  behave as a runtime effect.
  <https://doi.org/10.1145/581478.581488>

- **Dimoulas et al., "Correct Monotonic Contracts"** (2015).
  Semantics of contract systems where contract satisfaction is monotonic. Relevant to the
  interaction between contracts and the computation row.
  <https://doi.org/10.1007/s10990-015-9243-2>

- **Greenberg, Pierce, & Weirich, "Contracts Made Manifest"** (2010).
  Connecting contract systems to refinement types and higher-order contracts.
  <https://doi.org/10.1145/1863534.1863541>

- **Chugh, Rondon, & Jhala, "Nested Refinements: A Logic for Deductive Verification"**
  (2012).
  Liquid types / nested refinement types. Relevant to how refinement predicates compose
  with the computation row.
  <https://doi.org/10.1145/2398857.2378575>

- **Popper, "The Logic of Scientific Discovery"** (1934/1959).
  Falsificationism: scientific theories can never be verified, only corroborated (survive
  refutation attempts) or falsified. The epistemological basis for the distinction between
  proof and test evidence in Ash's contract system.
  <https://doi.org/10.4324/9780203994627>

- **Pacheco & Ernst, "Randoop: Feedback-Directed Random Testing"** (2007).
  Automated feedback-directed fuzzing for object-oriented programs. Relevant to how Ash's
  property/fuzzing regime can generate effective counter-evidence.
  <https://people.csail.mit.edu/cpacheco/publications/randoopjava.pdf>, <https://randoop.github.io/randoop/>

## 15. Changelog

| Date       | Change |
|------------|--------|
| 2026-06-23 | Initial version. Two-system analysis, discharge matrix, lowering rules, layering story, connection to NOTE-013 handler algebra. |
| 2026-06-23 | Added falsificationist epistemology (§4.3-4.4). Reframed evidence as Popperian counter-evidence gathering: proofs cover a critical subset, everything else survives testing. Updated discharge matrix and layering to distinguish proven vs. survived-testing vs. refuted. |
| 2026-06-23 | Added §8 attachment matrix: which contract types attach to which declaration forms. Interface methods and effect operations carry inherited Hoare proof obligations. Type invariants are the bridge between declaration-level and computation-site contracts. |
| 2026-06-23 | Added §11 contract lifetimes and operational semantics: pipeline stages, monotonicity principle, per-contract-type lifecycle, contract-guided optimizations (TCO through handlers, contract LTO, inlining without checks, row minimization, algebraic specialization, dead handler elimination), language boundary (Surface/Core/IR/Backend), module load time as cross-module evidence boundary. |
| 2026-06-23 | Added §12 gaps and missing angles: blame/accountability, monadic Hoare logic, contract subsumption/variance, evaluation mode interaction, concurrent/temporal contracts, contract failure observability, meta-level soundness, contract↔capability boundary, contract lowering surface→core. |
| 2026-06-24 | Added §8 capability/effect-owned extern boundary: capabilities can be expressed as effect operations plus handlers, while raw host/FFI externs remain effect-local unsafe implementation hooks. Clarified the split between Hoare contracts, handler laws, ABI safety, and authority admission. Updated GAP 8 and open questions with the resulting failure-taxonomy and placement questions. |
| 2026-06-24 | Expanded the effect-owned extern boundary with two placement alternatives and their contract utility: effect-level externs for canonical host ABIs, and trusted-handler externs for backend-specific adapters. Updated Open Question 8 to treat placement as a surface-syntax decision over a shared semantic invariant. |
| 2026-06-27 | Normalized target-row wording from effect row to computation row while leaving the detailed fact/evidence/obligation model to a separate follow-up track. |
| 2026-06-27 | Applied NOTE-022 decision: replaced all `effect Fs { ... }` declaration examples with `interface Fs { ... }`. Externs now shown as dispatch-side constructs with `for Fs` ownership annotation (Placement A) or handler-local (Placement B). The contract layering (Hoare contract, handler law, ABI safety, authority claim) is unchanged — only the declaration keyword changes. |
| 2026-06-27 | Consolidated host/FFI and extern placement into NOTE-024. Replaced the detailed §8 extern placement content (Placement A/B, typing rules, four obligation layers) with a pointer to NOTE-024. Preserved the contract layer separation table, updating the ABI safety mechanism column to reference `builtin(...)` and NOTE-024. Updated Open Question 8 to reference NOTE-024. The current target position: `extern` is reserved with no grammar production; `builtin(...)` is the only host-reaching mechanism. |
| 2026-06-28 | GAP 1 (blame) and GAP 3 (subsumption) resolved in NOTE-027. Blame labels formalized: party (Caller/Callee/Impl), polarity (Negative/Positive), module path, source span. Blame is immutable through handler composition. Subsumption rule: `P ⇒ P'` (precondition weakens) and `Q' ⇒ Q` (postcondition strengthens), checked eagerly at impl definition. Original gap descriptions preserved for context. |
| 2026-06-28 | GAP 4 (contracts × evaluation modes) resolved in NOTE-028. Purity is denotational: `strict`/`lazy`/`memo` and the handler marker are purity-preserving attributes; impurity comes from residual/latent rows. Contract timing: strict checks at call/return, lazy checks on every force, memo checks on first force and replays cached terminal outcomes. |
| 2026-06-28 | GAP 6 (contract failure observability and bottom behavior) resolved in NOTE-029. Default dynamic contract failure is structured bottom: `Trap { reason: ContractViolation(ContractDiagnostic) }`. `ContractViolation` is not a row item or implicit resumable effect; recoverable behavior lowers to explicit `fail` and row-accounts the failure. Diagnostics preserve blame, predicate, observed values, call chain, discharge history, handler decisions, and replay status. |
| 2026-06-28 | GAP 2 (monadic Hoare logic) resolved in NOTE-030. Rows compose by union, while contracts compose through predicate-transformer reasoning: producer postconditions discharge continuation preconditions (`∀a. Q(a) ⇒ R(a)`), and composed postconditions existentially thread the intermediate value (`∃a. Q(a) ∧ S(a, b)`). |
| 2026-06-29 | NOTE-031 resolved the `old(x)` snapshot open question and partially resolved GAP 9. Contract predicates are now classified before lowering: SMT-safe static predicates, pure dynamic predicates, and rejected effectful/unstable predicates. `old(...)` lowers to boundary-local snapshot metadata, and predicate faults are distinct from false predicates. |
