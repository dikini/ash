# TASK-2072: Parsed Import Resolution and Atomic Binding

**Status:** Planned
**Phase:** [PLAN-207](../PLAN-207-COMPLETE-MODULE-REALIZATION.md)
**Spec:** SPEC-103 §§6-7 (`M-IMPORT-EDGE`, `M-IMPORT-CYCLE`, `M-BIND`)
**Owned rule:** MOD-REAL-004 complete parsed imports, cycles, precedence, and binding
**Run-route impact:** prerequisite
**Semantic task record:** [semantic-task-records.json](../semantic-task-records.json)
**Semantic coverage map:** [TASK-2072](../SEMANTIC-RULE-COVERAGE.md#task-2072-parsed-import-resolution-and-atomic-binding)

## Semantic authority and axes

**Implementation:** partial
**Evidence:** none
**Parity:** below_spec
**Missing target-spec clauses:** The planned TASK-2072 complete parsed-import slice is `partial / none / below_spec`: it will resolve every admitted parsed use/path/visibility/alias/re-export grammar form against TASK-2071 provisional facts, stage canonical import edges and bindings, apply specified local/explicit/glob precedence and ambiguity/duplicate rules, detect complete cross-module cycles before publication, and stage `pub use` facts for TASK-2073 finalization. It will preserve target identity, namespace, origin, declaration/use spans, visibility, and source ordering, and atomically reject unsupported, inaccessible, ambiguous, duplicate, cyclic, or partial sibling inputs. It will not make staged bindings or `pub use` facts a final export closure, checked interface, Core/CPS artifact, Engine admission credential, runtime authority, or parity claim. Type and verification are partial; Core/CPS/admission-runtime are not_applicable; run-route impact is prerequisite. TASK-2073 owns final checked/export-closed publication, TASK-2069 consumes TASK-2073 only, and TASK-2064 owns parity.
**Layers:** Type `partial`; Core/CPS/admission-runtime `not_applicable`; verification `partial`.
**Next obligation:** Implement the complete parsed grammar and atomic binding domain, then hand only
staged resolved bindings and staged public-use facts to TASK-2073 for separately owned finalization.

## Description

Replace TASK-2068's deliberately isolated resolver/binder leaves with one complete parsed-import
realization. It consumes canonical provisional facts, never raw-source rediscovery or M-CHECK
private facts as import authority, and publishes no partial plan/binding set.

## Requirements

1. Cover all target parsed import grammar forms, structural traversal, visibility, aliases,
   re-exports, groups, globs, self/super/root forms, namespaces, and source-owned spans.
2. Apply complete precedence, ambiguity, duplicate-binding, and cycle rules deterministically.
3. Stage `pub use` identity/visibility/provenance facts without finalizing their export closure.
4. Reject every failure atomically across imports and sibling modules.

## TDD Steps

1. Add red all-grammar and identity/provenance tests.
2. Add red precedence/ambiguity/duplicate, staged-`pub use`, and complete-cycle atomicity tests.
3. Add file/inline normalized binding parity, a generated grammar/visibility property, and an
   authority fence.
4. Implement after RED, run focused tests/quality gates, then promote actual evidence only.

## Scope and non-goals

This task excludes provisional collection ownership, checked bodies, final interface/export closure,
Core/CPS, Engine transport/admission/runtime, execution, and client parity.

## Handoffs and completion checklist

- **Consumes:** TASK-2071 complete provisional facts, TASK-2070's narrow self alias if delivered,
  and TASK-2068's preserved foundation evidence.
- **Produces:** atomic resolved bindings/edges plus staged `pub use` facts, non-authorizing.
- **Downstream owner:** TASK-2073 alone validates bodies, private/public views, export closure, and
  final public-use publication; TASK-2069 consumes only that complete handoff.
- **Integration/proof:** TASK-2064 owns cross-layer/file-inline/client parity.
- [ ] Every grammar form, precedence/ambiguity/duplicate/cycle outcome, and staged public-use path
  has positive/negative/mutation/property evidence.
- [ ] No staged binding authorizes a final interface, Engine route, or direct evaluator.
