---
id: docs.notes.039
title: Proving Ash in Ash
kind: design-note
status: exploratory
authority: non-normative
date: 2026-07-12
tags:
  - type-system
  - proof-time
  - laws
  - constraints
  - effect-system
  - llm
  - tooling
---

# NOTE-039: Proving Ash in Ash

## Status and purpose

This note explores a possible direction; it does not define normative Ash semantics.

The central question is:

> If an Ash interpreter is available at proof time, can Ash laws, constraints, contracts, and proof procedures be written and evaluated in Ash itself?

The proposed direction is to reuse Ash as its own proof-time computation language. At proof time, selected program entities are available one semantic level higher: types can be manipulated as values, kinds classify those type-values, and a deliberately controlled subset of static values may be lifted into type-level computation. Computation rows isolate the fragment that is admissible at proof time.

This could make external systems such as Z3, Lean 4, and QuickCheck available through ordinary typed libraries and allowed effects rather than as language-specific built-ins. It also provides a foundation for an LLM to generate a workflow together with its laws and constraints, interactively discharge the resulting obligations, and co-execute the accepted program with tools or sub-agents.

The phrase **proving Ash in Ash** refers to this combination:

1. Ash states workflow logic and its required properties.
2. Ash functions implement proof-time computations and proof strategies.
3. The Ash checker controls which computations may run at proof time.
4. Ash libraries may use admitted proof-provider effects.
5. Accepted evidence discharges Ash obligations.
6. The resulting program may execute only through the remaining admitted boundaries.

## 1. Design pressure

A complex workflow generated from natural language contains more than an execution graph. It also carries requirements such as:

- tasks compose through compatible inputs and outputs;
- required artifacts are produced before their consumers execute;
- review gates cannot be bypassed;
- reviewers and authors satisfy separation-of-duty rules;
- actors have the required roles and operation authority;
- budgets, deadlines, and resource limits are feasible;
- claims have evidence with acceptable provenance;
- failures and unresolved obligations are reported rather than erased.

Writing the workflow and checking these properties in unrelated languages creates a semantic gap. The workflow language, constraint language, solver encoding, execution engine, and LLM-facing diagnostics can drift apart.

Ash already has relevant concepts:

- laws and constraints;
- contracts and evidence;
- types, kinds, and type-level computation;
- computation rows and distinct row-item families;
- handlers and providers;
- static, evidence-backed, and dynamic discharge modes;
- tasks/processes, roles, operations, and workflow admission.

The opportunity is to make these concepts compositional inside Ash instead of introducing a separate proof DSL for every proof provider.

## 2. Proof time as a general stage

**Proof time** is any stage at which an obligation must be discharged before a governed action may proceed. Compile time is one important proof-time boundary, but not the only one.

Possible boundaries include:

- definition time, where generic laws are checked;
- instantiation time, where concrete types and static parameters are known;
- planning or admission time, where a workflow graph, actors, providers, and resources are selected;
- pre-execution time, where concrete inputs and current attestations are available;
- execution checkpoints, where task results introduce new facts and evidence.

An obligation should therefore have a required discharge boundary rather than being classified only as “compile-time” or “runtime.”

Execution may introduce new facts. It must not silently weaken the law that required those facts.

## 3. Semantic lifting

At an ordinary evaluation level, one may write schematically:

```text
value : Type : Kind
```

At proof time, Ash should expose selected entities one level higher:

- a type is available as a proof-time value;
- its kind is the type of that value;
- type constructors are applicable proof-time functions;
- rows and their items are inspectable and composable;
- selected static values may be promoted or lifted under explicit rules.

The intent is not an unrestricted `Type : Type` universe. Ash needs stratification, whether visible in the surface language or maintained internally. Nor does the proposal require every runtime value to be usable in a type.

The desired capability is closer to:

```text
TypeValue<T> : KindOf<T>
RowValue<r>  : Row
```

with ergonomic surface syntax that does not necessarily expose representation wrappers.

Proof-time Ash should be able to perform operations such as:

- pattern matching over type constructors;
- inspecting record, variant, protocol, and computation rows;
- applying total type-level functions;
- normalizing aliases and applications;
- testing or constructing compatibility witnesses;
- deriving data-flow and composition obligations;
- computing residual constraints;
- constructing a type from explicitly liftable static data.

### 3.1 Controlled value-to-type lifting

Some workflow properties depend on static values:

- vector or collection sizes;
- retry bounds;
- declared budgets;
- role names;
- resource counts;
- deadlines or scheduling horizons;
- finite graph descriptions.

Ash may therefore need a controlled lifting relation:

```text
lift : Static<A> -> TypeLevel<A>
```

A value should be liftable only when its computation is available at proof time and meets declared requirements such as purity, termination, stable representation, and decidable equality. The exact relation remains open.

The lifting mechanism must not collapse all values into types or imply `Type : Type`.

## 4. Proof-time Ash as a row-isolated fragment

A separate proof syntax is not required merely to isolate proof-time computation. Computation rows can define the boundary.

Schematically:

```text
Γ ⊢ e : A ! ρ
ρ ≤ ProofTimeAllowed
--------------------
Γ ⊢proof e : A
```

A pure proof procedure has an empty operation-effect requirement. A solver-backed procedure may require an admitted provider operation:

```ash
fn compatible(a: Type, b: Type)
    -> {Z3::check} Decision<Composable<a, b>> {
    // exploratory syntax
}
```

The important separation is:

- a row describes what the proof-time computation requires;
- admission grants the relevant authority or provider;
- an evidence type describes what the result is allowed to establish.

Permission to call a provider does not make every provider response a theorem.

### 4.1 Candidate proof-time restrictions

The proof-time admission profile will likely require restrictions including:

- termination or an explicit bounded evaluation policy;
- phase closure over proof-time available inputs;
- deterministic semantics where deductive results require reproducibility;
- no undeclared authority;
- no untracked dependence on mutable external state;
- explicit treatment of timeout and provider `unknown`;
- preservation of the distinction between logical failure and operational failure.

Not every admitted effect must have the same epistemic status. For example, randomness may be admitted for property testing while remaining inadmissible for deductive normalization.

## 5. Laws as executable propositions

A law should distinguish:

1. the proposition being asserted;
2. one or more procedures that may try to discharge it;
3. the evidence grade required by the consumer;
4. the stage by which it must be discharged.

A computationally decidable law may normalize directly:

```text
compatible(A, B) ⇓ proved(witness)
compatible(A, B) ⇓ disproved(counterexample)
```

When variables or unsupported theories remain, evaluation should preserve an explicit residual obligation instead of treating failure to prove as false.

A useful conceptual result domain is:

```ash
type Decision<P> =
    | Proved(Proof<P>)
    | Disproved(Counterexample<P>)
    | Residual(Obligation<P>)
    | Unknown(Reason)
```

This syntax is illustrative. The semantic distinctions matter more than the constructors.

### 5.1 Evidence grades

Ash should not conflate all successful checks. Candidate evidence grades include:

- deductive proof;
- checked certificate;
- counterexample;
- model or witness;
- property-test evidence;
- external attestation;
- review evidence;
- evaluator or heuristic evidence;
- unresolved obligation.

Examples:

- a topological order can certify DAG acyclicity;
- an SMT model can witness satisfiability or provide a counterexample;
- an independently checked unsatisfiability certificate can support deductive discharge;
- Lean can check a translated theorem under an explicit bridge;
- a QuickCheck counterexample disproves a universal property;
- successful QuickCheck samples do not normally prove it;
- an LLM quality evaluation is advisory or review evidence unless policy explicitly assigns it another status.

Required evidence grade is part of the quality bar.

## 6. Library-defined proof providers

Z3, Lean 4, QuickCheck, model checkers, and domain-specific decision procedures should preferably appear through ordinary Ash interfaces and libraries.

Conceptually:

```ash
interface Smt {
    fn check(formula: Formula) -> Result<Model, Certificate, Unknown>
}

interface TheoremProver {
    fn check(theorem: Theorem) -> Result<CheckedTheorem, Diagnostic>
}

interface PropertyTester {
    fn test<A>(
        generator: Generator<A>,
        property: fn(A) -> Bool
    ) -> TestResult<A>
}
```

The concrete syntax and result types are open.

Libraries would provide:

- encodings from Ash propositions into provider languages;
- decoding of models and counterexamples;
- certificate verification where available;
- proof strategies and combinators;
- generators and shrinkers;
- diagnostics that map provider results back to Ash source entities.

This keeps provider knowledge out of the language core. The core needs staging, rows, type/kind exposure, controlled lifting, obligations, and evidence admission. Provider-specific semantics remain in libraries and handlers.

### 6.1 Trust boundary

A provider effect is an operational interface, not automatically a trusted oracle.

Different deployments may choose different trust policies:

- trust a provider result directly;
- require a checkable certificate;
- translate the proposition to Lean and trust Lean's kernel plus the translation;
- accept testing only as advisory evidence;
- require human review for propositions outside automated fragments.

The trust decision must remain explicit and auditable.

## 7. Interactive elaboration

The useful development model is not one-shot proof generation. It is interactive elaboration:

```text
requirements
  -> candidate Ash program
  -> inferred types, rows, and obligations
  -> proof-time evaluation
  -> proofs, counterexamples, or residual goals
  -> local repair
  -> admitted executable program
```

Ash should permit incomplete programs with explicit holes while keeping them non-executable past the relevant gate.

A structured proof state should expose:

- the proposition;
- typed local assumptions;
- the relevant law and contract context;
- available pure functions and proof providers;
- allowed proof-time row;
- required evidence grade;
- latest counterexample or failed derivation;
- the boundary by which the goal must be discharged.

This structure is useful to humans, IDEs, and LLMs alike.

## 8. LLM-authored proof-carrying workflows

A motivating scenario is an LLM receiving a natural-language description of a complex workflow and producing an Ash program containing:

- workflow computations;
- typed task inputs and outputs;
- roles and authority requirements;
- data-flow composition;
- laws, constraints, and contracts;
- proof procedures or selected proof strategies;
- execution-time evidence requirements;
- explicit unresolved holes when information is missing.

The LLM then participates in the elaboration and execution loop.

### 8.1 Example scenario

A prompt requests a researched report with independent review, supported factual claims, a fixed budget, and publication only after approval.

The LLM drafts Ash definitions for:

- research;
- evidence normalization;
- drafting;
- claim-to-source linking;
- independent review;
- publication.

The governing quality profile contributes non-removable requirements such as:

- every publishable factual claim has acceptable evidence;
- the reviewer is independent of the drafter;
- publication consumes an approved artifact;
- the workflow stays within budget;
- unresolved critical findings block publication.

Ash generates structural, type, role, scheduling, and evidence obligations. Pure proof-time Ash may discharge data compatibility and graph properties. Z3 may solve actor allocation and budget constraints. Property tests may find malformed evidence cases. Lean may be used for a selected inductive invariant. Some evidence obligations remain until research and review tasks execute.

The LLM receives structured goals and counterexamples, revises only the affected definitions, and retries elaboration.

After static admission, the LLM may co-execute the program. It can communicate with sub-agents, but delegation is represented by typed Ash task boundaries rather than opaque conversation alone.

A sub-agent receives:

- a task contract;
- typed inputs;
- admitted operations and resources;
- local laws and obligations;
- required output and evidence types.

It returns a value together with provenance, attestations, diagnostics, and residual obligations. Ash determines whether the result satisfies the boundary.

New execution facts may discharge later obligations. If a task result violates a contract, Ash blocks dependent tasks and exposes a local repair scope.

### 8.2 Preventing self-certification

An LLM must not be the sole author and judge of its own quality bar.

Requirements should have an authority order such as:

```text
language invariants
  > admission and organization policy
  > user requirements
  > program-local laws
  > LLM-proposed strengthening
```

The LLM may add or strengthen local constraints. It must not silently weaken higher-authority requirements.

Likewise, an LLM assertion is not automatically proof. It may:

- write a pure decision procedure;
- construct a proof accepted by a kernel;
- request a solver through an admitted effect;
- produce review or heuristic evidence;
- propose a repair;
- leave a visible obligation.

The Ash checker and governing evidence policy decide what is sufficient.

## 9. Co-execution and staged discharge

Static acceptance does not imply that every property is known before execution. Some contracts depend on produced values.

An obligation may therefore declare a discharge window:

```text
created: after draft
required: before publish
```

Co-execution alternates between:

1. selecting tasks whose type, data-flow, authority, and prerequisite obligations permit execution;
2. executing those tasks through admitted handlers, actors, or sub-agents;
3. adding typed outputs, trace facts, and evidence;
4. running newly enabled proof-time Ash procedures;
5. admitting dependent tasks or reporting counterexamples;
6. repairing and re-elaborating the smallest affected region.

Proof-time computation and workflow execution remain distinct even when they interleave. Proof computation determines whether a gate may open; ordinary execution performs the governed work.

## 10. Minimal language core

This direction suggests a relatively small core commitment:

1. A general notion of proof-time evaluation boundary.
2. Computation-row admission for proof-time functions.
3. Proof-time access to types, kinds, rows, and selected static values.
4. Stratification that avoids unrestricted `Type : Type`.
5. Controlled static value-to-type lifting.
6. Explicit propositions, obligations, and residual goals.
7. Evidence types or evidence classifications.
8. Rules connecting evidence grades to obligation discharge.
9. Stable structured diagnostics and proof state.

The following can remain library-defined:

- Z3 and SMT encodings;
- Lean 4 translation;
- QuickCheck generators and shrinking;
- proof tactics and search strategies;
- workflow-specific theories;
- scheduling models;
- LLM prompting and repair strategies;
- provider-specific certificate handling.

## 11. Relationship to existing Ash directions

This proposal should be reconciled with, not substituted for:

- NOTE-020, for computation-row taxonomy;
- NOTE-028, for purity, evaluation modes, and contract timing;
- NOTE-030, for monadic Hoare logic and compositional obligations;
- NOTE-031 through NOTE-035, for predicate boundaries, lowering, capability separation, and temporal contracts;
- NOTE-036, for gradual verification and proof providers;
- NOTE-037, for symbolic-connectionist cooperation;
- NOTE-038, for type-level proofs, Π-types, and Dijkstra monads;
- DESIGN-034, for total type computation;
- SPEC-059 and SPEC-065, for sealed type-level domains and promoted data;
- SPEC-064, for constraint propositions;
- SPEC-080 and SPEC-081, for evidence and law-test substrate;
- SPEC-096b, for target computation rows and discharge;
- the contract/evidence, process/concurrency, and application/runtime plans.

This note adds a synthesis: use row-admitted Ash interpretation itself as the common proof-time programming surface, then expose proof providers and LLM assistance through typed libraries and governed execution boundaries.

## 12. Open questions

### Language and staging

1. Are types exposed directly as proof-time values, or through an explicit representation type with privileged eliminators?
2. Which kind and universe stratification is required?
3. Which static values are liftable, and how is liftability expressed?
4. Does the same evaluator operate at multiple semantic levels, or do surface forms elaborate into a separate proof-time Core?
5. What counts as definitional equality after proof-time computation?

### Termination and effects

6. Must every deductive proof-time function be total, or may bounded evaluation yield residual obligations?
7. Is termination checked structurally, sized, fuel-bounded, or through a total fragment?
8. Which row-item families are admissible at each proof-time boundary?
9. How are provider timeout, nondeterminism, versioning, and environment dependence recorded?
10. Can proof-time handlers themselves introduce obligations?

### Evidence and trust

11. What is the minimal trusted kernel?
12. Which external results require certificates?
13. How are translation correctness and provider versions represented in provenance?
14. Which evidence grades may discharge each category of law or contract?
15. How are stale proofs invalidated when laws, providers, roles, or source artifacts change?

### LLM development

16. What proof-state representation is easiest for both humans and LLMs?
17. How are non-removable quality profiles selected and versioned?
18. How does Ash prevent an LLM from weakening a requirement indirectly through a definition?
19. Which counterexample formats best support local workflow repair?
20. How are sub-agent conversations reduced to typed task results, evidence, and trace facts?
21. When may an LLM-generated proof procedure itself enter the trusted path?

## 13. Suggested experimental path

A small prototype can test the idea without committing Ash to full dependent types.

### Experiment A: pure structural proof-time Ash

Expose computation rows and workflow graph descriptions as proof-time values. Implement total Ash functions for:

- row inclusion;
- task input/output compatibility;
- required-output reachability;
- DAG acyclicity with topological-order certificates;
- role separation checks.

Success criterion: the same Ash function/library surface can produce proofs or counterexamples with source-linked diagnostics.

### Experiment B: controlled promotion

Allow a small sealed domain of static natural numbers, symbols, finite sets, and graph values to participate in type-level constraints.

Success criterion: promotion is predictable, normalizing, and does not require unrestricted value dependency.

### Experiment C: one provider effect

Implement a library-defined Z3 interface for finite scheduling or resource constraints.

Success criterion: Z3 remains outside language semantics; Ash rows control access; models and unknown results remain explicit; certificate/trust policy is visible.

### Experiment D: LLM elaboration loop

Give an LLM a workflow prompt and a fixed quality profile. Let it:

1. generate an incomplete Ash program;
2. receive typed obligations and counterexamples;
3. fill proof holes or repair workflow composition;
4. reach static admission without weakening the profile.

Success criterion: progress is measured by discharged obligations and preserved laws, not by textual self-evaluation.

### Experiment E: typed co-execution

Execute admitted tasks and delegate one task to a sub-agent. Require a typed result with provenance and a residual evidence obligation before a dependent task becomes ready.

Success criterion: the runtime gates execution from Ash facts and evidence rather than from the coordinator LLM's assertion.

## 14. Provisional principles

The exploration currently suggests these principles:

1. **Use Ash as the proof-time programming language.**
2. **Use rows to isolate what proof-time computation may do.**
3. **Use evidence types or grades to state what a result may establish.**
4. **Expose types and kinds at proof time without collapsing the universe hierarchy.**
5. **Lift only explicitly admissible static values.**
6. **Keep proof providers library-defined and their trust explicit.**
7. **Preserve unknown and residual obligations; do not equate them with false.**
8. **Allow execution to add evidence, never to weaken laws.**
9. **Treat LLMs as elaborators, proof searchers, and workflow actors—not self-certifying authorities.**
10. **Make counterexamples and local proof state first-class interfaces for repair.**

The strongest claim of this note is not that Ash should immediately become a full dependent theorem prover. It is that Ash already has enough of the necessary structure to explore a smaller, coherent step:

> an effect-controlled, staged Ash interpreter that evaluates Ash laws over proof-time types, kinds, rows, and selected static values, while external proving and testing systems remain typed libraries.

That step would make “Ash proving Ash” a practical language architecture rather than only a metaphor.
