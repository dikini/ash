# TASK-2075: Two-Tier Complete Module Collection

**Status:** In progress
**Phase:** [PLAN-207](../PLAN-207-COMPLETE-MODULE-REALIZATION.md)
**Spec:** SPEC-103 §§2, 5–8 (`M-COLLECT`)
**Owned rule:** MOD-REAL-003/004 complete internal snapshot and provisional name view
**Run-route impact:** prerequisite
**Semantic task record:** [TASK-2075](../semantic-task-records.json)
**Semantic coverage map:** [TASK-2075](../SEMANTIC-RULE-COVERAGE.md#task-2075-two-tier-complete-module-collection)
**Design:** [Two-Tier Complete Module Collection](../../plans/2026-08-04-task-2075-two-tier-complete-module-collection-design.md)
**Implementation plan:** [TASK-2075 implementation plan](../../plans/2026-08-04-task-2075-two-tier-complete-module-collection-implementation-plan.md)

## Description

Collect every target declaration from TASK-2074's expanded graph into two deliberately different
outputs: a checker-internal `CanonicalCollectedModuleSnapshot` and an import-facing, name-only
`CanonicalProvisionalNameView`. Preserve `CanonicalProvisionalModuleScopes` only as a compatibility
projection for TASK-2068/TASK-2070.

## Semantic authority and axes

**Implementation:** not_implemented
**Evidence:** none
**Parity:** below_spec

**Missing target-spec clauses:** No complete two-tier collector, exhaustive target definition coverage, declared-visibility carrier alignment, canonical namespace/collision implementation, constructor/member scoping, source-order/shape/span/origin retention, drift revalidation, file/inline normalized projection, sibling atomicity, generated property, or authority-fence evidence exists. TASK-2072 cannot consume the internal snapshot, and TASK-2073 cannot treat the name-only view as checked facts.

**Layers:** Type `not_implemented`; Core `not_applicable`; CPS `not_applicable`;
admission-runtime `not_applicable`; verification `not_implemented`.

**Next obligation:** Implement the visibility-carrier prerequisites and the minimal canonical collection module that satisfies the approved RED domain contract, then continue the linked TDD plan without broadening into TASK-2072, TASK-2073, or TASK-2064 authority.

## Approved exhaustive RED checkpoint

`python3 -m unittest tools.docs.tests.test_task_2071_module_namespace_contract` verifies this
task lifecycle and evidence accounting. The new
`crates/ash-typeck/tests/task_2075_two_tier_module_collection.rs` target defines an approved
22-row domain table (all 21 current `Definition` variants plus `ModuleDecl`), proves exact
membership against `CanonicalDeclarationKind::ALL`, keeps `Impl` internal-only, requests separate
read-only internal/name-view APIs, and requires error-only atomic rejection for a supported sibling
paired with removed `Capability` syntax. Its focused command is now part of record verification:

```bash
cargo test -p ash-typeck --test task_2075_two_tier_module_collection
```

The command fails only with `E0432` because `ash_typeck::canonical_module_collection` does not yet
exist. This is intentional RED contract evidence, not passing test evidence: implementation remains
`not_implemented`, evidence remains `none`, the verification layer remains `not_implemented`, and
`TEST-MOD-REAL-003-004-COLLECTION-DOMAIN` remains deferred.

## Requirements

1. Cover all 21 current `Definition` variants plus structural `ModuleDecl`; reject `Capability` as
   removed target syntax.
2. Before complete collection, add or retain declared visibility carriers for policy, role, law,
   and proof as required by SPEC-103. Never infer public or inherited visibility.
3. Implement the canonical identity and lookup keys, every minimum namespace bucket, within-bucket
   duplicate rejection, cross-bucket ambiguity rule, parent-scoped members/constructors, and
   full-interface-application impl coherence.
4. Preserve expanded raw declaration/callable shapes, bodies/member spans, expansion origins and
   hygiene, and source ordinals only in `CanonicalCollectedModuleSnapshot`; store only the
   name/identity/namespace/visibility/exportability/origin-anchor/ordinal subset in
   `CanonicalProvisionalNameView`.
5. Keep impl entries internal-only; apply the specified constructor, macro, notation, policy, role,
   row, proposition, and evidence rules.
6. Rebuild and compare all collection inputs and reject name, kind, visibility, signature, body,
   order, or expansion-sidecar drift. Publish neither view if any sibling fails.
7. Establish normalized Type-layer file/inline projection equivalence and preserve the bounded
   TASK-2068/TASK-2070 compatibility behavior.

## TDD steps

1. Add an exhaustive RED variant table for 21 `Definition` variants plus `ModuleDecl`, including
   `Capability` rejection and visibility-carrier failures.
2. Add RED identity, namespace, collision, constructor/member, ambiguity, and impl-overlap tests.
3. Add RED source-order, raw-shape/body/member-span, expansion-origin/hygiene, and source-anchor
   retention tests for the internal snapshot; assert their absence from the name view.
4. Add RED drift mutations for name, kind, visibility, signature, body, order, and expansion
   sidecars; add valid-sibling/failing-sibling atomicity.
5. Add RED file/inline normalized projection, generated declaration/namespace property, authority
   fence, and TASK-2068/TASK-2070 regression tests.
6. Implement only after RED; promote only actual source/test evidence.

## Scope and non-goals

No syntax expansion ownership, parsed general import binding, body/type checking, final public or
private interface, export closure, Core/CPS, Engine transport/admission/execution, or client parity.

## Handoffs and completion checklist

- **Consumes:** TASK-2071's normative contract and TASK-2074's complete expanded graph.
- **Produces:** atomic internal snapshot plus minimal provisional name view, both non-authorizing.
- **Downstream owner:** TASK-2072 consumes only the name view; TASK-2073 consumes the internal
  snapshot plus TASK-2072 staging; TASK-2069 waits for TASK-2073.
- **Integration/proof:** TASK-2064 owns composed parity.
- [ ] Exhaustive variant, visibility, namespace, collision, and member/constructor evidence exists.
- [ ] Drift, atomicity, file/inline, property, compatibility, and authority-fence evidence exists.
- [ ] Import-facing output contains no signature, callable, body, type, equation, final-export, or
      runtime-authority fact.
