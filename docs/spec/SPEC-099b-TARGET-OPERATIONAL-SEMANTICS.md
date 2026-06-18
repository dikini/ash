---
id: spec.ash.operational-semantics.target
title: Ash Operational Semantics — Target State
description: Target big-step and small-step operational semantics with unified effect rows and kind-specific discharge
kind: spec
audience: [human, agent]
authority: design
status: draft
stability: alpha
owner: language
last_verified: 2026-06-18
verified_against:
  specs:
    - docs/spec/SPEC-095a-CURRENT-GRAMMAR.md
    - docs/spec/SPEC-095b-TARGET-GRAMMAR.md
    - docs/spec/SPEC-096a-CURRENT-EFFECT-SYSTEM.md
    - docs/spec/SPEC-096b-TARGET-EFFECT-SYSTEM.md
    - docs/spec/SPEC-097a-CURRENT-TYPE-SYSTEM.md
    - docs/spec/SPEC-097b-TARGET-TYPE-SYSTEM.md
    - docs/spec/SPEC-098a-CURRENT-IR.md
    - docs/spec/SPEC-098b-TARGET-IR.md
    - docs/spec/SPEC-099a-CURRENT-OPERATIONAL-SEMANTICS.md
---

# SPEC-099b: Ash Operational Semantics — Target State

**Status:** Draft — target operational semantics for unified effect rows
**Scope:** This document defines the runtime semantics we want Ash to have.
It is a goal-state living document that will be refined as implementation progresses.
**Depends on:** SPEC-095b (Target Grammar), SPEC-096b (Target Effect System), SPEC-097b (Target Type System), SPEC-098b (Target IR)

## 1. Summary

The target operational semantics unifies Ash's runtime behavior into one substrate with
effect-row annotations. The key changes:

1. Replace separate `Act`, `Proc`, and `Workflow` state machines with a unified `Computation`
   state machine carrying an effect row.
2. Add kind-specific discharge rules for capabilities, roles, policies, contracts, channels,
   process operations, failure, and evidence.
3. Add handler stack semantics for effect dispatch.
4. Add contract effect nodes for static/evidence/dynamic discharge tracking.
5. Preserve backward compatibility during migration.

## 2. Target Semantic Domains

### 2.1 Values

```text
Value      ::= Int(i) | Float(f) | String(s) | Bool(b) | Null
             | Time(t) | Ref(r) | List([v, ...]) | Record({k: v, ...})
             | Cap(c)
             | Variant(name, {k: v, ...})
             | Closure(params, body, env)
             | Computation(row, state)
             | HandlerStack(handlers)
             | ProcessHandle(id)
             | ChannelEndpoint(id, direction, type)
```

### 2.2 Effect Rows

```text
EffectRow  ::= { items: [EffectItem], tail: Option<RowVar> }

EffectItem ::= Capability(interface, operation)
             | Resource(resource, mode)
             | Role(role)
             | Policy(binding, decision_domain)
             | Contract(contract_kind, predicate)
             | Channel(channel, mode, message_type, guard)
             | Process(operation)
             | Failure(failure_type)
             | Evidence(sink, kind)
             | Group(group_ref)
```

### 2.3 Computation State

```text
ComputationState ::= Pure(v)
                    | Effect(item: EffectItem, args: [Value], k: Value -> ComputationState)
                    | Handle(handler: Handler, body: ComputationState, k: Value -> ComputationState)
                    | Blocked(reason: BlockReason)
                    | Stuck(error: Error)
```

### 2.4 Context

```text
Context    ::= Γ × E × C × P × Ω × π
  where Γ  = Variable -> Value
        E  = EffectEnvironment (ambient discharged rows)
        C  = Capability -> Implementation
        P  = PolicyEnv
        Ω  = ObligationState
        π  = Provenance
```

## 3. Target Big-Step Semantics

### 3.1 Pure Value

```text
------------------
Pure(v) ⇓ v
```

A pure value is already a result.

### 3.2 Effect Invocation (Handled)

```text
handler_stack = H :: hs
H.can_handle(item) = true
H.run(item, args) = Pure(v)
-----------------------------------
Effect(item, args, k) ⇓ k(v)
```

The effect is handled by the top handler on the stack that can handle it.

### 3.3 Effect Invocation (Unhandled)

```text
handler_stack = hs
no H in hs can handle(item)
-----------------------------------
Effect(item, args, k) ⇓ Stuck(UnhandledEffect(item))
```

If no handler can handle the effect, the computation is stuck.

### 3.4 Handler Boundary

```text
body ⇓ v
-----------------------------------
Handle(H, body, k) ⇓ k(v)
```

The handler is installed for the duration of the body. After the body completes, the handler
is removed and the continuation runs.

### 3.5 Contract Discharge (Static)

```text
static_prove(predicate) = true
-----------------------------------
Effect(Contract(requires, predicate), args, k) ⇓ k(())
```

If the predicate is statically provable, the contract effect is discharged without runtime cost.

### 3.6 Contract Discharge (Dynamic)

```text
runtime_check(predicate, args) = true
-----------------------------------
Effect(Contract(requires, predicate), args, k) ⇓ k(())

runtime_check(predicate, args) = false
-----------------------------------
Effect(Contract(requires, predicate), args, k) ⇓ Stuck(ContractViolation(predicate))
```

If the predicate is not statically provable, a runtime check is performed.

### 3.7 Channel Send

```text
channel_endpoint_exists(ch) = true
channel_direction(ch) = send
channel_type(ch) = T
value : T
-----------------------------------
Effect(Channel(ch, send, T, None), [value], k) ⇓ k(())
```

### 3.8 Channel Receive (Guarded)

```text
channel_endpoint_exists(ch) = true
channel_direction(ch) = receive
channel_type(ch) = T
message : T
guard_predicate(message) = true
-----------------------------------
Effect(Channel(ch, receive, T, guard), [], k) ⇓ k(message)

guard_predicate(message) = false
-----------------------------------
Effect(Channel(ch, receive, T, guard), [], k) ⇓ Blocked(WaitingForGuard(ch, guard))
```

If the guard fails, the computation blocks until a matching message arrives.

## 4. Target Small-Step Semantics

### 4.1 Configuration

```text
Configuration ::= (ComputationState, Context, HandlerStack, Mailbox)
```

### 4.2 Transition Rules

```text
(Pure(v), ctx, hs, mb) -->> v

(Effect(item, args, k), ctx, H::hs, mb)
  H.can_handle(item) = true
  H.run(item, args) = Pure(v)
  -->> (k(v), ctx, hs, mb)

(Effect(item, args, k), ctx, hs, mb)
  no H in hs can handle(item)
  -->> Stuck(UnhandledEffect(item))

(Handle(H, body, k), ctx, hs, mb)
  -->> (body, ctx, H::hs, mb)
  [when body completes, H is removed]

(Effect(Contract(requires, p), args, k), ctx, hs, mb)
  static_prove(p) = true
  -->> (k(()), ctx, hs, mb)

(Effect(Contract(requires, p), args, k), ctx, hs, mb)
  static_prove(p) = false
  runtime_check(p, args) = true
  -->> (k(()), ctx, hs, mb)

(Effect(Contract(requires, p), args, k), ctx, hs, mb)
  static_prove(p) = false
  runtime_check(p, args) = false
  -->> Stuck(ContractViolation(p))

(Effect(Channel(ch, send, T, None), [v], k), ctx, hs, mb)
  channel_exists(ch) = true
  -->> (send(ch, v); k(()), ctx, hs, mb)

(Effect(Channel(ch, receive, T, guard), [], k), ctx, hs, mb)
  message = mb.receive(ch)
  guard(message) = true
  -->> (k(message), ctx, hs, mb)

(Effect(Channel(ch, receive, T, guard), [], k), ctx, hs, mb)
  message = mb.receive(ch)
  guard(message) = false
  -->> (Blocked(WaitingForGuard(ch, guard)), ctx, hs, mb)
```

## 5. Row Profile Checking

### 5.1 Profile Rules

```text
Pure_profile(row) = row == {}

Act_profile(row) = row.items ⊆ {Capability, Resource, Failure, Evidence}

Proc_profile(row) = Act_profile(row) or row.items ⊆ {Capability, Resource, Failure, Evidence, Channel, Process}

Workflow_profile(row) = Proc_profile(row) or row.items ⊆ {Capability, Resource, Failure, Evidence, Channel, Process, Contract, Policy, Role}
```

### 5.2 Profile Violations

If a computation's row contains items outside its profile, the computation is stuck:

```text
Proc_profile(row) = false
row contains Process(item)
-----------------------------------
Stuck(ProfileViolation(Proc, item))
```

## 6. Handler Stack Semantics

### 6.1 Handler Installation

Handlers are installed by `Handle` expressions and by workflow/process boundaries:

```text
workflow boundary:
  installs handlers for: Contract, Policy, Role, Evidence

process boundary:
  installs handlers for: Channel, Process, Failure

Act boundary:
  installs handlers for: Capability, Resource
```

### 6.2 Handler Lookup

Handlers are searched from the top of the stack:

```text
handler_stack = H1 :: H2 :: ... :: Hn

lookup(item) = first Hi such that Hi.can_handle(item)
```

### 6.3 Handler Composition

Multiple handlers for the same effect type compose by stacking. The top handler takes precedence.

## 7. Migration Compatibility

### 7.1 Legacy State Machine Preservation

During migration, the old `ActState`, `ProcState`, and `WorkflowState` are preserved but are
lowered to the unified `ComputationState` before semantic analysis:

```text
ActState<T>   -> ComputationState with Act_profile row
ProcState<T>  -> ComputationState with Proc_profile row
WorkflowState<T> -> ComputationState with Workflow_profile row
```

### 7.2 Dual Semantics

A conforming implementation may maintain both semantics during migration:

- Old semantics for legacy code;
- New semantics for code with effect rows.

The choice is made by checking whether the code has effect-row annotations.

## 8. Open Decisions

1. Whether the unified state machine replaces or wraps the old state machines.
2. Whether handler stacks are first-class values or runtime-only constructs.
3. Whether contract discharge status is stored in the computation state or in a separate sidecar.
4. How row profile checking interacts with type checking.
5. Whether blocked computations are resumed automatically or require explicit `await`.
6. How process identity and mailbox ownership are represented in the unified semantics.

## 9. See Also

- [SPEC-099a: Current Operational Semantics](SPEC-099a-CURRENT-OPERATIONAL-SEMANTICS.md) — what the runtime does today
- [SPEC-004: Operational Semantics](SPEC-004-SEMANTICS.md) — full big-step spec
- [SPEC-025: Small-Step Operational Semantics](SPEC-025-SMALL-STEP-OPERATIONAL-SEMANTICS.md) — full small-step spec
- [SPEC-095b: Target Grammar](SPEC-095b-TARGET-GRAMMAR.md)
- [SPEC-096b: Target Effect System](SPEC-096b-TARGET-EFFECT-SYSTEM.md)
- [SPEC-097b: Target Type System](SPEC-097b-TARGET-TYPE-SYSTEM.md)
- [SPEC-098b: Target IR](SPEC-098b-TARGET-IR.md)

## 10. Changelog

- 2026-06-18: Created as target-state operational semantics document. Defined unified computation state, kind-specific discharge rules, handler stack semantics, and migration compatibility.
