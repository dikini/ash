# NOTE-032: Contract Soundness Obligations

**Date:** 2026-06-29
**Status:** Living document — design direction captured; resolves NOTE-014 GAP 7
**Purpose:** State the meta-level soundness obligations that make Ash's contract system trustworthy. NOTE-027 through NOTE-031 define the mechanisms: blame, subsumption, evaluation timing, structured bottom, monadic Hoare composition, and predicate well-formedness. This note defines what those mechanisms must preserve for static discharge, dynamic demotion, optimizer use, blame assignment, and predicate faults to be sound.

Companion to NOTE-014 (contract systems unification), NOTE-027 (blame and subsumption), NOTE-028 (purity and evaluation-mode timing), NOTE-029 (structured bottom), NOTE-030 (monadic Hoare composition), NOTE-031 (predicate well-formedness), SPEC-097b (target type system), SPEC-098b (target IR), SPEC-099 (Core language), and SPEC-100 (Core type checking).

## Pre-Spec Delta

This note is pre-spec and resolves NOTE-014 §12 GAP 7. When promoted into target specs, reconcile:

- **SPEC-097b Target Type System:** add the five soundness obligations as explicit meta-invariants over contract discharge, subsumption, sequencing, predicate classification, and evaluation modes.
- **SPEC-098b Target IR:** require `ContractDischarge`, `ComposedContract`, `SnapshotRef`, `BlameLabel`, `ContractDiagnostic`, and `PredicateFaultDiagnostic` metadata to be preserved by transformations that erase checks, demote checks, or reassociate computations.
- **SPEC-099 Core language:** state that Core dynamic checks, traps, and discharge records are the semantic boundary used by the soundness obligations.
- **SPEC-100 Core type checking:** require predicate well-formedness and discharge records before proof obligations are consumed; static proof, dynamic demotion, and predicate faults must produce distinguishable typed artifacts.

No new Ash surface syntax is introduced.

## 0. Motivation

The preceding contract notes stabilized the operational shape of Ash contracts. We can now say where a precondition is checked, how a postcondition composes through `bind`, what diagnostic is produced on failure, and which predicates are valid contract predicates.

That is still not enough. A contract system also needs meta-level obligations: claims about the checker, prover, runtime checks, blame labels, and optimizer. If these claims are not stated, later implementation work can pass local tests while breaking the reason the system exists.

The core question is:

```text
When Ash removes, moves, delays, or attributes a contract check, what must remain true?
```

This note answers that question with five obligations.

## 1. Core decision

Ash treats contract soundness as a set of explicit meta-obligations over the typed Core and CPS boundary.

```text
Contract soundness is not one theorem.
It is a family of preservation obligations.
```

The initial family is:

1. **Gradual verification soundness** — static discharge justifies removing the corresponding runtime check.
2. **Blame soundness** — a diagnostic's blame label names the party responsible for the violated obligation.
3. **Optimizer soundness** — transformations justified by contract or law evidence preserve values, rows, traps, diagnostics, and evidence boundaries.
4. **Dynamic demotion soundness** — an unknown static obligation demoted to runtime is checked at the same semantic boundary with the same predicate environment.
5. **Predicate-fault separation** — predicate evaluation faults are not collapsed into false predicates or caller/callee contract violations.

These obligations are design contracts for the compiler and runtime. A first implementation may not machine-prove them, but it must preserve the metadata needed to state and audit them.

## 2. Grammar impact

No new surface syntax is introduced.

The note constrains existing forms:

```ash
fn f(x: A) -> B
    requires: P(x)
    ensures: Q(x, result)
{ ... }
```

and existing internal proof metadata:

```text
∀a. Q(a) ⇒ R(a)
∃a. Q(a) ∧ S(a, b)
```

The `∀` and `∃` forms above remain proof metadata from NOTE-030, not source syntax.

## 3. Soundness objects and environments

The soundness obligations range over typed Core, discharge metadata, and runtime observations.

```text
TypedBoundary = {
  core_node: CoreNodeId,
  row: Row,
  contract: ContractEffect,
  predicate: PredicateSummary,
  discharge: ContractDischarge,
  blame: BlameLabel,
  snapshots: Vec<SnapshotRef>,
}
```

A runtime observation includes normal values, traps, recoverable failures, and diagnostics:

```text
Observation ::= Value(v)
              | Trap(ContractViolation(diagnostic))
              | Trap(ContractPredicateFault(diagnostic))
              | Fail(failure_value)
              | Diverge
```

For these obligations, two observations are equivalent only if they preserve the user-visible result and the contract-visible evidence boundary. In particular:

- `ContractViolation` is not equivalent to `ContractPredicateFault`;
- a recoverable `fail` is not equivalent to an unrecoverable trap;
- a redacted observed value is not equivalent to a missing diagnostic;
- two diagnostics with different blame parties are not equivalent;
- two `old(...)` snapshots from different boundaries are not equivalent.

## 4. Obligation 1: gradual verification soundness

Gradual verification soundness connects static discharge to runtime behavior.

```text
If Γ ⊢ P : StaticPredicate
and ProofEnv ⊢ P proven at boundary β
and lowering records ContractDischarge(P, Static/Evidence, β)
then execution need not install the dynamic check for P at β.
```

The obligation is:

```text
If the same boundary β were checked dynamically with the same predicate environment,
that dynamic check would not produce PredicateFalse.
```

This does not say the program cannot trap for another reason. It may still diverge, fail through an explicit `fail`, trap in the body, or hit a separate contract. It only says the statically discharged predicate itself is not allowed to be the source of a later false-predicate failure.

### 4.1 Static proof is scoped

A proof discharges a specific predicate under a specific environment:

```text
(PredicateRef, PredicateEnv, SnapshotEnv, BoundaryId)
```

Changing any of those invalidates the discharge unless a separate proof shows the change is semantics-preserving. This matters for:

- inlining functions with contracts;
- hoisting checks;
- reusing cross-module evidence;
- memo replay;
- bind reassociation.

### 4.2 Unknown is not false

If the prover returns `unknown`, Ash may reject, defer, or demote the check to runtime. The `unknown` result is not counter-evidence against the predicate.

```text
unknown(P) ≠ false(P)
```

Dynamic demotion is therefore a preservation move, not a refutation.

## 5. Obligation 2: blame soundness

Blame soundness connects a failed obligation to the party named in the diagnostic.

For a Hoare triple:

```text
{P} C {Q}
```

Ash uses the standard polarity rule:

```text
P false at entry  => blame caller / negative party
Q false at exit   => blame callee or impl / positive party
```

The obligation is:

```text
If Trap(ContractViolation(diagnostic)) is emitted,
then diagnostic.blame.party corresponds to the party that failed to establish
or preserve the violated contract obligation at that boundary.
```

### 5.1 Subsumption preserves blame

For interface-to-impl contracts, NOTE-027 gives:

```text
{P} C {Q} ⊑ {P'} C {Q'} iff P ⇒ P' and Q' ⇒ Q
```

Blame soundness requires the subsumption checker to preserve obligation ownership:

- If an impl strengthens guarantees and then fails them, the impl is blamed.
- If a caller fails the interface-visible precondition, the caller is blamed.
- If an impl rejects inputs accepted by the interface, the impl failed the subsumption check before runtime.

The runtime must not repair a bad impl contract by blaming the caller for an obligation the interface did not expose.

### 5.2 Composition preserves blame

For NOTE-030 sequencing:

```text
m ensures Q(a)
k requires R(a)
obligation: ∀a. Q(a) ⇒ R(a)
```

If the obligation is dynamically checked at the continuation boundary and fails, the diagnostic must distinguish:

- the continuation precondition that failed;
- the producer postcondition that was too weak to establish it;
- the composed caller boundary that fed `a` into `k`.

The blame label follows the violated precondition's polarity, but the diagnostic must retain the producer/continuation evidence chain. Without that chain, blame may be locally correct but globally opaque.

## 6. Obligation 3: optimizer soundness

Contract and law evidence may enable optimizations. Examples include:

- erasing statically discharged dynamic checks;
- inlining a function without duplicating already-discharged checks;
- specializing a handler using proven laws;
- reassociating binds;
- eliminating dead handler or failure paths that evidence proves unreachable.

The obligation is:

```text
If an optimization uses contract/law evidence E,
then the transformed program must preserve normal observations,
row requirements, trap/fail boundaries, blame labels, snapshot identities,
and discharge/evidence metadata relevant to E.
```

### 6.1 Evidence is part of the optimization input

An optimizer cannot use only the text of a predicate. It must use the checked evidence object:

```text
EvidenceKey = {
  predicate_ref,
  predicate_summary,
  discharge_mode,
  evidence_ref,
  boundary_id,
  snapshot_refs,
}
```

If any part of the key changes, the optimization must either revalidate the evidence or preserve the original check.

### 6.2 Bind reassociation

Bind reassociation is value-semantically valid for the ambient monad only when it preserves contract evidence and snapshot boundaries.

```text
bind(bind(m, k), h)  ~  bind(m, λa. bind(k(a), h))
```

For contracts, this equivalence is conditional:

- composed `ComposedContract` evidence must be transported or recomputed;
- `old_m`, `old_k`, and `old_h` snapshot identities must not be collapsed;
- dynamic checks must remain at the same semantic continuation boundary;
- diagnostics must still report the same violated obligation and blame party.

If evidence trees differ only by associativity but expose the same obligations, a later evidence-normalization pass may canonicalize them. Until such a pass exists, optimizers must preserve the conservative evidence tree.

### 6.3 Handler/law optimization

Handler optimizations may rely on laws, but a law proof is scoped to the handler, operation identity, and impl type that produced the evidence.

```text
operation identity = ImplType::op
```

A law for one impl type must not justify optimizing another impl type with the same interface method name. This follows the sort/impl identity model from NOTE-025.

## 7. Obligation 4: dynamic demotion soundness

Dynamic demotion converts an unproved static obligation into a runtime check.

```text
ProofEnv ⊢ P unknown
--------------------------------
insert dynamic check for P at β
```

The obligation is:

```text
The dynamic check must evaluate the same predicate P under the same boundary environment β
that the static obligation used, modulo explicit and recorded lowering transformations.
```

This requires preserving:

- source span;
- predicate text and structured predicate summary;
- `result`/message binders;
- lexical variables admitted into the predicate environment;
- `SnapshotRef`s for `old(...)`;
- blame label;
- discharge history showing demotion from static/unknown to dynamic.

Dynamic demotion must not hoist a check earlier if doing so forces lazy values. It must not sink a check later if doing so changes `old(...)` or blame. It must not convert an unrecoverable trap into recoverable `fail` unless the surface construct explicitly requested recoverability and row-accounted it.

## 8. Obligation 5: predicate-fault separation

NOTE-031 separates two outcomes:

```text
predicate returns false  => ContractViolation(predicate_false)
predicate traps/faults   => ContractPredicateFault(predicate_fault)
```

The obligation is:

```text
A predicate evaluation fault must not be reported as a false contract predicate.
A false predicate must not be hidden as a predicate evaluator fault.
```

This separation protects blame soundness. If `valid_ratio(n, d)` faults because it divides by zero internally, the caller did not necessarily violate the contract. The admitted predicate function is partial or malformed. Conversely, if `d != 0` evaluates cleanly to `false`, the caller did violate a precondition.

Predicate-fault diagnostics normally blame the contract author or admitted predicate-function provider. They are not caller/callee Hoare failures unless the predicate's own contract explicitly establishes that relationship.

## 9. Worked examples

### 9.1 Static discharge removes a check

```ash
fn divide(a: Int, b: Int) -> Int
    requires: b != 0
{ a / b }
```

If the caller proves `b != 0`, the call boundary records:

```text
ContractDischarge {
  contract: requires(b != 0),
  mode: Static,
  evidence: proof_ref,
  boundary: call_site,
}
```

Gradual verification soundness says the erased dynamic `b != 0` check would not have failed at that same boundary. It does not say `a / b` cannot fail for unrelated reasons introduced by another operation.

### 9.2 Dynamic demotion preserves `old(...)`

```ash
fn push(s: Stack<A>, a: A) -> Stack<A>
    ensures: result.len == old(s.len) + 1
{ ... }
```

If the checker cannot prove the postcondition statically, it may insert a dynamic check at the return boundary. The dynamic check must use:

```text
SnapshotRef(boundary = push.entry, path = s.len)
```

It must not read `s.len` after the body, and it must not reuse a caller or continuation snapshot with the same textual path.

### 9.3 Bad bind composition keeps the right blame chain

```ash
fn parse_maybe_empty(s: String) -> Parsed
    ensures: result.source_len >= 0
{ ... }

fn compile_nonempty(p: Parsed) -> Module
    requires: p.source_len > 0
{ ... }
```

The bind obligation is false:

```text
∀p. p.source_len >= 0 ⇒ p.source_len > 0
```

If the profile demotes to dynamic and the check fails, the violated condition is the continuation precondition. The diagnostic blames the composed caller boundary for invoking `compile_nonempty` with a bad `p`, while preserving the evidence that `parse_maybe_empty` only guaranteed `>= 0`.

### 9.4 Optimizer must preserve diagnostic boundary

Suppose an optimizer inlines `compile_nonempty` into its caller. It may remove call overhead, but it must not erase the precondition boundary unless the precondition is discharged. If the check remains dynamic, a failure must still point to the original `compile_nonempty` contract span and blame polarity.

Inlining may change code shape. It must not change contract authorship.

### 9.5 Predicate fault is not contract false

```ash
pred fn valid_ratio(n: Int, d: Int) -> Bool {
    n / d >= 0
}

fn f(n: Int, d: Int) -> Int
    requires: valid_ratio(n, d)
{ ... }
```

For `d == 0`, evaluating the predicate faults. Ash reports:

```text
Trap { reason: ContractPredicateFault(...) }
```

It does not report:

```text
Trap { reason: ContractViolation(... PredicateFalse ...) }
```

The predicate function failed to define a safe observer for that input. That is different from the caller failing a well-defined precondition.

## 10. Design decisions

1. **Soundness is a family of obligations.** Ash does not collapse gradual verification, blame, optimization, demotion, and predicate-fault behavior into one vague theorem.
2. **Static discharge is scoped by boundary environment.** Proofs are keyed by predicate summary, snapshot environment, boundary, and evidence.
3. **Blame labels are semantic evidence.** Optimizers and handlers must preserve blame identity, not just predicate text.
4. **Dynamic demotion preserves the static obligation's environment.** Unknown proof results become checks at the same semantic boundary, not arbitrary runtime assertions.
5. **Optimizers consume evidence, not claims.** Contract/law-based transformations must retain or recompute evidence metadata.
6. **Predicate faults are not false predicates.** Collapsing them would break blame soundness and misclassify checker/predicate-definition failures as caller/callee contract violations.

## 11. Open questions

1. **Mechanized proof boundary.** Which subset of these obligations should eventually be mechanized, and in what system: Lean, Coq, F*, or a smaller executable metatheory?
2. **Evidence normalization.** What is the canonical form for `ComposedContract` evidence trees after bind reassociation?
3. **Cross-module trust.** How are proof certificates, test evidence, and discharge records invalidated when an imported module changes?
4. **Falsification evidence scope.** How should survived-testing evidence participate in optimizer soundness, if at all? The conservative rule is: never use statistical evidence to justify semantic rewrites.
5. **Temporal extension.** GAP 5 will need temporal/process soundness obligations. This note covers sequential Core/Act-style contracts only.

## 12. References

### Internal references

- [NOTE-014: Contract Systems Unification](NOTE-014-CONTRACT-SYSTEMS-UNIFICATION.md)
  — source gap register; this note resolves GAP 7.
- [NOTE-027: Contract Blame and Subsumption](NOTE-027-CONTRACT-BLAME-AND-SUBSUMPTION.md)
  — blame labels, polarity, subsumption, and the link to blame soundness.
- [NOTE-028: Purity, Evaluation Modes, and Contract Timing](NOTE-028-PURITY-EVALUATION-MODES-AND-CONTRACT-TIMING.md)
  — strict/lazy/memo timing and replay boundaries.
- [NOTE-029: Structured Bottom and Contract Diagnostics](NOTE-029-STRUCTURED-BOTTOM-AND-CONTRACT-DIAGNOSTICS.md)
  — structured bottom, diagnostic payloads, explicit `fail`, and GAP 7 reminder.
- [NOTE-030: Monadic Hoare Logic for Ash Computations](NOTE-030-MONADIC-HOARE-LOGIC-FOR-ASH-COMPUTATIONS.md)
  — bind-level contract composition and `ComposedContract` metadata.
- [NOTE-031: Contract Predicate Well-Formedness and Snapshot Semantics](NOTE-031-CONTRACT-PREDICATE-WELL-FORMEDNESS-AND-SNAPSHOTS.md)
  — predicate classification, snapshot semantics, and predicate-fault separation.
- [SPEC-097b: Target Type System](../spec/SPEC-097b-TARGET-TYPE-SYSTEM.md)
  — target predicate summaries, subsumption, sequencing contracts, and evaluation modes.
- [SPEC-098b: Target IR](../spec/SPEC-098b-TARGET-IR.md)
  — `ContractDischarge`, `ComposedContract`, `SnapshotRef`, `ContractDiagnostic`, and `ContractPredicateFault` metadata.
- [SPEC-099: Core Language](../spec/SPEC-099-CORE-LANGUAGE.md)
  — Core dynamic contract checks, traps, and predicate evaluation boundary.
- [SPEC-100: Core Type Checking](../spec/SPEC-100-CORE-TYPE-CHECKING.md)
  — predicate well-formedness, obligation generation, dynamic demotion, and contract trap typing.

### External references

- C. A. R. Hoare, "An Axiomatic Basis for Computer Programming" (1969).
  Original Hoare-triple foundation for pre/postcondition reasoning.
  <https://doi.org/10.1145/363235.363259>
- Robert Bruce Findler and Matthias Felleisen, "Contracts for Higher-Order Functions" (2002).
  Foundational higher-order contract and blame setting. DOI target was reachable through DOI but ACM page was bot-protected during this session.
  <https://doi.org/10.1145/581478.581484>
- Christos Dimoulas, Robert Bruce Findler, Cormac Flanagan, and Matthias Felleisen, "Correct Blame for Contracts: No More Scapegoating" (2012).
  Background for the blame-soundness obligation.
  <https://doi.org/10.1145/2103621.2103697>
- Danel Ahman, Cătălin Hriţcu, Kenji Maillard, Guido Martínez, Gordon Plotkin, Jonathan Protzenko, Aseem Rastogi, and Nikhil Swamy, "Dijkstra Monads for Free" (POPL 2017).
  Verified by browser during this session at the F* project page; relevant to weakest-precondition reasoning for effects.
  <https://www.fstar-lang.org/papers/dm4free/>
- Nikhil Swamy et al., "Dependent Types and Multi-monadic Effects in F*" (POPL 2016).
  Verified by browser during this session at the F* project page; relevant to combining SMT automation, dependent types, and effectful verification.
  <https://www.fstar-lang.org/papers/mumon/>
- Xavier Leroy et al., CompCert project.
  Verified by browser during this session; relevant as prior art for optimizer/compiler transformations justified by machine-checked semantic preservation.
  <https://compcert.org/>

## 13. Changelog

| Date | Change |
|------|--------|
| 2026-06-29 | Initial note. Resolves NOTE-014 GAP 7 by stating five meta-level soundness obligations: gradual verification soundness, blame soundness, optimizer soundness, dynamic demotion soundness, and predicate-fault separation. |
