---
status: drafting
created: 2026-03-30
last-revised: 2026-03-30
related-plan-tasks: []
tags: [functions, capabilities, semantics, design]
---

# MCE-003: Do We Need Functions or Are Capabilities Enough?

## Problem Statement

Ash currently has both workflows and a nascent function concept. This exploration asks: can we eliminate functions entirely and express everything through workflows + capabilities? Or do we need both for different use cases?

This is a fundamental language design decision affecting:
- Syntax complexity
- Semantics clarity
- Implementation effort
- User mental model

## Scope

- **In scope:**
  - Role of functions vs workflows
  - Role of capabilities as the "function-like" abstraction
  - Call semantics (sync vs async)
  - Composition patterns

- **Out of scope:**
  - First-class functions (higher-order)
  - Closures and capture semantics
  - Function types as values

- **Related but separate:**
  - MCE-001: Entry point (may be a workflow or capability)
  - MCE-005: Small-step semantics (call semantics)

## Current Understanding

### What we know

- Workflows are first-class, async-capable, track effects/obligations
- Capabilities provide structured access to external resources
- `call(w)` synchronously invokes a workflow
- `spawn(w)` asynchronously creates a workflow instance
- Sync calls enforce isolation (no obligation inheritance)

### What we're uncertain about

- Can all "function-like" behaviors be expressed as workflows?
- Do users need a lighter-weight abstraction than workflows?
- Is there a semantic difference between "pure function" and "pure workflow"?
- How does this affect the IR (Call form semantics)?

## Design Dimensions

| Dimension | Pure Workflows | Functions + Workflows | Capabilities as Functions |
|-----------|----------------|----------------------|---------------------------|
| Abstractions | 1 | 2 | 2 (but unified interface) |
| Sync calls | `call(w)` | Function call | `call(cap)` |
| Async calls | `spawn(w)` | N/A for functions | `spawn(cap)` |
| Effects tracking | Workflow-level | Function-level? | Capability interface |
| Implementation | Simpler | More complex | Medium |

## Proposed Approaches

### Approach 1: Pure Workflows (No Functions)

Everything is a workflow. "Functions" are just pure workflows (no effects, no obligations).

```ash
-- A "function" is just a workflow
workflow add(a: Int, b: Int) -> Int {
  return a + b
}

-- Called synchronously
let x = call(add(1, 2))
```

**Pros:**
- Single abstraction
- No semantic bifurcation
- Call semantics are uniform

**Cons:**
- All calls have workflow overhead
- May confuse users expecting "just a function"
- No static guarantee of purity

### Approach 2: Functions as Distinct Abstraction

Functions are compile-time, inlined or lightweight. Workflows are runtime units.

```ash
-- Function (compile-time)
fn add(a: Int, b: Int) -> Int {
  a + b
}

-- Workflow (runtime)
workflow greet(name: String) requires io.Stdout {
  act io.Stdout.write("Hello, {name}")
}
```

**Pros:**
- Familiar to programmers
- Functions can be optimized differently
- Clear static/dynamic boundary

**Cons:**
- Two abstractions to learn
- Interop complexity (can workflows call functions? vice versa?)
- More surface syntax

### Approach 3: Capabilities as the Interface

Capabilities define interfaces; workflows implement them. No standalone functions.

```ash
-- Capability defines interface
cap Calculator {
  fn add(a: Int, b: Int) -> Int
}

-- Workflow implements capability
workflow calc(a: Int, b: Int) implements Calculator {
  return a + b
}

-- Usage through capability
let c = acquire Calculator
call c.add(1, 2)
```

**Pros:**
- Unified with capability system
- Interface/implementation separation
- Testable via capability mocking

**Cons:**
- Verbose for simple cases
- Indirection
- Requires capability system to be ergonomic

## Open Questions

1. What is the performance cost of treating everything as a workflow?
2. Can we optimize pure workflows to function-like performance?
3. How do higher-order patterns work in pure-workflow model?
4. What do other capability languages do? (Pony, E, Fuchsia)
5. Is there a "third way" — workflows as the only abstraction, but with compile-time purity tracking?

## Related Explorations

- MCE-001: Entry point (is main a workflow, function, or capability?)
- MCE-002: IR audit (Call form semantics depends on this)

## Decision Log

| Date | Decision | Rationale |
|------|----------|-----------|
| 2026-03-30 | Exploration created | Fundamental design question |

## Next Steps

- [ ] Survey capability language designs
- [ ] Prototype examples in all three approaches
- [ ] Performance analysis of workflow-only model
- [ ] User ergonomics comparison
