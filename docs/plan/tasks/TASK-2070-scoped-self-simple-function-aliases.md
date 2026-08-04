# TASK-2070: Scoped Self Simple Function Aliases

**Status:** Complete
**Phase:** [PLAN-207](../PLAN-207-COMPLETE-MODULE-REALIZATION.md)
**Spec:** SPEC-103 §§5-7 (`M-IMPORT-EDGE`, `M-IMPORT-CYCLE`, `M-BIND`)
**Owned rule:** MOD-REAL-004 bounded M-SELF-SIMPLE-ALIAS
**Run-route impact:** prerequisite
**Semantic task record:** [semantic-task-records.json](../semantic-task-records.json)
**Semantic coverage map:** [TASK-2070](../SEMANTIC-RULE-COVERAGE.md#task-2070-scoped-self-simple-function-aliases)

## Semantic authority and axes

**Implementation:** partial
**Evidence:** tested
**Parity:** below_spec
**Missing target-spec clauses:** The delivered M-SELF-SIMPLE-ALIAS slice is `partial / tested / below_spec`: it admits zero or more individually eligible inherited `UsePath::Simple` statements per root or nested module, each exactly `use self::<ordinary_function> as <different_alias>;` with two segments. A module with no such statement produces an empty dedicated result; every encountered `use` must be eligible, so groups, globs, mixed imports, and every other import form are `Unsupported`. A direct `self::<child_module>` target is a nonfunction `Unsupported` boundary, not a traversal route. Each candidate resolves only the importing module's direct same-`ModuleKey` ordinary function, requires an explicit different alias, and applies `is_visible_from` for that importer. Distinct aliases stage together; repeated aliases report `DuplicateBinding`, and a local collision reports `LocalDeclarationCollision`. The dedicated resolver returns `CanonicalResolvedSelfOrdinaryFunctionAliases`; only the dedicated binder calls its private `into_bound_alias_set` to return `CanonicalBoundSelfOrdinaryFunctionAliasSet`. Each `CanonicalSelfOrdinaryFunctionAliasBinding` retains the local alias, defining identity, declaration span, origin, exact declared visibility, and full `Use::span` as `use_span`. The result has no import-edge field, no `CanonicalSimpleImportEdge` is created, and no cycle check runs; `ImportCycle` is unreachable by construction and the tested source fence. Any invalid module atomically publishes no dedicated result. The route consumes only the canonical graph and provisional scopes, leaves `CanonicalBoundModuleBinding` and the generic binder unchanged, and does not use private M-CHECK facts as import authority. Cross-module traversal/edges/cycles, complete import grammar and precedence, final interfaces, Core/CPS, Engine, admission/runtime, and client parity remain deferred to TASK-2072 through TASK-2064. Type and verification remain partial; Core/CPS/admission-runtime are not_applicable; run-route impact is prerequisite.
**Record-mirrored exact missing clause:** The delivered M-SELF-SIMPLE-ALIAS slice is `partial / tested / below_spec`: its dedicated resolver and binder admit zero or more individually eligible inherited two-segment `use self::<ordinary_function> as <different_alias>;` statements in root or nested modules. They resolve only direct same-`ModuleKey` ordinary functions, apply exact declared visibility through `is_visible_from`, stage distinct aliases together, reject duplicate/local collisions atomically, and retain identity, declaration span, origin, visibility, and full `use_span`. `CanonicalResolvedSelfOrdinaryFunctionAliases` has no import-edge field; only the dedicated binder calls private `into_bound_alias_set` to produce `CanonicalBoundSelfOrdinaryFunctionAliasSet`. No `CanonicalSimpleImportEdge` or cycle check is created, and `ImportCycle` is unreachable by construction and the tested source fence. Groups, globs, mixed/other forms, direct child modules, and all out-of-domain shapes reject without publishing a result. The generic binder and private M-CHECK facts remain outside its authority. Focused evidence is 8/8, including the exact 16-case property with alias count `1..3`; predecessor evidence is 32/32 and `ash-typeck` library evidence is 477/477. Complete import grammar/precedence/cycles, final interfaces/export closure, Core/CPS, Engine, admission/runtime, and client parity remain deferred to TASK-2072 through TASK-2064.
**Layers:** Type `partial`; Core/CPS/admission-runtime `not_applicable`; verification `partial`.
**Next obligation:** TASK-2072 consumes this delivered non-authorizing Type handoff while completing parsed imports and atomic binding. TASK-2069 consumes only TASK-2073's complete checked handoff, and TASK-2064 owns integration parity.

## Description

Deliver the bounded M-SELF-SIMPLE-ALIAS leaf without widening it. The dedicated resolver and
binder admit zero or more individually eligible direct same-module aliases per
module, preserving target facts and full use spans without creating an import edge or cycle
behavior.

## Requirements

1. Accept zero or more individually eligible inherited, two-segment
   `use self::<ordinary_function> as <different_alias>;` statements in a root or nested module;
   distinct aliases bind together, while duplicate aliases report `DuplicateBinding`.
2. Resolve only the importer's direct ordinary-function target and apply same-`ModuleKey`
   `is_visible_from` before staging the alias.
3. Return only `CanonicalResolvedSelfOrdinaryFunctionAliases` and
   `CanonicalBoundSelfOrdinaryFunctionAliasSet`, connected by the resolver-private
   `into_bound_alias_set` used only by the binder; each
   `CanonicalSelfOrdinaryFunctionAliasBinding` exposes its local alias, defining identity,
   declaration span, origin, visibility, and full `Use::span` as `use_span`.
4. Encode no-edge behavior structurally: the resolved result has no `import_edges` field, and the
   route neither creates a `CanonicalSimpleImportEdge` nor runs a cycle check; it returns the
   shared `CanonicalStructuralImportError`, with `ImportCycle` unreachable by construction/source
   fence.
5. Reject groups, globs, mixed/other import forms, every stated shape/visibility boundary, and
   local collisions atomically across the graph; direct `self::<child_module>` is a nonfunction
   `Unsupported`, and a duplicate eligible alias must reach `DuplicateBinding`, not `Unsupported`.
6. Keep `CanonicalBoundModuleBinding`, the generic binder, and M-CHECK private facts out of this
   route.

## TDD Steps

1. Make the eight existing deferred witnesses red in
   `crates/ash-typeck/tests/task_2070_scoped_self_ordinary_function_aliases.rs`, including zero
   aliases, multiple distinct aliases, duplicate-alias `DuplicateBinding`, and direct-child-module
   `Unsupported`; verify RED with
   `cargo test -p ash-typeck --test task_2070_scoped_self_ordinary_function_aliases`.
2. Add only the dedicated alias binding/result types and
   `resolve_scoped_self_ordinary_function_imports_with_scopes`; prove provenance, `use_span`,
   no-edge structure, visibility, distinct aliases, duplicate diagnostics, and graph atomicity.
3. Add only `bind_scoped_self_ordinary_function_imports` plus its export; prove the generic binder
   and `CanonicalBoundModuleBinding` remain unchanged, and that shared
   `CanonicalStructuralImportError` keeps `ImportCycle` unreachable by the dedicated source fence.
4. Verify GREEN with `cargo test -p ash-typeck --test
   task_2070_scoped_self_ordinary_function_aliases`, then run `cargo fmt --check`, `cargo clippy
   -p ash-typeck --test task_2070_scoped_self_ordinary_function_aliases -- -D warnings`, and
   `git diff --check` before promotion.

## Verification evidence

- Focused TASK-2070 target: 8/8 passed, including the exact 16-case property with alias count
  `1..3`; the 32 predecessor tests and the `ash-typeck` library suite (477/477) also passed.
- Focused clippy completed with warnings denied; formatting and `git diff --check` were clean.
- Spec and Rust-quality review approved the bounded implementation.
- Recorded source fingerprints: planner
  `sha256:0e7131c8fa00458a6de421c6ef54e041d715df11b2ffca3af4fc8a01777e4025`, structural
  binder `sha256:a80da73c9c86b66237bcca59bb33b1494aa5f1bb1cce5e32d41fa29763518b76`, public
  exports `sha256:307975ee0f5da786a47068c0aef6ef00cc1e7d7f7674ca1ba774ec55711a303a`, and focused
  test `sha256:1e9aa6317f2fdc44257bc04398d7a2c155d0261e1bc85c7715e94370f8a499f0`.

## Scope and non-goals

The delivered M-SELF-SIMPLE-ALIAS slice excludes natural-name or equal self aliases, same-module
child traversal, direct child-module/nonfunction targets, cross-module import traversal/edges/
cycles, public/restricted use/re-exports, crate/super/unprefixed paths, groups/globs/mixed or
other import forms, successful duplicate or local-colliding bindings, final interfaces,
generic-binder changes, M-CHECK private-fact authority, Core/CPS, Engine, admission/runtime, and
parity. Zero or more individually eligible aliases remain in scope; direct child modules are
`Unsupported` and duplicate eligible aliases are rejected as `DuplicateBinding`.

## Handoffs and completion checklist

- **Consumes:** TASK-2067 canonical graph/unit facts and TASK-2068's completed provisional-scope
  foundation only.
- **Produces:** `CanonicalResolvedSelfOrdinaryFunctionAliases` and the binder-produced
  `CanonicalBoundSelfOrdinaryFunctionAliasSet` projection, or no result on any graph failure.
- **Downstream owner:** TASK-2072 owns complete import grammar, cross-module cycles, precedence,
  ambiguity/duplicates, and staged `pub use` facts; TASK-2069 consumes only TASK-2073.
- **Integration/proof:** TASK-2064 alone proves file/inline/client terminal parity.
- [x] The implementation node and all eight inherited deferred test nodes were promoted after
  focused tests passed.
- [x] Multiple distinct aliases bind; duplicate aliases report `DuplicateBinding`; no
  cross-module edge/cycle or generic-binder authority is introduced.
- [x] Dedicated bindings retain `use_span`; no resolved result has an `import_edges` field, only
  the binder uses private `into_bound_alias_set`, and no dedicated API returns
  `CanonicalResolvedSimpleImports` or `CanonicalBoundModuleSet`.
- [x] This task remains `partial / tested / below_spec` after delivery; it cannot complete
  MOD-REAL-004.
