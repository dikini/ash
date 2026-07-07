# NOTE-036: Gradual Verification and Proof Provider Architecture for Ash

**Date:** 2026-07-07
**Status:** Living document — design direction captured; research and future-spec guide
**Purpose:** Frame Ash's multi-tier verification model (automatic proof, property testing,
runtime checking) as a single gradual-assurance system. Define what predicates Ash should encode,
how proof obligations surface in syntax and Core IR, how evidence is propagated/erased/transformed,
and how external proof providers (SMT, Lean, symbolic execution, LLM-assisted) integrate without
breaking fail-closedness.

Companion to NOTE-030 (monadic Hoare composition), NOTE-031 (predicate well-formedness),
NOTE-033 (surface-to-Core lowering), NOTE-034 (contract-capability boundary), NOTE-035 (temporal
contracts), NOTE-037 (symbolic-connectionist duality), SPEC-064 (constraint/proposition layer),
SPEC-080 (interface evidence constraints), SPEC-081 (law test evidence), SPEC-085
(proof-producing synthesis deferral), SPEC-096b (target effect system), SPEC-098b (target IR),
SPEC-099/SPEC-099b (Core language and operational semantics), SPEC-100 (Core type checking),
PLAN-145, PLAN-149, PLAN-165, PLAN-194, PLAN-195, and SPEC-038 (LSP/MCP stack research).

## Pre-Spec Delta

This note is pre-spec and research. When promoted into target specs, reconcile:

- **SPEC-095b Target Grammar:** add `proof` body alternatives (`by solver`, `by symbolic`,
  `by proof_term`, `by lean`) and any quantifier/law surface reserved by the automatic tier.
- **SPEC-096b Target Effect System:** add `verified` contract discharge and proof-provider/evidence
  row kinds.
- **SPEC-097b Target Type System:** add proof-obligation typing rules, predicate AST well-formedness,
  and quantifier profile for statically discharged implications.
- **SPEC-098b Target IR:** add `ProofObligation`, `ProofEvidence`, `ProofProviderRef`, and
  `VerifiedDischarge` sidecars.
- **SPEC-099 / SPEC-099b Core Language and Operational Semantics:** specify erasure semantics for
  verified predicates and dynamic-check insertion for non-verified predicates.
- **SPEC-100 Core Type Checking:** add obligation generation from `requires`/`ensures`/laws and
  evidence revalidation rules.
- **SPEC-081 / PLAN-145 Law Test Evidence:** position `by test` as the empirical tier beneath
  automatic proof tiers.
- **SPEC-085 / PLAN-149 Proof-Producing Synthesis:** replace the deferred placeholder with a
  concrete proof-evidence family and provider interface.
- **docs/plan/PLAN-INDEX.md:** register a new implementation phase for automatic proof integration
  once this note stabilizes.

## 0. Motivation

Ash already has three verification/assurance mechanisms in different stages of maturity:

1. **Runtime contract checking** — `requires`/`ensures` on `fn` lower to dynamic checks with
   structured blame (PLAN-194).
2. **Empirical law evidence** — `proof ... { by test ... }` supports authored tests, property
   tests, and small-world tests (SPEC-081, PLAN-145).
3. **Type-level propositions** — a conservative constraint/proposition layer handles equality,
   disequality, interface bounds, and named predicates (SPEC-064).

What is missing is the **automatic proof tier**: SMT-backed, symbolic-execution-backed,
proof-assistant-backed, or model-checked discharge of obligations. The project has deliberately
kept this deferred (SPEC-085) until the trust boundary, syntax, and evidence model were clear.

This note makes them clear enough to write a spec. It argues that Ash should not treat automatic
verification as a separate feature bolted onto the language. Instead, Ash should expose one
**gradual verification** continuum:

```text
unverified  ->  tested  ->  monitored  ->  verified
     |              |            |             |
   omitted     by test      by monitor    by solver/symbolic/proof_term/lean
```

The same predicate can be discharged at different tiers in different compilation contexts,
depending on available providers, trust settings, and decidability. The compiler must report,
use, erase, or transform each obligation consistently.

## 0.1 Symbolic-connectionist duality

This note treats proof providers as the **symbolic** side of a broader dual system. Ash is also
intended to integrate **connectionist** assistance — primarily LLMs — as a heuristic companion that
suggests but does not verify. See NOTE-037 for the full thesis.

In the proof subsystem the interaction is a triple:

```text
Ash compiler  (orchestrator)
       |
       ├── Prover  (symbolic checker: SMT, Lean, ...)
       └── LLM     (connectionist suggester)
```

The prover returns `verified` / `refuted` / `deferred`. The LLM returns suggestions. A suggestion
becomes evidence only after the prover checks it. The compiler records `Hybrid` evidence when an
LLM suggestion was validated by a prover. LLM calls are non-deterministic operations and should
appear in effect rows.

Short form: automatic proof is symbolic; LLM assistance is connectionist; the compiler is the
orchestrator; fail-closedness remains on the symbolic side.

## 1. Gradual verification as an evidence lattice

Every proof obligation in Ash should carry an **evidence outcome**. The outcome determines what the
compiler does with the obligation:

| Outcome | Produced by | Compile-time action | Runtime action | Evidence record |
|---|---|---|---|---|
| `verified` | Trusted proof provider (SMT, Lean, symbolic, proof term) | Erase dynamic check; record proof metadata | Nothing (zero-cost) | Provider id, proof fingerprint, trust assumptions |
| `verified (hybrid)` | LLM suggestion checked by a prover | Same as `verified`; also record LLM invocation | Nothing | Prover evidence + LLM invocation metadata |
| `tested` | `by test` authored/property/small-world (SPEC-081) | Lower to optional or mandatory test linkage | Depends on contract strategy | Test result, seed, counterexample, coverage |
| `monitored` | Runtime monitor / trace contract (NOTE-035) | Insert monitor instrumentation | Monitor consumes trace facts | Monitor plan, observed facts, verdict |
| `deferred` | Provider declined or unsupported fragment | Emit diagnostic; require explicit strategy | Usually lower to dynamic check or fail | Deferral reason, residual obligation |
| `refuted` | Counterexample or proof of violation | Compile-time error or explicit failure path | Statically-inserted failure path | Counterexample, blame label |
| `untested` | No evidence attempted | Warning or error depending on policy | Dynamic check if contract requires it | Untested status |

The default policy should be **fail-closed**: a `deferred` or `untested` outcome for a contract
that has no dynamic-check strategy is a compile-time error, not a silent omission.

## 2. What to encode

Ash should not aim for "any predicate about any Ash program" in the first slice. The initial
assertion language should be the set of predicates Ash can already evaluate at runtime, plus the
type-level propositions already in SPEC-064:

### 2.1 Value predicates

- Arithmetic and equality over primitive types (`Int`, `Float`, `String`, `Bool`).
- Field/selector observation on records and ADTs.
- Pure total helper functions marked as contract-safe (NOTE-031).
- `old(x)` snapshots at a single contract boundary (NOTE-031).
- `result` in postconditions.

These are exactly the predicates PLAN-194 already evaluates dynamically.

### 2.2 Effect-row containment

- "This computation does not require operation `O`."
- "This computation's row is a subrow of `{...}`."

These are predicates over computation rows (SPEC-096b). They are essential for sandboxing and
authority reasoning.

### 2.3 Interface laws

- Parametric equations over interface methods, e.g. `Functor.identity`, `Monad.associativity`.
- Laws already synthesized for algebra interfaces (PLAN-145 follow-on work).

### 2.4 Type-level propositions

- Equality, disequality, interface bounds, named predicates (SPEC-064).
- These bridge value-level contracts with the type system.

### 2.5 Temporal/trace properties

- Safety and liveness over trace facts (NOTE-035).
- These discharge primarily through monitors, with bounded finite-state fragments eligible for
  model-checking discharge.

### 2.6 Asymptotic goal

Eventually Ash may support richer quantification, separation logic, and program-equivalence
predicates. Those require proof-assistant integration and should be added only after the provider
architecture and trust model are stable.

## 3. Surface and Core syntax

### 3.1 Surface

Ash already has the right syntactic anchors. Extend `proof` bodies and keep contract syntax stable:

```ash
-- existing runtime contracts
fn sqrt(x: Int) requires { x >= 0 } ensures { result * result <= x } -> Int { ... }

-- existing empirical law proof
proof associativity(a: T, b: T, c: T) where T: Semigroup {
    by test property { ... }
}

-- future automatic proof modes (illustrative, not committed keywords)
proof associativity(a: T, b: T, c: T) where T: Semigroup {
    by solver z3 { timeout: 30s }
}

proof safety(p: Packet) {
    by symbolic { produce proof_artifact }
}

proof termination(n: Int) {
    by lean { theorem: "Ash.Termination.foo" }
}

proof identity(x: Option<Int>) {
    by proof_term { ... }
}
```

The `proof` body should be an **evidence family enum**, not a string label. The existing
`ByTest` family (SPEC-081) becomes one variant of a larger `ProofEvidence` type.

### 3.2 Core predicate AST

There must be **one Core predicate AST** that feeds every backend. Do not create a separate AST for
SMT, Lean, runtime evaluation, and property testing.

```rust
pub enum Predicate {
    Bool(bool),
    Not(Box<Predicate>),
    And(Vec<Predicate>),
    Or(Vec<Predicate>),
    Implies(Box<Predicate>, Box<Predicate>),
    ForAll(Binder, Box<Predicate>),   // internal/Core only initially
    Exists(Binder, Box<Predicate>),   // internal/Core only initially
    Eq(Term, Term),
    Lt(Term, Term),
    Le(Term, Term),
    Contains(RowRequirement, RowSet), // effect-row containment
    InterfaceBound(Term, InterfaceApp),
    Named(NamedPredicateId, Vec<Term>),
    Old(SnapshotRef, Term),
    Result(Term),
}

pub enum Term {
    Var(VarId),
    Literal(Value),
    Field(Box<Term>, FieldId),
    Call(PureFnRef, Vec<Term>),       // contract-safe pure helpers only
}
```

The surface grammar need not expose `forall`/`exists` in the first slice; those remain internal to
the predicate transformer (NOTE-030) and proof backends. Surface contracts keep their current
first-order expression shape.

### 3.3 Proof obligation sidecar

Core should carry proof obligations as sidecar metadata, not new term forms:

```rust
pub struct ProofObligation {
    pub id: ObligationId,
    pub source_span: Span,
    pub predicate: Predicate,
    pub origin: ObligationOrigin,
    pub discharge: ProofDischarge,
}

pub enum ObligationOrigin {
    RequiresClause { fn_id: FnId },
    EnsuresClause { fn_id: FnId },
    Law { interface: InterfaceId, law_name: String },
    BindContinuation { producer: FnId, continuation: FnId },
    InterfaceEvidenceConstraint { interface: InterfaceId },
    UserProof { proof_id: ProofId },
}

pub enum ProofDischarge {
    Verified(VerifiedEvidence),
    Tested(TestEvidence),           // from SPEC-081
    Monitored(MonitorPlanRef),      // from NOTE-035
    Dynamic(RuntimeCheckPlan),      // from PLAN-194
    Deferred(DeferralReason),
    Refuted(RefutationEvidence),
    Untested,
}
```

## 4. Evidence propagation: report, use, erase, transform

### 4.1 Report

The compiler, LSP, and `ash test`/`ash check` must report evidence outcomes in a stable schema.
JSON output should include:

```json
{
  "obligation_id": "sqrt.post.0",
  "predicate": "result * result <= x",
  "origin": "ensures_clause",
  "evidence_family": "solver",
  "provider": "z3",
  "outcome": "verified|tested|monitored|deferred|refuted|untested",
  "trust": "builtin|mcp|local|unchecked",
  "fingerprint": "sha256:...",
  "residual": null
}
```

### 4.2 Use

Evidence is used during row discharge and compilation:

- `verified` on a `requires`/`ensures` contract removes the runtime check.
- `verified` on an interface evidence constraint (SPEC-080) removes the runtime evidence lookup
  requirement and may enable specialization.
- `tested` on a law satisfies the law obligation for the test run but does not erase runtime
  checks elsewhere.
- `monitored` on a temporal contract inserts monitor instrumentation (NOTE-035).

### 4.3 Erase

Erasure is the zero-cost benefit of verification. Rules:

- Erase only when the outcome is `verified` and the provider is trusted by the current compilation
  profile.
- Record the erased proof metadata in the module summary so downstream consumers can revalidate.
- Do not erase `tested` or `monitored` checks by default; those are empirical, not deductive.

### 4.4 Transform to dynamic checks

When an obligation is not `verified`, the compiler transforms it:

- `requires`/`ensures` value predicates -> runtime check plan (PLAN-194).
- Temporal/trace properties -> monitor plan (NOTE-035).
- `deferred` with no dynamic strategy -> compile-time error.
- `refuted` -> compile-time error or explicit failure path, depending on surface recoverability.

This is the **gradual** part: the same source predicate can become a static proof, a runtime
check, a monitor, or a test oracle depending on the outcome.

## 5. System architecture for proof providers

### 5.1 Do not embed all provers as libraries

Ash should use a **provider model**. In the dual-system framing from NOTE-037, providers are the
symbolic side; the compiler orchestrates them together with optional connectionist assistants:

```text
Ash compiler
    |
    ├── Prover providers  (symbolic: SMT, Lean, symbolic execution)
    └── LLM assistant     (connectionist: suggestion only)
```

- **Built-in, lightweight checks** in `ash-core` / `ash-typeck` for decidable fragments:
  SPEC-064 normalization, constructor disjointness, simple arithmetic, row subsumption.
- **External prover servers** connected via MCP (SPEC-038 recommends `rmcp`).
- **Optional embedded solvers** only when binary size and dependency trust are acceptable.
- **LLM suggestion servers** exposed as MCP tools, never as evidence producers.

### 5.2 Provider interface

Each proof provider exposes a stable interface. An MCP server is the default shape:

```rust
pub struct ProofProvider {
    pub id: ProviderId,
    pub version: String,
    pub accepted_fragments: Vec<FragmentKind>,
    pub trust_class: TrustClass,
}

pub enum ProofRequest {
    Prove { obligation: Predicate, timeout: Duration, hints: Vec<Hint> },
    Check { obligation: Predicate, proof: ProofArtifact, timeout: Duration },
    Suggest { obligation: Predicate, timeout: Duration },
}

pub enum ProofResponse {
    Verified { evidence: ProofArtifact, fingerprint: String },
    Refuted { counterexample: Option<TermSubstitution> },
    Deferred { reason: String },
    Error { diagnostic: ProviderDiagnostic },
}
```

Tools:

- `prove`: attempt full discharge.
- `check`: validate an externally supplied proof term or artifact.
- `suggest`: return proof hints/tactics without claiming discharge.

### 5.3 Provider registry

A module or workspace declares acceptable providers:

```toml
[proof_providers]
accepted = ["ash.builtin", "mcp://localhost:7777/z3", "mcp://leanprover/lean4"]
trust_model = "revalidate"   # or "trust", "record-only"
```

- `revalidate`: the compiler re-runs `check` on returned proof artifacts.
- `trust`: accept provider verdicts without revalidation (useful for fast LSP feedback only, not
  release builds).
- `record-only`: provider suggestions are treated as `deferred` unless independently checked.

### 5.4 LSP and MCP integration

- The language server can call providers asynchronously for diagnostics and hover information.
- MCP servers for `z3`, `lean4`, and symbolic engines are separate processes; Ash connects via the
  protocol researched in SPEC-038.
- The compiler is the authority for release builds; LSP feedback is advisory.

### 5.5 Lean4 specifically

The `lean_reference/` directory already contains a Lean reference interpreter. The proof-provider
architecture should reuse or extend this:

- Embed a shallow embedding of Ash predicates in Lean.
- Provide a translator from Core `Predicate` to Lean expressions.
- Use Lean's kernel to check proof terms returned by the provider.
- Do not require end users to write Lean; surface proofs remain in Ash.

## 6. LLM integration

This section is the proof-specific instance of the broader symbolic-connectionist duality
described in NOTE-037. See that note for the language-design thesis, effect-row treatment of LLM
calls, and the `Symbolic` / `Connectionist` / `Hybrid` evidence taxonomy.

LLMs such as Leanstral should be treated as **proof suggestion assistants**, not evidence
producers. The allowed integration is:

1. User writes a law or proof sketch in Ash.
2. LLM suggests a proof term, SMT encoding, tactic sequence, or lemma set.
3. Ash checks the suggestion through a trusted provider.
4. If the check passes, the obligation becomes `verified (hybrid)` with the trusted provider's
   fingerprint plus LLM invocation metadata.
5. If the check fails, the suggestion is discarded or downgraded to `deferred`.

LLM output must never directly discharge an obligation. LLM calls are non-deterministic
operations and should carry an effect row (e.g., `{LLM::complete model}`). This preserves
fail-closedness and reproducibility. The LLM may be exposed as an MCP tool called `suggest_proof`
with response type `Suggestion`, not `ProofResponse::Verified`.

## 7. Trust boundary and reproducibility

Every `verified` outcome must be reproducible:

- Record provider id, version, predicate hash, proof artifact hash, timeout, and trust model.
- Store proof artifacts alongside module summaries for downstream revalidation.
- Reject provider version drift if the predicate hash matches but the provider version changed.
- Distinguish "provider said verified" from "Ash compiler independently checked" in evidence
  metadata.

This is essential because external solvers and proof assistants can have bugs or version-dependent
behavior.

## 8. Relation to existing work

- **NOTE-037:** frames the proof provider subsystem as the symbolic side of a
  symbolic-connectionist dual system. This note's SMT/Lean providers and LLM assistants are an
  instance of that broader thesis.
- **PLAN-194 / NOTE-033:** runtime contract checks are the dynamic fallback for non-verified value
  predicates.
- **NOTE-030:** monadic Hoare composition generates the `∀a. Q(a) ⇒ R(a)` proof obligations that
  the automatic tier attempts to discharge.
- **NOTE-031:** predicate well-formedness restricts the assertion language to authority-free,
  stable observers; this is the same restriction the SMT/Lean backends rely on.
- **NOTE-034:** contract evaluators and monitors consume facts rather than acquiring authority;
  proof providers are also fact consumers.
- **NOTE-035:** temporal contracts discharge primarily through monitors, with bounded fragments
  eligible for model-checking providers.
- **SPEC-064:** type-level propositions are the first automatic-discharge fragment; they already
  use `Deferred` as a conservative outcome.
- **SPEC-080:** interface evidence constraints become proof obligations that a provider can
  discharge, reducing runtime evidence lookup.
- **SPEC-081 / PLAN-145:** `by test` is the empirical tier; this note positions it beneath the
  automatic tiers.
- **SPEC-085 / PLAN-149:** replaces the deferred placeholder with concrete architecture.
- **SPEC-096b:** effect rows include contract/evidence items; `verified` is a new discharge kind.
- **SPEC-038:** MCP/LSP stack research recommends `rmcp` and `tower-lsp-server` for provider
  integration.

## 9. Design decisions

1. **One predicate AST for all backends.** Runtime evaluator, property tests, SMT, Lean, and
   symbolic execution all consume the same Core `Predicate`.
2. **Evidence outcomes are first-class compiler values.** Every obligation has an outcome that
   determines erasure, dynamic insertion, or failure.
3. **Fail-closed by default.** `deferred` or `untested` without an explicit strategy is an error.
4. **External providers via MCP by default.** Built-in checks handle only lightweight decidable
   fragments.
5. **LLMs are suggestion assistants only.** Their output must pass a trusted checker before it
   becomes `verified`.
6. **Reproducibility metadata is mandatory for `verified`.** Provider version, predicate hash, and
   proof artifact must be recorded.
7. **Surface syntax evolves from existing `proof` bodies.** No new top-level declaration form is
   needed in the first slice.
8. **Quantifiers remain internal initially.** Surface contracts stay first-order; `forall`/
   `exists` appear in Core predicate metadata and proof backends.
9. **LLM assistance is connectionist, not evidentiary.** A verified outcome may be `hybrid` if an
   LLM suggestion was checked by a prover, but the evidence is the prover's check, not the
   suggestion. See NOTE-037.

## 10. Open questions

1. **Predicate language boundary.** Which fragments does each backend accept in MVP? Should Ash
   reject a predicate that mixes real arithmetic with heap reasoning, or defer it?
2. **Proof artifact size.** How are large Lean proof terms transported in module summaries?
3. **Counterexample usability.** When a solver refutes a predicate, how is the counterexample
   rendered in Ash diagnostics?
4. **Provider timeouts and determinism.** How does Ash guarantee reproducible timeouts across
   platforms?
5. **Effect-row proofs.** What is the proof theory for row subsumption and disjointness, and which
   solver handles it best?
6. **Lean embedding strategy.** Do we deep-embed Ash terms or shallow-embed via a generated DSL?
7. **LSP feedback vs. compiler authority.** How do we prevent fast-but-unchecked LSP suggestions
   from leaking into release builds?
8. **Law synthesis interaction.** Can synthesized algebra laws be automatically routed to a solver
   provider, or do they always require explicit `proof` bodies?

## 11. References

### Internal references

- [NOTE-030: Monadic Hoare Logic for Ash Computations](NOTE-030-MONADIC-HOARE-LOGIC-FOR-ASH-COMPUTATIONS.md)
- [NOTE-031: Contract Predicate Well-Formedness and Snapshots](NOTE-031-CONTRACT-PREDICATE-WELL-FORMEDNESS-AND-SNAPSHOTS.md)
- [NOTE-033: Surface-to-Core Contract Lowering](NOTE-033-SURFACE-TO-CORE-CONTRACT-LOWERING.md)
- [NOTE-034: Contract Capability Boundary](NOTE-034-CONTRACT-CAPABILITY-BOUNDARY.md)
- [NOTE-035: Temporal and Concurrent Contracts](NOTE-035-TEMPORAL-AND-CONCURRENT-CONTRACTS.md)
- [NOTE-037: Ash as a Symbolic-Connectionist Duality](NOTE-037-SYMBOLIC-CONNECTIONIST-DUALITY.md)
- [SPEC-038: Rust LSP & MCP Stack Research 2025](../spec/SPEC-038-RUST-LSP-MCP-RESEARCH-2025.md)
- [SPEC-064: Constraint and Proposition Layer](../spec/SPEC-064-CONSTRAINT-PROPOSITION-LAYER.md)
- [SPEC-080: Interface Evidence Constraints](../spec/SPEC-080-INTERFACE-EVIDENCE-CONSTRAINTS.md)
- [SPEC-081: Law Test Evidence Substrate](../spec/SPEC-081-LAW-TEST-EVIDENCE-SUBSTRATE.md)
- [SPEC-085: Proof-Producing Synthesis Todo Spec](../spec/SPEC-085-PROOF-PRODUCING-SYNTHESIS-TODO-SPEC.md)
- [SPEC-096b: Target Effect System](../spec/SPEC-096b-TARGET-EFFECT-SYSTEM.md)
- [SPEC-098b: Target IR](../spec/SPEC-098b-TARGET-IR.md)
- [SPEC-099: Core Language](../spec/SPEC-099-CORE-LANGUAGE.md)
- [SPEC-099b: Target Operational Semantics](../spec/SPEC-099b-TARGET-OPERATIONAL-SEMANTICS.md)
- [SPEC-100: Core Type Checking](../spec/SPEC-100-CORE-TYPE-CHECKING.md)
- [PLAN-145: Law Test Evidence Substrate](../plan/PLAN-145-LAW-TEST-EVIDENCE-SUBSTRATE.md)
- [PLAN-149: Proof-Producing Synthesis Todo Spec](../plan/PLAN-149-PROOF-PRODUCING-SYNTHESIS-TODO-SPEC.md)
- [PLAN-165: Contract System Implementation Handoff](../plan/PLAN-165-CONTRACT-SYSTEM-IMPLEMENTATION-HANDOFF.md)
- [PLAN-194: Contract and Evidence System](../plan/PLAN-194-CONTRACT-AND-EVIDENCE-SYSTEM.md)
- [PLAN-195: Process and Concurrency Model](../plan/PLAN-195-PROCESS-AND-CONCURRENCY-MODEL.md)

### External references

- C. A. R. Hoare. "An Axiomatic Basis for Computer Programming." Communications of the ACM, 1969.
  <https://doi.org/10.1145/363235.363259>
- Leonardo de Moura and Nikolaj Bjørner. "Z3: An Efficient SMT Solver." TACAS 2008.
  <https://doi.org/10.1007/978-3-540-78800-3_24>
- Leonardo de Moura et al. "The Lean Theorem Prover." CADE 2015.
  <https://leanprover.github.io/papers/lean.pdf>
- N. Swamy et al. "Dependent Types and Multi-Monadic Effects in F*." POPL 2016.
  <https://www.fstar-lang.org/papers/mumon/>
- Model Context Protocol specification. <https://modelcontextprotocol.io/>

## 12. Changelog

| Date | Change |
|---|---|
| 2026-07-07 | Initial note. Frames Ash gradual verification: one predicate AST, evidence-outcome lattice, external proof providers via MCP, LLM-as-suggestion-assistant, and trust/reproducibility requirements. Positions automatic proof as the next tier above existing runtime checks and empirical law tests. |
| 2026-07-07 | Added symbolic-connectionist duality summaries and references to NOTE-037. Clarified `verified (hybrid)` evidence and the `(compiler, prover, LLM)` triple. |
