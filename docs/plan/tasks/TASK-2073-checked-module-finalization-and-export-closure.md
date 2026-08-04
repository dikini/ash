# TASK-2073: Checked Module Finalization and Export Closure

**Status:** Planned
**Phase:** [PLAN-207](../PLAN-207-COMPLETE-MODULE-REALIZATION.md)
**Spec:** SPEC-103 §§6-8 (`M-CHECK`, final export closure)
**Owned rule:** MOD-REAL-003 complete checked bodies/private-public/export-closed interface
**Run-route impact:** prerequisite
**Semantic task record:** [semantic-task-records.json](../semantic-task-records.json)
**Semantic coverage map:** [TASK-2073](../SEMANTIC-RULE-COVERAGE.md#task-2073-checked-module-finalization-and-export-closure)

## Semantic authority and axes

**Implementation:** partial
**Evidence:** none
**Parity:** below_spec
**Missing target-spec clauses:** The planned TASK-2073 finalization slice is `partial / none / below_spec`: it will consume TASK-2071 complete provisional namespace/callable facts and TASK-2072 staged resolved bindings/public-use facts to check all supported module bodies, retain complete private and public checked facts, validate export closure, and atomically publish one versioned final interface plus final `pub use` projection only after every dependency succeeds. It will preserve canonical module/declaration identity, origin, declaration/body/use spans, visibility, namespace, signature/body results, binding provenance, and dependency versions; it will reject incomplete, stale, forged, failed, cyclic, or export-inconsistent facts before publication. It will establish normalized Type-layer file/inline final-interface parity, not Core/CPS/runtime parity. Type and verification are partial; Core/CPS/admission-runtime are not_applicable; run-route impact is prerequisite. TASK-2069 consumes this complete checked handoff for lowering, TASK-2063 awaits TASK-2069, and TASK-2064 owns executed/client parity.
**Layers:** Type `partial`; Core/CPS/admission-runtime `not_applicable`; verification `partial`.
**Next obligation:** Complete checked finalization/export closure, then hand TASK-2069 one complete,
non-authorizing checked module closure; TASK-2063 must await TASK-2069 and TASK-2064 owns parity.

## Description

Own the final Type-layer interface boundary omitted from TASK-2068: complete body checking,
private/public fact retention, export closure, and final public-use projection. This task alone may
turn staged facts into a versioned final interface; it still cannot lower, admit, or execute.

## Requirements

1. Check every supported body and callable/namespace fact against complete provisional and binding
   inputs while preserving canonical provenance and diagnostic anchoring.
2. Keep private checked facts distinct from public projections and validate complete export closure,
   including final `pub use`, only after all inputs succeed.
3. Reject stale, forged, incomplete, failed, cyclic, or export-inconsistent dependencies before any
   final interface publishes.
4. Establish normalized Type-layer file/inline final-interface parity.

## TDD Steps

1. Add red complete-body/private-public collection tests.
2. Add red final `pub use`/export-closure and stale/forged/incomplete rejection tests.
3. Add red atomic-finalization, normalized file/inline final-interface parity, generated closure,
   and authority-fence tests.
4. Implement after RED, run focused Type tests/quality gates, then promote actual evidence only.

## Scope and non-goals

This task excludes parser acquisition/graph construction, import grammar/binding ownership,
Core/CPS lowering, Engine transport/link/admission/execution, direct evaluation, and CLI/daemon
terminal parity.

## Handoffs and completion checklist

- **Consumes:** TASK-2071 complete provisional facts and TASK-2072 atomic resolved bindings/staged
  `pub use` facts.
- **Produces:** complete versioned final checked module/interface/export closure, non-authorizing.
- **Downstream owner:** TASK-2069 exclusively consumes this handoff for source-to-Core/CPS and
  Engine transport fencing; TASK-2063 awaits TASK-2069; TASK-2064 consumes TASK-2073/2069/2063.
- **Integration/proof:** TASK-2064 proves end-to-end file/inline/client terminal parity.
- [ ] Complete body/private/public/export-closure and file/inline interface evidence is recorded.
- [ ] No final interface is treated as an admission credential or execution fallback.
