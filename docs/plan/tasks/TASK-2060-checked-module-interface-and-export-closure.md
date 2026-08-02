# TASK-2060: Checked Module Interface and Export Closure

**Status:** Planned
**Phase:** [PLAN-207](../PLAN-207-COMPLETE-MODULE-REALIZATION.md)
**Spec:** SPEC-103 §§5, 7-8
**Owned rule:** MOD-REAL-003
**Run-route impact:** prerequisite

## Description

Make a core-owned checked module interface the single public semantic boundary for exported modules and declarations. The interface must be export-closed, versioned, identity-preserving, and source-origin-aware.

## Dependencies

- 📝 TASK-2058 — canonical module artifacts.
- 📝 TASK-2059 — common module-unit route.

## Current → target

**Current files:** `crates/ash-core` semantic-summary carriers, `crates/ash-engine/src/module_loader.rs`, `crates/ash-typeck` summary consumers.

**Current state:** selected ordinary type and type-computation summaries exist, while Engine-private export tables and feature-specific summaries remain part of import behavior.

**Target state:** one `PublicModuleInterface` projection contains public child modules and every supported public declaration namespace. One private view remains available only while checking the defining module. Engine transports interfaces but does not define their semantics.

## Requirements

1. Define a versioned core-owned interface schema with stable module/declaration identities, visibility, origins, dependencies, and public summaries.
2. Include public modules, values/callables, types/constructors, interfaces/impls, and existing syntax-phase macro/notation summaries without making macros runtime callable.
3. Validate export closure before publication; reject public signatures that reference inaccessible facts.
4. Preserve defining identities through `pub use`; aliases alter paths only.
5. Quarantine or remove Engine-private export ownership and raw-source collection for ordinary declarations.
6. Consume SPEC-057/SPEC-062 summary carriers through a declared compatibility/version amendment;
   do not recreate their type identities, closure validation, versioning, or import-order rules.
7. Retire the Engine export scanners listed in AUDIT-207, or fence each as a disagreement-only
   check with no route to interface publication.

## TDD Steps and evidence

1. Write tests for public/private views, re-exports, aliases, constructor exposure, callable rows, interface references, macro/notation metadata, and version rejection.
2. Add property tests: public projection never contains a private declaration; re-export identity equals defining identity; declaration order does not alter normalized interfaces.
3. Add malformed-summary and cache-version tests that reject before partial registration.

## Completion checklist

- [ ] Public and private views are distinct and export closure is checked.
- [ ] Re-exports preserve defining identities.
- [ ] Engine-private semantic export ownership is removed or fenced.
- [ ] Every export scanner in AUDIT-207 has a removal/fence result and no unclassified caller.
- [ ] Focused core/typecheck/Engine tests, fmt, and clippy pass.

## Handoffs

- **Consumes:** module units and canonical identities.
- **Produces:** checked public/private interface facts for TASK-2061 and module-lowering facts for TASK-2062.
- **Downstream owner:** TASK-2061 binds imports; TASK-2063 transports only checked linked artifacts.
- **Non-goals:** runtime authority, dynamic imports, package resolution, or type-level semantics beyond preserving existing summary facts.

## Files and verification

**Files:** `crates/ash-core/src/semantic_summary*`, `crates/ash-engine/src/module_loader.rs`, relevant `ash-typeck` summary-registration modules and tests.

```text
cargo test -p ash-core semantic_summary
cargo test -p ash-typeck
cargo test -p ash-engine module
cargo clippy -p ash-core -p ash-typeck -p ash-engine --all-targets -- -D warnings
cargo fmt --check
```
