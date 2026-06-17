# Protocol-gated type-directed LLM execution

**Status:** research note / idea seed  
**Date:** 2026-06-16  
**Use:** research expansion, brainstorming, agent handoff, and future Ash planning

## Elevator pitch

Large language models are good at proposing code, decompositions, types, tests, and proofs. They are not reliable judges of whether those proposals are correct.

Protocol-gated type-directed LLM execution treats the LLM as a proposal source inside a deterministic development protocol. The protocol owns the state machine, transition rules, oracles, evidence admission, and trace. The LLM can suggest the next move, but only the controller can advance the semantic state.

For coding-style problems, this gives the system real teeth. A parser, resolver, type checker, test runner, property runner, proof checker, or protocol checker can serve as a deterministic oracle. The LLM may be probabilistic, but each accepted transition is backed by checkable evidence.

The goal is not to make the model smarter by prompt style alone. The goal is to make the model useful under a protocol that can reject bad moves, preserve good evidence, and replay why progress was admitted.

## The idea in one paragraph

Constrain LLM-assisted development to a typed protocol. The controller decomposes a task into explicit states such as `ProblemStated`, `GoalGraphAdmitted`, `TypesAdmitted`, `ImplementationTypeChecked`, `PropertiesChecked`, and `Verified`. At each state, the LLM may propose only the moves that the protocol allows. A deterministic transition checker validates the proposal with one or more oracles, records the evidence, and either admits the transition or keeps the protocol in the previous valid state with a structured rejection trace.

## Why this is worth exploring

Most coding agents blend three roles that should be separate:

1. **Proposer** — invent candidate code, types, tests, repairs, or decompositions.
2. **Judge** — decide whether the candidate is correct.
3. **Historian** — remember what was checked and why it was accepted.

LLMs are useful proposers. They are weak judges and unreliable historians. This note proposes making the judge and historian deterministic.

For Ash, the useful core is not mainly the semantic tower. The tower and effect lattice remain useful internal structure, especially for effect and authority boundaries, but they are not the source of reliability. The bite comes from:

- explicit types;
- explicit state machines and protocols;
- deterministic transition checking;
- parser/resolver/type/test/property/proof oracles;
- evidence admission;
- replayable traces.

## What this is not

This is not “ask the LLM to think step by step.”

This is also not “wrap the LLM in a state-machine metaphor” while still accepting free-form self-certification. A protocol-gated loop must reject a proposal when the deterministic checker cannot admit it.

This is closer to:

```text
LLM proposes a move.
Controller checks the move.
Oracle produces evidence.
Protocol admits or rejects the transition.
Trace records the result.
```

The accepted artifact is not just code. It is code plus evidence plus the transition that admitted it.

## Core architecture

### Roles

| Role | Responsibility |
| --- | --- |
| LLM | Proposes decompositions, types, functions, workflows, tests, laws, proofs, and repairs. |
| Controller | Owns the authoritative state machine and legal transition relation. |
| Transition checker | Validates that a proposed move is legal from the current state. |
| Oracle | Runs deterministic checks such as parse, resolve, typecheck, test, property, law, proof, or protocol validation. |
| Evidence store | Records accepted oracle output, rejected proposals, dependency hashes, and replay data. |
| Trace | Preserves the ordered history of proposals, checks, admissions, rejections, and state changes. |

The LLM can also serve as a **reviewer** or **judge** for subjective, aesthetic, or underspecified criteria, but only when its judgment is recorded as a reviewable proposal. A judgment without reviewability is not a valid transition.

### Layered architecture sketch

The protocol-gated system can be viewed as several layers with structurally similar but non-overlapping concerns:

```text
Layer A: Task analysis and prompt preparation
  Input: raw task description + project context
  Output: sanitized LLM prompts + structured context + protocol state initialization
  Mechanism: RLM-style analysis with type discipline

Layer B: Proposal generation (proposer LLM)
  Input: sanitized prompt + context + current protocol state
  Output: candidate artifacts (code, types, tests, constraints)
  Mechanism: constrained or unconstrained generation, possibly grammar-gated

Layer C: Proposal validation (judge LLM + deterministic oracles)
  Input: candidate artifact + admitted constraints + current protocol state
  Output: judgment + evidence + transition decision
  Mechanism: deterministic oracles first, LLM review for subjective gaps, evidence recording

Layer D: State advancement and trace
  Input: transition decision + evidence
  Output: updated protocol state + recorded trace
  Mechanism: controller enforces state machine, evidence store persists, trace logs history

Layer E: Recursive decomposition and subagent dispatch
  Input: complex task that requires subproblems
  Output: subagent prompts + context + budget + termination conditions
  Mechanism: RLM-style recursive calls with protocol-gated subagent boundaries
```

Each layer has its own engineering concerns and tradeoffs. Layer A is about context management and prompt hygiene. Layer B is about generation quality and constraint enforcement. Layer C is about validation correctness and evidence quality. Layer D is about state integrity and replayability. Layer E is about recursive control and resource limits.

The layers are not strictly sequential. A recursive subagent (Layer E) may trigger a new Layer A analysis for its subproblem. A failed validation (Layer C) may return to Layer B with repair guidance. A state advancement (Layer D) may trigger a new constraint specification (Layer A) for the next phase.

### State is authoritative

The LLM may keep scratch notes, summaries, candidate plans, and subcall outputs. None of that is authoritative.

The controller owns the semantic state:

```text
ProblemStated
GoalGraphProposed
GoalGraphAdmitted
TypesProposed
TypesAdmitted
SignaturesProposed
SignaturesAdmitted
ObligationsGenerated
ImplementationProposed
ImplementationTypeChecked
PropertiesAttached
PropertiesChecked
ProofsAdmitted
PatchIntegrated
Verified
Rejected
Blocked
```

These names are illustrative. A real implementation should choose states by the artifacts and oracles it can actually check.

### Constraint specification is a first-class state

For a task like "implement a function satisfying this spec," the protocol must not assume the constraint is already given. The constraint specification itself must be proposed, checked, and admitted:

```text
ProblemStated
  -> ConstraintSpecProposed        (LLM proposes the formal constraint)
  -> ConstraintSpecAdmitted        (oracle checks the spec is well-formed and matches intent)
  -> ImplementationProposed
  -> ImplementationTypeChecked
  -> ConstraintSatisfied          (oracle checks implementation against admitted spec)
  -> Verified
```

This matters because a bad constraint is as dangerous as a bad implementation. The LLM may propose a constraint that is vacuous, inconsistent, or misaligned with the user's intent. The protocol must admit the constraint before it can be used to judge implementations.

For Ash programs, this is especially hard. Specifying constraints for an Ash function involves:
- type signatures (checkable by the type checker);
- effect annotations (checkable by the effect checker);
- capability requirements (checkable by the capability checker);
- preconditions and postconditions (`requires`, `ensures` — checkable by contract checker or test);
- algebraic laws (checkable by property runner or proof checker);
- behavioral invariants (harder — may require human review or model checking).

The protocol should treat each of these as a separate constraint-admission step, not bundle them into one opaque "spec" transition.

### Transitions are admitted, not asserted

A transition should be represented as data, not as an English claim.

Illustrative schema:

```text
TransitionProposal {
  from_state: StateId,
  kind: TransitionKind,
  artifact: ArtifactId,
  claims: List<Claim>,
  required_checks: List<Check>,
}
```

The controller evaluates it:

```text
TransitionDecision =
  Accepted {
    to_state: StateId,
    evidence: List<EvidenceId>,
    state_patch: StatePatch,
  }
| Rejected {
    stays_at: StateId,
    reason: RejectionReason,
    diagnostics: List<Diagnostic>,
  }
```

A failed proposal is still useful. It becomes trace data for repair, comparison, and future agent handoff.

## Deterministic oracles

The architecture should be oracle-polymorphic. The type checker is the most important early oracle, but not the only one.

| Oracle | What it can admit |
| --- | --- |
| Parser | The proposed artifact is syntactically valid. |
| Resolver | Names, imports, modules, and referenced artifacts resolve. |
| Type checker | The artifact inhabits the claimed type under the current context. |
| Protocol checker | The proposed move is legal from the current protocol state. |
| Effect/capability checker | The artifact does not widen authority or use unavailable effects. |
| Test runner | Concrete examples pass. |
| Property runner | Generated cases satisfy declared properties. |
| Law checker | Algebraic or semantic laws hold for admitted evidence mode. |
| Proof checker | A proof term, solver certificate, or proof artifact is valid. |
| Trace/replay checker | The accepted result can be reproduced from recorded context. |

The type checker provides the first strong boundary:

```text
Given Γ and expected type T,
accept candidate e only if Γ ⊢ e : T.
```

Tests, properties, laws, and future proof types strengthen that boundary.

## Example: type-directed synthesis loop

The user asks for a function or workflow. The controller does not ask the LLM to “write the feature” in one step. It creates a protocol run.

```text
1. ProblemStated
2. GoalGraphProposed
3. GoalGraphAdmitted
4. TypesProposed
5. TypesAdmitted
6. SignaturesProposed
7. SignaturesAdmitted
8. ObligationsGenerated
9. ImplementationProposed
10. ImplementationTypeChecked
11. PropertiesChecked
12. PatchIntegrated
13. Verified
```

At each stage, the LLM proposes the next artifact. The controller decides whether the move is legal and which oracle must check it.

For example:

```text
State: SignaturesAdmitted
Allowed move: propose implementation for goal G
Required oracle: parser + resolver + type checker
Success: ImplementationTypeChecked
Failure: stay at SignaturesAdmitted with diagnostics
```

The LLM can repair from diagnostics, but it cannot advance the state by saying the repair is correct.

## Example: property-backed development

Suppose the admitted signature is:

```text
normalize : Expr -> Expr
```

The controller may generate or request obligations such as:

```text
idempotence:
  normalize(normalize(e)) == normalize(e)

preservation:
  type_of(normalize(e)) == type_of(e)
```

The LLM can propose the implementation and the property harness. The controller admits progress only when the deterministic checks pass:

```text
parse implementation
resolve referenced names
typecheck normalize : Expr -> Expr
run examples
run generated properties
record counterexample or pass evidence
```

If a generated property fails, the protocol does not become confused. It records a rejected transition with the counterexample and returns to a repair state.

## Example: proof-producing synthesis

Future proof types can make the same pattern stronger.

Illustrative notation only:

```text
synthesize normalize :
  (e : Expr) -> { e2 : Expr | Normal(e2) && TypeOf(e2) = TypeOf(e) }
```

A proposal is not admitted because the LLM explains why it should work. It is admitted only if the proof oracle accepts the evidence:

```text
candidate term parses
candidate term typechecks
proof artifact checks
obligation coverage is complete
trace records artifact hashes and checker version
```

The proof may come from a proof term, a solver certificate, a bounded symbolic executor, or another future evidence family. The protocol should treat each as a distinct oracle with explicit trust boundaries.

## Example: protocol-gated patch integration

A patch should not move directly from “LLM generated a diff” to “done.”

A safer protocol is:

```text
PatchProposed
  -> PatchApplies
  -> CodeParses
  -> CodeTypeChecks
  -> FocusedTestsPass
  -> PropertiesPass
  -> BroadGateReconciled
  -> Integrated
```

Each transition has its own evidence. If focused tests pass but a property fails, the patch remains useful but not integrated.

## Relationship to Recursive Language Models

The Recursive Language Models paper is highly relevant as an execution substrate. It treats the prompt as external state, gives the model programmatic handles into that state, and allows recursive subcalls over selected slices. That is a good operational shape for long-horizon development.

The key change for protocol-gated development is to replace an open-ended REPL with a typed transition API.

RLM-style execution:

```text
model writes code
code inspects prompt variable
code launches recursive subcalls
model sets Final
```

Protocol-gated execution:

```text
model proposes typed move
controller checks legal transition
oracle admits evidence
controller mutates semantic state
terminal state permits final answer
```

RLMs show that external state and programmatic recursive decomposition matter. Protocol gating adds deterministic authority over state advancement.

## Relationship to state-machine generation work

The UML state-machine generation paper is less directly an implementation substrate, but it is useful motivation. It shows that LLMs can propose useful state-machine structure from natural language, while still struggling with guards, actions, parallel regions, and history states.

That failure mode supports the protocol-gated design. Guards, actions, and history are exactly the pieces that should be checked by a deterministic protocol rather than trusted as generated prose.

The paper also shows that prompt decomposition helps some models and hurts others. That is a warning: the controller should constrain semantic progress, not micromanage every token of reasoning.

## Research questions to expand

This note should seed search and brainstorming around several research clusters.

### State-machine and protocol control

- typed state machines for agent execution;
- protocol languages for tool-using agents;
- session types and typestate as agent-control models;
- automata-guided program synthesis;
- runtime monitors and shielded execution;
- constrained decoding versus constrained transition admission.

### Type-directed synthesis

- type inhabitation and proof search;
- holes/metavariables and typed repair loops;
- compiler diagnostics as synthesis guidance;
- type-directed program generation;
- proof-producing synthesis;
- refinement types and liquid types as oracle surfaces.

### Evidence and oracle design

- proof-carrying code;
- certified compilation;
- solver certificates;
- property-based testing as empirical evidence;
- test-generation traces and counterexample shrinking;
- reproducible build/test/proof traces.

### Agent and long-context scaffolds

- recursive language models;
- LLM subagent delegation;
- external memory and scratchpad environments;
- context offloading;
- tool-use traces;
- deterministic controllers for probabilistic agents.

## Design principles

1. **The LLM proposes; the protocol disposes.**
   The model can suggest a move, but only the controller can admit it.

2. **No self-certifying transitions.**
   A transition must cite oracle evidence, not only model explanation.

3. **Keep semantic state outside the model.**
   The model may summarize; the controller stores authoritative state.

4. **Failures are first-class trace events.**
   Rejections, counterexamples, parse errors, type errors, and proof failures are useful data.

5. **Use the strongest cheap oracle first.**
   Parse and typecheck before running expensive tests or proof search.

6. **Separate artifact generation from evidence admission.**
   Code, tests, laws, and proofs become meaningful only when admitted with evidence.

7. **Constrain progress, not creativity.**
   The LLM can explore freely inside proposal states. It cannot mutate verified state without passing gates.

8. **The LLM can judge, but its judgments must be reviewable.**
   The LLM can act as a judge for subjective, aesthetic, or underspecified criteria, but only when the judgment is recorded as a proposal that can be reviewed, challenged, or overridden by a stronger oracle or a human. The LLM's judgment is not self-certifying.

9. **Constraint specification is itself a protocol step.**
   For a task like "implement a function satisfying this spec," the protocol must first admit the constraint specification, then verify that the proposed implementation satisfies it. Constraint specification is not a given; it is a state transition with its own oracle requirements.

## Open design choices

- What is the minimal transition schema that is useful without becoming heavy?
- Should protocols be authored manually, generated from task type, or inferred from repository conventions?
- How much of the goal graph should require human approval before synthesis begins?
- Which evidence kinds are stable enough for first-class admission?
- How should traces identify model calls, tool calls, dependency versions, source hashes, and oracle versions?
- When a property/test/proof fails, should the protocol return to the previous state or enter a distinct repair state?
- How should recursive subagents be budgeted and prevented from over-producing low-quality candidates?
- What is the smallest Ash slice that can demonstrate this loop without inventing future proof infrastructure?

## Candidate first slice for Ash

A practical first implementation should be small and honest:

```text
Input:
  expected type/signature + local context + optional tests

Protocol:
  ProblemStated
  -> SignatureAdmitted
  -> CandidateProposed
  -> CandidateParses
  -> CandidateTypeChecks
  -> FocusedTestsPass
  -> Verified

Oracles:
  parser, resolver, type checker, focused test runner

Trace:
  proposal text, patch hash, diagnostics, command output, accepted evidence
```

This first slice would not claim full semantic correctness. It would prove that protocol-gated admission is workable and gives better handoff data than ordinary agent loops.

## References

- Alex L. Zhang, Tim Kraska, and Omar Khattab. “Recursive Language Models.” arXiv:2512.24601v3, 2026. <https://arxiv.org/abs/2512.24601v3>. The paper motivates external prompt state, programmatic decomposition, recursive subcalls, and context offloading as an inference-time scaffold.
- Samer Abdulkarim, Evan Boyd, Karl Bridi, Alec Tufenkjian, Boqi Chen, and Gunter Mussbacher. "Structure- and Event-Driven Frameworks for State Machine Modeling with Large Language Models." arXiv:2604.00275v1, 2026. <https://arxiv.org/abs/2604.00275v1>. The paper is useful motivation for why LLM-generated state-machine structure needs deterministic validation, especially around guards, actions, and advanced state-machine features.
- Brandon T. Willard and Rémi Louf. "Efficient Guided Generation for Large Language Models." arXiv:2307.09702, 2023. <https://arxiv.org/abs/2307.09702>. Outlines library: grammar-constrained LLM decoding via finite-state automata, regex, and context-free grammars. Model-agnostic structured generation.
- Terry Koo, Frederick Liu, and Luheng He. "Automata-based constraints for language model decoding." COLM 2024. <https://openreview.net/forum?id=BDBdblmyzY>. Provably correct automata-based constrained decoding with ~7,000x faster compilation than prior work, extended to deterministic context-free languages.
- Niels Mündler, Jingxuan He, Hao Wang, Koushik Sen, Dawn Song, and Martin Vechev. "Type-Constrained Code Generation with Language Models." Proc. ACM Program. Lang. 9, PLDI, Article 171 (June 2025), 26 pages. <https://doi.org/10.1145/3729274>. Type-constrained decoding using prefix automata and type inhabitation search, reducing compilation errors by more than half on HumanEval and MBPP.

## Related work and implementation landscape

This section maps the broader research and implementation landscape onto the protocol-gated architecture. The goal is not to survey every project, but to identify which pieces provide useful primitives, which provide cautionary lessons, and where the gaps remain.

### Outlines: grammar-constrained decoding

Outlines is a practical library for structured LLM generation. It compiles regular expressions, JSON schemas, and context-free grammars into finite-state automata that constrain the model's token sampling at each step. This guarantees that the generated output conforms to the specified grammar.

Key capabilities:
- Regex-constrained generation: `model(prompt, regex_pattern)`
- JSON schema generation: `model(prompt, json_schema)`
- Pydantic model generation: `model(prompt, MyPydanticModel)`
- Context-free grammar support
- Works across model providers (OpenAI, vLLM, Ollama, etc.)

What Outlines provides for protocol-gated execution:

**1. A deterministic generation boundary, not just a post-hoc check.**
Outlines constrains the model *during* generation, not after. This is stronger than "generate then validate" because it prevents invalid structures from ever being produced. For a protocol-gated loop, this means the LLM's raw output can be trusted to parse correctly, reducing the oracle load.

**2. Grammar-as-protocol for proposal shapes.**
A protocol state can define not just "what moves are legal" but "what syntax the proposal must have." For example, a `TypesProposed` state could require the LLM to emit output conforming to a grammar of valid type declarations. Outlines makes that enforceable at the token level.

**3. Structured evidence formats.**
When the LLM must produce evidence (test cases, property specifications, proof sketches), Outlines can enforce the structure of that evidence. A JSON schema for test metadata, a regex for law propositions, or a grammar for proof terms all become admissible constraints.

Limitations and gaps:
- Outlines handles syntax and schema, not semantics. It guarantees well-formed JSON, not that the JSON represents a valid type.
- It does not model type systems beyond what can be expressed in a grammar. The type-constrained decoding paper addresses this gap.
- It does not provide state-machine execution, transition checking, or evidence admission. It is a generation primitive, not a protocol controller.
- It does not track traces, manage multi-step protocols, or handle rejection and repair loops.

For Ash, Outlines-style primitives could be useful inside the proposal layer: when the LLM emits a candidate type, function, or test, the system could enforce that the emission conforms to a grammar. But the semantic admission still requires the type checker, test runner, or other oracle.

### LangGraph: stateful agent orchestration

LangGraph is a low-level orchestration framework for building stateful, long-running agents. It models execution as a graph of nodes and edges, where each node is a computation step and edges define transitions. It supports cycles, branching, human-in-the-loop, memory, and persistence.

Key capabilities:
- Graph-based execution: nodes (functions) + edges (transitions)
- Stateful execution: shared state object passed between nodes
- Cycles and branching: not just DAGs
- Human-in-the-loop: interrupts for inspection/modification
- Persistence: checkpoint and resume execution
- Subgraphs: nested graph composition

What LangGraph provides for protocol-gated execution:

**1. A practical graph execution substrate.**
LangGraph shows that graph-based agent execution is workable at scale. The protocol-gated architecture can adopt a similar graph model, but with stricter transition semantics. Instead of "any node can run next if the edge condition is true," the protocol would use "only transitions with oracle evidence are admitted."

**2. Stateful execution with external state.**
LangGraph's shared state object is analogous to the controller's semantic state. The key difference is that LangGraph state is typically a Python dict or Pydantic object that nodes can mutate freely. Protocol-gated execution would require that state mutations pass through the transition checker.

**3. Human-in-the-loop as a protocol feature.**
LangGraph's interrupt mechanism is a useful pattern. In protocol-gated execution, human approval could be modeled as an oracle: a transition requiring human sign-off does not advance until the human oracle emits evidence.

**4. Persistence and replay.**
LangGraph's checkpointing is useful, but protocol-gated execution needs stronger replay semantics: not just "resume from here" but "reproduce exactly why this transition was admitted" with full evidence.

Limitations and gaps:
- LangGraph edges are typically Python conditions, not formal protocol checks. A node can mutate state in ways that are not externally validated.
- There is no built-in oracle layer. Nodes call tools, but there is no systematic evidence admission framework.
- The state is not typed in a way that enforces protocol invariants. A node can add arbitrary keys to the state dict.
- There is no built-in trace of rejections. Failed paths are not first-class data.
- LangGraph is Python-specific and LangChain-centric. The concepts are portable, but the implementation is not.

For Ash, LangGraph's graph model is a useful reference for the execution substrate, but the transition semantics need to be much stricter. The protocol should not be "nodes and edges" but "states, legal moves, oracles, and evidence."

### Automata-based constrained decoding (Koo et al., COLM 2024)

This paper provides a rigorous theoretical foundation for constrained LLM decoding using automata theory. It reformulates the entire problem in terms of finite-state automata (FSAs) and finite-state transducers (FSTs), solving tokenization alignment problems through transduction.

Key contributions:
- Detokenization as FST transduction: the connection between tokens and characters is modeled as a transducer
- Closed-form solutions for regular languages via FSA operations
- Extensions to deterministic context-free languages (DPDAs)
- ~7,000x faster constraint compilation than prior work (Outlines)
- Provably correct, modular extensibility

What this provides for protocol-gated execution:

**1. Faster, more reliable grammar constraints.**
The 7,000x speedup matters for interactive protocols. If the LLM proposes a move and the protocol must compile a new grammar constraint for the next proposal, slow compilation becomes a bottleneck. Fast constraint compilation makes grammar-gated proposals practical.

**2. Context-free language support.**
Regular expressions are insufficient for many programming language constructs. The extension to deterministic context-free languages means that grammar constraints for code generation can be more expressive. This is directly relevant for enforcing that LLM-generated code conforms to language syntax.

**3. Tokenization alignment as a solved problem.**
The FST-based detokenization approach elegantly handles the mismatch between subword tokenizers and formal language tokens. This means grammar constraints can be specified at the character/lexer level and automatically compiled to token-level masks.

**4. Modularity and correctness.**
The paper's reformulation in terms of standard automata operations means the approach is extensible and verifiable. New constraints can be added by composing standard automata operations, not by writing bespoke token-masking code.

Limitations and gaps:
- Like Outlines, this handles syntax and formal languages, not semantics. Type checking, name resolution, and semantic validation are outside the scope.
- It does not address multi-step protocols, state machines, or evidence admission.
- It is a generation-time primitive, not an architectural framework for agent control.

For Ash, this paper provides the theoretical foundation for a fast, correct grammar-constrained proposal layer. The protocol could use these techniques to enforce that LLM outputs conform to Ash surface syntax, but semantic admission still requires the Ash parser, resolver, and type checker.

### Type-constrained code generation (Mündler et al., PLDI 2025)

This is the strongest paper for the protocol-gated architecture. It goes beyond syntax constraints to type constraints, using prefix automata and type inhabitation search to guide LLM code generation.

Key contributions:
- Prefix automata for type-constrained decoding: novel automata that track type information during generation
- Type inhabitation search: search over inhabitable types to guide the model toward well-typed completions
- Sound approach for a foundational simply-typed language
- Extension to TypeScript with practical evaluation
- Reduces compilation errors by 74.8% (HumanEval) and 56.0% (MBPP)
- Significantly increases functional correctness across synthesis, translation, and repair tasks

What this provides for protocol-gated execution:

**1. The strongest oracle primitive: type checking during generation.**
This is the closest existing work to "deterministic oracle-guided LLM execution." The type checker is not just a post-hoc validator; it actively guides the model's sampling by rejecting tokens that would lead to type errors. This is exactly the kind of oracle that should sit at the core of a protocol-gated loop.

**2. Type inhabitation as a synthesis primitive.**
The search over inhabitable types is a form of type-directed synthesis. For a protocol-gated loop, this suggests that the controller can not only check types but also guide the LLM toward type-correct completions by enumerating or suggesting inhabitable types.

**3. Repair as a protocol transition.**
The paper evaluates repair tasks (fixing non-compiling code). In protocol terms, this is a transition from `ImplementationTypeChecked` (failed) to `ImplementationTypeChecked` (repaired), with the type checker guiding the repair.

**4. Evidence that type constraints beat syntax constraints.**
The paper's key finding is that syntax errors are only 6% of compilation errors; type errors are 94%. This strongly supports the protocol-gated emphasis on type checking as the primary oracle, not just parsing.

Limitations and gaps:
- The approach is specialized to code generation and type systems. It does not generalize to arbitrary protocols, test validation, property checking, or proof admission.
- It does not provide a state machine or transition framework. It is a generation primitive, not a protocol controller.
- It does not track evidence, traces, or multi-step development workflows.
- The TypeScript extension is practical but not fully general. Higher-order types and some advanced features are not fully covered.

For Ash, this paper is the strongest evidence that type-constrained LLM execution is both theoretically sound and practically effective. It should be a primary reference for designing the type-checker-as-oracle component of the protocol.

### Synthesis: how these pieces fit together

The four additional sources complement the original two papers in a layered architecture:

```text
Layer 1: Generation-time constraints (what the LLM can emit)
  - Outlines: grammar/schema constraints via FSA
  - Koo et al.: fast, correct automata-based constraint compilation
  - Mündler et al.: type-constrained decoding via prefix automata

Layer 2: Execution substrate (how proposals are managed)
  - RLM paper: external state, recursive decomposition, context offloading
  - LangGraph: graph-based stateful execution, persistence, human-in-the-loop

Layer 3: Protocol authority (what transitions are legal)
  - Protocol-gated architecture: state machine, transition checker, oracles
  - UML generation paper: motivation for why generated structure needs validation

Layer 4: Evidence and trace (why progress was admitted)
  - Protocol-gated architecture: evidence store, replay, rejection history
```

The gap that Ash could fill is Layer 3 and 4, with strong Layer 1 integration. Most existing work stops at Layer 1 or 2. No existing system combines:
- type-constrained generation (Layer 1)
- external state and recursive decomposition (Layer 2)
- formal protocol with deterministic transition checking (Layer 3)
- evidence admission and replayable traces (Layer 4)

### What Ash should borrow, what it should build

**Borrow directly:**
- Outlines-style grammar constraints for proposal syntax (adapt to Ash surface syntax)
- Automata-based constraint compilation for fast, correct grammar gates (Koo et al.)
- Type-constrained decoding primitives for type-directed synthesis (Mündler et al.)
- LangGraph-style graph execution for the controller substrate (but with stricter transitions)
- RLM-style external state and recursive subcalls for long-horizon tasks

**Build specifically:**
- Typed protocol definition language for Ash development workflows
- Transition checker that integrates parser, resolver, type checker, test runner, property runner
- Evidence store with artifact hashes, oracle versions, and replay data
- Trace format for proposal/rejection/admission history
- Budget and termination controls for recursive subagent calls
- Human-in-the-loop as an explicit oracle with structured approval evidence

**Avoid:**
- LangGraph's free-form state mutation (replace with typed state patches)
- Outlines' limitation to syntax-only (extend with semantic oracles)
- RLM's open-ended REPL (replace with typed transition API)
- UML generation paper's prompt-only decomposition (replace with protocol-gated decomposition)

### New research directions opened by these sources

1. **Grammar-gated proposal syntax.**
   Can Ash surface syntax be compiled into an FSA constraint for LLM proposals? This would guarantee that LLM-generated Ash code is syntactically valid before parsing, reducing parser error handling and improving proposal quality.

2. **Type inhabitation as synthesis guidance.**
   Can the Ash type checker enumerate inhabitable types for a given goal, guiding the LLM toward type-correct completions? This would make the type checker an active synthesis oracle, not just a passive validator.

3. **Fast constraint compilation for interactive protocols.**
   The 7,000x speedup from Koo et al. suggests that interactive protocol steps (where the LLM proposes and the system must quickly compile a new constraint) are feasible. Can Ash's grammar constraints be compiled with similar efficiency?

4. **Typed protocol composition.**
   LangGraph shows that subgraphs compose. Can Ash protocols compose? A protocol for "generate a function" could be a subgraph of a protocol for "generate a module," with shared oracles and transition patterns.

5. **Evidence formats for grammar/type/test/proof oracles.**
   Each oracle produces different evidence. Can Ash define a unified evidence format that accommodates parser output, type judgments, test results, property traces, and proof certificates?

6. **Recursive subagent budget and quality control.**
   RLMs and LangGraph both support recursive calls, but neither provides strong budget or quality controls. Can the protocol define subagent budgets, minimum evidence thresholds, and automatic rejection of low-quality recursive proposals?

7. **Human oracle integration.**
   LangGraph's human-in-the-loop is a pattern; the protocol should make it a first-class oracle. Can human approvals be recorded as evidence with the same structure as automated oracle evidence?

8. **Layered prompt preparation and context sanitization.**
   Can the task analysis layer (Layer A) be formalized as a type-disciplined context extraction protocol? This would involve: parsing the raw task, extracting relevant code context, sanitizing sensitive information, and producing structured prompts with explicit protocol state initialization.

9. **Proposer-judge separation with shared context.**
   How should the proposer LLM (Layer B) and judge LLM (Layer C) share context without leaking privileged information or creating circular dependencies? Can the protocol define a shared context schema that both layers consume but neither controls?

10. **Recursive subagent protocol inheritance.**
    When a recursive subagent is dispatched (Layer E), how much of the parent protocol state should be inherited? Can the protocol define a scoped inheritance model where subagents receive a constrained view of the parent state, with their own local protocol states that must be merged on completion?

### Updated open design choices

- How much of Ash surface syntax can be compiled into FSA constraints for proposal generation?
- Can the Ash type checker be extended with type inhabitation search for synthesis guidance?
- What is the minimal protocol schema that supports composition (subgraphs) without becoming heavy?
- How should grammar constraints, type constraints, and semantic oracles be ordered in the oracle pipeline?
- Should the LLM generate raw text and then be constrained, or should constraints be compiled before generation?
- How can recursive subagent calls be budgeted and quality-gated within the protocol?
- What evidence format can unify parser, type, test, property, law, and proof oracle outputs?
- How should human approval be integrated as a first-class oracle with structured evidence?
- What is the smallest end-to-end Ash demonstration that combines grammar constraint + type check + test pass + trace recording?

### Constraint specification and verification

- How should the LLM propose formal constraints for an Ash task, and what oracles check that the constraint is well-formed and matches intent?
- Should constraint specification be a separate protocol subgraph that can be reused across tasks?
- How can the protocol verify that a proposed `requires`/`ensures` contract is not vacuous or inconsistent?
- How should algebraic laws be proposed, checked for well-formedness, and admitted before they are used to judge implementations?
- What is the oracle for checking that a behavioral invariant is actually enforced by the implementation?
- Can the LLM's judgment on subjective constraints (e.g., "this API is ergonomic") be recorded as reviewable evidence?
- How should constraint drift be detected when the implementation evolves but the admitted constraint does not?
