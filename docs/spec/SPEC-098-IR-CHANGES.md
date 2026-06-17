---
id: spec.ash.ir-changes
title: IR Changes for Unified Effect System
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
  code:
    - crates/ash-core/src/ast.rs
    - crates/ash-core/src/value.rs
    - crates/ash-interp/src/eval.rs
---

# SPEC-098: IR Changes for Unified Effect System

## 1. Summary

This spec documents the changes to the Ash Intermediate Representation (IR) required to support the unified effect system. The IR is the core AST (`crates/ash-core/src/ast.rs`) that the parser lowers to and the interpreter/typechecker operates on. The key question: does the unified effect system require IR changes, or is it purely a surface syntax + type system feature?

## 2. Assessment: IR Changes Required

**Answer: Yes, but minimal.**

The unified effect system is primarily a **type system** and **surface syntax** change. The core computational structure (expressions, workflows, pattern matching) remains the same. However, the IR needs:

1. **Effect row tracking** on function types and expressions
2. **Contract effect nodes** for runtime discharge
3. **Handler stack representation** for effect dispatch
4. **Unified monad representation** (single `Eff` instead of `Act`/`Proc`/`Workflow`)

## 3. Current IR Baseline

### 3.1 Core AST (from `crates/ash-core/src/ast.rs`)

```rust
pub enum Workflow {
    Observe { capability, pattern, continuation },
    Receive { mode, arms, control },
    Orient { expr, continuation },
    Propose { action_name, action_arguments, continuation },
    Decide { expr, policy, continuation },
    Check { obligation, continuation },
    Act { provider_name, action_name, arguments, guard, provenance, result_name, continuation },
    Call { target, arguments, continuation },
    Oblig { role, workflow },
    Let { pattern, expr, continuation },
    If { condition, then_branch, else_branch },
    Seq { first, second },
    ForEach { pattern, collection, body },
    Ret { expr },
    With { capability, workflow },
    Maybe { primary, fallback },
    Must { workflow },
    Set { capability, channel, value },
    Send { capability, channel, value },
    Spawn { workflow_type, init, pattern, continuation },
    Split { expr, pattern, continuation },
    Kill { target, continuation },
    Pause { target, continuation },
    Resume { target, continuation },
    CheckHealth { target, continuation },
    Oblige { name, span },
    CheckObligation { name, span },
    Yield { role, request, expected_response_type, continuation, span, resume_var },
    ProxyResume { value, value_type, correlation_id, span },
    Done,
}
```

### 3.2 Current Value Types (from `crates/ash-core/src/value.rs`)

```rust
pub enum Value {
    Int(i64),
    Float(f64),
    String(String),
    Bool(bool),
    Null,
    Time(...),
    Ref(...),
    List(Vec<Value>),
    Record(Vec<(Name, Value)>),
    Variant { name, payload },
    Closure { params, body, env },
    Cap { name, effect },
    ProcessHandle(...),
    Proc(...),
    Instance(...),
    InstanceAddr(...),
    ControlLink(...),
    Stream(...),
    ActEnvToken(...),
}
```

### 3.3 Current Runtime Representation

The runtime has three separate monad implementations:
- `Act<T>` — for capability effects
- `Proc<T>` — for process effects  
- `Workflow<T>` — for workflow effects

These are distinct types in the runtime, not unified.

## 4. Proposed IR Changes

### 4.1 Minimal Change: Add Effect Row to Types

The smallest change: add effect row to `TypeExpr` and function signatures, but keep the AST mostly unchanged.

```rust
pub enum TypeExpr {
    -- ... existing variants ...
    EffectRow { effects: Vec<EffectItem>, row_var: Option<Name> },
}

pub struct FnType {
    params: Vec<TypeExpr>,
    return_type: TypeExpr,
    effect_row: TypeExpr,  -- NEW
}
```

**Impact:** Type checker and parser only. Runtime unchanged initially.

### 4.2 Moderate Change: Unified Monad Type

Replace `Act`/`Proc`/`Workflow` with a single `Eff` type:

```rust
pub enum Value {
    -- ... existing variants ...
    -- REMOVE: ActEnvToken, Proc, Instance, InstanceAddr, ControlLink
    -- ADD:
    Eff { effect_row: TypeExpr, value: Box<Value> },  -- NEW: unified effect monad
}
```

**Impact:** Runtime changes. All three monads become one.

### 4.3 Full Change: Effect Handler Stack

Add handler stack to the runtime environment:

```rust
pub struct Env {
    -- ... existing fields ...
    handler_stack: Vec<Handler>,  -- NEW: effect handlers
}

pub struct Handler {
    effect_name: Name,
    handler_fn: Value,  -- Closure that handles the effect
}
```

**Impact:** Major runtime change. Effects are dispatched dynamically.

## 5. Recommended Approach: Phased IR Changes

### 5.1 Phase 1: Type-Only Changes (No Runtime)

Changes to `TypeExpr` only. The runtime still uses `Act`/`Proc`/`Workflow`.

```rust
pub enum TypeExpr {
    -- ... existing ...
    EffectRow { effects: Vec<EffectItem>, row_var: Option<Name> },
}
```

The type checker tracks effect rows, but the runtime ignores them. This is a **type erasure** approach.

**Pros:**
- Minimal runtime changes
- Can implement type system first
- Backward compatible

**Cons:**
- Runtime doesn't enforce effect safety
- Contracts are not checked at runtime
- Can't add dynamic handlers yet

### 5.2 Phase 2: Unified Runtime Type

Replace `Act`/`Proc`/`Workflow` with `Eff`:

```rust
pub enum Value {
    -- ... existing ...
    Eff { effect_row: TypeExpr, value: Box<Value> },
}
```

The runtime now has one monad. The effect row is still mostly ignored at runtime (type erasure), but the type is unified.

**Pros:**
- Single monad implementation
- Simpler runtime
- Can add effect dispatch later

**Cons:**
- Still no dynamic effect checking
- Requires updating all runtime code

### 5.3 Phase 3: Full Effect Handler Stack

Add dynamic effect dispatch:

```rust
pub struct Env {
    bindings: Vec<(Name, Value)>,
    handler_stack: Vec<Handler>,
}

pub struct Handler {
    effect_name: Name,
    handler_fn: Value,
}
```

Effects are dispatched dynamically at runtime. The handler stack is searched for each effect invocation.

**Pros:**
- Full effect safety at runtime
- Dynamic handlers work
- Contracts can be checked dynamically

**Cons:**
- Runtime overhead
- Complex implementation
- Requires careful design

## 6. IR Node Changes Detail

### 6.1 Workflow Nodes: What Changes

| Node | Current | Proposed | Change |
|------|---------|----------|--------|
| `Act` | `Act { provider, action, args, guard, ... }` | `Act { provider, action, args, guard, effect_row, ... }` | Add effect_row field |
| `Call` | `Call { target, args, ... }` | `Call { target, args, effect_row, ... }` | Add effect_row field |
| `Let` | `Let { pattern, expr, ... }` | `Let { pattern, expr, effect_row, ... }` | Add effect_row field |
| `Ret` | `Ret { expr }` | `Ret { expr, effect_row }` | Add effect_row field |
| `Spawn` | `Spawn { workflow_type, init, ... }` | `Spawn { workflow_type, init, effect_row, ... }` | Add effect_row field |
| `Send` | `Send { cap, channel, value }` | `Send { cap, channel, value, effect_row }` | Add effect_row field |
| `Receive` | `Receive { mode, arms, ... }` | `Receive { mode, arms, effect_row, ... }` | Add effect_row field |
| `Yield` | `Yield { role, request, ... }` | `Yield { role, request, effect_row, ... }` | Add effect_row field |
| `Done` | `Done` | `Done { effect_row }` | Add effect_row field |

**Pattern:** Every workflow node gets an `effect_row` field. This is the minimal IR change.

### 6.2 Expression Nodes: What Changes

| Node | Current | Proposed | Change |
|------|---------|----------|--------|
| `FnDef` | `FnDef { params, return_type, body }` | `FnDef { params, return_type, effect_row, body }` | Add effect_row field |
| `Call` | `Call { func, args }` | `Call { func, args, effect_row }` | Add effect_row field |
| `DoBlock` | `DoBlock { target, stmts }` | `DoBlock { effect_row, stmts }` | Replace target with effect_row |
| `Literal` | `Literal { value }` | `Literal { value, effect_row }` | Add effect_row field (always `{}`) |

### 6.3 New IR Nodes

```rust
pub enum Workflow {
    -- ... existing nodes ...
    
    -- NEW: Handle effect
    Handle {
        effect: Name,
        handler: Box<Workflow>,
        body: Box<Workflow>,
        continuation: Box<Workflow>,
    },
    
    -- NEW: Raise effect
    Raise {
        effect: Name,
        arguments: Vec<Expr>,
        continuation: Box<Workflow>,
    },
    
    -- NEW: Contract check
    ContractCheck {
        contract: ContractEffect,
        body: Box<Workflow>,
        continuation: Box<Workflow>,
    },
}
```

## 7. Lowering Changes

### 7.1 Parser to IR Lowering

The parser currently lowers surface syntax to core AST. With unified effects:

```ash
-- Surface syntax
fn foo(x: Int) -> {fs} Int {
    do { y <- fs.read("x"); return y }
}

-- Lowered IR
FnDef {
    params: [("x", TypeExpr::Named("Int"))],
    return_type: TypeExpr::Named("Int"),
    effect_row: TypeExpr::EffectRow {
        effects: vec![EffectItem::Capability("fs")],
        row_var: None,
    },
    body: DoBlock {
        effect_row: TypeExpr::EffectRow {
            effects: vec![EffectItem::Capability("fs")],
            row_var: None,
        },
        stmts: vec![
            Let { pattern: "y", expr: Call { func: "fs.read", args: ["x"], effect_row: ... } },
            Ret { expr: "y", effect_row: ... }
        ]
    }
}
```

### 7.2 IR to Runtime

The interpreter evaluates the IR. With unified effects:

```rust
fn eval_workflow(workflow: Workflow, env: Env) -> Result<Value, EvalError> {
    match workflow {
        Act { provider, action, args, effect_row, ... } => {
            -- Check that env.handler_stack can handle effect_row
            -- If not, error
            -- If yes, dispatch to handler
            eval_act(provider, action, args, env)
        }
        Handle { effect, handler, body, ... } => {
            -- Push handler onto env.handler_stack
            -- Evaluate body
            -- Pop handler
            eval_handle(effect, handler, body, env)
        }
        Raise { effect, args, ... } => {
            -- Search env.handler_stack for effect handler
            -- If found, call handler with args
            -- If not, error
            eval_raise(effect, args, env)
        }
        -- ... existing cases ...
    }
}
```

## 8. Backward Compatibility

### 8.1 Compatibility Strategy

The IR changes are additive. Old IR without effect rows is still valid.

```rust
pub enum TypeExpr {
    -- ... existing ...
    EffectRow { ... },  -- NEW
}

-- Old code: TypeExpr::Named("Int") — still valid
-- New code: TypeExpr::EffectRow { ... } — new variant
```

### 8.2 Migration Path

1. **Phase 1**: Add `EffectRow` to `TypeExpr`. Old code ignores it.
2. **Phase 2**: Add `effect_row` field to workflow nodes. Default to `None` (old behavior).
3. **Phase 3**: Update interpreter to use `effect_row`. Old code still works (default behavior).
4. **Phase 4**: Remove old `Act`/`Proc`/`Workflow` types. Breaking change.

## 9. Examples

### 9.1 Pure Function IR

```ash
fn add(a: Int, b: Int) -> {} Int { a + b }
```

IR:
```rust
FnDef {
    params: [("a", Named("Int")), ("b", Named("Int"))],
    return_type: Named("Int"),
    effect_row: EffectRow { effects: vec![], row_var: None },  -- empty = pure
    body: Binary { op: Add, left: "a", right: "b" }
}
```

### 9.2 Effectful Function IR

```ash
fn readFile(path: String) -> {fs} String {
    do { x <- fs.read(path); return x }
}
```

IR:
```rust
FnDef {
    params: [("path", Named("String"))],
    return_type: Named("String"),
    effect_row: EffectRow { effects: vec![Capability("fs")], row_var: None },
    body: DoBlock {
        effect_row: EffectRow { effects: vec![Capability("fs")], row_var: None },
        stmts: vec![
            Let {
                pattern: "x",
                expr: Call { func: "fs.read", args: ["path"], effect_row: EffectRow { effects: vec![Capability("fs")], row_var: None } },
                effect_row: EffectRow { effects: vec![Capability("fs")], row_var: None }
            },
            Ret { expr: "x", effect_row: EffectRow { effects: vec![Capability("fs")], row_var: None } }
        ]
    }
}
```

### 9.3 Function with Handler IR

```ash
fn safeDivide(a: Int, b: Int) -> {} Int {
    handle Contract with {
        requires(pred) -> if pred() then () else return 0
    };
    divide(a, b)
}
```

IR:
```rust
FnDef {
    params: [("a", Named("Int")), ("b", Named("Int"))],
    return_type: Named("Int"),
    effect_row: EffectRow { effects: vec![], row_var: None },
    body: Handle {
        effect: "Contract",
        handler: FnDef {
            params: [("pred", Named("Fn(() -> Bool)"))],
            return_type: Named("()"),
            effect_row: EffectRow { effects: vec![], row_var: None },
            body: If {
                condition: Call { func: "pred", args: [] },
                then_branch: Ret { expr: Literal(Null), effect_row: EffectRow { effects: vec![], row_var: None } },
                else_branch: Ret { expr: Literal(Int(0)), effect_row: EffectRow { effects: vec![], row_var: None } }
            }
        },
        body: Call { func: "divide", args: ["a", "b"], effect_row: EffectRow { effects: vec![], row_var: None } },
        continuation: Ret { expr: Literal(Null), effect_row: EffectRow { effects: vec![], row_var: None } }
    }
}
```

## 10. Summary of Changes

| Component | Change | Size |
|-----------|--------|------|
| `TypeExpr` | Add `EffectRow` variant | Small |
| `FnDef` | Add `effect_row` field | Small |
| `Workflow` nodes | Add `effect_row` field to all | Medium |
| `Expr` nodes | Add `effect_row` field to some | Small |
| `Value` | Add `Eff` variant, remove `Act`/`Proc`/`Workflow` | Medium |
| `Env` | Add `handler_stack` | Small |
| Interpreter | Add `Handle`, `Raise`, `ContractCheck` cases | Medium |
| Lowerer | Update to populate `effect_row` | Medium |

## 11. See Also

- [SPEC-096: Unified Effect System](SPEC-096-UNIFIED-EFFECT-SYSTEM.md)
- [SPEC-097: Type System Changes](SPEC-097-TYPE-SYSTEM-CHANGES.md)
- [SPEC-099: Operational Semantics](SPEC-099-OPERATIONAL-SEMANTICS.md)

## 12. Changelog

- 2026-06-17: Initial draft
