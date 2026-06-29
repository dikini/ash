# Hoare Clauses in Ash: Static and Dynamic Contracts

## Status

Design note. Explores whether Hoare logic (preconditions, postconditions, invariants) is sufficient to express Ash's contract system, and how static vs dynamic checking maps to Hoare clauses.

## Summary

Ash has multiple contract mechanisms:
- `requires` / `ensures` on functions
- `where` on policies
- `laws` for algebraic properties
- `properties` for QuickCheck testing

Hoare clauses (`{P} C {Q}`) are **sufficient for dynamic contracts** (runtime checking) but **not sufficient for static laws and properties** without extending to theorem proving.

## Current Ash Contracts

### Function Contracts (SPEC-027)

```ash
fn safe_div(a: Int, b: Int) -> Int
    requires: b != 0
{
    a / b
}
```

- `requires`: Precondition (Hoare `P`)
- `ensures`: Postcondition (Hoare `Q`)

### Policy Invariants (SPEC-006)

```ash
policy BoundedResource {
    min: Int,
    max: Int,
} where {
    min <= max
}
```

- `where`: Invariant (Hoare `I`)

### Laws (SPEC-079)

```ash
law associativity<A: Semigroup> {
    forall x, y, z: A.
    (x <> y) <> z == x <> (y <> z)
}
```

- `law`: Universal property (theorem, not Hoare triple)

### Properties (QuickCheck)

```ash
property commutativity<A: Commutative> {
    forall x, y: A.
    x <> y == y <> x
}
```

- `property`: Statistical hypothesis (tested, not proven)

## Hoare Logic: The Basics

A Hoare triple: `{P} C {Q}`

- `P`: Precondition (must hold before `C`)
- `C`: Command/computation
- `Q`: Postcondition (must hold after `C`)

For loops: `{I} while B do C {I && !B}`
- `I`: Loop invariant (must hold before, during, and after)

## Mapping Ash Contracts to Hoare Clauses

| Ash Contract | Hoare Clause | Static Check | Dynamic Check |
|-------------|-------------|-------------|--------------|
| `requires: P` | Precondition `P` | SMT solver | Runtime assertion |
| `ensures: Q` | Postcondition `Q` | SMT solver | Runtime assertion |
| `where { I }` | Invariant `I` | SMT solver | Runtime assertion |
| `law { forall x. P(x) }` | Universal theorem | Theorem prover (Lean/Coq) | QuickCheck (testing) |
| `property { forall x. P(x) }` | Statistical hypothesis | N/A | QuickCheck (testing) |

## Static vs Dynamic Hoare Clauses

### Static Checking (Compile-Time)

```ash
-- Static requires: checked by SMT at compile time
fn safe_div(a: Int, b: Int) -> Int
    static requires: b != 0
    static ensures: result * b == a
{
    a / b
}
```

The SMT solver proves:
- `b != 0` implies `a / b * b == a`
- If the solver cannot prove, compilation fails

### Dynamic Checking (Runtime)

```ash
-- Dynamic requires: checked at runtime
fn safe_div(a: Int, b: Int) -> Int
    dynamic requires: b != 0
    dynamic ensures: result * b == a
{
    a / b
}
```

Generated code:
```rust
fn safe_div(a: i64, b: i64) -> i64 {
    assert!(b != 0, "requires failed: b != 0");
    let result = a / b;
    assert!(result * b == a, "ensures failed: result * b == a");
    result
}
```

### Both Static and Dynamic

```ash
-- Checked at compile time AND runtime
fn safe_div(a: Int, b: Int) -> Int
    requires: b != 0          -- static (SMT)
    ensures: result * b == a  -- static (SMT)
    dynamic requires: b != 0  -- runtime assertion
    dynamic ensures: result * b == a  -- runtime assertion
{
    a / b
}
```

## Are Hoare Clauses Sufficient?

### For Dynamic Contracts: Yes

Dynamic `requires`, `ensures`, and `where` are exactly Hoare clauses checked at runtime.

```ash
-- Dynamic Hoare triple
fn transfer(from: Account, to: Account, amount: Money) -> Result
    dynamic requires: from.balance >= amount
    dynamic ensures: from.balance == old(from.balance) - amount
    dynamic ensures: to.balance == old(to.balance) + amount
{
    ...
}
```

### For Static Contracts: Partially

Static `requires` and `ensures` are Hoare clauses checked by SMT. But:
- SMT can only check **decidable** fragments (arithmetic, arrays, bitvectors)
- SMT cannot check **universal quantification** over unbounded types (e.g., `forall x: Int. P(x)`)
- SMT cannot check **recursive properties** (e.g., `length(append(xs, ys)) == length(xs) + length(ys)`)

For these, we need a **theorem prover** (Lean, Coq, Isabelle).

### For Laws: No

Laws are **universal theorems**, not Hoare triples:

```ash
-- Law: must hold for all values, all time
law associativity<A: Semigroup> {
    forall x, y, z: A.
    (x <> y) <> z == x <> (y <> z)
}
```

This is not `{P} C {Q}`. It is `forall x, y, z. P(x, y, z)`.

Hoare logic cannot express this directly. We need:
- **Algebraic reasoning** (equational logic)
- **Induction** (for recursive types)
- **Theorem prover** (for proof)

### For Properties: No

Properties are **statistical hypotheses** tested by QuickCheck:

```ash
-- Property: tested on random samples
property commutativity<A: Commutative> {
    forall x, y: A.
    x <> y == y <> x
}
```

This is not Hoare logic. It is **property-based testing**.

## The Design for Ash

### Core: Hoare Clauses

Core Ash supports:
- `requires` (precondition)
- `ensures` (postcondition)
- `invariant` (loop/data structure invariant)

Both static (SMT) and dynamic (runtime) checking.

### Sugar: Laws and Properties

Laws and properties are **sugar** for:
- Laws: `theorem` in Lean/Coq
- Properties: `property` in QuickCheck

They are not Hoare clauses. They are separate mechanisms.

### The Surface Syntax

```ash
-- Hoare clauses (core)
fn safe_div(a: Int, b: Int) -> Int
    requires: b != 0
    ensures: result * b == a
{
    a / b
}

-- Laws (sugar for theorem prover)
law associativity<A: Semigroup> {
    forall x, y, z: A.
    (x <> y) <> z == x <> (y <> z)
}

-- Properties (sugar for QuickCheck)
property commutativity<A: Commutative> {
    forall x, y: A.
    x <> y == y <> x
}
```

## Can Laws Be Embedded in Hoare Logic?

Partially. We can make laws into invariants:

```ash
-- Law as invariant: checked at specific points
fn semigroup_op<A>(x: A, y: A, z: A) -> A
    invariant: (x <> y) <> z == x <> (y <> z)
{
    ...
}
```

But this is **weaker** than a law:
- Invariant: checked at specific points
- Law: must hold for all values, all time

## The Verdict

| Contract Type | Hoare Clause? | Static Check | Dynamic Check |
|-------------|-------------|-------------|--------------|
| `requires` | Yes (precondition) | SMT | Runtime assertion |
| `ensures` | Yes (postcondition) | SMT | Runtime assertion |
| `where` | Yes (invariant) | SMT | Runtime assertion |
| `law` | No (universal theorem) | Theorem prover | QuickCheck |
| `property` | No (statistical hypothesis) | N/A | QuickCheck |

Hoare clauses are **sufficient for the core contract system** (requires, ensures, where). They are **not sufficient for laws and properties**, which require separate mechanisms (theorem prover, QuickCheck).

## Static vs Dynamic: Unified View

Static Hoare logic is a **type system extension** (refinement types). Dynamic Hoare logic is an **effect system extension** (contract violation effects).

### Static Hoare as Type System Extension

Static Hoare clauses refine types:

```text
-- Base type
fn safe_div(a: Int, b: Int) -> Int

-- Refined type (Hoare as refinement)
fn safe_div(a: Int | b != 0, b: Int | b != 0) -> Int | result * b == a
```

The type checker collects constraints and passes to SMT:
- `requires: b != 0` → constraint `b != 0` on parameter types
- `ensures: result * b == a` → constraint `result * b == a` on return type

**SMT result handling:**

| SMT Result | Action |
|-----------|--------|
| `sat` (proven) | Compile with refinement (static only) |
| `unsat` (disproven) | Compilation error (counterexample provided) |
| `unknown` | **Demote to dynamic** — runtime check inserted |

This is **gradual verification**: what can be proven statically is checked at compile time; what cannot is checked at runtime. The programmer can also force dynamic checking with the `dynamic` keyword.

### Dynamic Hoare as Effect System Extension

Dynamic Hoare clauses are effects:

```text
-- Dynamic contracts install runtime checks.
-- Default false-predicate failure is structured bottom, not a row item.
fn safe_div(a: Int, b: Int) -> Int
    dynamic requires: b != 0
    dynamic ensures: result * b == a
```

The function body:
1. Checks `requires` at entry → `Trap { reason: ContractViolation(...) }` if false
2. Computes result
3. Checks `ensures` at exit → `Trap { reason: ContractViolation(...) }` if false

If the surface requests recoverability explicitly, the failure path must lower to a visible
row-accounted `fail`, not to a hidden `ContractViolation` row item.

### Unified Surface Syntax

```ash
-- Static only (default): checked by SMT at compile time
fn safe_div(a: Int, b: Int) -> Int
    requires: b != 0
    ensures: result * b == a

-- Dynamic only: checked at runtime; default failure is structured bottom
fn safe_div(a: Int, b: Int) -> Int
    dynamic requires: b != 0
    dynamic ensures: result * b == a

-- Both: checked at compile time AND runtime
fn safe_div(a: Int, b: Int) -> Int
    requires: b != 0          -- static (SMT)
    ensures: result * b == a  -- static (SMT)
    dynamic requires: b != 0  -- runtime (effect)
    dynamic ensures: result * b == a  -- runtime (effect)
```

## The Core Ash Lowering

Static Hoare clauses lower to **type refinements** (no runtime code):

```text
-- Surface:
fn safe_div(a: Int, b: Int) -> Int
    requires: b != 0

-- Core (refined type):
fn safe_div(a: Int | b != 0, b: Int | b != 0) -> Int
```

Dynamic Hoare clauses lower to **Raise/Handle** (runtime code):

```text
-- Surface:
fn safe_div(a: Int, b: Int) -> {ContractViolation} Int
    dynamic requires: b != 0

-- Core (effect):
fn safe_div(a: Int, b: Int) -> {ContractViolation} Int {
    if b == 0 {
        Raise { op: ContractViolation, args: ["requires: b != 0"], resume: k }
    };
    a / b
}
```

## Laws and Properties: Compile-Time Metadata Artifacts

Laws and properties are **not Hoare clauses**. They are **compile-time metadata** that generate evidence, which is **lowered to refinements** in core.

### Laws: Evidence Lowered to Refinements

Laws are universal properties that produce **evidence** — refinement predicates that can be used in types:

```ash
-- Law: produces evidence "associative(append)"
law associativity<A> {
    forall x, y, z: List<A>.
    append(append(x, y), z) == append(x, append(y, z))
}
```

**Law verification pipeline:**

| Solver Result | Evidence | Action |
|--------------|----------|--------|
| Proved | `associative(append)` | Lowered to refinement, usable in types |
| Disproved | — | Compile error (law violation) |
| Unknown | Advisory | Warning; may be used as dynamic check |

The evidence is a **refinement predicate**:

```ash
-- Evidence used in type refinement
fn fold<A>(xs: List<A>, init: A, op: (A, A) -> A | associative(op)) -> A {
    ...
}

-- Or in Hoare clause
fn fold<A>(xs: List<A>, init: A, op: (A, A) -> A) -> A
    requires: associative(op)  -- evidence from law
{
    ...
}
```

### Properties: Statistical Evidence (Advisory)

Properties produce **statistical evidence** — confidence builders, not proofs:

```ash
-- Property: produces advisory evidence
property commutativity<A: Commutative> {
    forall x, y: A.
    x <> y == y <> x
}
```

| Result | Evidence | Action |
|--------|----------|--------|
| Passed (100 cases) | "tested: 100/100 passed" | Advisory warning; not usable in types |
| Failed | Counterexample | Compile error or warning |

Properties are **never** lowered to refinements. They are advisory only.

### Metadata, Not Core Constructs

Laws and properties are **metadata**, not core language constructs. They generate evidence that is used in core refinements.

```text
Law associativity<Int>
  → SMT query → Proved
  → Evidence: associative(append_Int)
  → Lowered to refinement: (A, A) -> A | associative(append_Int)

Law associativity<List<A>>
  → SMT query → Unknown
  → Warning: "not proved, using dynamic check"
  → Optional: SmallCheck (depth 3) + QuickCheck (100 cases)
  → Advisory evidence only

Property commutativity<Int>
  → QuickCheck (100 cases) + SmallCheck (depth 3)
  → Advisory: "tested: 100/100 passed"
  → Not usable in types
```

### The Endgame

| Contract Type | Static | Dynamic | Evidence | Fallback |
|-------------|--------|---------|----------|----------|
| `requires` | SMT refinement | Runtime assertion | — | — |
| `ensures` | SMT refinement | Runtime assertion | — | — |
| `where` | SMT invariant | Runtime assertion | — | — |
| `law` | SMT proof | — | Refinement predicate | QuickCheck + SmallCheck (advisory) |
| `property` | — | — | Advisory only | QuickCheck + SmallCheck |

## Open Questions

1. Should `static` be implicit (default) and `dynamic` be explicit?
2. How do refinements interact with row polymorphism? (e.g., `fn f<A | P>(x: A) -> {r} B`)
3. Should `ContractViolation` be a built-in effect or user-defined?
4. Can we generate dynamic checks from static refinements automatically?
5. How do we handle `old(x)` in postconditions? (snapshot of pre-state)
6. Should the SMT solver be integrated into the compiler or run as a separate pass?
7. How do Hoare clauses interact with effect rows? (e.g., `requires` in a pure function vs effectful function)

## References

- [SPEC-027: Pure Functions](../spec/SPEC-027-PURE-FUNCTIONS.md) — `requires`/`ensures` on functions
- [SPEC-006: Policy Definitions](../spec/SPEC-006-POLICY-DEFINITIONS.md) — `where` on policies
- [SPEC-079: Standard Algebra — Comonad and Kleisli Helpers](../spec/SPEC-079-STANDARD-ALGEBRA-COMONAD-AND-KLEISLI-HELPERS.md) — laws
- [core-ash.md](core-ash.md) — Core Ash language

## Changelog

- 2026-06-20: Created design note exploring Hoare clauses in Ash, comparing static vs dynamic checking, and determining sufficiency for Ash's contract system.
