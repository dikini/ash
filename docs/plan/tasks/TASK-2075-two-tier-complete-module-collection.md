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

**Missing target-spec clauses:** Visibility/carrier prerequisites plus Tasks 5–7 graph-wide atomic paired collection, internal-fact/minimal-view retention, and keyed/span-anchored drift revalidation are implemented and tested. Internal entries retain expanded raw definitions, callable bodies, nested member spans through direct source anchors, deterministic ordinals, and module-owned expansion/hygiene sidecars; the provisional view remains the exact name/identity/namespace/visibility/exportability/origin-anchor/ordinal subset. Impl coherence remains bounded to interfaces found in the current module or lexical canonical-module ancestors; imported interface binding fails closed until TASK-2072 supplies the defining identity. Normalized collected file/inline projection, generated-name suppression/property evidence, TASK-2068/TASK-2070 compatibility evidence, and the complete later-layer authority fence remain absent. TASK-2072 and TASK-2073 have non-authorizing carrier inputs but still own binding and finalization.

**Layers:** Type `partial`; Core `not_applicable`; CPS `not_applicable`;
admission-runtime `not_applicable`; verification `partial`.

**Next obligation:** Implement Task 8 normalized collected file/inline projection, generated-name/property, compatibility, and complete later-layer authority-fence evidence without broadening into TASK-2072, TASK-2073, or TASK-2064 authority.

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
so half-publication is unrepresentable. `CanonicalCollectedEntry` retains one raw definition,
derives callable bodies exhaustively instead of duplicating them, and exposes its retained source
anchor read-only. Construction remains private.

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

## Delivered Task 5 collection checkpoint

The graph collector now stages every canonical module before publishing one paired collection. It
classifies all supported top-level declarations and structural modules, retains source order and
module expansion/hygiene sidecars, assigns unique parent-scoped identities to constructors and
members, and keeps impl entries and members out of the provisional name view. Parent-aware
duplicates reject without suppressing valid cross-namespace spellings. Impl coherence uses a
module-qualified lexical interface identity and alpha-normalized, span-free full type applications;
computation rows are normalized as sets, and unresolved interface identity fails closed. Notation
lookup retains a public read-only typed pattern/fixity key rather than requiring display-string
reparsing. A late failing sibling discards the graph-wide staged result.

The full focused command is now required-success verification and passes 24/24:

```bash
cargo test -p ash-typeck --test task_2075_two_tier_module_collection
```

This test evidence covers exhaustive namespace disposition, paired publication, same-bucket and
parent-scoped collision behavior, constructor/member placement, typed notation identity,
internal-only impl members, alpha-renamed/permuted computation-row overlap, graph-wide sibling
impl coherence, unresolved-interface rejection, removed syntax, and late-sibling atomicity. It is
not proof, collected file/inline parity, drift revalidation, generated-name suppression, or final
import/interface authority. The Task 5 checkpoint itself passed 22/22; the Task 6 additions below
raise the current required-success target to 24/24.

**Fingerprints:** collector implementation
`sha256:b1773b14365d1029c7425d9f1d369cec82c8bc6216ec7b58eced7b3638ec966b`;
focused contract
`sha256:06e10776ed3eeabddc845b1fd25d05e8e0a623bbf2ed455ea0965a7049616df6`;
private validation tests
`sha256:6a1ecd290d6d451b27f1decbf81748d1ded89c528b8d4b80298a588c49ae7896`.

## Delivered Task 6 internal-fact and minimal-view checkpoint

Task 6 is implemented and tested within the task's overall `partial / tested / below_spec` axes.
The internal snapshot retains expanded raw callable shapes and bodies, interface/impl parent shapes,
nested member spans through `CanonicalCollectedEntry::source_anchor`, deterministic member/source
ordinals, and per-module expansion-origin and hygiene sidecars. Structural `ModuleDecl` entries also
retain a direct source anchor even though they intentionally have no raw `Definition`.

The syntax-aware carrier fence fixes the internal entry to private fields with read-only accessors
and keeps `CanonicalProvisionalNameEntry` at exactly eight private fields and eight read-only
accessors. The provisional view contains only name, identity, lookup key/namespace, visibility,
exportability, origin anchor, and source ordinal; it contains no raw definition, callable body,
signature, checked type, equation, final export, or runtime authority.

Positive evidence is the inline-child sidecar/raw-callable test and the nested interface/impl
member raw-fact/span/body/ordinal test. The representative carrier and structural-module cases
also compare the internal source anchor with raw or provisional anchors. These are tests, not a
proof, normalized collected file/inline parity, generated/property evidence, compatibility
evidence, imported-interface binding, or later-layer authority evidence.

## Delivered Task 7 revalidation and atomic-publication checkpoint

Task 7 is implemented and tested within the task's overall `partial / tested / below_spec` axes.
`CanonicalModuleCollection::revalidate_against` rebuilds the candidate expanded graph through the
same collector and compares module keys, declaration name/kind/visibility/signature/body/source
order, expansion origins, and hygiene sidecars before any replacement publication. Every drift
failure is reported as `SourceDrift` with the canonical module key, changed declaration name, and
candidate source span; a changed sibling leaves the already-published baseline pair intact.

The focused target now passes 32/32. Its eight Task 7 mutation cases cover name, kind, visibility,
signature, body, source order, expansion-sidecar, and valid-sibling/candidate-sibling drift. The
exact private carrier fence remains green, including the internal source-anchor accessor and the
provisional view's strict eight-field/eight-accessor boundary. This is test evidence, not a proof
or normalized file/inline projection, generated/property, TASK-2068/TASK-2070 compatibility,
imported-interface binding, or later-layer authority evidence.

**Fingerprints:** collector implementation
`sha256:67963434deb3d1f61dbb1323df7f983629ff4136a51152fe6c5cf2a76c22c3f7`;
focused contract
`sha256:0976312880548b30984f3627a883973a82b408adfa6378ba8299d4148797b6cf`.

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
- [x] Exhaustive variant, namespace, collision, and member/constructor evidence exists.
- [x] Internal raw facts, source anchors/ordinals, sidecars, and strict minimal-view evidence exists.
- [x] Keyed/span-anchored `SourceDrift` revalidation and sibling atomic-publication evidence exists.
- [ ] Normalized file/inline, generated/property, compatibility, and complete authority-fence evidence exists.
- [x] Import-facing output contains no signature, callable, body, type, equation, final-export, or
      runtime-authority fact.
