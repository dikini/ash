---
id: spec.ash.operational-semantics
title: Operational Semantics for Unified Effect System
kind: spec
audience: [human, agent]
authority: design
status: draft
stability: alpha
owner: language
last_verified: 2026-06-17
verified_against:
  specs:
    - docs/spec/SPEC-096-UNIFIED-EFFECT-SYSTEM.md
    - docs/spec/SPEC-097-TYPE-SYSTEM-CHANGES.md
    - docs/spec/SPEC-098-IR-CHANGES.md
  code:
    - crates/ash-core/src/ast.rs
    - crates/ash-core/src/value.rs
    - crates/ash-interp/src/eval.rs
---

# SPEC-099: Operational Semantics for Unified Effect System

## 1. Summary

This spec defines the operational semantics for the unified effect system. It covers both small-step semantics (for implementation) and big-step semantics (for reasoning). The key insight: Act, Proc, and Workflow are the same monad, so they share the same operational rules.

## 2. Current Operational Semantics

### 2.1 Current State Machine

The current system has three separate operational semantics:

| Stratum | State | Transitions |
|---------|-------|-------------|
| **Act** | `ActState<T>` | `Pure`, `Effect(cap, args, k)` |
| **Proc** | `ProcState<T>` | `Pure`, `Effect(cap, args, k)`, `Spawn(wf, k)`, `Send(ch, v, k)` |
| **Workflow** | `WorkflowState<T>` | `Pure`, `Effect(cap, args, k)`, `Spawn(wf, k)`, `Send(ch, v, k)`, `Decide(pol, k)`, `Check(obl, k)` |

### 2.2 Current Transition Rules

```
-- Act transitions
Act::Pure(v) -->> v
Act::Effect(cap, args, k) -->> cap.execute(args) >>= k

-- Proc transitions
Proc::Pure(v) -->> v
Proc::Effect(cap, args, k) -->> cap.execute(args) >>= k
Proc::Spawn(wf, k) -->> spawn(wf) >>= k
Proc::Send(ch, v, k) -->> send(ch, v) >>= k

-- Workflow transitions
Workflow::Pure(v) -->> v
Workflow::Effect(cap, args, k) -->> cap.execute(args) >>= k
Workflow::Spawn(wf, k) -->> spawn(wf) >>= k
Workflow::Send(ch, v, k) -->> send(ch, v) >>= k
Workflow::Decide(pol, k) -->> policy.evaluate(pol) >>= k
Workflow::Check(obl, k) -->> obligation.check(obl) >>= k
```

## 3. Unified Operational Semantics

### 3.1 Unified State Machine

Replace three state machines with one:

```
EffState<T> ::= Pure(v : T)
              | Effect(e : EffectName, args : Vec<Value>, k : Value -> EffState<T>)
              | Handle(h : Handler, body : EffState<T>, k : Value -> EffState<T>)
```

### 3.2 Small-Step Semantics

#### 3.2.1 Pure Value

```
------------------
Pure(v) -->> v
```

A pure value is already a result.

#### 3.2.2 Effect Invocation

```
handler_stack = H :: hs
H.can_handle(e) = true
H.run(e, args) = Pure(v)
-----------------------------------
Effect(e, args, k) -->> k(v)
```

The effect is handled by the top handler on the stack.

#### 3.2.3 Effect Invocation (Unhandled)

```
handler_stack = hs
forall H in hs. H.can_handle(e) = false
-----------------------------------
Effect(e, args, k) -->> UnhandledEffect(e, args)
```

No handler can handle the effect — error.

#### 3.2.4 Handle Block

```
-----------------------------------
Handle(h, body, k) -->> body with h pushed on handler_stack
```

Push the handler onto the stack, then evaluate the body.

#### 3.2.5 Handler Pop

```
body -->> Pure(v)
-----------------------------------
Handle(h, body, k) -->> k(v) with h popped from handler_stack
```

Pop the handler after the body completes.

### 3.3 Big-Step Semantics

#### 3.3.1 Pure Value

```
------------------
Pure(v) ==> v
```

#### 3.3.2 Effect Invocation

```
handler_stack = H :: hs
H.can_handle(e) = true
H.run(e, args) ==> v
k(v) ==> result
-----------------------------------
Effect(e, args, k) ==> result
```

#### 3.3.3 Handle Block

```
handler_stack' = h :: handler_stack
body (with handler_stack') ==> v
handler_stack' = handler_stack
k(v) ==> result
-----------------------------------
Handle(h, body, k) ==> result
```

### 3.4 Contract Semantics

#### 3.4.1 Static Discharge

```
Γ ⊢ p = true
-----------------------------------
{requires {p}} <: {}
```

If the type system can prove `p` is always true, the contract is discharged.

#### 3.4.2 Dynamic Discharge

```
Γ ⊢ p = unknown
handler_stack = H :: hs
H.can_handle("Contract") = true
H.run("requires", [p]) ==> ()
-----------------------------------
{requires {p}} <: {} (dynamically)
```

If the type system can't prove `p`, a dynamic handler checks it.

#### 3.4.3 Contract Violation

```
Γ ⊢ p = false
-----------------------------------
{requires {p}} <: {} (violation)
```

If `p` is false, the contract is violated.

## 4. Operational Rules for Surface Syntax

### 4.1 `do` Notation

```
-- do { return e }
-----------------------------------
do { return e } -->> Pure(e)

-- do { x <- e; rest }
e -->> Pure(v)
rest[v/x] -->> result
-----------------------------------
do { x <- e; rest } -->> result

-- do { x <- e; rest }
e -->> Effect(eff, args, k)
-----------------------------------
do { x <- e; rest } -->> Effect(eff, args, \v. do { x <- k(v); rest })
```

### 4.2 `handle` Block

```
-----------------------------------
handle h with { e1 -> k1; e2 -> k2 } in body
-->> Handle(h, body, \v. v)
```

### 4.3 `raise` Expression

```
-----------------------------------
raise e(args) -->> Effect(e, args, \v. Pure(v))
```

### 4.4 Function Application

```
fn f(x: A) -> {r} B { body }
-----------------------------------
f(v) -->> body[v/x] with effect_row = {r}
```

## 5. Effect Handler Semantics

### 5.1 Handler Stack

The handler stack is a list of handlers, searched from top to bottom:

```
HandlerStack ::= Empty
               | Handler(effect_name, handler_fn) :: HandlerStack
```

### 5.2 Handler Lookup

```
lookup(e, Empty) = None
lookup(e, H :: hs) = if H.effect_name == e then Some(H) else lookup(e, hs)
```

### 5.3 Handler Execution

```
H = Handler(e, handler_fn)
H.run(e, args) = handler_fn(args)
```

### 5.4 Contract Handler

```
H = Handler("Contract", contract_handler)
contract_handler("requires", [pred]) = if pred() then Pure(()) else raise ContractViolation
contract_handler("ensures", [pred]) = if pred() then Pure(()) else raise ContractViolation
```

## 6. Concurrency Semantics

### 6.1 Spawn

```
spawn(wf) -->> Effect("Spawn", [wf], \pid. Pure(pid))
```

### 6.2 Send

```
send(ch, v) -->> Effect("Send", [ch, v], \_. Pure(()))
```

### 6.3 Receive

```
receive { p -> k } -->> Effect("Receive", [], \v. match v with p -> k)
```

## 7. Equivalence with Current Semantics

### 7.1 Act Equivalence

```
Act::Effect(cap, args, k) == Eff::Effect(cap, args, k)
```

### 7.2 Proc Equivalence

```
Proc::Effect(cap, args, k) == Eff::Effect(cap, args, k)
Proc::Spawn(wf, k) == Eff::Effect("Spawn", [wf], k)
Proc::Send(ch, v, k) == Eff::Effect("Send", [ch, v], k)
```

### 7.3 Workflow Equivalence

```
Workflow::Effect(cap, args, k) == Eff::Effect(cap, args, k)
Workflow::Spawn(wf, k) == Eff::Effect("Spawn", [wf], k)
Workflow::Send(ch, v, k) == Eff::Effect("Send", [ch, v], k)
Workflow::Decide(pol, k) == Eff::Effect("Decide", [pol], k)
Workflow::Check(obl, k) == Eff::Effect("Check", [obl], k)
```

## 8. Examples

### 8.1 Simple Pure Function

```ash
fn add(a: Int, b: Int) -> {} Int { a + b }
```

Small-step:
```
add(1, 2)
-->> Pure(1 + 2)
-->> Pure(3)
```

### 8.2 Effectful Function

```ash
fn readFile(path: String) -> {fs} String {
    do { x <- fs.read(path); return x }
}
```

Small-step:
```
readFile("x.txt")
-->> do { x <- fs.read("x.txt"); return x }
-->> Effect("fs.read", ["x.txt"], \v. do { x <- Pure(v); return x })
-->> Effect("fs.read", ["x.txt"], \v. Pure(v))
```

After handler executes:
```
-->> Pure("contents")
```

### 8.3 Function with Handler

```ash
fn safeDivide(a: Int, b: Int) -> {} Int {
    handle Contract with {
        requires(pred) -> if pred() then () else return 0
    };
    divide(a, b)
}
```

Small-step:
```
safeDivide(10, 0)
-->> Handle(ContractHandler, divide(10, 0), \v. v)
-->> divide(10, 0) with ContractHandler on stack
-->> Effect("Contract", ["requires", \_. 0 != 0], \_. divide(10, 0))
-->> ContractHandler.run("requires", [\_. 0 != 0])
-->> Pure(())  -- wait, 0 != 0 is false
-->> ContractHandler.run("requires", [\_. 0 != 0])
-->> Pure(0)  -- handler returns 0
```

## 9. Summary

| Feature | Small-Step Rule | Big-Step Rule |
|---------|-----------------|---------------|
| Pure value | `Pure(v) -->> v` | `Pure(v) ==> v` |
| Effect | `Effect(e, args, k) -->> k(H.run(e, args))` | `Effect(e, args, k) ==> k(H.run(e, args))` |
| Handle | `Handle(h, body, k) -->> body with h` | `Handle(h, body, k) ==> k(body with h)` |
| Contract (static) | `{requires {p}} <: {} if Γ ⊢ p` | `{requires {p}} <: {} if Γ ⊢ p` |
| Contract (dynamic) | `{requires {p}} <: {} if H.run("requires", [p])` | `{requires {p}} <: {} if H.run("requires", [p])` |

## 10. See Also

- [SPEC-096: Unified Effect System](SPEC-096-UNIFIED-EFFECT-SYSTEM.md)
- [SPEC-097: Type System Changes](SPEC-097-TYPE-SYSTEM-CHANGES.md)
- [SPEC-098: IR Changes](SPEC-098-IR-CHANGES.md)

## 11. Changelog

- 2026-06-17: Initial draft
