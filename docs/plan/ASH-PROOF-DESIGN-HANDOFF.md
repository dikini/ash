# Ash Proof-Design Handoff

**Status:** Design entry contract, not an implementation specification
**Evidence source:** TASK-1994 closeout and PLAN-202 pilot reports
**Authority:** Planning (A5). The Canonical Core and CPS calculus remain the semantic authorities.

## Entry decision

A separate Ash-native `spec`/`proof` design programme may start only as a design effort.  It must
not implement grammar, parser, typechecker, runtime erasure, proof checker, or provider protocol
until its own approved task/specification packet exists.  Its inputs are the canonical Core/CPS
rules, the traceability graph, and the measured Verus reports—not Verus syntax or the current Rust
carrier layout.

## Required design decisions

| Topic | Starting evidence | Required decision before implementation | Current boundary |
|---|---|---|---|
| Predicate fragments | `NOTE-031`, `NOTE-033`, `NOTE-036` | Define one typed, pure, stable lowered predicate AST; choose the initial decidable subset and explicit rejected forms. | Source text is diagnostic-only; arbitrary/effectful/recursive predicate execution is not admitted. |
| Provider routing | `NOTE-036`, trace proof metadata, TASK-1991 TCB | Specify provider identity, input/output schema, trust class, deterministic replay data, timeout/resource policy, and checker validation. | Verus is an isolated Rust-model provider experiment, not Ash semantic authority or a default runtime dependency. |
| Proof scripts | `NOTE-036`, `ASH-CPS-CALCULUS.md` theorem ladder | Decide whether scripts are declarative obligations, checked proof terms, or provider-specific attachments; give every obligation a stable trace ID. | No Ash proof syntax or script grammar has been selected. |
| Termination and erasure | `SPEC-027`, `NOTE-036` | State termination obligations and the exact rule for erasing a dynamic check only after trusted `verified` evidence. | Recursive/metatheoretic termination and executable erasure correctness are not proved by PLAN-202. |
| Holes and trust | PLAN-202 §2/§3, `NOTE-036`, Verus TCB reports | Make `verified`, `refuted`, `deferred`, `tested`, `monitored`, `admitted`, `holed`, and `timed-out` non-interchangeable; define whether and where holes can execute. | Pilot logical escapes are empty; that fact does not define an Ash hole policy. |
| Diagnostics | `NOTE-027`, `NOTE-033`, `NOTE-036` | Define stable obligation IDs, source spans, boundary/binder context, provider diagnostics, and remedial actions without treating text as semantics. | Current notes describe proposals; cross-layer implementation and projection remain deferred. |
| LLM synthesis evidence | PLAN-202 §3/§11, TASK-1993 report | Record model/provider/prompt-or-absence, candidate fingerprint, checker command/output, repair iteration, and acceptance result; benchmark at least one replayable failure/repair. | The frame benchmark proves only checker rejection then checker acceptance; it contains no evidenced LLM provenance. |

## Evidence model

Every future obligation must link a canonical rule or explicitly named model to a stable `PROOF-*`
node.  Its record must include provider/version/options, assumptions and trusted boundaries,
artifact fingerprint, implementation revision or an explicit `model-only` marker, and result
status.  A checked proof of a model may refine a canonical rule only when the model boundary is
named; it must not be presented as a direct Rust proof without a checked adapter/refinement.

The first design review must reject proposals that:

- infer Ash semantics from Rust collection order or Verus encoding choices;
- erase checks for `tested`, `deferred`, `admitted`, `holed`, or untrusted outcomes;
- allow a provider, LLM, or tool invocation to become evidence without a deterministic checker;
- hide resource limits, solver/tool versions, assumptions, external bodies, or adapter boundaries;
- turn unresolved runtime/provider behavior into an implicit trusted helper.

## Minimum entry artifacts for the next programme

1. An approved A1/A2 specification for the predicate, obligation, and evidence schemas.
2. A provider protocol and trust policy with replayable checker outputs.
3. A hole/timeout/admission policy and operational erasure contract.
4. A diagnostics contract joining source spans, trace IDs, provider output, and dynamic fallback.
5. A benchmark corpus with declared LLM provenance (or an explicit absence), checker feedback, and
   reproducible acceptance/rejection outcomes.
6. A refinement plan that first closes one direct Rust/model bridge or explicitly justifies a
   model-only design experiment.

## Non-goals carried forward

This handoff does not approve a general theorem prover, unrestricted first-order logic, recursive
proof search, a runtime SMT dependency, provider execution semantics, or compatibility semantics
for forms removed by Phase 201.  It also does not close the Phase-202 follow-up tasks listed in the
closeout review.
