# TASK-2068 Local-over-Simple Precedence Design

**Status:** Implemented with focused Type-layer evidence
**Scope:** M-SIMPLE local-over-explicit precedence leaf
**Semantic accounting:** implementation `partial`; evidence `tested`; parity `below_spec`.

## Decision

Add one dedicated Type-layer route for exactly one inherited, unaliased
`UsePath::Simple` import of the form
`use crate::<public structural-child>...::<public ordinary-function>;`. Its natural final name is
the only possible import binding. A same-module ordinary function with that name removes the import
from the returned binding projection. A non-colliding import binds normally.

The resolver retains an edge only for a selected cross-module target, completes deterministic
cycle checking over those cross-module edges, and only then removes a locally shadowed binding. A
same-module selected target emits no self-edge and does not participate in cycle detection.
Thus all-shadowed cross-module candidates succeed with no import binding and retained edges, while
a real hidden two-module cross-module cycle returns atomic
`ImportCycle { edges: CanonicalImportCycle }` before any result is published. The binder only
delegates to the resolver and maps its result through `into_bound_set`.

## Approaches considered

1. Keep using the delivered M-SIMPLE route and relax its local-collision error. This would change
   a tested route with a wider grammar: root functions, aliases, and several visibility cases. It
   also makes the current regression expectation unclear. Not recommended.
2. **Add a dedicated narrow resolver and delegating binder.** It isolates one precedence rule,
   preserves the existing route's rejection behavior, and makes edge retention before cycle
   detection explicit. Recommended.
3. Bind the import, then overwrite it with the local function. This risks publishing an
   intermediate result and makes cycle handling depend on binding order. Rejected.

## Boundaries and handoffs

The new public APIs are
`resolve_scoped_simple_local_precedence_imports_with_scopes` and
`bind_scoped_simple_local_precedence_imports`. They consume only the canonical graph and
provisional module scopes, produce only a non-authorizing Type-layer plan/bound set, and do not
use M-CHECK private facts or generic-binder authority.

The exact route excludes root functions, aliases, multiple uses, groups, globs, `self`, `super`,
restricted/private targets or structural paths, re-exports, nonfunctions, lexical body bindings,
final interfaces, Core/CPS, admission/runtime, and client parity. Existing M-SIMPLE remains
unchanged and continues to reject local collisions. TASK-2069 owns later lowering; TASK-2064 owns
file/inline artifact and client terminal parity. The focused file/inline witness, once added,
claims only normalized Type-layer source-form scope/binding parity.

## Focused evidence

The focused `task_2068_local_over_simple_precedence` target passes 9/9. Its witnesses cover
local-wins/noncollision, cross-module identity/edge retention, all-shadowed empty bindings,
actual hidden two-module cycle atomicity, visibility/shape rejection, Type-only normalized
file/inline source-form scope/binding parity, a 16-case depth 1–3/name/collision-mask/source-form
property, an authority fence, and the regression that the existing M-SIMPLE route still rejects a
local collision. These are Type-layer tests, not proof, final-interface, Core/CPS, Engine,
admission/runtime, or client-parity evidence.

**Source traceability:** planner
`sha256:7fb241da5b3bf35595e7cf3054f06dcbc9c9dc08dc9701c047d0d2c045a393d3`; dedicated binder
`sha256:500d00d4de399eaac9c6ad19b74d79a2ec694b724014fbea8cdea02470a0d470`; `lib.rs` export
boundary `sha256:68f8c3410b8bb92ee72cc85b91501a877dd357dca1456c27622f7996c150162c`; unchanged generic
binder fence `sha256:aea47f6aae83b4b7e3bfaa9ee9a7561d76b12a5f82b50c42d3fd2d98e23ed8b6`.
