# Phase 55 Cross-Crate Boundaries Design

**Date:** 2026-03-27

## Goal

Add real cross-crate boundaries to Ash by introducing source-defined crate identity and dependency declarations, loading dependency crates through the parser/module resolver, and enforcing cross-crate visibility in import resolution and type checking.

## Scope

This design includes:

- Root-file crate declaration syntax
- Root-file dependency declaration syntax
- Dependency-aware crate loading in `ash-parser`
- Crate-aware module graph data in `ash-core`
- Cross-crate import resolution and visibility enforcement
- Cross-crate type-checker parity tests

This design does **not** include:

- CLI entrypoints or packaging commands
- Registry/version resolution
- Binary artifact caching
- New re-export semantics beyond existing import resolver scope

## User-Facing Syntax

The crate root file declares its crate name and file-path dependencies:

```ash
crate app;

dependency util from "../util/main.ash";
dependency policy from "../policy/main.ash";

mod workflows;
```

Cross-crate imports use an explicit external prefix:

```ash
use external::util::sanitize::normalize;
use external::policy::rules::review_gate;
```

This keeps `crate::...` reserved for the current crate and avoids ambiguity with relative paths.

## Design Decision

Use a single crate-aware `ModuleGraph` rather than introducing a separate workspace graph abstraction. Each module node will carry crate identity, and the graph will track crate metadata and declared dependency aliases. That minimizes churn in the current `ImportResolver`, which already expects one graph object.

## Architecture

### 1. Crate Root Metadata

Add a lightweight parser for crate-root metadata:

- `crate <name>;`
- `dependency <alias> from "<path>";`

Only crate root files may declare this metadata. Non-root modules remain regular Ash module files.

### 2. Crate-Aware Module Graph

Extend `ash_core::module_graph` with:

- `CrateId`
- crate metadata records keyed by `CrateId`
- crate ownership on each `ModuleNode`
- dependency alias mapping from one crate to another

The graph still stores module parent/child relationships exactly once, but now modules can be grouped by crate.

### 3. Dependency-Aware Module Loading

Extend `ModuleResolver::resolve_crate(...)` to:

1. Parse crate-root metadata from the entry file
2. Register the current crate in the graph
3. Resolve in-crate `mod` declarations as today
4. Resolve declared dependency crate roots recursively
5. Detect duplicate crate names, duplicate dependency aliases, and dependency cycles

### 4. Cross-Crate Import Resolution

`ImportResolver` should support:

- `crate::...` for current-crate imports
- `external::<alias>::...` for declared dependency imports

Cross-crate visibility rule:

- `pub` is importable across crates
- `pub(crate)`, `pub(super)`, `pub(self)`, and `pub(in ...)` are never importable from another crate

### 5. Type Checker Alignment

The type checker should stop relying on loose string heuristics for “external” and instead model external crate references explicitly in tests and path parsing. It should preserve the same boundary rule as import resolution: cross-crate access requires `pub`.

## Error Handling

Add explicit errors for:

- missing crate declaration in dependency root
- duplicate crate name
- duplicate dependency alias in one crate
- undeclared external crate alias in an import
- dependency cycle across crate roots

Error text should stay library-grade and actionable.

## Testing Strategy

TDD split:

1. Parser tests for root metadata syntax
2. `ModuleGraph` tests for crate identity and dependency edges
3. `ModuleResolver` tests using `MockFs` for multi-crate loading
4. `ImportResolver` tests for `external::...` path resolution and visibility rejection
5. `ash-typeck` tests for explicit external crate path semantics
6. Workspace regression pass

Use in-memory/mock file systems for most loader tests. Only add real fixture files when parser/import integration needs realistic source layouts.

## Recommended Task Split

1. Parse crate root and dependency syntax
2. Add crate-aware graph structures
3. Load dependency crates recursively
4. Resolve external imports and enforce visibility
5. Align type checker and add integration coverage
6. Close out Phase 55
