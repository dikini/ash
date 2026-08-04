# TASK-2070: Scoped Self Simple Function Aliases

**Status:** In progress
**Phase:** [PLAN-207](../PLAN-207-COMPLETE-MODULE-REALIZATION.md)
**Spec:** SPEC-103 §§5-7 (`M-IMPORT-EDGE`, `M-IMPORT-CYCLE`, `M-BIND`)
**Owned rule:** MOD-REAL-004 bounded M-SELF-SIMPLE-ALIAS
**Run-route impact:** prerequisite
**Semantic task record:** [semantic-task-records.json](../semantic-task-records.json)
**Semantic coverage map:** [TASK-2070](../SEMANTIC-RULE-COVERAGE.md#task-2070-scoped-self-simple-function-aliases)

## Semantic authority and axes

**Implementation:** partial
**Evidence:** none
**Parity:** below_spec
**Missing target-spec clauses:** The planned M-SELF-SIMPLE-ALIAS slice is `partial / none / below_spec`: it will admit zero or more individually eligible inherited `UsePath::Simple` statements per root or nested module, each exactly `use self::<ordinary_function> as <different_alias>;` with two segments. A module with no such statement produces an empty dedicated result; every encountered `use` must be eligible, so groups, globs, mixed imports, and every other import form are `Unsupported`. A direct `self::<child_module>` target is a nonfunction `Unsupported` boundary, not a traversal route. Each candidate resolves only the importing module's direct same-`ModuleKey` ordinary function, requires an explicit different alias, and applies `is_visible_from` for that importer. Distinct aliases stage together; repeated aliases report `DuplicateBinding`, and a local collision reports `LocalDeclarationCollision`. The resolver returns `CanonicalResolvedSelfOrdinaryFunctionAliases`, which has no `import_edges` field and exposes alias bindings; its private `into_bound_alias_set` is callable only by the dedicated binder, and it never constructs or returns `CanonicalResolvedSimpleImports`. The private conversion and the dedicated binder return `CanonicalBoundSelfOrdinaryFunctionAliasSet`, never `CanonicalBoundModuleSet`. Each `CanonicalSelfOrdinaryFunctionAliasBinding` exposes its local alias, defining identity, declaration span, origin, declared visibility, and full `Use::span` as `use_span`. The resolver and binder use shared `CanonicalStructuralImportError`; `ImportCycle` is unreachable by construction and the source fence. Any invalid module atomically publishes no dedicated result. It consumes canonical graph and provisional scopes only, leaves `CanonicalBoundModuleBinding` and the generic binder unchanged, and does not use private M-CHECK facts as import authority. Cross-module traversal/edges/cycles, final interfaces, Core/CPS, Engine, admission/runtime, and parity remain deferred. Planned Type and verification are partial; Core/CPS/admission-runtime are not_applicable; run-route impact is prerequisite; TASK-2069 owns lowering and TASK-2064 owns parity. The deferred implementation node and eight deferred test witnesses cover zero/multiple aliases across root/nested same-module visibility regions, identity/provenance, no-edge/no-false-cycle, shape/visibility rejections including direct child modules, duplicate/local-collision and atomic valid-sibling failure, normalized Type-layer file/inline scope/binding parity, an exactly-16-case names/aliases/root-nested/source/visibility property, and the authority fence.
**Layers:** Type `partial`; Core/CPS/admission-runtime `not_applicable`; verification `partial`.
**Next obligation:** Implement this bounded self-alias route with its existing deferred node and eight witnesses, then hand its non-authorizing Type facts to TASK-2072's complete parsed-import/binding owner; TASK-2069 consumes only TASK-2073's complete checked handoff and TASK-2064 owns integration parity.

## Description

Take ownership of the existing planned M-SELF-SIMPLE-ALIAS leaf without widening it. The future
resolver and binder admit zero or more individually eligible direct same-module aliases per
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

## Scope and non-goals

The planned M-SELF-SIMPLE-ALIAS slice excludes natural-name or equal self aliases, same-module
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
- [ ] All eight inherited deferred trace nodes are promoted only after focused tests pass.
- [ ] Multiple distinct aliases bind; duplicate aliases report `DuplicateBinding`; no
  cross-module edge/cycle or generic-binder authority is introduced.
- [ ] Dedicated bindings retain `use_span`; no resolved result has an `import_edges` field, only
  the binder uses private `into_bound_alias_set`, and no dedicated API returns
  `CanonicalResolvedSimpleImports` or `CanonicalBoundModuleSet`.
- [ ] This task remains `partial / tested / below_spec` after delivery; it cannot complete
  MOD-REAL-004.
