---
id: spec.ash.effect-system.current
title: Ash Effect System — Current State
description: The current four-stratum effect system with separate Act, Proc, and Workflow types
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
  code:
    - crates/ash-core/src/effect.rs
    - crates/ash-core/src/ast.rs
    - crates/ash-interp/src/eval.rs
    - crates/ash-typeck/src/
---

# SPEC-096a: Ash Effect System — Current State

**Status:** Active — records the live effect system as of main HEAD
**Scope:** This document is the authority for what the compiler and runtime do today.
It does not propose changes.
**Frozen against:** `e61f2792`

## 1. Summary

Ash currently tracks effects through two separate mechanisms:

1. A **4-point effect lattice** (`Epistemic < Deliberative < Evaluative < Operational`) used for coarse workflow-node classification.
2. A **four-stratum tower** (`Pure < Act < Proc < Workflow`) with separate types, syntax, and runtime representations for each stratum.

These mechanisms are not unified. The lattice classifies effects; the tower separates computation kinds. There is no row polymorphism, no effect rows on function types, and no common accounting layer across capabilities, roles, policies, contracts, channels, or process operations.

## 2. Current Effect Lattice

### 2.1 Definition

From `crates/ash-core/src/effect.rs`:

```rust
pub enum Effect {
    Epistemic = 0,      -- Read-only
    Deliberative = 1,   -- Analysis/planning
    Evaluative = 2,     -- Policy evaluation
    Operational = 3,    -- Side effects
}
```

### 2.2 Semantics

The lattice is a total order:

```text
Epistemic < Deliberative < Evaluative < Operational
```

Each workflow node carries an effect grade. The runtime uses this grade for:

- capability invocation classification;
- audit trail effect recording;
- coarse provenance tracking.

The lattice is **not** used for:

- function-type effect tracking;
- row polymorphism;
- capability/role/policy/contract/channel accounting;
- static discharge of contracts or policies.

## 3. Current Tower

### 3.1 Strata

| Stratum | Type | Syntax | Runtime |
|---------|------|--------|---------|
| **Pure** | No dedicated type | `fn` | Direct evaluation |
| **Act** | `Act<T>` | `do:Act { ... }`, `act { ... }` | `ActEnv` / capability dispatch |
| **Proc** | `Proc<T>` | `do:Proc { ... }` | Process handles, spawn, await |
| **Workflow** | `Workflow<T>` | `workflow { ... }`, `do:Workflow { ... }` | Workflow boundary, contracts, reporting |

### 3.2 Type Representation

From `crates/ash-core/src/ast.rs` and related type surfaces:

```rust
-- Pure function type
Type::Fn(args, ret)

-- Effectful/capability-linked callable type
Type::Fun(args, ret, effect)

-- Act computation
Type::Constructor { name: "Act", args: [T] }

-- Proc computation
Type::Constructor { name: "Proc", args: [T] }

-- Workflow computation
Type::Constructor { name: "Workflow", args: [T] }
```

### 3.3 Runtime Representation

From `crates/ash-core/src/value.rs` and interpreter surfaces:

```rust
pub enum Value {
    -- ... primitives ...
    Closure { params, body, env },
    Cap { name, effect },
    ProcessHandle(...),
    Proc(...),
    ActEnvToken(...),
    -- ...
}
```

The runtime has separate state machines for Act, Proc, and Workflow:

```text
ActState<T>   ::= Pure(v) | Effect(cap, args, k)
ProcState<T>  ::= Pure(v) | Effect(cap, args, k) | Spawn(wf, k) | Send(ch, v, k)
WorkflowState<T> ::= Pure(v) | Effect(cap, args, k) | Spawn(wf, k) | Send(ch, v, k)
                     | Decide(pol, k) | Check(obl, k)
```

## 4. Current Capability System

### 4.1 Capability Declarations

```ash
capability fs {
    read(path: String) -> String;
    write(path: String, content: String) -> ();
}
```

Capabilities are declared at module level. They bind to Rust `CapabilityProvider` implementations or Ash `capability impl` recipes (SPEC-052).

### 4.2 Capability Invocation

```ash
act fs.read("/tmp/config.txt")
```

Capability calls are workflow/Act statements, not ordinary expressions. They require an `Act` or higher context.

### 4.3 Capability Binding

Workflow headers declare capability bindings:

```ash
workflow processor
    capabilities: [fs @ { paths: ["/tmp/*"] }]
{
    act fs.read("/tmp/config.txt");
}
```

Bindings are admission-time associations, not first-class values.

## 5. Current Role System

### 5.1 Role Declarations

```ash
role ai_agent {
    capabilities: [file @ { paths: ["/tmp/*"], read: true, write: false }]
}
```

Roles declare authority (capabilities) and obligations. They are static and assigned at spawn time.

### 5.2 Role Inclusion

```ash
workflow processor
    plays role(ai_agent)
{
    act file.read("/tmp/config.txt");
}
```

Role inclusion is a workflow-header clause. It is not represented in function types or effect rows.

### 5.3 Runtime Enforcement

The runtime checks role authority before capability invocation. Authority denial is a hard error. Obligations are tracked and must be discharged before workflow completion.

## 6. Current Policy System

### 6.1 Policy Declarations

```ash
policy RateLimit {
    requests: Int,
    window_secs: Int
}
```

Policies are named schemas with fields and optional `where` invariants.

### 6.2 Policy Bindings

```ash
policy production_rate = RateLimit { requests: 1000, window_secs: 60 };
```

Named policy bindings are the canonical boundary between syntax and lowering.

### 6.3 Policy Usage

```ash
workflow api_call {
    decide request_meta under production_rate then {
        act http_get with url: "https://api.example.com";
    }
}
```

Policies are consumed by `decide` statements. They are not first-class values and do not appear in function types.

## 7. Current Contract System

### 7.1 Function Contracts

```ash
fn divide(a: Int, b: Int) -> Int
    requires: {b != 0}
{
    a / b
}
```

Contracts are `requires` and `ensures` clauses on function definitions. They are not tracked in types or effect rows.

### 7.2 Workflow Contracts

```ash
workflow processor
    requires: {input.valid}
    ensures: {output.processed}
{
    ...
}
```

Workflow contracts are header clauses. They are checked at workflow boundaries, not at function call sites.

## 8. Current Channel and Process System

### 8.1 Channels

```ash
send message to channel;
receive wait {
    pattern -> { ... }
}
```

Channels are workflow statements. They are not typed as effects and do not appear in function types.

### 8.2 Process Operations

```ash
spawn workflow_type;
await process_handle;
join process_handle;
```

Process operations are workflow/Proc statements. They require a `Proc` or `Workflow` context.

## 9. Current Do-Notation

### 9.1 Syntax

```ash
do:Act { return expr }
do:Proc { return expr }
do:Workflow { return expr }
```

Each stratum has its own `do` target. There is no unified `do` form.

### 9.2 Semantics

`do:K` elaborates to nested `bind`/`return` calls for the target `K`. The target must be a known computation constructor (`Act`, `Proc`, `Workflow`).

## 10. Known Limitations

1. No effect rows on function types.
2. No row polymorphism.
3. No unified accounting for capabilities, roles, policies, contracts, channels, or process operations.
4. No static discharge of contracts or policies.
5. No effect aliases or groups.
6. No user-defined algebraic effects.
7. No resumable continuations.
8. `Act`, `Proc`, and `Workflow` are separate runtime implementations, not views over a shared substrate.

## 11. See Also

- [SPEC-096b: Target Effect System](SPEC-096b-TARGET-EFFECT-SYSTEM.md) — unified effect rows
- [SPEC-004: Operational Semantics](SPEC-004-SEMANTICS.md) — big-step semantics for current system
- [SPEC-025: Small-Step Operational Semantics](SPEC-025-SMALL-STEP-OPERATIONAL-SEMANTICS.md) — small-step semantics for current system
- [SPEC-019: Role Runtime Semantics](SPEC-019-ROLE-RUNTIME-SEMANTICS.md)
- [SPEC-024: Capability-Role-Workflow Syntax](SPEC-024-CAPABILITY-ROLE-REDUCED.md)
- [SPEC-052: Capability Interfaces and Implementations](SPEC-052-CAPABILITY-INTERFACES-AND-IMPLEMENTATIONS.md)
- [SPEC-006: Policy Definition Syntax](SPEC-006-POLICY-DEFINITIONS.md)
- [SPEC-007: Policy Combinators](SPEC-007-POLICY-COMBINATORS.md)

## 12. Changelog

- 2026-06-18: Split from combined SPEC-096 into current-state document. Frozen against `e61f2792`. Added explicit description of current lattice, tower, capability, role, policy, contract, channel, process, and do-notation systems.
- 2026-06-17: Initial draft.
