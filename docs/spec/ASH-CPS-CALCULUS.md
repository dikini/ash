---
id: spec.ash.lambda-cps.calculus
title: λAsh-CPS Calculus
kind: semantic-rule-set
audience: [human, agent]
authority: canonical-detail
status: active
stability: alpha
owner: language-semantics
last_verified: 2026-07-27
---

# λAsh-CPS Calculus

> **TASK-2041 execution boundary:** This calculus is not an execution route. `ash run`, `ash
> test`, and REPL each use a local Engine instance and do not communicate with the daemon. The
> daemon executes submitted descriptors with its own local Engine instance and manages
> long-running programs.

**Status:** Frozen kernel calculus for TASK-1989. This is the executable-detail companion to the
[Ash Canonical Core](CANONICAL-CORE.md#core-and-cps-syntax), which remains the single owner of
`core-cps.syntax`. Its machine-readable rule, stage, theorem, and example record is
[ASH-CPS-CALCULUS.json](ASH-CPS-CALCULUS.json). Current Rust Core/CPS code is prototype-only
realization evidence; it neither defines this calculus nor establishes a refinement proof.

## Scope and stage boundary

`λAsh-CPS₀` is the admitted kernel and the mathematical account of the CPS control fragment. It is
not a complete model of the parser, host runtime, or future proof language. `λAsh-Effect` is the
next conservative CPS extension: it must define the effectful CPS syntax, state, and transitions
that correspond to the target operational semantics and the Engine executor. Its proof obligations
remain distinct from its semantic definition. Later features remain deferred and cannot be assumed
by a kernel or effect proof.

The target relation is:

```text
surface Ash --lowering--> Core --lowering--> CPS --realization--> Rust Engine executor
                                             │
                                             └--mathematical semantics--> λAsh-CPS₀ → λAsh-Effect
                                                                 --terminal projection--> observable result
```

The calculi are not production IR stages or alternative evaluators. They explain CPS terms and
configurations, guide conformance cases, and provide the future refinement targets for Rust.

Rows express requirements only. They never install a provider, grant authority, or stand in for a
handler/provider frame.

## Mathematical syntax and state

Let `x` range over variables, `k` over labels or affine continuation closures, `p` over admitted
total primitive operators, and `ρ` over closed requirement rows.

```text
a ::= x | () | n | b | s | C
v ::= a | (v*) | {l = v*} | C(v*) | closure(x*, k, t, η)
t ::= LetVal x = v in t
    | LetPrim x = p(a*) in t
    | LetCont k(x) = t in t
    | LetContCall k(x) = t in t
    | Jump k a | Call a(a*; k)
    | If a then t else t | Match a with cases
    | Return v | Trap reason
```

`Return` is a kernel terminal observation, not a direct-style source form and not a CPS call
result. This resolves the apparent conflict between PLAN-202's kernel `Return` and SPEC-098b's
“no direct return”: surface `return` lowers through its continuation, while completed kernel
evaluation projects to `Return`.

A kernel configuration is `⟨t, η, κ, α, ρ⟩`, where `η` is a mathematical value environment,
`κ` a continuation store, `α` an affine-use map, and `ρ` the closed-row environment. These are
mathematical objects. Rust allocation, captured-environment layout, `Rc`/`RefCell`, maps,
timestamps, serialization, and helper functions are excluded from the state and trusted base.

## Judgments and kernel rules

The frozen judgments are `wf-value`, `wf-term`, `wf-configuration`, `type-row`, small step
`→`, and terminal projection `⇓obs`. The effect gate additionally names frame lookup and an
external provider-boundary transition.

The machine artifact owns stable identifiers for the following kernel rules:

- `SEM-CPS-LETVAL-001` and `SEM-CPS-PRIM-001`: bind an evaluated value or the result of an
  admitted total primitive, respectively.
- `SEM-CPS-LETCONT-001` and `SEM-CPS-LETCONTCALL-001`: extend the mathematical continuation
  store and affine-use state; they do not specify a Rust closure representation.
- `SEM-CPS-JUMP-001` and `SEM-CPS-CALL-001`: transfer to a continuation or invoke a CPS closure
  with its continuation argument.
- `SEM-CPS-IF-001` and `SEM-CPS-MATCH-001`: choose a unique well-formed branch or constructor
  case; an invalid checked-case situation is a structured trap, never unclassified stuckness.
- `SEM-CPS-RETURN-001`: project a completed terminal value.
- `SEM-CPS-TRAP-001`: project structured bottom without granting ordinary row requirements.

The primary relation is deterministic small step. Big step is derived only for terminating kernel
configurations and is not a second operational authority.

## Effect extension

`λAsh-Effect` is the complete conservative extension of `λAsh-CPS₀` for the declared target
effectful CPS subset. It is mathematical semantics for existing CPS, not a production IR, lowerer,
direct evaluator, Engine execution route, or client-local execution route. Its machine-readable
correspondence contract is `effect_correspondence` in
[ASH-CPS-CALCULUS.json](ASH-CPS-CALCULUS.json).

The frozen `admitted_fragment` remains kernel-only. Its separate
`effect_extension_coverage` record is complete for this extension and explicitly prevents the
kernel exclusion list from being misread as an omission or a second admission boundary. The formal
contract names effect configuration well-formedness, effect typing, and every effect transition by
notation and stable rule identity.

An effect configuration is
`⟨t, η, κ, α, F, δ, ρ, ξ⟩`: kernel term/value environment/continuation store; affine
continuation-consumption map; an ordered sequence `F` of `HandlerFrame` and `ProviderFrame`; a
discharge record `δ`; structural residual closed rows `ρ`; and a declared external outcome `ξ`.
The added syntax is `Raise(op, a*, resume)`, `Handle(clause, body, k)`, and administrative
`RecordDischarge(discharge, body)`, plus mathematical frame, affine-resume, and external-outcome
forms. No component names Rust storage, scheduling, a clock, a host provider, a signal, or a
transport.

`SEM-EFFECT-LOOKUP-001` scans the ordered frames innermost-first across both frame kinds.
`SEM-EFFECT-RAISE-001` routes a raise to that selected frame, while
`SEM-EFFECT-DISCHARGE-001` records a structural closed-row discharge. Rows are requirements only:
TASK-2013 checked handler facts may be carried to the boundary, but only TASK-2014's separately
authorized Path-B admission/frame instructions may install a frame. `SEM-EFFECT-ADMISSION-001`
therefore rejects missing or malformed/unchecked entry before execution and never selects a
direct-evaluator fallback.

The canonical machine data names a `HandlerFrame` by its ordered clauses, done clause, residual
row, and captured affine resume; a `ProviderFrame` by its operation identity, authority, persistent
frame identity, and success/failure continuations. A captured affine resume records its binding,
one-use consumption, and handler reinstallation position. Matching is explicitly innermost-frame
then clause order, with the selected frame's done clause and residual row retained as distinct
fields. Every declared effect transition has a source configuration, target configuration, and
closed endpoint vocabulary in the JSON artifact; this is mathematical syntax, not a Rust runtime
representation.

For a matching handler, `SEM-EFFECT-HANDLE-001` removes the selected handler frame while its
operation clause evaluates. On `resume`, `SEM-EFFECT-RESUME-001` reinstates that original handler
in its original position around the resumed tail; the captured continuation consumes at most once.
A second consumption is a structured trap. Only handled-computation completion and resumed-tail
completion enter the selected `done` clause exactly once. An abortive operation-clause result
returns directly while that frame remains absent and bypasses `done`. A trap in the handler body is
propagated by `SEM-EFFECT-HANDLERTRAP-001`, not reinterpreted as a discharge.
`SEM-EFFECT-MISSDISCHARGE-001` makes absent matching discharge a structured outcome rather than
ordinary stuckness.

The determinism claim is deliberately local: it applies only to the one raised-configuration chain
`Raise → Lookup → Dispatch`, whose selected `Dispatch` configuration takes exactly one tagged
handler, provider, or missing-discharge branch under the mutually exclusive innermost-selection
premise. It does not claim that every effect configuration globally has one successor: terminal,
handler-completion, and provider-outcome relations have separately stated domains. Handler entry
and provider invocation therefore do not compete as independent `Raise` transitions.

`SEM-EFFECT-PROVIDER-001` carries the operation arguments and captured continuation `r` from the
selected provider invocation. The provider frame remains at its original ordered-stack position.
An external success explicitly resumes `r(value)` under the retained configuration state; an
operational failure, timeout (`SEM-EFFECT-TIMEOUT-001`), or cancellation
(`SEM-EFFECT-CANCEL-001`) produces the retained `ExternalOutcome(ξ)` state. This is an abstract,
bounded external transition and makes no claim about provider implementation, storage, timer,
signals, or scheduler. `SEM-EFFECT-TERMINAL-001` classifies `Return`, `Trap`, and external outcomes
for separately owned terminal-envelope projection. Normal return remains the inherited
`SEM-CPS-RETURN-001` kernel projection after a handled or resumed completion's exactly-once
`done` clause, or directly for an abortive operation-clause result.

Provider success is not terminalized by this correspondence: it follows
`ExternalSuccess(value, r) → r(value)` as a nonterminal CPS resumption. Generic provider failure,
timeout, and cancellation each use a typed, endpoint-continuous external path:
`ExternalOutcome(ξ) → TerminalReady(ExternalOutcome(ξ)) → Terminal(ExternalOutcome(ξ))`, with the
corresponding `timeout` and `cancelled` labels. These chains preserve the selected provider frame
through outcome classification and remain mathematical non-authorizing handoffs, not provider
execution or a second route. The three provider-specific external terminal transitions are the
only canonical owners of `Terminal(ExternalOutcome(...))`; generic terminalization deliberately
excludes `ExternalOutcome` states, preventing duplicate or lossy external projection.

The completion and terminal phases are explicit and acyclic. A handled completion starts as
`HandledReturn`, a resumed tail as `ResumedTailReturn`, and an abortive clause as
`AbortiveClauseReturn`; none is the generic `Return` configuration. Their non-looping successor
chains are `done(v) → TerminalReady(Return(v)) → Terminal`,
`handler-result(v) → TerminalReady(Return(v)) → Terminal`, and
`MissingDischarge(op) → TerminalReady(MissingDischarge(op)) → Terminal`.
`HandlerBodyTrap(reason)` is likewise a distinct phase step to `TerminalReady(Trap(reason))`, not
a generic `Trap → Trap` self-loop. These are CPS-machine relations only; they do not grant a row,
frame, or task a second execution route or terminal-projection authority.

The canonical examples and rule-indexed conformance obligations cover normal return, missing
admission, malformed/unchecked CPS, handler-body trap, timeout, and cancellation. They are an
evidence plan only: they claim neither an active generic run route nor CLI/daemon parity. Selected
Verus candidates—TASK-2031 authorization, affine use, and terminal projection—are marked
deferred/unproved in traceability; the graph does not report a proof for this task. The lookup
candidate is a deferred correspondence bridge, distinct from the existing
`PROOF-CPS-FRAME-LOOKUP-001`. That proof remains a limited `λAsh-CPS₀` frame-lookup model result
whose declared scope is the declared `SEM-CPS-FRAME-LOOKUP-MODEL-001` model and excludes
`SEM-EFFECT-LOOKUP-001`, so it is not a proof of this correspondence.

## Admitted fragment and exclusions

The admitted fragment consists exactly of the ten kernel term forms in the JSON artifact. Effects,
recursive bindings, thunks and memo stores, traces, monitors, processes, open rows, aliases,
groups, inference, contracts, snapshots, provenance, execution records, and concurrency are not
admitted. Lexer/parser recovery, formatting, macro hygiene, host/FFI internals, scheduler and
network behavior, optimizer correctness, floating-point and unfrozen primitive behavior, and the
future Ash proof language are also excluded.

No current implementation behavior may fill an omission: a storage choice or a helper is evidence
for a later refinement only after a checked view relation names the corresponding mathematical
object.

## Theorem ladder

The theorem identifiers and their statuses are machine-readable in the artifact. Kernel proof work
may use `THM-CPS-WF-001` as frozen syntax/state scope, but determinism, progress, preservation,
substitution/row normalization, primitive determinism, and big-step correspondence remain target
obligations. Effect lookup, shadowing, affine consumption, and the machine-certified local
`Raise → Lookup → Dispatch` determinism route are admitted extension obligations. No theorem claims
determinism for the full effect fragment. Trace/provenance, terminal execution-record projection,
bounded helper nondeterminism, and lowering preservation are later/deferred obligations.

This status distinction is intentional: no theorem is claimed proved merely because a Rust test or
prototype evaluator currently passes.

## Canonical derivation examples

`EX-CPS-RETURN-UNIT-001` projects `Return unit` to `{ kind: return, value: unit }` under
`SEM-CPS-RETURN-001`. `EX-CPS-TRAP-PRIM-001` projects the declared primitive-domain failure to a
structured trap. `EX-CPS-JUMP-001` witnesses the control path through a local continuation to the
same terminal return. Their full, stable rule references and expected projections are in the JSON
artifact, so conformance work can cite identities rather than prose headings.

## Reconciliation and implementation boundary

This calculus narrows the relevant portions of SPEC-098b and SPEC-099b. Their recursion, lazy,
trace, monitor, contract, and runtime-record material is deferred rather than silently included.
The legacy CPS reference remains explanatory; workflow-first formalization material remains
historical. Observable execution-record contracts are later projection work, not a present kernel
axiom.

TASK-1988 found current Rust lowering/interpreter surfaces useful as prototype evidence but not a
semantic proof. TASK-2003 through TASK-2008 supply composable production handoffs; PLAN-203 owns
their executable integration through the single Engine CPS path. No task is closed by this document
except the calculus-freeze documentation task itself.
