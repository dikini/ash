# DESIGN-018: Module-Scoped Capability Resolution Closure

## Status: Draft

## Overview

Close the remaining architectural gap in Phase 71 by making shared symbolic capability resolution
explicitly module-scoped end-to-end. The parser/lowering and type-checking paths must both consume
the same `CapabilityResolutionContext` with an explicit current `ModuleId`, instead of relying on
global search helpers or fallback local resolvers.

## Problem Statement

Phase 71 removed the bridge helper methods and introduced:

- `CapabilityExport`
- `CapabilityResolutionContext`
- `CapabilityPipeline`
- `LoweringContext`

But two architectural gaps remain:

1. `resolve_for_lowering()` still performs module-agnostic lookup instead of requiring the current
   module identity.
2. Type checking still carries a fallback local resolver path and does not exclusively depend on the
   shared pipeline context.

That means the implementation is closer to the intended contract, but it still does not enforce one
fully module-scoped symbolic resolution boundary.

## Design Goals

1. Make all unqualified symbolic capability resolution require an explicit current `ModuleId`.
2. Make lowering and capability checking consume the same shared context contract.
3. Remove the remaining local fallback symbolic resolver path from type checking.
4. Keep explicit `provider:action(...)` outside symbolic lookup.
5. Preserve explicit unresolved-name failures.

## Non-Goals

1. Fix unrelated interpreter conditional-execution failures in `ash-engine`.
2. Redesign runtime provider dispatch.
3. Reintroduce any bridge-style built-in symbolic resolver helpers.

## Design Decisions

### Decision 1: Unqualified Resolution Requires `ModuleId`

Every unqualified symbolic lookup must supply the current module identity:

```text
resolve_unqualified(current_module, visible_name) -> Option<(provider, action)>
```

Lookup order:

1. imported aliases visible in `current_module`
2. local declarations in `current_module`

No global cross-module search is allowed for unqualified names.

### Decision 2: Qualified Resolution Is Explicit

Module-qualified symbolic lookups must use an explicit qualified-resolution path:

```text
resolve_qualified(target_module, visible_name) -> Option<(provider, action)>
```

This is separate from unqualified lookup and must not silently fall back to any global search.

### Decision 3: Type Checking Uses Shared Context Only

`CapabilityChecker` and related type-checking surfaces must accept:

- the shared `CapabilityResolutionContext`
- the current `ModuleId`

and resolve symbolic ACT targets through that shared context. The local fallback symbolic resolver
must be removed.

## Success Criteria

This design is realized when:

1. lowering passes explicit `ModuleId` into symbolic resolution
2. type checking passes explicit `ModuleId` into symbolic resolution
3. no module-agnostic symbolic lookup helper remains
4. no local fallback symbolic resolver remains in type checking
5. Phase 71 can be marked complete without qualification
