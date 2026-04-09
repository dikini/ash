# DESIGN-016: Capability Call Dispatch Split and Operational Call Sugar

## Status: Draft

## Overview

Split operational capability execution into an explicit `(provider, action)` target, and add
act-less workflow sugar for operational calls. This design removes the remaining runtime overload
where one name is used both for provider lookup and provider-local action dispatch.

## Problem Statement

### Current State

The current ACT path still overloads one name:

- core `Workflow::Act` stores `action_name`
- interpreter execution builds `Action { name: action_name, ... }`
- provider lookup also uses that same `action_name`

That makes the current runtime contract effectively:

```text
lookup provider by action_name
then provider.execute(Action { name: action_name, ... })
```

This is semantically muddled. A provider and a provider-local action are different concepts.

### Why This Blocks Cleaner Syntax

The desired surface forms are:

```ash
capability(args)
capability(args) when guard

provider:action(args)
provider:action(args) when guard
```

These forms only stay honest if the implementation preserves a resolved `(provider, action)` pair.
Otherwise `provider:action(...)` becomes cosmetic sugar over an overloaded flat name.

## Design Goals

1. Make provider lookup and provider-local action dispatch explicit.
2. Preserve one canonical internal ACT representation across parser, lowering, interpreter, engine,
   and future compiled backends.
3. Support both explicit `provider:action(...)` and symbolic capability calls such as
   `io::fs_read(...)`.
4. Keep name resolution separate from runtime dispatch.
5. Update the surface, big-step, and small-step specs together.

## Non-Goals

1. Redesign `observe`, `set`, or `send` in the same phase.
2. Expose provider internals as the primary user-facing abstraction.
3. Add dynamic provider/action discovery features.

## Design Decisions

### Decision 1: Canonical ACT Target Is `(provider, action)`

Core operational execution will carry:

```rust
Workflow::Act {
    provider_name: Name,
    action_name: Name,
    action_arguments: Vec<Expr>,
    guard: Guard,
    provenance: Provenance,
}
```

This shape is canonical regardless of whether the source used:

- `act provider:action(args) with guard`
- `provider:action(args) when guard`
- `capability(args)`
- `io::capability(args) when guard`

### Decision 2: Symbolic Capability Calls Resolve to `(provider, action)`

Symbolic capability names remain part of the surface language and module system. Resolution is a
separate concern from execution.

The resolver should produce a target shape equivalent to:

```rust
ResolvedCapabilityTarget {
    provider: Name,
    action: Name,
}
```

Examples:

```ash
fs_read("file.txt")
io::fs_read("file.txt")
```

Both resolve through the normal symbol/module machinery, then lower to the same canonical ACT
target form as explicit `provider:action(...)`.

### Decision 3: Explicit `provider:action(...)` Exists in the Surface Language

The language should also allow explicit qualified operational calls:

```ash
fs:write_file("file.txt", "hello")
mcp:call("tools/call", params) when approved(request_id)
```

This explicit form is cleaner for:

- tests
- debugging
- docs and examples
- low-level runtime-facing workflows

It also gives the module system a stable canonical target shape to resolve symbolic calls into.

### Decision 4: Act-less Operational Calls Are Workflow Sugar

Add workflow-position sugar:

```ash
capability(args)
capability(args) when guard
provider:action(args)
provider:action(args) when guard
```

These are operational workflow forms, not general expression calls.

Canonical lowering intent:

```text
capability(args)              => ACT(resolved provider, resolved action, args, always)
capability(args) when g       => ACT(resolved provider, resolved action, args, g)
provider:action(args)         => ACT(provider, action, args, always)
provider:action(args) when g  => ACT(provider, action, args, g)
```

### Decision 5: Provider Trait Must Be Provider-local

The provider trait should not receive its own provider name again.

Instead of:

```rust
async fn execute(&self, action: &Action) -> Result<Value, CapabilityError>;
```

the canonical provider-local execution surface should become:

```rust
async fn execute(
    &self,
    action_name: &str,
    args: &[Value],
) -> Result<Value, CapabilityError>;
```

Provider lookup happens outside the trait:

```text
registry.get(provider_name) -> provider.execute(action_name, args)
```

This keeps the trait boundary aligned with the abstraction boundary.

### Decision 6: Small-step Semantics Are Refined, Not Replaced

This design affects small-step semantics narrowly.

The semantic family does not change: ACT still evaluates guard, evaluates arguments, and crosses a
host-interaction boundary. What changes is the call target shape and helper contract.

`SPEC-025` should therefore be updated from an overloaded single-name ACT target to an explicit
`(provider, action, values)` helper-owned interaction boundary.

## Surface Syntax Summary

### Canonical User-facing Forms

```ash
capability(args)
capability(args) when guard

provider:action(args)
provider:action(args) when guard
```

### Compatibility Form

Existing explicit `act ...` forms remain during the migration, but should lower to the same split
provider/action representation.

## Architecture

### Before

```text
surface action name
  -> core Act(action_name, args, guard)
  -> evaluate args
  -> registry lookup(action_name)
  -> provider.execute(Action { name: action_name, arguments })
```

### After

```text
surface symbolic or explicit operational call
  -> resolve target to (provider, action)
  -> core Act(provider_name, action_name, args, guard)
  -> evaluate args
  -> registry lookup(provider_name)
  -> provider.execute(action_name, values)
```

## Spec Impact

This design requires coordinated updates to:

- `SPEC-001`: core `Workflow::Act` contract
- `SPEC-002`: surface syntax and sugar
- `SPEC-003`: resolver/type-system treatment of symbolic capability call targets
- `SPEC-004`: big-step ACT execution contract
- `SPEC-010`: embedding/runtime trait expectations
- `SPEC-017`: capability integration and symbolic capability target contract
- `SPEC-025`: small-step ACT helper boundary

## Risks

### Risk 1: Legacy Flat ACT Forms Drift During Migration

Mitigation: make one spec-first task freeze the compatibility story before parser/runtime edits.

### Risk 2: Resolver Metadata Is Too Weak for `(provider, action)`

Mitigation: add a dedicated resolver/typechecker task rather than burying the change in parser
work.

### Risk 3: Runtime Trait Churn Breaks Engine Providers

Mitigation: migrate interpreter/runtime dispatch first, then engine providers, then final
verification.

## Success Criteria

1. Core ACT execution uses explicit provider lookup and provider-local action dispatch.
2. The surface language supports both symbolic capability calls and explicit `provider:action(...)`
   operational calls.
3. Specs agree on the split call target contract.
4. Resolver-backed symbolic capability names can target `(provider, action)` pairs.
5. Providers no longer rely on overloaded `Action.name` for both lookup and dispatch.
