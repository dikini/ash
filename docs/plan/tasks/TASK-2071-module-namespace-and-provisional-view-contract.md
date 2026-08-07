# TASK-2071: Module Namespace and Provisional View Contract

**Status:** Complete
**Semantic task classification:** semantic-contract-definition
**Phase:** [PLAN-207](../PLAN-207-COMPLETE-MODULE-REALIZATION.md)
**Spec:** SPEC-103 §§2, 5–8 (`M-SYNTAX-PREPASS`, `M-EXPAND`, `M-COLLECT`)
**Owned rule:** MOD-REAL-001–004 syntax-prepass, expansion, namespace, identity, collision, and provisional-view contract
**Run-route impact:** prerequisite
**Semantic task record:** [semantic-task-records.json](../semantic-task-records.json)
**Semantic coverage map:** [TASK-2071](../SEMANTIC-RULE-COVERAGE.md#task-2071-module-namespace-and-provisional-view-contract)

## Description

Define the normative boundary that was missing before complete module expansion and collection can
be implemented. This task specifies the canonical expanded graph, the checker-internal collected
snapshot, the import-facing name-only view, declaration identity and collision rules, syntax-import
ordering, constructor/member treatment, and retained declaration visibility for the target
definition domain.

This is a completed specification handoff, not an implementation task. It creates no Rust carrier,
collector, import binding, checked interface, lowering artifact, or runtime authority.

## Semantic authority and axes

**Implementation:** not_implemented
**Evidence:** none
**Parity:** below_spec

**Missing target-spec clauses:** TASK-2071 completes the normative syntax-prepass, expansion, namespace, and provisional-view contract only. TASK-2074 must implement the AST-only syntax prepass and one-to-one `CanonicalExpandedModuleGraph`; TASK-2075 must implement `CanonicalCollectedModuleSnapshot` and the name-only `CanonicalProvisionalNameView`, including complete revalidation, file/inline normalized projection, and atomic failure. TASK-2072 must consume only the provisional name view for parsed import binding; TASK-2073 must consume the internal snapshot plus TASK-2072 staging for checked finalization. No production implementation or test, proof, lowering, admission, runtime, or parity evidence is supplied by this contract task.

**Layers:** Type `not_implemented`; Core `not_applicable`; CPS `not_applicable`;
admission-runtime `not_applicable`; verification `not_implemented`.

**Non-goals:** Rust carriers or behavior, binding, body checking, final interfaces, Core/CPS, Engine transport/admission/execution, and client parity.

**Next obligation:** TASK-2074 is complete for its non-authorizing parser-stage handoff while the broader target remains partial/tested/below-spec. TASK-2075 is complete for its partial/tested/below-spec paired collection handoff; TASK-2072 consumes the name-only view, and TASK-2073 consumes the internal snapshot plus TASK-2072 staging.

## Normative contract

### Expansion and collection carriers

1. `CanonicalExpandedModuleGraph` owns one `CanonicalModuleGraph` and exactly one shallowly
   expanded `ModuleBody` per canonical `ModuleKey`. It retains parsed uses, source order, and
   per-module expansion origin/hygiene sidecars. Inline-child sidecars belong only to the child key.
2. `CanonicalCollectedModuleSnapshot` is checker-internal. It may retain raw expanded declaration
   and callable shapes, bodies/member spans, source ordinals, and expansion origin/hygiene, but no
   checked type or body result.
3. `CanonicalProvisionalNameView` is import-facing. An entry contains only lookup key/name,
   defining identity and `ModuleKey`, namespace kind, declared visibility/exportability,
   origin/source anchor, and source ordinal. It contains no signature, callable/body/type/equation,
   final-export, or runtime-authority fact.
4. `CanonicalProvisionalModuleScopes` remains a compatibility facade/projected behavior for the
   completed TASK-2068/TASK-2070 routes. It is not the complete collector.

### Identity and collision

- Canonical identity key: `(ModuleKey, declaration kind, canonical parent, origin key)`.
- Lookup key: `(namespace bucket, visible local key)`.
- Duplicates reject within one collision bucket. Cross-bucket spelling is allowed unless the
  referenced syntax context cannot select a bucket; that reference then rejects as ambiguous.
- Nested members collide only within their canonical parent. Implementation coherence compares the
  full canonical interface application for overlap, never only a spelling.
- Required buckets are structural module; type/domain; type computation; promoted kind;
  value/callable/eligible constructor; interface; row name; proposition; macro; notation;
  implementation registry; and evidence. Dedicated role/policy forms are removed and are not
  collection namespaces.

### Definition-domain decisions

- `ModuleDecl` is structural. `Capability` is removed target syntax and rejects.
- `ResourceType`, `Type`, `Newtype`, and `SealedDomain` names share type/domain collision.
  `TypeFn` remains separate. `EffectAlias` and `EffectGroup` share the row-name bucket.
- `Function`, `Handler`, `BuiltinFn`, and eligible ordinary/newtype constructors share the value
  bucket. Sealed-domain constructors remain parent-scoped and non-standalone; promoted
  constructors remain parent-scoped/type-level.
- Interface and implementation members are parent-scoped. `Impl` is internal-only and absent from
  the provisional view. Macro-generated identifiers are hygienic and not source-spellable keys;
  item-generating macros are unsupported.
- A macro key is its name. A notation key is normalized pattern plus fixity and precedence and
  obeys notation-overlap rules.
- Module law and proof use the evidence namespace and allow imports only when explicitly visible.
  Collection preserves declared visibility for retained declarations and never infers missing
  visibility. Dedicated role/policy forms are outside this contract.

### Syntax prepass

Before expansion, an AST-only prepass gathers public macro/notation summaries, resolves only syntax
imports through canonical module keys and parsed `Use` spans, rejects syntax-dependency cycles, and
topologically expands providers before consumers. It creates no general import binding or runtime
authority. It cannot read source text, the filesystem, path caches, or the Engine module loader.
Imported notation requires a canonical summary and remains inactive without one.

## Handoffs

- **Consumes:** SPEC-103, TASK-2067's canonical parsed graph, TASK-2068/TASK-2070 compatibility
  facts, and the current parser definition domain.
- **Produces:** the normative expansion, namespace, identity, collision, visibility, and two-view
  collection contract implemented separately by TASK-2074 and TASK-2075.
- **Downstream owner:** TASK-2074 owns the expanded graph; TASK-2075 owns collection; TASK-2072
  owns import resolution/binding; TASK-2073 owns checked finalization/export closure.
- **Does not own:** any Rust carrier or behavior, parsed binding, body checking, final interface,
  Core/CPS lowering, Engine transport/admission/execution, or client parity.
- **Integration/proof responsibility:** TASK-2064 owns composed file/inline and client parity.

## TDD and activation steps

1. TASK-2074 was separately activated with its own semantic record, coverage section,
   traceability nodes, and parser-focused TDD evidence; it is now complete for that non-authorizing
   handoff. TASK-2075 is independently complete for its paired collection handoff with exact
   `**Status:** Complete` and `partial / tested / below_spec` accounting.
2. Completed TASK-2074 publishes no partial expanded graph.
3. TASK-2075 follows its typechecker-focused TDD plan and publishes neither view on any failure.
4. TASK-2072 and TASK-2073 update their activation records to consume only their declared views.

## Completion checklist

- [x] SPEC-103 defines syntax-only prepass ordering and the canonical expanded graph.
- [x] SPEC-103 separates the internal snapshot from the import-facing name-only view.
- [x] Identity, namespace, collision, constructor/member, and retained-visibility rules are explicit.
- [x] TASK-2074 and TASK-2075 have separate bounded task and TDD plan documents.
- [x] No implementation, test, proof, or parity evidence is fabricated.

## Documentation verification

- `python3 -m unittest tools.docs.tests.test_task_2071_module_namespace_contract`
- `python3 tools/docs/validate_semantic_task_records.py --root . --manifest docs/plan/semantic-task-records.json`

These commands validate the documentation contract and record only. Runtime evidence remains
`none`.
