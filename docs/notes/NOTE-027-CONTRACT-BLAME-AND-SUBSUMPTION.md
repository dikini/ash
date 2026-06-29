# NOTE-027: Contract Blame and Subsumption

**Date:** 2026-06-28
**Status:** Living document — design direction captured; resolves NOTE-014 GAP 1 (blame) and
GAP 3 (subsumption)
**Purpose:** Formalize the two CRITICAL contract-system gaps that together unblock impl
contract verification: (1) blame assignment — determining WHO violated a contract; and
(2) contract subsumption — the behavioral subtyping rule that determines WHEN a contract
obligation transfers from interface to impl. These are designed together because blame
polarity is exactly what subsumption controls.

Companion to NOTE-014 (contract systems unification), NOTE-013 (handler composition
algebra), NOTE-025 (effect identity), and NOTE-023 (handler marker).

## Pre-Spec Delta

This note is pre-spec and resolves NOTE-014 §12 GAP 1 and GAP 3. When the project moves to
spec updates, reconcile:

- **Blame labels in the IR (SPEC-098b):** the `ContractDischarge` struct and `Raise` node
  for `ContractViolation` gain a `BlameLabel` carrying the party role, module path, function
  name, and source span. The `TrapReason::ContractViolation` carries the same label.
- **Subsumption checking (SPEC-097b):** the type system gains a contract-subsumption rule
  checked eagerly at impl definition time: `impl_contract ⊑ interface_contract`. The
  existing §6.5 "Contract subsumption" section covers row-item discharge; this note adds the
  Hoare-triple subsumption rule.
- **Impl verification (SPEC-097b):** the checker verifies that each impl method satisfies
  the interface's Hoare contracts via subsumption — this is the formalization of NOTE-014
  §8.2\*\*\* ("impl contracts must be consistent with, no weaker than").

## 0. Motivation

NOTE-014 unified contracts into a row-item model with discharge modes. But two CRITICAL gaps
remained undefined:

1. **Blame.** When a dynamic contract fires, the system needs to know WHO violated it.
   Without blame labels, a violation says "something broke." With blame, it says "module A
   called `safe_div` without establishing `b != 0`" or "module B's `sort` impl failed its
   sortedness postcondition."

2. **Subsumption.** The informal principle "impl contracts must be no weaker than interface
   contracts" (NOTE-014 §8.2\*\*\*) was stated but never formalized. Without a checkable rule,
   the type checker cannot verify that an impl satisfies its interface's contracts.

These are not independent. The behavioral subtyping rule (contravariant precondition,
covariant postcondition) determines which party carries which obligation, and that directly
determines blame direction on failure. **Subsumption IS blame polarity.**

## 1. Contract Subsumption (GAP 3)

### 1.1 The Rule

The standard behavioral subtyping rule for Hoare triples (Liskov-Wing, Hoare logic
sequencing):

```text
{P} C {Q} ⊑ {P'} C {Q'}  iff  P' ⇒ P   (contravariant precondition)
                            and  Q ⇒ Q'  (covariant postcondition)
```

An impl may:
- **Weaken the precondition** (`P'` is weaker than `P`): accept more inputs than the
  interface required. The caller gets less obligation.
- **Strengthen the postcondition** (`Q'` is stronger than `Q`): guarantee more than the
  interface promised. The callee takes on more obligation.

The impl CANNOT strengthen the precondition (reject inputs the interface accepted) or weaken
the postcondition (deliver less than the interface promised). These are contract violations
at the impl level.

### 1.2 What is being subsumed

When an interface declares a contract on a method, and an impl provides a method body with
its own contract, the subsumption rule applies:

```ash
interface Stack<A> {
    pop() -> A
        requires: not_empty()
        ensures: result == old(peek())
}

impl Stack<String> for ArrayStack {
    fn pop() -> String
        requires: not_empty() && is_valid()    -- STRENGTHENED precondition
        ensures: result == old(peek())         -- same postcondition
    { ... }
}
```

The impl's `pop` has `requires: not_empty() && is_valid()`. Is this legal? The direction of
the rule is subtle and error-prone — it is stated precisely in §1.3.

### 1.3 The Rule — Precisely

The subsumption rule for behavioral subtyping is:

```text
Impl contract {P'} C {Q'} is a valid refinement of interface contract {P} C {Q} iff:

    P ⇒ P'      (impl precondition is no stronger than interface precondition)
    and
    Q' ⇒ Q      (impl postcondition is no weaker than interface postcondition)
```

**Reading:** `P ⇒ P'` means "wherever the interface precondition `P` holds, the impl
precondition `P'` also holds." The impl accepts **at least** all inputs the interface
accepted. It may accept more (weakening).

`Q' ⇒ Q` means "the impl postcondition `Q'` implies the interface postcondition `Q`." The
impl guarantees **at least** what the interface promised. It may guarantee more
(strengthening).

**Memory aid:** precondition contravariant (weaken going down), postcondition covariant
(strengthen going down). "Pre-conditions can only get weaker, post-conditions can only get
stronger."

### 1.4 Worked example

```ash
interface SafeDiv {
    fn divide(a: Int, b: Int) -> Int
        requires: b != 0                         -- P
        ensures: result * b == a                 -- Q
}

impl SafeDiv for CheckedDiv {
    fn divide(a: Int, b: Int) -> Int
        requires: b != 0 && b.abs() <= MaxInt    -- P' = P ∧ (b.abs() ≤ MaxInt)
        -- VIOLATION: P ⇒ P'? Does (b != 0) imply (b != 0 && b.abs() <= MaxInt)?
        -- No — b could be MaxInt+1. P' is STRONGER. This is a subsumption error.
}

impl SafeDiv for IntDiv {
    fn divide(a: Int, b: Int) -> Int
        requires: b != 0                         -- P' = P. Legal (trivially: P ⇒ P).
        ensures: result * b == a                 -- Q' = Q. Legal (trivially: Q' ⇒ Q).
}

impl SafeDiv for GenerousDiv {
    fn divide(a: Int, b: Int) -> Int
        requires: true                           -- P' = true. Legal: P ⇒ true (always).
        ensures: result * b == a && result >= 0  -- Q' = Q ∧ (result ≥ 0).
        -- Legal postcondition strengthening: Q' ⇒ Q holds.
}
```

The third impl is the interesting case: it **weakens** the precondition (accepts `b == 0`
too) and **strengthens** the postcondition (adds non-negativity). Both are legal. The impl
must handle `b == 0` internally (since it claimed `requires: true`).

### 1.5 When the check runs

**Eagerly, at impl definition time.** When the type checker processes an `impl` block, it:

1. Looks up the interface method signatures and their declared contracts (`requires`, `ensures`).
2. Reads the impl method's declared contracts (if any).
3. Verifies `P ⇒ P'` and `Q' ⇒ Q` for each method.
4. If an impl method has no explicit contracts, it inherits the interface's contracts
   exactly (`P' = P`, `Q' = Q`), which trivially satisfies subsumption.

This is the same timing as Rust's trait impl checking. It requires the interface's contracts
to be visible at impl-processing time (same module or imported). It surfaces errors at the
impl site, not at call sites.

### 1.6 Interaction with the effect row

Contract subsumption is about **values** (preconditions on arguments, postconditions on
results). It is orthogonal to row subtyping. If an impl adds effects not in the interface's
row, that is a separate row-subtyping violation, checked independently by the effect system.

An impl may have a stricter row (fewer effects) than the interface — this is row narrowing
and is always legal. The impl may not add effects the interface didn't declare (row
widening), because callers would be unprepared to handle them.

### 1.7 `old(x)` in postconditions

An `ensures` clause referencing `old(x)` snapshots the pre-state of `x` at function entry.
For subsumption, `old(x)` refers to the same variable in both the interface and impl
postconditions. The implication `Q' ⇒ Q` is evaluated with `old(x)` bound to the same
pre-call value in both predicates.

If the impl changes the post-state shape (e.g., the interface uses `old(stack.size)` and the
impl represents the stack differently), the impl's `ensures` must still be expressible in
terms of the interface's observable state. This is the Liskov substitutability principle: the
impl is usable wherever the interface is expected, so postconditions must be in terms of the
interface's abstraction, not the impl's internal representation.

## 2. Blame Assignment (GAP 1)

### 2.1 The Two Parties

Every Hoare contract triple `{P} C {Q}` involves two parties:

- **The caller** (negative party): responsible for establishing `P` before calling.
- **The callee** (positive party): responsible for delivering `Q` after execution.

When a contract fires, blame is determined by polarity:

| Contract | Violated condition | Blame | Reason |
|---|---|---|---|
| `requires: P` | `P` is false at entry | **Caller** | Caller didn't establish the precondition |
| `ensures: Q` | `Q` is false at exit | **Callee** | Callee didn't deliver the postcondition |
| `invariant: I` | `I` is false at boundary | **Depends** | At loop entry: caller (same as `requires`). At loop exit: callee (same as `ensures`). At data-structure boundary: the code that mutated the structure. |

This is Findler-Felleisen higher-order contract blame theory (2002). The key property is
**blame soundness**: if blame is assigned to party X, then party X actually violated the
contract. This is GAP 7 (meta-level soundness) from NOTE-014 §12.

### 2.2 Blame Labels

A `BlameLabel` carries the diagnostic state needed to attribute a contract violation:

```rust
pub struct BlameLabel {
    pub party: Party,           // Caller or Callee
    pub polarity: Polarity,     // Negative (requires) or Positive (ensures)
    pub module_path: String,    // Full module path of the blamed party
    pub function_name: String,  // Function/method where the contract lives
    pub contract_text: String,  // Human-readable predicate text
    pub source_span: Span,      // Source location of the contract declaration
}

pub enum Party {
    Caller,    // negative party — failed to establish precondition
    Callee,    // positive party — failed to deliver postcondition
    Impl,      // specialized callee for impl-level blame (impl failed interface contract)
}

pub enum Polarity {
    Negative,  // requires / precondition
    Positive,  // ensures / postcondition
}
```

The `Party::Impl` variant captures the interface→impl contract obligation. When an impl
violates the interface's postcondition, the blame points to the impl (not the original
interface method). This is how subsumption and blame connect: the covariant postcondition
strengthening means the impl takes on more obligation, and blame follows the obligation.

### 2.3 Blame in the IR

The `ContractDischarge` struct (SPEC-098b §4.1) and dynamic contract diagnostics carry the
blame label:

```rust
// Extended ContractDischarge (SPEC-098b §4.1)
pub struct ContractDischarge {
    pub contract: ContractEffect,
    pub mode: DischargeMode,
    pub evidence: Option<EvidenceRef>,
    pub source_span: Span,
    pub blame: BlameLabel,          // NEW: who is blamed on failure
}
```

For dynamic contracts, the runtime check plan carries the blame label so the trap diagnostic
or explicit recoverable `fail` path knows who to attribute the failure to:

```rust
// Dynamic contract violation at runtime, default unrecoverable path
Trap {
    reason: ContractViolation(ContractDiagnostic {
        contract: ContractEffect::Requires(PredicateRef("b != 0")),
        blame: BlameLabel {
            party: Party::Caller,
            polarity: Polarity::Negative,
            module_path: "myapp::payments",
            function_name: "safe_div",
            contract_text: "requires: b != 0",
            source_span: Span { file: "math.ash", line: 12, col: 5 },
        },
        observed_values: vec![ /* actual argument values for diagnostics */ ],
        ...
    })
}
```

### 2.4 Worked example — blame flow

```ash
-- In module myapp::payments
fn process_payment(amount: Int) -> {ContractViolation} Unit {
    -- calls safe_div with b = 0
    safe_div(100, 0)     -- caller fails to establish requires: b != 0
}

-- In module stdlib::math
fn safe_div(a: Int, b: Int) -> {ContractViolation} Int
    dynamic requires: b != 0
    dynamic ensures: result * b == a
{
    a / b
}
```

At runtime, `safe_div(100, 0)` raises `ContractViolation` with blame:

```
BlameLabel {
    party: Caller,
    polarity: Negative,
    module_path: "myapp::payments",        -- the CALLER's module
    function_name: "safe_div",             -- where the contract lives
    contract_text: "requires: b != 0",
    source_span: Span { file: "math.ash", line: 42, col: 5 },
}
```

The diagnostic message:

```
Contract violation in safe_div (math.ash:42):
  requires: b != 0
  Blame: caller (myapp::payments::process_payment)
  Actual: b = 0
```

If the `ensures` had failed instead (impossible for `safe_div` unless the division is buggy),
the blame would point to the callee:

```
BlameLabel {
    party: Callee,
    polarity: Positive,
    module_path: "stdlib::math",           -- the CALLEE's module
    function_name: "safe_div",
    contract_text: "ensures: result * b == a",
}
```

### 2.5 Blame through subsumption

When an impl strengthens a postcondition (covariant), the impl takes on MORE obligation. If
the strengthened postcondition fails, blame points to the impl:

```ash
interface SafeDiv {
    fn divide(a: Int, b: Int) -> Int
        requires: b != 0
        ensures: result * b == a            -- Q
}

impl SafeDiv for GenerousDiv {
    fn divide(a: Int, b: Int) -> Int
        requires: true                      -- P': weakened (legal)
        ensures: result * b == a && result >= 0   -- Q': strengthened (legal)
    {
        // if this body returns a negative result, the strengthened Q' fails
        // Blame: Impl (GenerousDiv failed its own stronger postcondition)
        ...
    }
}
```

If `GenerousDiv::divide` returns a negative result, the blame is:

```
BlameLabel {
    party: Impl,
    polarity: Positive,
    module_path: "myapp::generous",        -- the IMPL's module
    function_name: "divide",
    contract_text: "ensures: result * b == a && result >= 0",
}
```

The impl is blamed because it declared a stronger postcondition than the interface and failed
to deliver it. The interface's original postcondition (`result * b == a`) might still hold,
but the impl's additional guarantee (`result >= 0`) was violated.

**Key principle:** blame follows the obligation. The party that declared the contract clause
that failed is the party that is blamed. If the impl added a clause, the impl is blamed for
that clause. If the interface declared a clause and the impl inherited it unchanged, the
blame depends on polarity (caller for `requires`, callee/impl for `ensures`).

## 3. Blame Through Handler Composition

When a contract handler catches a `ContractViolation` and resumes (NOTE-013 §7.2), the
question is: who carries the blame?

### 3.1 The principle: blame is immutable

**The original blame label is never changed by handler composition.** If module A called
`safe_div` with `b = 0`, the blame points to A. If a contract handler catches the violation
and resumes with a default value, the blame still points to A — the handler didn't fix the
cause, it papered over the symptom.

This is critical for diagnostic integrity. A handler that swallows contract violations must
not erase the blame trail. The audit log records both the original violation (blamed to A)
and the handler's decision to resume.

### 3.2 Three handler strategies and their blame semantics

| Handler strategy | Resume? | Blame effect | Audit record |
|---|---|---|---|
| **Propagate** (no resume) | no | Original blame preserved; propagated to outer handler/trap | Violation logged, blame preserved |
| **Resume with default** | yes | Original blame preserved; handler decision logged | Violation logged + resume decision logged |
| **Escape** (transform answer) | no | Original blame preserved; answer type changed | Violation logged, blame preserved, answer transformed |

In all three cases, the original blame label is immutable. The handler's *decision* (resume,
propagate, escape) is recorded separately as handler metadata, not as blame.

### 3.3 Nested handler blame propagation

Given a handler stack H₁ (outer) ∘ H₂ (inner), if a contract violation is raised:

1. H₂ (innermost) sees it first. If H₂ has a `ContractViolation` clause, it decides: resume,
   propagate, or escape.
2. If H₂ propagates (no clause or explicit re-raise), the violation flows to H₁ with the
   **same** blame label.
3. If H₁ also propagates, the violation traps with `TrapReason::ContractViolation` carrying
   the blame label.

At no point does the blame label change. Nesting only affects *whether* the violation is
caught, not *who* is blamed.

### 3.4 Worked example — blame through handler nesting

```ash
handler contract_handler<A, r: Row>(
    comp: Unit -> {ContractViolation | r} A
) -> {r} Result<A, ContractError> {
    on comp() {
        ContractViolation(label, _resume) => Err(ContractError {
            blame: label,           -- original blame preserved
            message: label.contract_text,
            recovered: false,       -- we did not resume
        })
        done(value) => Ok(value)
    }
}

handler defaulting_handler<A, r: Row>(
    comp: Unit -> {ContractViolation | r} A
) -> {r} A {
    on comp() {
        ContractViolation(label, resume) => {
            log("Contract violated: {}, blamed to: {}, resuming with default",
                label.contract_text, label.module_path);
            resume(default_value())   -- swallows violation, resumes
            -- Blame is STILL label. The log records it. The computation continues.
        }
        done(value) => value
    }
}
```

The `defaulting_handler` is the dangerous case: it swallows the violation and resumes. The
computation continues as if nothing happened, but the audit log records the violation and
its blame. This handler is legal but should be used with care — it masks correctness
failures.

## 4. Diagnostic State on Contract Failure (connects to GAP 6)

Contract failure is a structured bottom. The `BlameLabel` is part of a larger diagnostic
payload that survives the failure. This formalizes what GAP 6 (NOTE-014 §12) described
informally.

### 4.1 The full diagnostic payload

```rust
pub struct ContractDiagnostic {
    pub blame: BlameLabel,
    pub contract: ContractEffect,
    pub actual_values: Vec<DiagnosticValue>,   // actual arguments/results at failure
    pub call_chain: Vec<CallFrame>,            // continuation chain at point of violation
    pub discharge_history: DischargeHistory,   // was this demoted from static? always dynamic?
    pub handler_decisions: Vec<HandlerDecision>, // did any handler catch and resume?
}

pub struct DiagnosticValue {
    pub name: String,
    pub type_name: String,
    pub value: String,      // Debug-formatted value for diagnostics
}

pub struct CallFrame {
    pub function_name: String,
    pub module_path: String,
    pub source_span: Span,
}

pub struct DischargeHistory {
    pub original_mode: DischargeMode,      // Static, Evidence, Dynamic
    pub demotion_chain: Vec<DischargeMode>, // if demoted from static → dynamic
}

pub struct HandlerDecision {
    pub handler_name: String,
    pub decision: HandlerAction,           // Resume, Propagate, Escape
    pub source_span: Span,
}
```

### 4.2 The boundary: trap vs. resumable

This also clarifies NOTE-014 Open Question 5 (`ContractViolation` as trap vs. effect):

- **No handler installed for `ContractViolation`:** traps with
  `TrapReason::ContractViolation(diagnostic)`. Terminal bottom. The computation cannot
  resume.
- **Handler installed for `ContractViolation`:** raises as an operation. The handler decides
  whether to resume, propagate, or escape. The diagnostic payload is available to the handler
  clause.

The boundary is determined by the **effect row**: if `{ContractViolation | r}` is in the
row and no handler peels it, it traps. If a handler peels it, it is a recoverable raise. The
row type determines the boundary — same as any other effect.

## 5. The Verification Algorithm

Putting subsumption and blame together, the type checker performs these steps at impl
definition time:

### 5.1 At impl definition

For each method `m` in `impl I for T`:

1. **Retrieve interface contract.** Look up `I::m`'s declared `requires: P` and `ensures: Q`.
2. **Retrieve impl contract.** Read `T::m`'s declared `requires: P'` and `ensures: Q'` (or
   inherit `P' = P`, `Q' = Q` if none declared).
3. **Check subsumption.** Verify `P ⇒ P'` (precondition weakening) and `Q' ⇒ Q`
   (postcondition strengthening) via SMT.
   - If SMT proves both: subsumption holds. Record `ContractDischarge` with `Static` mode.
   - If SMT disproves either: compile error at the impl site with a counterexample.
   - If SMT is `unknown`: demote the subsumption check to runtime (the impl's contracts
     become dynamic, carrying blame labels).
4. **Attach blame labels.** For each contract clause (static or dynamic), generate a
   `BlameLabel` recording the party and polarity.

### 5.2 At call site

For a call `T::m(args)` where `m` has `requires: P`:

1. If `P` is static and discharged: caller's obligation is erased (SMT proved it).
2. If `P` is dynamic: insert a runtime check at the call site with
   `BlameLabel { party: Caller, ... }`. The caller is blamed if the precondition fails.
3. After the call returns `result`: if `Q` is dynamic, insert a runtime check at the return
   site with `BlameLabel { party: Callee, ... }` (or `Impl` if the impl
   strengthened the postcondition).

### 5.3 Gradual verification interaction

The gradual verification flow from NOTE-014 §9.3 is extended:

```text
requires: P
  │
  ├─ SMT proves P at call site → erase, RecordDischarge(Static)
  │                              No blame label needed (never fires)
  │
  ├─ SMT disproves P → compile error (counterexample)
  │                     Blame: Caller (failed to establish)
  │
  └─ SMT unknown / explicit dynamic
       │
       ├─ Insert runtime check at call site
       │   with BlameLabel { party: Caller, polarity: Negative, ... }
       ├─ Default false predicate traps with ContractViolation diagnostic
       └─ Explicit recoverability lowers to row-accounted fail
              (blame label preserved through failure handling)
```

## 6. Open Questions

1. **Blame for `invariant` violations.** An invariant can fire at multiple boundary types
   (loop entry, loop exit, data-structure mutation). The blame polarity depends on which
   boundary fired. Should the `Party` enum gain an `Invariant` variant, or is the existing
   Caller/Callee/Impl sufficient with the polarity disambiguating?

2. **Blame across newtype wrappers.** If a newtype (NOTE-026) wraps a type and the wrapper's
   impl violates a contract, is blame assigned to the newtype's module or the underlying
   type's module? The newtype is a distinct type with its own impl, so blame should point to
   the newtype's impl. Confirm.

3. **Cross-module evidence caching with blame.** When law evidence or contract discharge is
   cached across module boundaries (NOTE-014 Open Question 6), do blame labels need to be
   re-validated? If module A trusts module B's static discharge, and B's discharge was wrong,
   does A get blamed? This connects to GAP 7 (meta-level soundness).

4. **Blame for concurrent contracts (GAP 5).** When contracts span multiple processes
   (temporal, supervision, obligation), the two-party model breaks down — there may be many
   callers and many callees. The blame model will need extension for the Proc/Workflow tier.
   Deferred until GAP 5 is addressed.

5. **Contract clauses as effects — blame identity.** Should `ContractViolation` carry an
   impl-qualified identity (per NOTE-025), or is it a single global effect? Currently it is
   modeled as `TrapReason::ContractViolation(ContractEffect)` — a single trap reason. If
   multiple contract handlers coexist (like multiple `Fs` handlers), they would need distinct
   identities. Deferred — the current single-effect model suffices until multiple contract
   handlers are needed.

## 7. Working Principle

```text
Contract subsumption (GAP 3):
  {P} C {Q} ⊑ {P'} C {Q'}  iff  P ⇒ P'  (precondition weakens) and Q' ⇒ Q (postcondition strengthens).
  Checked eagerly at impl definition time.
  Impl with no explicit contracts inherits the interface's contracts exactly.

Blame assignment (GAP 1):
  requires: P violated → blame the Caller (negative party).
  ensures: Q violated → blame the Callee/Impl (positive party).
  Blame follows the obligation: the party that declared the failing clause is blamed.

Blame labels:
  BlameLabel { party, polarity, module_path, function_name, contract_text, source_span }.
  Carried in ContractDischarge and dynamic ContractDiagnostic / explicit fail payloads.

Blame through handler composition:
  The original blame label is immutable.
  Handler decisions (resume, propagate, escape) are recorded separately, never as blame.
  Nesting affects whether a violation is caught, not who is blamed.

Diagnostic state:
  ContractDiagnostic { blame, actual_values, call_chain, discharge_history, handler_decisions }.
  Default dynamic false predicate = terminal bottom Trap.
  Explicit recoverability = row-accounted fail with the same blame label.
```

## 8. References

Internal references:

- [NOTE-014: Contract Systems Unification](NOTE-014-CONTRACT-SYSTEMS-UNIFICATION.md) — §12
  GAP 1 (blame), GAP 3 (subsumption), GAP 6 (failure observability), GAP 7 (meta-level
  soundness); §8.2\*\*\* (impl contract inheritance); §9 (IR lowering)
- [NOTE-013: Ambient Monad and Handler Composition Algebra](NOTE-013-AMBIENT-MONAD-AND-HANDLER-COMPOSITION-ALGEBRA.md)
  — §7 (handler composition, nesting order)
- [NOTE-025: Effect Identity via Sorts and Impls](NOTE-025-EFFECT-IDENTITY-VIA-SORTS-AND-IMPLS.md)
  — §7.7 coherence
- [NOTE-026: Newtype and Phantom Types](NOTE-026-NEWTYPE-AND-PHANTOM-TYPES.md)
- [SPEC-097b: Target Type System](../spec/SPEC-097b-TARGET-TYPE-SYSTEM.md) — §6.5 contract
  subsumption (row-item discharge)
- [SPEC-098b: Target CPS IR](../spec/SPEC-098b-TARGET-IR.md) — §4.1 ContractDischarge,
  TrapReason, ContractDiagnostic, and explicit `fail` path

External references:

- Findler & Felleisen, "Contracts for Higher-Order Functions" (2002).
  https://doi.org/10.1145/581478.581484
- Liskov & Wing, "Behavioral Subtyping Using Invariants and Constraints" (1994).
  https://doi.org/10.1016/0167-6423(94)90026-4
- Dimoulas, Findler, Flatt & Felleisen, "Correct Blame for Contracts: No More Scapegoating"
  (2012). https://doi.org/10.1145/2103621.2103697
- Atkey, Bach Poulsen & McKinna, "Step-Indexed Relational Interpretation of Information Flow
  and Monadic Hoare Logic" (2014). Monadic Hoare logic for effectful contracts.
  https://doi.org/10.1007/978-3-662-45231-8_3

## 9. Changelog

- 2026-06-28: Initial version. Resolves NOTE-014 GAP 1 (blame) and GAP 3 (subsumption).
  Formalizes behavioral subtyping (contravariant precondition, covariant postcondition),
  blame labels (party, polarity, module path, source span), blame through handler composition
  (immutable labels, handler decisions recorded separately), and diagnostic state
  (ContractDiagnostic struct). Connects to GAP 6 (failure observability) and GAP 7 (blame
  soundness). Five open questions flagged: invariant blame, newtype blame, cross-module
  evidence, concurrent blame, contract violation as impl-qualified effect.
- 2026-06-29: Reconciled dynamic-blame examples with NOTE-029/NOTE-033. Dynamic false
  predicates now produce `ContractDiagnostic` in a `Trap` by default; explicit recoverability
  uses row-accounted `fail` while preserving the same blame label.
