---
id: spec.ash.operational-semantics.current
title: Ash Operational Semantics — Current State
description: Current big-step and small-step operational semantics for the four-stratum tower
code_commit: e61f2792
kind: spec
audience: [human, agent]
authority: derived-from-code
status: active
stability: beta
owner: language
last_verified: 2026-06-18
verified_against:
  git_commit: e61f2792
  specs:
    - docs/spec/SPEC-004-SEMANTICS.md
    - docs/spec/SPEC-025-SMALL-STEP-OPERATIONAL-SEMANTICS.md
---

# SPEC-099a: Ash Operational Semantics — Current State

**Status:** Active — records the live operational semantics as of main HEAD
**Scope:** This document is the authority for how the Ash runtime evaluates programs today.
It does not propose changes.
**Frozen against:** `e61f2792`

## 1. Summary

Ash currently has two operational semantics specifications:

1. **SPEC-004**: Big-step semantics for whole-workflow meaning.
2. **SPEC-025**: Small-step semantics for workflow-first stepwise execution.

Both semantics describe the four-stratum tower (`Pure < Act < Proc < Workflow`) with separate
state machines for each stratum. There is no unified effect-row semantics.

## 2. Current Big-Step Semantics (SPEC-004)

### 2.1 Semantic Domains

```text
Value      ::= Int(i) | Float(f) | String(s) | Bool(b) | Null
             | Time(t) | Ref(r) | List([v, ...]) | Record({k: v, ...})
             | Cap(c)
             | Variant(name, {k: v, ...})

Effect     ::= Epistemic | Deliberative | Evaluative | Operational

Trace      ::= ε | TraceEvent :: Trace

EffectTrace ::= EffectTrace { terminal: Effect, reached: Set(Effect) }

Provenance ::= Prov { id, parent, lineage, ... }

Result<Value, Error> ::= Ok(Value) | Err(Error)

PolicyEnv  ::= PolicyName → Policy

ObligationState ::= Set(Obligation)

CompletionPayload ::= {
  result: Result<Value, Error>,
  obligations: ObligationState,
  provenance: Provenance,
  effects: EffectTrace,
}

Error      ::= PolicyViolation(policy, v)
             | ObligationViolation(obligation)
             | GuardViolation(action, guard)
             | PatternBindFailure
             | PatternMatchFailure(v)
             | TerminalControl(action, target, reason)
             | RuntimeFailure(reason)

Context    ::= Γ × C × P × Ω × π
  where Γ  = Variable → Value
        C  = Capability → Implementation
        P  = PolicyEnv
        Ω  = ObligationState
        π  = Provenance

WorkflowOutcome ::= Return(Value, Effect, Trace, ObligationState, Provenance)
                  | Reject(Error, Effect, Trace, ObligationState, Provenance)
```

### 2.2 Key Rules

The big-step semantics defines the meaning of canonical workflow forms:

- `Act` transitions: capability execution with effect recording;
- `Proc` transitions: spawn, send, await;
- `Workflow` transitions: decide, check, yield, proxy resume.

Pure expressions are atomic and do not produce effects.

## 3. Current Small-Step Semantics (SPEC-025)

### 3.1 Configuration

```text
Configuration ::= (WorkflowState, Context, HandlerStack, Mailbox)
```

### 3.2 Transition Rules

The small-step semantics defines explicit configuration transitions:

```text
(Act::Pure(v), ctx, hs, mb) -->> v
(Act::Effect(cap, args, k), ctx, hs, mb) -->> cap.execute(args) >>= k

(Proc::Spawn(wf, k), ctx, hs, mb) -->> spawn(wf) >>= k
(Proc::Send(ch, v, k), ctx, hs, mb) -->> send(ch, v) >>= k

(Workflow::Decide(pol, k), ctx, hs, mb) -->> policy.evaluate(pol) >>= k
(Workflow::Check(obl, k), ctx, hs, mb) -->> obligation.check(obl) >>= k
```

### 3.3 Blocked vs Stuck

A configuration is **blocked** if it is waiting for an external event (e.g., `receive`).
A configuration is **stuck** if it has reached an error state with no valid transition.

## 4. Current Effect Classification

The operational semantics uses the 4-point `Effect` lattice:

```text
Epistemic < Deliberative < Evaluative < Operational
```

Each transition is classified by its effect grade. The runtime records the effect trace for
audit and provenance.

## 5. Known Limitations

1. No unified effect-row semantics.
2. No row polymorphism in operational rules.
3. No kind-specific discharge rules.
4. No contract effect nodes.
5. No handler stack semantics for user-defined effects.
6. No resumable continuations.
7. `Act`, `Proc`, and `Workflow` have separate state machines.

## 6. See Also

- [SPEC-099b: Target Operational Semantics](SPEC-099b-TARGET-OPERATIONAL-SEMANTICS.md) — unified semantics with effect rows
- [SPEC-004: Operational Semantics](SPEC-004-SEMANTICS.md) — full big-step spec
- [SPEC-025: Small-Step Operational Semantics](SPEC-025-SMALL-STEP-OPERATIONAL-SEMANTICS.md) — full small-step spec
- [SPEC-096a: Current Effect System](SPEC-096a-CURRENT-EFFECT-SYSTEM.md)
- [SPEC-097a: Current Type System](SPEC-097a-CURRENT-TYPE-SYSTEM.md)
- [SPEC-098a: Current IR](SPEC-098a-CURRENT-IR.md)

## 7. Changelog

- 2026-06-18: Created as current-state operational semantics document. Frozen against `e61f2792`. Summarized SPEC-004 and SPEC-025 with explicit current limitations.
