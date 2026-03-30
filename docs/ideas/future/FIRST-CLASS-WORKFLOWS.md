---
status: deferred
created: 2026-03-30
last-revised: 2026-03-30
related-plan-tasks: []
tags: [workflows, first-class, types, existentials, deferred]
---

# FIRST-CLASS-WORKFLOWS: First-Class Workflow Values

## Status: Deferred (Post-Minimal-Core)

This exploration is **deferred** until after the minimal core execution environment is complete.

## Problem Statement

Currently, workflows are invoked by **name** with static `spawn`:

```ash
spawn child_task { init: args }
```

First-class workflows would allow workflows as **values** — passing, storing, returning workflows dynamically.

## Motivation

First-class workflows enable:
- **Dynamic workflow selection:** Load workflow from config, spawn at runtime
- **Higher-order workflows:** Workflows that take/return workflows
- **Workflow collections:** Lists, maps of workflows
- **Plugin systems:** Load and spawn user-provided workflows

## Deferred Rationale

From MCE-001 discussion:

> "Postpone first class workflow. It is not worth it for the time being. At startup we can do some shenaningans, as long as we can provide 'semantic' guarantees that we meet the contract for spawn."

The minimal core uses **gradual typing** at the startup boundary (Rust loads/checks, provides typed capability to Ash), avoiding the need for first-class workflow types in the language.

## Key Design Challenges (For Future)

### 1. Existential Types

To return a workflow of unknown input/output type:

```ash
cap Loader {
  fn load(path: String) -> Result<exists In Out. Workflow<In, Out>, LoadError>
}
```

Requires **existential types** (`exists In Out. ...`) or equivalent.

### 2. Type of Workflow

What is the type of a workflow value?

```ash
-- Option: Parametrized by input record shape
type Workflow<In, Out> = ...

-- Problem: Each distinct input shape = distinct type
-- Cannot store [workflow1, workflow2] in a list if different inputs
```

### 3. Higher-Order Workflows

```ash
workflow map_workflows(
  workflows: List<Workflow<In, Out>>,
  f: Fun(Out) -> Out
) -> List<Handle<Out>> {
  -- Cannot express this without first-class workflows + existentials
}
```

### 4. Effect/Obligation Tracking

If workflows are values, how do effects/obligations propagate?

```ash
let w: Workflow<In, Out> = ...;
-- What is the effect of `w` itself (before spawning)?
-- How do we track that `spawn w` has effect join(parent, w.effect)?
```

## When to Revisit

Consider revisiting when:
- Minimal core is complete and stable
- Plugin/extension system becomes a requirement
- User demand for dynamic workflow loading
- Type system has existentials or row polymorphism

## Related Explorations

- MCE-001 (Entry Point): Deferred first-class workflows, used gradual typing instead
- MCE-003 (Functions vs Capabilities): Overlaps with "what is the type of a workflow"
- SPEC-004 (Semantics): Big-step judgment context (Γ, C, P, Ω, π) may inform workflow typing

## Notes

- Current Ash has `Workflow` as an IR enum (Done, Call, Spawn, Seq, Par, etc.)
- Surface language may need a different model for first-class workflows
- Consider interaction with capability passing and effect tracking
