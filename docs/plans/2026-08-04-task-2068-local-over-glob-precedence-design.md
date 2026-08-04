# TASK-2068 Local-over-Glob Precedence Design

**Status:** Implemented with focused Type-layer evidence
**Scope:** M-GLOB local-over-glob precedence leaf
**Semantic accounting:** implementation `partial`; evidence `tested`; parity `below_spec`.

## Decision

Within exactly one existing inherited M-GLOB route, a same-module ordinary function wins over an
imported public ordinary function with the same natural name only in returned public bindings.
Non-colliding imports bind normally. The resolver records one selected cross-module edge for every
imported target, including shadowed targets, and cycle-checks that complete edge set before
filtering bindings. All-shadowed input returns no import bindings but retains its edges; a hidden
cycle returns atomic ImportCycle { edges: CanonicalImportCycle }.

Only canonical graph units and provisional module scopes authorize this slice. Private M-CHECK
facts are not import authority.

## Approaches considered

1. Reject every local/glob collision. This preserves the current boundary but does not realize
   the requested precedence rule or reveal cycles hidden by a shadowed binding.
2. **Select all imports, retain their edges, then omit bindings shadowed by same-module ordinary
   functions.** This gives the requested local-wins result while preserving cycle detection and
   atomicity. Recommended.
3. Bind the glob first and overwrite it with the local declaration. This risks publishing an
   incorrect intermediate binding and makes edge/cycle handling depend on binding order.

## Boundaries and handoffs

The delivered resolver/binder is Type-only and returns only a non-authorizing opaque plan/bound
set.
It excludes other imports, multiple globs, aliases/re-exports, `self`, `super`, non-`crate`
paths, nonfunctions, the generic binder, final interfaces, Core/CPS, admission/runtime, and
client parity. TASK-2069 owns lowering; TASK-2064 owns end-to-end parity.

The focused target passes 8/8: local-wins plus non-colliding binding, identity and retained edges,
all-shadowed empty bindings, actual ImportCycle atomicity, retained visibility/shape failures,
normalized Type-layer file/inline scope/binding parity, a 16-case property varying names,
collision subsets, source form, and depth 1–3, and the authority fence. Existing M-GLOB behavior
remains separately rejecting; private M-CHECK facts, generic binder, final-interface, Core/CPS,
Engine, admission/runtime, and parity authority remain excluded.

**Source traceability:** planner sha256:17b2ffe653d196ba295ea1e93bd57ad8c193596918f3787c3f43a1e2e6299f2a;
dedicated binder sha256:652062ee3430667a1259f92777cf7e369b9b5e7ce167151941ade225fb0f8bf1;
lib.rs export boundary sha256:99e7a4c81c34ced69e5fb78830176406e2b30c37b7bf8a9b5617eb78e4664aa6;
unchanged generic binder fence sha256:aea47f6aae83b4b7e3bfaa9ee9a7561d76b12a5f82b50c42d3fd2d98e23ed8b6.
