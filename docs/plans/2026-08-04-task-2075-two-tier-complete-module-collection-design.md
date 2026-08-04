# TASK-2075 Two-Tier Complete Module Collection Design

**Date:** 2026-08-04
**Status:** Approved design; implementation is In progress with `not_implemented / none / below_spec` accounting
**Authority:** SPEC-103 and TASK-2071

## Problem

Import resolution needs only names, identities, namespaces, visibility, and anchors. Body checking
needs raw declaration and callable shapes, bodies, member spans, expansion sidecars, and order.
Putting both purposes into one provisional interface would leak checker facts into import authority
and make later checked interfaces indistinguishable from unvalidated collection data.

## Selected design

TASK-2075 consumes only `CanonicalExpandedModuleGraph` and atomically builds:

- `CanonicalCollectedModuleSnapshot`: checker-internal, with raw expanded declaration/callable
  shapes, bodies/member spans, per-module expansion origins/hygiene, source anchors and ordinals;
- `CanonicalProvisionalNameView`: import-facing, with only lookup key/name, defining identity/key,
  namespace, declared visibility/exportability, origin/source anchor, and source ordinal.

The internal snapshot has no checked types or body results. The provisional view has no signature,
callable shape, body, checked type, equation, final export, or runtime-authority field. Construction
is private and graph-wide; either both views publish or neither does.

## Namespace and identity model

Canonical declaration identity is `(ModuleKey, declaration kind, canonical parent, origin key)`.
Lookup is `(namespace bucket, visible local key)`. The minimum buckets and their definition mapping
are exactly those in SPEC-103/TASK-2071. Duplicate lookup keys reject within a bucket; the same
spelling across buckets is permitted until a syntax context cannot choose, when lookup rejects as
ambiguous. Nested members and constructors include their canonical parent. Impl coherence compares
full interface applications and impl entries never enter the provisional view.

`Capability` rejects. Eligible ordinary/newtype constructors join values; sealed-domain and
promoted constructors remain parent-scoped in their specified levels. Macro-generated names are
not source-spellable. Policy/role/evidence importability depends on retained declared visibility.
TASK-2075 must first repair AST visibility carriers for policy, role, law, and proof where absent.

## Compatibility and drift

`CanonicalProvisionalModuleScopes` stays available only as a projection for TASK-2068/TASK-2070.
The new collector does not widen those APIs or treat their ordinary-function-only snapshot as
complete.

Before publication, revalidation compares the expanded graph and staged facts for name, kind,
visibility, signature, body, source order, and expansion-sidecar drift. Equivalent file/inline
inputs compare through a normalized projection that omits source-layout-only paths/offsets while
retaining all semantic identities and relative provenance.

## Failure model

Errors identify module key, declaration/member identity, namespace bucket, canonical parent,
source anchor, and the violated rule. A late failure in one module discards both staged products for
all siblings. Import binding, body checking, and final export closure are not run by collection.

## Evidence design

One exhaustive table covers all 21 current `Definition` variants plus `ModuleDecl`. Additional
tests cover visibility carriers, every collision bucket, constructor/member placement, impl
overlap, raw-shape/order/sidecar retention, deliberate absence of internal fields from the name
view, drift mutations, sibling atomicity, normalized file/inline projection, generated inputs,
authority fences, and TASK-2068/TASK-2070 regressions.
