# TASK-2074: Canonical Expanded Module Graph

**Status:** In progress
**Phase:** [PLAN-207](../PLAN-207-COMPLETE-MODULE-REALIZATION.md)
**Spec:** SPEC-103 §§2, 5, and 8 (`M-SYNTAX-PREPASS`, `M-EXPAND`)
**Owned rule:** MOD-REAL-001/002 canonical syntax dependency and expanded graph handoff
**Run-route impact:** prerequisite
**Semantic task record:** [TASK-2074](../semantic-task-records.json)
**Semantic coverage map:** [TASK-2074](../SEMANTIC-RULE-COVERAGE.md#task-2074-canonical-expanded-module-graph)
**Design:** [Canonical Expanded Module Graph](../../plans/2026-08-04-task-2074-canonical-expanded-module-graph-design.md)
**Implementation plan:** [TASK-2074 implementation plan](../../plans/2026-08-04-task-2074-canonical-expanded-module-graph-implementation-plan.md)
**Notation-import design:**
[Parenthesized Notation Import](../../plans/2026-08-04-task-2074-parenthesized-notation-import-design.md)
**Notation-import implementation plan:**
[Parenthesized Notation Import implementation plan](../../plans/2026-08-04-task-2074-parenthesized-notation-import-implementation-plan.md)

## Description

Build the parser-owned expansion boundary between the completed canonical parsed graph and complete
module collection. The complete target carrier consumes and owns `CanonicalModuleGraph`, performs a syntax-only
macro/notation import prepass, shallowly expands each keyed `ModuleBody`, preserves uses and source
order, and publishes an exact one-to-one expanded module map only when the whole graph succeeds.

## Semantic authority and axes

**Implementation:** partial
**Evidence:** tested
**Parity:** below_spec

**Missing target-spec clauses:** The delivered bounded parser expansion is `partial / tested / below_spec`: public `CanonicalExpandedModuleGraph` consumes the exact parsed graph and performs an AST-only prepass for invocation-backed simple canonical public macro imports, public structural provider paths, macro-namespace priority, duplicate-alias rejection, deterministic provider ordering and syntax-cycle provenance, transitive provider closure, provider-owned diagnostics, and read-only syntax-import provenance sidecars. It preserves uses, module declarations, source order, per-key sidecars, exact keys, and atomic failure; unsupported item-generation attempts reject as required. Syntax-prepass evidence is 17/17, shallow-graph evidence is 5/5, and `ash-parser` library evidence is 463/463. The approved 8/8 completion target additionally tests normalized file/inline child projections, acquired typed units after all source files are overwritten and deleted, alias/provider-template mutations, ordinary callable-import notation nonactivation, anchored graph-wide atomic nonmacro rejection, a direct-orchestration/manifest authority fence, and an exhaustive 64-case projection. Parenthesized notation-import selector syntax has 12/12 focused evidence, structured declaration parsing has 3/3 evidence, and a separate resolver witness confirms notation imports remain syntax-only and create no ordinary binding. Active parser and LSP identity consumers use a typed, span-free token/hole key rather than diagnostic raw text; rendering is confined to textual boundaries. Valid-path canonical public notation-summary transport has 3/3 focused evidence, while notation dependency validation has 12/12 focused evidence with typed atomic failures, mixed macro/notation cycle provenance, same-class conflict rejection, compatible cross-class transport, and conflict-group-local anchors. Prepass-validated rows activate only in their importing module, including imported-macro output, and retain declaration/use provenance and macro-to-notation origin ancestry without callable binding or scope leakage; the focused notation-import target passes 21/21. The complete TASK-2075 handoff remains absent pending an independent completion audit. This parser-stage evidence creates no filesystem, Engine, raw-text semantic authority, general binding, checked-interface, Core/CPS, runtime, proof, final-interface, or client-parity authority. TASK-2074 remains partial / tested / below_spec.

**Layers:** Type `partial`; Core `not_applicable`; CPS `not_applicable`;
admission-runtime `not_applicable`; verification `partial`.

## Delivered bounded expansion evidence

- **Implementation:** `IMPL-MODULE-CANONICAL-EXPANDED-GRAPH`,
  `IMPL-MODULE-CANONICAL-SYNTAX-PREPASS`, and `IMPL-MODULE-SHALLOW-BODY-EXPANSION`.
- **Positive:** `TEST-MOD-REAL-001-002-LOCAL-SHALLOW-ORDER`,
  `TEST-MOD-REAL-001-002-INLINE-SIDECAR-OWNERSHIP`,
  `TEST-MOD-REAL-001-002-EXACT-KEY-ATOMIC-PUBLICATION`, and
  `TEST-MOD-REAL-001-002-GENERATED-SHALLOW-ORDER-PROPERTY`.
- **Negative:** `TEST-MOD-REAL-001-002-ANCHORED-LATE-EXPANSION-FAILURE`.
- **Mutation:** `TEST-MOD-REAL-001-002-MISSING-DEFINITION-CARDINALITY` and
  `TEST-MOD-REAL-001-002-EXTRA-DEFINITION-CARDINALITY`.
- **Syntax-prepass positive:** `TEST-MOD-REAL-001-002-LOCAL-PUBLIC-MACRO`,
  `TEST-MOD-REAL-001-002-CANONICAL-PUBLIC-MACRO-ALIAS`,
  `TEST-MOD-REAL-001-002-PROVIDER-ORDER`,
  `TEST-MOD-REAL-001-002-TRANSITIVE-PROVIDER-CLOSURE`,
  `TEST-MOD-REAL-001-002-SYNTAX-IMPORT-PROVENANCE`,
  `TEST-MOD-REAL-001-002-MACRO-NAMESPACE-PRIORITY`, and
  `TEST-MOD-REAL-001-002-PUBLIC-MACRO-ALIAS-PROPERTY`.
- **Syntax-prepass negative:** `TEST-MOD-REAL-001-002-PRIVATE-MACRO`,
  `TEST-MOD-REAL-001-002-PRIVATE-STRUCTURAL-PATH`,
  `TEST-MOD-REAL-001-002-NON-MACRO-SYNTAX-IMPORT`,
  `TEST-MOD-REAL-001-002-MISSING-MACRO-SUMMARY`,
  `TEST-MOD-REAL-001-002-DUPLICATE-MACRO-ALIAS`,
  `TEST-MOD-REAL-001-002-PROVIDER-OWNED-DIAGNOSTIC`,
  `TEST-MOD-REAL-001-002-NOTATION-NONLEAKAGE`, and
  `TEST-MOD-REAL-001-002-ITEM-GENERATION-REJECTION`.
- **Syntax-prepass mutation:** `TEST-MOD-REAL-001-002-TWO-MODULE-SYNTAX-CYCLE` and
  `TEST-MOD-REAL-001-002-THREE-MODULE-SYNTAX-CYCLE`.
- **Syntax-prepass verification:**
  `cargo test -p ash-parser --test task_2074_canonical_syntax_prepass` passed 17/17, including an
  exact 16-case key-order property.
- **Focused verification:** `cargo test -p ash-parser --test task_2074_canonical_expanded_module_graph`
  passed 5/5, including an exact 16-case property.
- **Private invariant verification:** `cargo test -p ash-parser --lib` passed 463/463, including
  the two separate missing/extra definition-cardinality units.
- **Regression verification:** `cargo test -p ash-parser --test task_1725_expanded_surface_boundary`;
  `cargo test -p ash-parser --test task_1732_local_notation_table_resolution`;
  `cargo test -p ash-parser --test task_1755_macro_registry_scope`;
  `cargo test -p ash-parser --test task_1756_expression_macro_expansion`;
  `cargo test -p ash-parser --test task_1757_macro_origin_hygiene`;
  `cargo test -p ash-parser --test task_1769_hygienic_binder_macros`;
  `cargo test -p ash-parser --test task_2059_file_inline_module_unit_parity`; and
  `cargo test -p ash-parser --test task_2067_canonical_module_graph` passed 56/56 in aggregate
  (6 + 8 + 7 + 6 + 6 + 3 + 8 + 12).
- **Macro summary/identity regressions:**
  `cargo test -p ash-parser --test task_1763_macro_summary_carriers` and
  `cargo test -p ash-parser --test task_1786_macro_identity` passed 6/6 in aggregate (2 + 4).
- **Quality verification:** `cargo clippy -p ash-parser --all-targets --all-features -- -D warnings`,
  `cargo fmt --check`, and `git diff --check` passed.
- **Proof/parity:** no proof. The approved normalized file/inline child projection is parser-stage
  test evidence only; it is not a final-interface, lowered/admitted/runtime, or CLI/daemon parity
  relation. TASK-2064 separately owns composed parity.
- **Fingerprints:** graph `sha256:719121d36c1631cec5d8eac86f3180f06d83f263bc18351588ff5c4cda9143cb`;
  syntax prepass `sha256:4a56c3500c4c794b73a80bf7e7360a82eb463b242aa62a29f4c6c7bcdff5d59f`;
  shallow seam `sha256:86fda1f445a828acb8685786acee20594f4384964c5a819ee2514fe2ae395053`;
  exports `sha256:7640971d936f5189e7319a2f09ed9a9419b6c932a59484685db74ab5e3c2a53a`;
  syntax-prepass test `sha256:76929348ca710fc260742213494cd6eaab3a5454e11dabc5d786ba38ff3426dc`;
  shallow-graph test `sha256:2e81aebd0a0bbc1bfff113270fcd48eb395a690809389f9f732057e8d400fe0e`.

**Next obligation:** Independently audit the complete notation-inclusive expanded graph and promote its atomic TASK-2075 handoff only if every TASK-2074 contract clause remains satisfied. TASK-2075 remains planned and inactive until that audit completes.

## Tested parenthesized notation-import parser

- **Implementation:** `IMPL-MODULE-NOTATION-IMPORT-PARSER` supplies shared structured
  `NotationPatternPart` declaration/import parts, `NotationImportSelector`, and the syntax-only
  `UsePath::Notation` carrier. `IMPL-MODULE-STRUCTURED-NOTATION-PATTERN-KEY` supplies the
  typed, deterministic span-free token/hole identity used by active parser and LSP consumers;
  its renderer is restricted to textual boundaries, while raw spelling and the legacy token cache
  remain diagnostic only.
- **Evidence:** `TEST-MOD-REAL-001-002-NOTATION-IMPORT-PARSER` passed 12/12 for exact symbolic and
  mixfix parts/spans, trivia normalization, `_` disambiguation, `as` inside a selector, exact `(*)`
  versus ordinary `::*`, malformed selectors, aliases, visibility rejection, and ordinary-import
  regressions.
- **Typed-key evidence:** `TEST-MOD-REAL-001-002-TYPED-NOTATION-KEY` distinguishes a hole from the
  literal `_` token and one token containing a space from two adjacent tokens, preventing rendered
  spelling from becoming semantic equality or hash authority.
- **Boundary:** this parser milestone does not collect, transport, match, or activate provider
  summaries and does not bind or authorize their callable targets.
- **Raw-consumer audit:** the remaining production `pattern.raw` reads are presentation-only: the
  LSP document-symbol label and hover declaration rendering. Parser tests retain raw/token checks
  solely for diagnostic backward compatibility. Matching, completion identity, navigation, symbol
  indexing, local notation conflicts, and syntax-prepass classification use structured parts.
- **Resolver characterization:**
  `TEST-MOD-REAL-001-002-NOTATION-IMPORT-RESOLVER-FENCE` passes separately and confirms a notation
  use creates no ordinary binding even when the provider exports both the target callable and an
  operator-spelled ordinary name. The witness passed immediately against the already-added no-op
  resolver arm, so it is characterization evidence rather than a RED/GREEN claim.
- **Verification:** `cargo test -p ash-parser --test task_2074_parenthesized_notation_import_parser`;
  `cargo test -p ash-parser parse_use::tests`;
  `cargo test -p ash-parser --test task_1730_notation_declaration_parser_ast`;
  `cargo test -p ash-parser --test task_2067_canonical_module_graph`;
  `cargo test -p ash-parser import_resolver::tests::notation_import_creates_no_ordinary_binding_or_callable_authority`;
  and `cargo test -p ash-parser --lib` passed 12/12, 14/14, 3/3, 12/12, 1/1, and 463/463
  respectively.
- **LSP verification:** `cargo check -p ash-lsp-core` and
  `cargo clippy -p ash-lsp-core --lib --all-features -- -D warnings` passed. The requested
  `cargo test -p ash-lsp-core` and all-target LSP clippy remain blocked by the pre-existing
  `crates/ash-lsp-core/src/symbols.rs:382` test-only `ModuleFile` initializer missing
  `crate_metadata`; TASK-2074 does not modify that unrelated fixture.

**Parser fingerprint:** `parse_use.rs`
`sha256:b6c07de72c55f9a17baf6e347b891f2a3c2fc46b63e0617f14a696a683d25e51`;
focused test
`sha256:d8817775a8ccd3f3b796c8f91c1185aa7cc1c4be64b00e18e0e731faf4e4c385`.

## Tested canonical notation-summary carrier

- **Implementation:** `IMPL-MODULE-CANONICAL-NOTATION-SUMMARY-CARRIER` collects direct public
  notation declarations from acquired typed provider ASTs, exact-matches structured token/hole
  selectors, and transports every matching full-fixity variant into a private expanded-record
  sidecar. The read-only carrier retains the typed key, callable target, visibility, declaration
  span, provider key/path/artifact origin, and exact consumer use span.
- **Evidence:** `TEST-MOD-REAL-001-002-CANONICAL-NOTATION-SUMMARY` passed 3/3, covering exact public
  symbolic and mixfix summaries/provenance, all six source-order permutations, and generated source
  trivia. Matching never reads or reparses diagnostic raw spelling.
- **Boundary:** This valid-path slice creates no ordinary binding, callable authority, notation
  activation, generalized mixfix expression parsing, invalid-edge diagnostic, Core/CPS/runtime, or
  parity claim. TASK-2074 remains `partial / tested / below_spec`.
- **Verification:** `cargo test -p ash-parser --test task_2074_canonical_notation_import` passed
  3/3.
- **Fingerprints:** carrier/prepass
  `sha256:4a56c3500c4c794b73a80bf7e7360a82eb463b242aa62a29f4c6c7bcdff5d59f`;
  expanded graph `sha256:719121d36c1631cec5d8eac86f3180f06d83f263bc18351588ff5c4cda9143cb`;
  typed pattern parts `sha256:ac458fb5f91c8ab07b524b322849532c7acefed9dcef9615bbb133881b499b37`;
  public exports `sha256:7640971d936f5189e7319a2f09ed9a9419b6c932a59484685db74ab5e3c2a53a`;
  focused test `sha256:d7b55e6ed5b62a098e41754ed8aeb34117d6f3d8eafb6f4595dc04ad84d88fb4`.

## Implemented parenthesized notation-import contract

- **Implemented dependency validation:** `IMPL-MODULE-CANONICAL-NOTATION-IMPORT` rejects private
  structural paths and declarations, missing exact summaries, local/imported and imported/imported
  pattern conflicts, and combined macro/notation cycles atomically. Typed failures retain complete
  consumer/provider source and artifact context, exact use spans, and applicable declaration spans.
- **Tested evidence:** `TEST-MOD-REAL-001-002-NOTATION-DEPENDENCY-REJECTION` passes 12/12,
  including compatible local/imported and cross-provider prefix/infix variants.
- **Implemented activation:** `IMPL-MODULE-IMPORTED-NOTATION-ACTIVATION` installs only canonical
  prepass-validated rows into the importing module's existing notation table. It does not rescan
  source, re-resolve paths, bind callables, or create runtime/admission authority.
- **Tested activation:** `TEST-MOD-REAL-001-002-IMPORTED-NOTATION-ACTIVATION` covers supported
  operator-section contexts, full-key carrier retention including mixfix, consumer-local scope,
  imported-macro composition with macro-to-notation origin ancestry, declaration/use-order
  determinism, and callable
  import nonactivation. The focused notation-import target passes 21/21.
- **Deferred completion:** `TEST-MOD-REAL-001-002-EXPANDED-GRAPH-COMPLETION` remains deferred until
  an independent completion audit promotes the whole atomic handoff.
- The selector is one exact normalized parsed token/hole pattern and carries no fixity,
  associativity, or precedence. `NotationPattern.raw` and selector raw spelling are diagnostic
  only and must never be reparsed or scanned as semantic matching authority.
- Every eligible public provider variant for the selected pattern is transported deterministically
  with its full key: pattern, fixity, associativity, and precedence. Target callable identity and
  provider/use provenance are retained without binding or authorizing the callable.
- Notation imports support neither `as` nor a notation glob. Ordinary callable imports never
  activate notation. Missing, private, malformed, conflicting, and cyclic dependencies reject the
  graph atomically with the required declaration/use/cycle anchors.
- A provider exports the summary only through `pub` on its notation declaration. TASK-2074 supports
  only inherited-visibility `use module::(pattern)`; `pub use module::(pattern)` and every other
  visibly qualified notation use reject as unsupported. Notation re-export needs a separate future
  contract and owner.
- Activation installs imported summaries in the existing syntax-phase table and preserves hole
  order for downstream resolution. TASK-2074 does not own generalized mixfix use-site parsing or
  elaboration.

## Approved completion-test checkpoint

`cargo test -p ash-parser --test task_2074_expanded_graph_completion` passed **8/8**. This
approved non-notation checkpoint supplies bounded parser-stage test evidence for:

- normalized file/inline child projection parity;
- expansion from acquired typed graph units after source files are overwritten and deleted, with no
  expansion reread;
- observable alias and provider-template mutations;
- ordinary callable import nonactivation of provider notation;
- anchored graph-wide atomic rejection of a nonmacro syntax edge;
- the direct orchestration/manifest authority fence, including no loader, scanner, filesystem, or
  later-layer dependency; and
- exhaustive 64-case depth, source-form, declaration-order, alias, provider-template, and
  function-count projection.

The test does not transport or activate a canonical public notation summary. It does not prove
the target rule or establish final-interface, lowered/admitted/runtime, or client parity; therefore
TASK-2074 remains **In progress** and `partial / tested / below_spec`.

**Fingerprint:** `task_2074_expanded_graph_completion.rs`
`sha256:897c979d1ff025beea266f9ae1633adc43b7e83c61298898ed6b5185264ef347`.

## Tested bounded syntax-prepass target

- **Command:** `cargo test -p ash-parser --test task_2074_canonical_syntax_prepass`.
- **Current state:** GREEN, 17/17, including one exact 16-case key-order property.
- **Delivered boundary:** invocation-backed simple canonical public macro aliases; public
  structural provider visibility; deterministic provider-first ordering and cycle provenance;
  transitive closed-provider consumption; macro-namespace priority; duplicate-alias rejection;
  provider-owned failure context; and read-only syntax-import provenance sidecars.
- **Fail-closed boundary:** notation without a canonical public summary remains inactive, and
  unsupported item-generation attempts reject as required by SPEC-103 §5. This evidence does not
  authorize general binding, filesystem discovery,
  raw-source fallback, Engine behavior, runtime behavior, or parity.

## Requirements

1. Add a parser-owned `CanonicalExpandedModuleGraph` that consumes/owns the input
   `CanonicalModuleGraph` and exposes exactly one expanded module record per parsed `ModuleKey`.
2. Add a shallow `ModuleBody` expansion seam: expand only direct definitions owned by that key;
   retain parsed uses, nested structural declarations, and source order unchanged.
3. Gather public macro and notation summaries from AST only; notation declaration and import
   matching uses structured parsed token/hole parts, never reparsed raw spelling. Resolve only
   syntax imports through canonical keys and exact `Use` spans; reject syntax cycles; expand
   providers before consumers.
4. Parse inherited-visibility parenthesized exact notation selectors; reject visibly qualified
   notation uses, aliases, and globs; transport every
   eligible public full-key variant deterministically, and activate it in the existing syntax-phase
   table without binding or authorizing its target callable. Reject item-generating macro behavior
   outside the declared target domain.
5. Retain source path/artifact origin plus per-module expansion diagnostics, origins, and hygiene.
   Inline-child sidecars occur only in the child record.
6. Validate exact parsed/expanded key equality and fail atomically on prepass, dependency, expansion,
   or invariant errors. Never reuse `ash-engine` module loading.

## TDD steps

1. Add RED tests for the shallow `ModuleBody` API and direct-definition expansion.
2. Add RED AST-only macro/notation summary, topological-order, and syntax-cycle tests.
3. Add RED use/order retention, per-key inline sidecar, and file/inline normalization tests.
4. Add RED mutation, graph-wide atomicity, generated graph property, and no-filesystem/authority
   fence tests.
5. Run existing expansion and canonical-graph regressions before implementation.
6. Implement only after RED; promote only actual source/test evidence.

## Scope and non-goals

No generalized mixfix use-site parsing/elaboration, namespace collection, provisional view,
general import binding, body/type checking, final interface, Core/CPS lowering, Engine
transport/admission/execution, filesystem discovery, source text fallback, or client parity.

## Handoffs and completion checklist

- **Consumes:** TASK-2071's contract and TASK-2067's canonical parsed graph.
- **Target produces:** one complete atomic, parser-owned `CanonicalExpandedModuleGraph`,
  non-authorizing. The delivered bounded syntax-prepass slice is not yet this complete handoff.
- **Downstream owner:** TASK-2075 alone consumes the completed graph for collection; it remains
  planned and inactive while TASK-2074 is partial.
- **Integration/proof:** TASK-2064 owns composed parity.
- [x] Positive, negative, mutation, normalized file/inline child projection, property,
  acquired-graph no-reread, and direct-orchestration/manifest authority-fence evidence exists
  (8/8 completion target).
- [x] Focused completion evidence passes; previously recorded expansion and graph regressions remain
  task-owned verification.
- [x] The graph-wide nonmacro syntax-edge witness returns one anchored failure rather than a partial
  expanded graph.
- [x] Canonical public notation-summary transport and eligible notation activation are implemented
  and tested.
- [ ] The complete atomic expanded-graph handoff is ready for TASK-2075.
