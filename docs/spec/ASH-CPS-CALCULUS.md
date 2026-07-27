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

`λAsh-Effect` extends the kernel with `Raise`, `Handle`, and administrative `RecordDischarge`;
ordered handler/provider frames; source-ordered deep affine resume; structural residual-row
subtraction; and the labelled provider boundary. `SEM-EFFECT-LOOKUP-001`,
`SEM-EFFECT-RAISE-001`, `SEM-EFFECT-HANDLE-001`, and `SEM-EFFECT-MISSDISCHARGE-001` remain stable
identities. The next calculus task must give them complete grammar, configuration, judgments,
transition rules, examples, and an explicit CPS/operational/Rust correspondence contract. Lookup
is innermost-first across both frame kinds; missing discharge is a structured terminal outcome, not
ordinary stuckness. Determinism is relative to declared provider outcomes, not a Rust helper axiom.

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
obligations. Effect lookup, shadowing, affine consumption, and fixed-oracle determinism are
admitted extension obligations. Trace/provenance, terminal execution-record projection, bounded
helper nondeterminism, and lowering preservation are later/deferred obligations.

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
