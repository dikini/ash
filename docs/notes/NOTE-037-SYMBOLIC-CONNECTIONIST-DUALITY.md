> # NOTE-037: Ash as a Symbolic-Connectionist Duality

**Date:** 2026-07-07
**Status:** Living document — design thesis and architectural guide
**Purpose:** Frame Ash as a cooperative dual system: a **symbolic** core
(deterministic, type-driven, provable) and a **connectionist** companion
(non-deterministic, heuristic, LLM-based). Define how the two sides interact
through the compiler, effect rows, evidence rows, and explicit orchestration.
This note is broader than proof providers; it applies to code generation,
diagnostics, specification, and verification.

Companion to NOTE-036 (gradual verification and proof providers), SPEC-096b
(target effect system), the `std::llm` module, NOTE-020 (computation row
taxonomy), NOTE-030 (monadic Hoare composition), and NOTE-035 (temporal
contracts).

## 0. Motivation

Most programming languages treat LLMs as external API clients. Ash is different:
its design already contains the scaffolding for a deeper integration:

- `requires` / `ensures` contracts are symbolic predicates that can also be
  read as intent by a connectionist system.
- `law` / `proof` bodies are explicit evidence obligations that can be
  discharged by either a prover or a test — or suggested by an LLM and checked
  by a prover.
- Effect rows track what a computation requires; LLM calls are non-deterministic
  operations that should appear in rows.
- Evidence rows record not only *that* an obligation was discharged, but *how*.

This note makes the implicit design thesis explicit: **Ash is a
symbolic-connectionist hybrid language.** The symbolic side guarantees
fail-closed correctness. The connectionist side provides heuristic power. The
compiler orchestrates both.

## 1. The dual-system thesis

Every significant Ash computation can be viewed through two lenses:

| Symbolic | Connectionist |
|---|---|
| Deterministic | Non-deterministic |
| Type-driven | Pattern-driven |
| Verifiable by proof | Suggestible by model |
| Exact effect rows | Heuristic intent |
| SMT / Lean | LLM |
| `verified` / `refuted` | `suggested` / `assisted` |
| Compile-time discharge | Runtime / interactive assistance |

Neither side is supreme. The symbolic side is fail-closed but brittle. The
connectionist side is flexible but unreliable. Ash's value is in making their
interaction explicit and auditable.

## 2. Ash constructs mapped to the duality

| Ash construct | Symbolic role | Connectionist role |
|---|---|---|
| Type system | Enforces invariants | Provides schema for prompts |
| `requires` / `ensures` | Provable predicates | Expressible intent |
| `law` | Axioms / theorems | Synthesis targets |
| `proof` body | Evidence family | Assistance mode |
| Effect rows | Authority/requirement tracking | LLM operation tracking |
| Evidence rows | Prover verdicts | LLM invocation records |
| `deferred` | Unknown symbolic proof | Open heuristic target |
| Module summaries | Reproducible contracts | Model prompt fingerprints |

The `std::llm` module already occupies the connectionist side: pure prompt
types, dispatch workflows, tool use, and conversation orchestration. The
missing piece is formal integration with the symbolic side through effect rows
and evidence rows.

## 3. The compiler as orchestrator

The Ash compiler does not "believe" either side. It coordinates them:

```text
1. Generate obligation (symbolic)
2. Try prover discharge (symbolic)
3. If deferred, ask LLM for suggestion (connectionist)
4. Send suggestion to prover (symbolic check)
5. Record combined evidence
6. Decide: erase / dynamic-check / monitor / fail
```

This pattern generalizes beyond proofs:

```text
code generation:  type constraints + LLM snippet + parser/type-checker
diagnostics:      error codes + LLM explanation + compiler verification
spec writing:     law templates + LLM draft + prover/test validation
```

The compiler is the only component with final authority. The prover checks. The
LLM suggests.

## 4. LLM calls as effects

An LLM completion is not a pure function. It is:

- non-deterministic,
- possibly remote/networked,
- possibly costly,
- observational (exposes the prompt),
- time-dependent (models change).

Therefore it must carry an effect row:

```ash
fn suggest_proof(pred: Predicate) -> {LLM::complete} Suggestion

fn explain_diagnostic(err: Diagnostic, model: LlmProvider)
    -> {LLM::complete model, Network::outbound} String
```

The `std::llm` module's pure prompt constructors are separate from the effectful
dispatch operations. This separation is intentional: you can build prompts
purely, but invoking a model is an operation that must be admitted at a
boundary.

## 5. Evidence source taxonomy

Every discharge or suggestion must record its origin:

```rust
pub enum EvidenceSource {
    Symbolic(ProverEvidence),
    Connectionist(LlmInvocation),
    Hybrid {
        symbolic: ProverEvidence,
        connectionist: LlmInvocation,
    },
}
```

- `Symbolic`: a prover produced and checked the evidence.
- `Connectionist`: an LLM produced a suggestion that has not yet been checked,
  or produced a natural-language artifact (e.g., diagnostic explanation).
- `Hybrid`: an LLM produced a candidate that a prover verified.

A `Hybrid` source is the normal case for LLM-assisted proof. The connectionist
system proposes; the symbolic system closes.

## 6. Design locks

1. **The symbolic side is fail-closed.** If neither the prover nor an explicit
   fallback discharges an obligation, compilation fails.
2. **The connectionist side has no authority.** An LLM suggestion does not
   erase checks, grant capabilities, or change rows.
3. **LLM calls are effect-tracked.** They appear in computation rows like any
   other operation.
4. **Evidence records the source.** Every artifact records whether it is
   symbolic, connectionist, or hybrid.
5. **Reproducibility rests on the symbolic side.** A `Hybrid` proof is
   reproducible because the prover's check is deterministic; the LLM suggestion
   is audit metadata.
6. **Determinism is opt-in.** Programs without LLM effects are deterministic
   modulo declared async/randomness. Programs with LLM effects are explicitly
   non-deterministic.
7. **The compiler is the orchestrator.** It does not trust either side blindly.

## 7. Relation to verification

NOTE-036 defines the proof-provider architecture (SMT, Lean, symbolic
execution). This note provides the higher-level framing: those provers are the
symbolic side; the LLM is the connectionist side; the compiler orchestrates the
triple `(compiler, prover, LLM)`.

See NOTE-036 for:

- concrete provider interfaces (`prove`, `check`, `suggest`);
- SMT-LIB and Lean4 integration specifics;
- evidence caching and manifest design;
- fallback strategies.

This note adds the requirement that LLM-assisted proof must be recorded as
`Hybrid` evidence and must appear as an effect in the computation row.

## 8. Relation to existing Ash design

- **SPEC-096b Target Effect System:** rows must be able to express LLM
  operations and other connectionist effects. A row item is not limited to
  hardware operations; it can represent any dischargeable requirement,
  including a call to an external model.
- **NOTE-020 Computation Row Taxonomy:** the taxonomy already separates
  operation, resource, role, policy, contract, channel, process, failure, and
  evidence rows. A connectionist-effect row family (e.g., `llm`, `model`,
  `suggest`) is a natural extension.
- **NOTE-030 Monadic Hoare Composition:** the predicate-transformer rule
  composes symbolic contracts. Connectionist suggestions do not alter the rule;
  they merely attempt to discharge the symbolic obligation heuristically.
- **NOTE-035 Temporal and Concurrent Contracts:** monitors consume trace facts.
  A connectionist system could suggest monitor formulas, but the monitor itself
  is a symbolic checker over a deterministic trace.
- **`std::llm` module:** already provides the connectionist primitives. The
  next step is to connect `std::llm::complete` to effect rows and to the
  compiler's orchestration loop.

## 9. Open questions

1. **Row name.** Should the connectionist effect family be `LLM`, `Model`,
   `Suggest`, or something more general like `Heuristic`?
2. **Prompt provenance.** How much of the prompt and response is recorded in
   evidence metadata? Prompts may contain source code; redaction policies apply.
3. **Local vs. remote models.** Remote models are a stronger effect (network,
   third-party observation). Local models are weaker but still
   non-deterministic. Should rows distinguish them?
4. **Deterministic sampling.** If an LLM call uses `seed` and `temperature = 0`,
   is it deterministic enough to omit from the row, or still non-deterministic
   due to model updates?
5. **Cost and quotas.** Should effect rows track budget/cost, or is that a
   runtime policy outside the language?
6. **Connectionist codegen.** If the LLM generates Ash source, does the
   generated code inherit a connectionist provenance marker?
7. **Debugging hybrid proofs.** How does a user inspect why a hybrid proof
   succeeded or failed? The symbolic check gives a yes/no; the connectionist
   step may be opaque.

## 10. References

### Internal references

- [NOTE-020: Computation Row Taxonomy](NOTE-020-COMPUTATION-ROW-TAXONOMY.md)
- [NOTE-030: Monadic Hoare Logic for Ash Computations](NOTE-030-MONADIC-HOARE-LOGIC-FOR-ASH-COMPUTATIONS.md)
- [NOTE-035: Temporal and Concurrent Contracts](NOTE-035-TEMPORAL-AND-CONCURRENT-CONTRACTS.md)
- [NOTE-036: Gradual Verification and Proof Provider Architecture](NOTE-036-GRADUAL-VERIFICATION-AND-PROOF-PROVIDERS.md)
- [SPEC-096b: Target Effect System](../spec/SPEC-096b-TARGET-EFFECT-SYSTEM.md)
- [`std::llm` module](../../std/src/llm/mod.ash)

### External references

- Jürgen Schmidhuber. "Deep Learning in Neural Networks: An Overview." Neural
  Networks, 2015. <https://doi.org/10.1016/j.neunet.2014.09.003> — connectionist
  foundations.
- George B. Dantzig. "Linear Programming and Extensions." 1963 — symbolic
  optimization and verification lineage.
- Gary Marcus. "The Next Decade in AI: Four Steps Towards Robust Artificial
  Intelligence." 2020 — arguments for neuro-symbolic integration.

## 11. Changelog

| Date | Change |
|---|---|
| 2026-07-07 | Initial note. Frames Ash as a symbolic-connectionist hybrid, maps Ash constructs to both sides, defines the compiler as orchestrator, requires LLM effects to be row-tracked, and locks fail-closedness on the symbolic side. |
