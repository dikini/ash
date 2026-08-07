# TASK-2075 Two-Tier Complete Module Collection Design

**Date:** 2026-08-04
**Status:** Approved design; implementation is In progress with `partial / tested / below_spec` accounting after Task 5 atomic collection
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

The implemented Task 4 boundary gives these views private fields and read-only accessors, keeps
layout-sensitive spans outside identity keys, retains module expansion/hygiene sidecars, derives
callable bodies from one raw definition, and makes half-publication unrepresentable through one
paired module map. The internal snapshot has no checked types or body results. The provisional
view has no signature, callable shape, body, checked type, equation, final export, or runtime
authority field. Task 5 now populates and publishes the paired views graph-wide and atomically;
complete raw-fact/drift/file-inline/generated-name/compatibility/authority evidence remains later.

## Namespace and identity model

Canonical declaration identity is `(ModuleKey, declaration kind, canonical parent, origin key)`.
Lookup is `(namespace bucket, visible local key)`. The minimum buckets and their definition mapping
are exactly those in SPEC-103/TASK-2071. Duplicate lookup keys reject within a bucket; the same
spelling across buckets is permitted until a syntax context cannot choose, when lookup rejects as
ambiguous. Nested members and constructors include their canonical parent. Impl coherence compares
full interface applications and impl entries never enter the provisional view.

Task 5 implements those rules with unique parent-scoped member identities, typed duplicate and
unresolved-interface diagnostics, a read-only typed notation pattern/fixity key, and a
module-qualified lexical interface identity. Full impl heads alpha-normalize every surface `Type`
shape without spans; computation rows are sorted/deduplicated fixed-item sets with explicit open
tails, so alpha-renamed/permuted rows and compatible open/closed extensions overlap.

`Capability` rejects. Eligible ordinary/newtype constructors join values; sealed-domain and
promoted constructors remain parent-scoped in their specified levels. Macro-generated names are
not source-spellable. TASK-2075 consumes the declared visibility facts of retained declarations;
it does not add role/policy forms, authority, or compatibility work for removed constructs.

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

One exhaustive table covers all current retained `Definition` variants plus `ModuleDecl`. Additional
tests cover retained declaration visibility, every collision bucket, constructor/member placement, impl
overlap, raw-shape/order/sidecar retention, deliberate absence of internal fields from the name
view, drift mutations, sibling atomicity, normalized file/inline projection, generated inputs,
authority fences, and TASK-2068/TASK-2070 regressions.
