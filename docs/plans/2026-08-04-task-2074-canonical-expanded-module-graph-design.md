# TASK-2074 Canonical Expanded Module Graph Design

**Date:** 2026-08-04
**Status:** Approved design; implementation in progress with a verified initial local-only slice
**Authority:** SPEC-103 and TASK-2071

## Problem

The canonical parser graph stores complete ordered `ModuleBody` values, but existing expansion APIs
take `ModuleFile`, which omits parsed uses and the unified source order. Existing expansion also
recurses through inline module bodies, although the canonical graph already owns each inline child
under its own `ModuleKey`. Engine macro loading is path/source/filesystem based and cannot authorize
the SPEC-103 route.

## Selected design

Add a parser-owned `CanonicalExpandedModuleGraph` that consumes and owns the exact
`CanonicalModuleGraph` it expands. A private builder creates a `BTreeMap<ModuleKey,
CanonicalExpandedModule>` and publishes only after its key set exactly equals the parsed unit key
set. Each record retains its shallowly expanded `ModuleBody`, source path/artifact origin, expansion
diagnostics, origins, and hygiene. `ExpansionId` remains module-local; the enclosing `ModuleKey`
disambiguates every sidecar.

Add a parser-internal shallow `ModuleBody` expander. It expands direct definitions only, retains
uses and nested structural declarations unchanged, rebuilds the ordered snapshot, and never
recursively re-expands an inline child owned by another key.

Before expansion, run an AST-only syntax prepass. It gathers public macro and notation summaries,
resolves only syntax imports through canonical keys and parsed `Use` spans, rejects dependency
cycles, and orders providers before consumers. An imported notation remains inactive without a
canonical summary. The prepass supplies no general import binding or runtime authority.

## Public boundary

The target API is intentionally read-only after construction:

```rust
pub struct CanonicalExpandedModuleGraph { /* private */ }

impl CanonicalExpandedModuleGraph {
    pub fn try_expand(
        parsed: CanonicalModuleGraph,
    ) -> Result<Self, CanonicalModuleExpansionError>;
    pub fn parsed_graph(&self) -> &CanonicalModuleGraph;
    pub fn module(&self, key: &ModuleKey) -> Option<CanonicalExpandedModuleRef<'_>>;
    pub fn modules(&self) -> impl Iterator<Item = CanonicalExpandedModuleRef<'_>>;
}
```

The exact syntax-summary carrier stays parser-owned and syntax-only. It retains provider
`ModuleKey`, declaration identity/origin, visibility, and source anchor so an imported string path
cannot replace canonical identity.

## Failure model

`CanonicalModuleExpansionError` wraps existing expansion failures with module key, source path,
artifact origin, and exact span. Dedicated variants cover invalid syntax imports, syntax dependency
cycles with ordered key/span edges, and graph/expanded-map invariant failures. Any error discards
the staged map. There is no partial carrier.

## Rejected alternatives

- Projecting `ModuleBody` to `ModuleFile` would split authority, lose ordered uses from the input,
  and risk duplicate recursive inline expansion.
- Reusing the Engine module loader would introduce filesystem, text, path-cache, and second-route
  authority forbidden by SPEC-103.
- A detached expanded map plus content hash would permit stale graph pairing. Owning the parsed graph
  makes that mismatch unrepresentable for this non-cache boundary.

## Evidence design

Focused tests cover shallow expansion, AST-only syntax dependency order/cycles, use/order retention,
per-key sidecars, exact map equality, normalized file/inline results, mutations, sibling atomicity,
generated graph shapes, no filesystem/Engine dependency, and current expansion/graph regressions.
This is parser-stage evidence only.
