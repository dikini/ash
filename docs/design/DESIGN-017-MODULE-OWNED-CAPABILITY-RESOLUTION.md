# DESIGN-017: Module-Owned Capability Resolution

## Status: Draft

## Overview

Replace the Phase 70 bridge resolver with module-system-owned capability resolution. Symbolic
operational calls such as `fs_read(...)` and `io::fs_read(...)` should resolve from declared and
imported capability metadata carried by the module/import pipeline, rather than from parser-local
built-in tables.

## Problem Statement

Phase 70 established the split-dispatch runtime contract:

```text
lookup(provider) -> execute(action_name, args)
```

and added surface support for symbolic and explicit operational calls:

```ash
fs_read("file.txt")
io::fs_read("file.txt")
io:fs_read("file.txt")
```

However, symbolic call resolution is still implemented as a bridge:

- lowering constructs a built-in `CapabilityResolver`
- capability checking constructs a matching built-in `CapabilityResolver`
- capability declarations and imports do not yet own the resolution metadata

This is coherent enough to support the new syntax, but it is not the intended architectural
boundary. Symbolic operational names should be a module/import resolver concern, not a parser-local
lookup table.

## Design Goals

1. Make module/import resolution the source of truth for symbolic capability targets.
2. Ensure capability declarations and re-exports produce explicit resolver metadata.
3. Pass capability-resolution state through lowering and type checking instead of constructing
   ad hoc built-in resolvers in each subsystem.
4. Preserve explicit `provider:action(...)` as a direct surface form that bypasses symbolic
   resolution.
5. Keep the canonical semantic contract in `docs/spec/`.

## Non-Goals

1. Redesign runtime provider dispatch beyond the Phase 70 split-dispatch contract.
2. Remove explicit `provider:action(...)` syntax.
3. Introduce dynamic provider discovery.
4. Redesign `observe`, `set`, or `send` in the same phase.

## Design Decisions

### Decision 1: Capability Declarations Export Resolution Metadata

Every capability declaration that is usable as an operational symbol must contribute a canonical
resolution target:

```text
CapabilitySymbol {
  visible_name,
  declaring_module,
  provider,
  action,
  visibility,
}
```

At minimum, the system must carry enough metadata to answer:

- what symbolic name is visible in a given module scope
- what `(provider, action)` pair that name resolves to
- whether the symbol is visible through direct definition, import, or re-export

This phase does not require redesigning capability declaration syntax. It requires making
declarations the authoritative source of symbolic operational metadata.

### Decision 2: Imports and Re-exports Control Capability Visibility

Module-local symbolic resolution should follow the existing module/import model:

- local capability declarations are visible in their declaring module
- `pub capability` and `pub use` determine export visibility
- `use` and aliasing determine imported local names
- module-qualified names such as `io::fs_read` resolve through the same module graph and import
  rules as other qualified names

The key rule is that `io::fs_read(...)` is still a symbolic module-qualified call, not a provider
encoding. Its target pair comes from module-owned metadata.

### Decision 3: Resolver State Becomes a Pipeline Artifact

The system should build a capability-resolution context once from the module graph plus import
resolution, then pass that context to:

- lowering
- type checking
- capability checking

These phases must stop constructing their own built-in resolver tables.

Conceptually:

```text
module resolver + import resolver
  -> CapabilityResolutionContext
  -> lowering / type checking / capability checking
```

This context may still be materialized as a simple map in the first iteration, but ownership must
move to the module pipeline.

### Decision 4: Standard Library Capability Symbols Enter Through the Same Boundary

Built-in capabilities such as `print` or `fs_read` should not remain special parser/typechecker
tables. They must enter symbolic resolution through the same authoritative path as user code.

Acceptable implementations include:

- standard-library capability declarations in `std/src/...`
- a compiler bootstrap step that loads std capability metadata into the module graph

What is not acceptable is continuing to construct separate built-in mappings in lowering and
type checking.

### Decision 5: Explicit `provider:action(...)` Remains Direct

Explicit provider-qualified operational calls remain a direct target form:

```ash
io:fs_read("file.txt")
```

These do not require symbolic capability lookup. They still flow into the same canonical lowered
`Act { provider_name, action_name, ... }` representation, but they bypass module-owned symbolic
resolution.

### Decision 6: Unresolved Symbolic Calls Fail Before Runtime

Unresolved symbolic operational names must fail explicitly during compile-time phases that require
resolved targets. No fallback provider, no convention-based splitting, and no silent defaulting are
allowed.

Different phases may surface different error types, but they must agree on the core reason:

```text
symbolic operational capability name does not resolve to a `(provider, action)` target
```

## Data Flow

### Current Bridge

```text
surface parse
  -> lowering builds builtin resolver table
  -> type checking builds matching builtin resolver table
  -> runtime executes lowered `(provider, action)`
```

### Target End State

```text
parse module declarations + use statements
  -> build module graph
  -> compute visible capability symbols per module
  -> construct CapabilityResolutionContext
  -> lowering resolves symbolic targets with that context
  -> type checking / capability checking reuse the same context
  -> runtime executes lowered `(provider, action)`
```

## Integration Boundaries

### Parser / Module Parsing

`parse_module` already parses capability definitions and module items. This phase extends the
downstream use of those declarations rather than inventing a second declaration source.

### Module Graph / Import Resolution

The module graph and import resolver should carry enough per-module export and binding information
to derive visible capability symbols and aliases. This is the right ownership layer for symbolic
operational resolution.

### Lowering

Lowering should accept an externally constructed capability-resolution context. Symbolic and
module-qualified targets resolve through that context; explicit `provider:action` targets lower
directly.

### Type Checking and Capability Checking

Type checking and capability declaration checks should use the same capability-resolution context as
lowering so that compile-time validation and lowering agree on what names are visible and what
targets they denote.

## Compatibility / Migration

The bridge introduced in Phase 70 remains the compatibility baseline until this phase is complete.

Migration rules:

1. Do not break explicit `provider:action(...)` forms.
2. Preserve accepted symbolic forms while moving their source of truth.
3. Keep bridge behavior documented as bridge behavior until the module-owned path is complete.
4. Update specs and status docs only when the implementation actually changes ownership.

## Testing Strategy

This phase needs cross-layer tests, not just parser tests:

- module-local capability declaration resolution
- imported capability symbol resolution
- aliased import resolution
- re-exported capability resolution
- qualified symbolic calls resolving through module paths
- std capability symbols entering through the same pipeline
- unresolved symbolic names failing consistently in lowering and capability checking

## Success Criteria

This design is realized when:

1. symbolic operational capability resolution is built from module/import metadata
2. lowering no longer constructs a built-in capability resolver
3. type checking/capability checking no longer construct a built-in capability resolver
4. `docs/spec/` describes the final module-owned contract accurately
5. Phase 70 bridge notes can be removed because the bridge no longer exists
