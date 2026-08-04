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

**Implementation:** partial
**Evidence:** tested
**Parity:** below_spec

**Missing target-spec clauses:** The declared-visibility prerequisite and private/read-only carrier boundary are implemented and tested, including the closed disposition domain, layout-stable typed keys, mandatory paired map, module sidecars, derived callable-body query, and private removed-Capability rejection. No graph collector publishes either view. Namespace/collision behavior, constructor/member scoping, complete raw-fact population, drift revalidation, file/inline normalized projection, graph-wide sibling atomicity, generated property, compatibility, and authority-fence evidence remain absent. TASK-2072 and TASK-2073 still have no consumable collection handoff.

**Layers:** Type `partial`; Core `not_applicable`; CPS `not_applicable`;
admission-runtime `not_applicable`; verification `partial`.

**Next obligation:** Implement TASK-2075 Task 5 namespace classification, collision, parent/member, constructor, and impl-overlap semantics; replace the fail-closed collector stub with atomic paired publication so the representative focused test passes, then add the full focused command to required-success verification without broadening into TASK-2072, TASK-2073, or TASK-2064 authority.

## Delivered visibility-carrier checkpoint

This bounded Type-layer prerequisite is implemented and tested:

- `PolicyDef`, `RoleDef`, `LawDef`, and `ProofDef` require an explicit declared `Visibility`.
- Module-scope role, law, and proof declarations retain inherited, `pub`, `pub(crate)`,
  `pub(super)`, `pub(self)`, and exact `pub(in path)` visibility, and their spans cover the
  visibility prefix through declaration end.
- Interface-nested laws and impl-nested proofs remain inherited and parent-scoped. A visibility
  prefix on either nested form remains rejected.
- Policy has no active declaration grammar; its evidence is construction-only.

The focused `crates/ash-parser/tests/task_2075_collection_visibility_carriers.rs` target passes
5/5. This evidence implements only the declaration-carrier prerequisite. It does not implement or
test collection, the 22-row domain, namespaces, atomicity, normalized file/inline projection, or
authority fences.

**Fingerprints:** visibility carriers
`sha256:d3b70b78b3daf4fb0adee5d1eba58ba485c51ee8e14627a1ba4bd3db7614f911`;
module/nested parsing
`sha256:5f33c04c3df001094d6bf5d7ff2c7bbb9959b3a07ebc34595a6afd63ef53a1a3`;
focused test
`sha256:e60c75e3acb84167e90c1782911f6a781e0582e719ebd97a74f929d6d4c6019a`.

## Delivered private carrier checkpoint

`python3 -m unittest tools.docs.tests.test_task_2071_module_namespace_contract` verifies this
task lifecycle and evidence accounting.
`crates/ash-typeck/tests/task_2075_two_tier_module_collection.rs` target defines an approved
22-row domain table (all 21 current `Definition` variants plus `ModuleDecl`), proves exact
membership against `CanonicalDeclarationKind::ALL`, keeps `Impl` internal-only, requests separate
read-only internal/name-view APIs. Task 4 now implements the closed declaration, namespace, and
disposition enums; typed layout-stable identity, parent, lookup, and origin keys; eight named
private-field carriers; an exact eight-field/eight-accessor provisional-name entry; module-owned
expansion-origin and hygiene sidecars; and one mandatory map whose private record pairs both views
so half-publication is unrepresentable. `CanonicalCollectedEntry` retains one raw definition and
derives callable bodies exhaustively instead of duplicating them. Construction remains private.

The private `validate_definition_batch` shares the exhaustive no-wildcard definition
classification path with the graph entry point. Its two unit tests pass for a supported batch and
for exact keyed/name/span rejection of removed `Capability` syntax:

```bash
cargo test -p ash-typeck --lib canonical_module_collection::tests
```

The domain and two syntax-aware source-fence tests pass independently:

```bash
cargo test -p ash-typeck --test task_2075_two_tier_module_collection definition_domain_is_closed_exhaustive_and_has_one_collection_disposition_per_kind
cargo test -p ash-typeck --test task_2075_two_tier_module_collection carrier_source_fence_enforces_private_construction_and_exact_name_view
cargo test -p ash-typeck --test task_2075_two_tier_module_collection syn_fence_handles_adversarial_source_without_substring_false_results
```

The full focused command remains intentionally excluded from the manifest's required-success
verification:

```bash
cargo test -p ash-typeck --test task_2075_two_tier_module_collection
```

The command runs four tests: the domain and both source fences pass, while only
`representative_expanded_graph_publishes_separate_internal_and_name_only_views` fails at runtime
with the deliberate `CollectorNotImplemented` Task 5 boundary. This is passing carrier evidence,
not passing collection/publication evidence. The complete
`IMPL-MODULE-CANONICAL-TWO-TIER-COLLECTION`, representative
`TEST-MOD-REAL-003-004-COLLECTION-CARRIER-SHAPE`, graph-wide atomicity, and later witnesses
remain deferred. Add the full command only after production collection makes every test pass.

**Fingerprints:** carrier implementation
`sha256:51be177a4c64c8f944b004da57416d59ae257cb4e580b9bad18d2fad4fd580d3`;
focused contract
`sha256:9df0e45fade891d467cf742e36153f5105aca1b39acb31ebd5d8b737a9ff68a5`;
private validation tests
`sha256:6a1ecd290d6d451b27f1decbf81748d1ded89c528b8d4b80298a588c49ae7896`.

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
- [x] Declared visibility-carrier and module/nested parser evidence exists.
- [x] Private carrier/domain/source-fence and removed-`Capability` validation evidence exists.
- [ ] Exhaustive variant, namespace, collision, and member/constructor evidence exists.
- [ ] Drift, atomicity, file/inline, property, compatibility, and authority-fence evidence exists.
- [ ] Import-facing output contains no signature, callable, body, type, equation, final-export, or
      runtime-authority fact.
