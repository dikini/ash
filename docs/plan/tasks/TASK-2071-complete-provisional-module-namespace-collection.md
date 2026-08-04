# TASK-2071: Complete Provisional Module Namespace Collection

**Status:** Planned
**Phase:** [PLAN-207](../PLAN-207-COMPLETE-MODULE-REALIZATION.md)
**Spec:** SPEC-103 §§5-6 (`M-COLLECT`, checked namespace prerequisites)
**Owned rule:** MOD-REAL-003 complete provisional namespace/callable collection
**Run-route impact:** prerequisite
**Semantic task record:** [semantic-task-records.json](../semantic-task-records.json)
**Semantic coverage map:** [TASK-2071](../SEMANTIC-RULE-COVERAGE.md#task-2071-complete-provisional-module-namespace-collection)

## Semantic authority and axes

**Implementation:** partial
**Evidence:** none
**Parity:** below_spec
**Missing target-spec clauses:** The planned TASK-2071 M-COLLECT completion slice is `partial / none / below_spec`: it will extend TASK-2068's immutable ordinary-function provisional scopes to complete non-public namespace and callable facts needed by later checking and binding, preserving canonical `ModuleKey`, defining identity, declaration/body spans, origin, declared visibility, namespace kind, signature/callable shape, and source-order facts. It will rebuild and compare graph/unit declaration snapshots before publication and atomically reject malformed, duplicate, drifted, or incomplete collection. It will not check bodies, finalize public/private interfaces or export closure, resolve imports, authorize re-exports, lower Core/CPS, admit runtime artifacts, or claim parity. Type and verification are partial; Core/CPS/admission-runtime are not_applicable; run-route impact is prerequisite. TASK-2073 consumes complete collected facts for checking/finalization, TASK-2072 consumes them for binding, TASK-2069 consumes only TASK-2073's complete handoff, and TASK-2064 owns parity.
**Layers:** Type `partial`; Core/CPS/admission-runtime `not_applicable`; verification `partial`.
**Next obligation:** Complete canonical provisional namespace/callable collection and hand stable facts independently to TASK-2072 and TASK-2073; neither consumer may treat a provisional fact as a final interface or runtime credential.

## Description

Complete the collection boundary that TASK-2068 intentionally limited to ordinary-function facts.
The output is an immutable, canonical scope/namespace snapshot for later import binding and checked
finalization, never a public interface or execution authority.

## Requirements

1. Collect every target namespace and callable fact required by SPEC-103 while preserving identity,
   provenance, visibility, spans, source order, and module ownership.
2. Rebuild declaration snapshots from current graph units and reject drift, duplicates, malformed
   namespace entries, or incomplete units before any scope publishes.
3. Keep private/non-public facts available only as Type-layer collection data; do not project or
   authorize them as imports, exports, interfaces, or runtime facts.
4. Publish atomically across sibling modules.

## TDD Steps

1. Add a red complete-namespace/non-public-callable collection target.
2. Add identity/visibility/source-order, snapshot-drift/duplicate atomicity, and file/inline
   normalized scope tests plus a generated namespace/callable property.
3. Add an authority-fence test rejecting final-interface/import/runtime use of a provisional fact.
4. Implement the minimal collector, run focused tests, fmt, and strict clippy, then promote only
   actual witnesses.

## Scope and non-goals

This task excludes body checking, final public/private interface publication, export closure,
parsed import resolution/binding, `pub use` staging, Core/CPS lowering, Engine transport/admission,
execution, and terminal parity.

## Handoffs and completion checklist

- **Consumes:** TASK-2067 canonical graph/module units and TASK-2068 tested foundation facts.
- **Produces:** complete immutable provisional namespace/callable facts, non-authorizing.
- **Downstream owner:** TASK-2072 resolves/binds imports; TASK-2073 checks/finalizes interfaces;
  TASK-2069 waits for TASK-2073.
- **Integration/proof:** TASK-2064 owns composed parity.
- [ ] Positive, negative, mutation, file/inline, property, and authority-fence evidence exists.
- [ ] No collection result is a final interface, import credential, or runtime authority.
