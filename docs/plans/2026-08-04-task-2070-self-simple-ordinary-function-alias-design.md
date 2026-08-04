# TASK-2070 Self Simple Ordinary-Function Alias Design

**Status:** Active planning
**Task owner:** [TASK-2070](../plan/tasks/TASK-2070-scoped-self-simple-function-aliases.md)
**Scope:** M-SELF-SIMPLE-ALIAS
**Semantic accounting:** implementation `partial`; evidence `none`; parity `below_spec`.

## Decision

Add a dedicated Type-layer route for zero or more individually eligible inherited
`UsePath::Simple` statements per module: `use self::<ordinary_function> as <different_alias>;`.
Every accepted statement has exactly two segments and is valid in either the root or a nested
module. A module with no accepted statement produces an empty dedicated result; a module with any
group, glob, mixed, or other `use` form rejects as `Unsupported`. A direct
`self::<child_module>` target is a nonfunction `Unsupported` boundary, not a traversal route.
Each accepted statement selects only the importing module's direct ordinary function and preserves that function's defining
identity, declaration span, origin, declared visibility, and complete `Use::span`. The declaration
must satisfy `is_visible_from` for that same `ModuleKey`; this retains every same-module visibility
region without treating a private M-CHECK fact as import authority. Distinct aliases may bind in
the same module, while a repeated alias reaches `DuplicateBinding` and an alias colliding with a
local declaration reaches `LocalDeclarationCollision`.

The dedicated result contract is `CanonicalResolvedSelfOrdinaryFunctionAliases`,
`CanonicalBoundSelfOrdinaryFunctionAliasSet`, and `CanonicalSelfOrdinaryFunctionAliasBinding`.
The binding exposes the local alias, defining identity, declaration span, origin, visibility, and
full `Use::span` as `use_span`. The resolved type exposes its bindings but has no `import_edges`
field; only the dedicated binder calls its private `into_bound_alias_set` to produce the dedicated
bound-set type. The resolver and binder use shared `CanonicalStructuralImportError`; this is not a
traversal rule, so `ImportCycle` is unreachable by construction and source fence. One invalid
module causes atomic failure with no dedicated result for a valid sibling. It never constructs or
returns `CanonicalResolvedSimpleImports` or `CanonicalBoundModuleSet`; the existing
`CanonicalBoundModuleBinding`, generic binder, and all delivered `crate`, `super`, glob, and
local-precedence routes remain unchanged.

## Approaches considered

1. Extend the existing M-SIMPLE resolver. That route owns wider `crate` grammar, cross-module
   edges, and cycle behavior; extending it would blur its tested authority boundary. Rejected.
2. **Add dedicated result types, a resolver, and a one-line delegating binder.** The repeated
   eligible-self-alias grammar, same-module visibility check, no-edge guarantee, full-use-span
   provenance, duplicate diagnosis, and atomic projection stay isolated. Recommended.
3. Reuse the generic binder after synthetically converting `self` to a local binding. That would
   hide the required full-use-span provenance and make rejection/atomicity ordering ambiguous.
   Rejected.

## Boundary and handoff

The planned APIs are `resolve_scoped_self_ordinary_function_imports_with_scopes`, returning
`Result<CanonicalResolvedSelfOrdinaryFunctionAliases, CanonicalStructuralImportError>`, and
`bind_scoped_self_ordinary_function_imports`, returning
`Result<CanonicalBoundSelfOrdinaryFunctionAliasSet, CanonicalStructuralImportError>`.
`CanonicalResolvedSelfOrdinaryFunctionAliases` exposes only its
`CanonicalSelfOrdinaryFunctionAliasBinding` values and has no `import_edges` field; its private
`into_bound_alias_set` is callable only by the dedicated binder and returns the dedicated bound-set
type. They consume only the canonical graph and provisional module scopes and produce only
non-authorizing Type-layer facts. They admit zero or more individually eligible self aliases per
module, reject duplicate aliases as `DuplicateBinding`, and exclude natural-name/equal aliases,
public or restricted `use`/re-exports, `self::child::fn`, direct child-module/nonfunction targets,
`crate`/`super`/unprefixed paths, groups, globs, mixed or other import forms, and
alias/local-declaration collisions. They neither construct nor return
`CanonicalResolvedSimpleImports` or `CanonicalBoundModuleSet`, leave
`CanonicalBoundModuleBinding` and the generic binder untouched, and make `ImportCycle`
unreachable by construction and source fence.

Cross-module traversal, edges, and cycles; final interfaces; generic-binder changes; Core/CPS;
Engine; admission/runtime; and client parity stay outside the slice. The run-route impact is a
Type-only `prerequisite`: TASK-2069 owns lowering and TASK-2064 owns parity.

## Deferred evidence

The future `task_2070_scoped_self_ordinary_function_aliases` target has eight witnesses: zero,
root, nested, and multiple-distinct aliases across same-module visibility regions;
identity/provenance including `use_span`; no edge and no false cycle; shape/visibility and
mixed-form and direct-child-module/nonfunction rejections; duplicate/local-collision and atomic valid-sibling failure; normalized
Type-layer file/inline scope/binding parity; exactly 16 generated
name/alias/root-nested/source/visibility cases; and an authority fence. These are planned tests,
not evidence, proof, or parity.

## Trace promotion target

TASK-2070 owns promotion of the deferred implementation node and these eight witnesses only after
the focused target passes. This plan does not promote TASK-2068 or alter its closed foundation.
