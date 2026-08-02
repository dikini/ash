# TASK-2061: Interface Import Resolution and Visibility

**Status:** Planned
**Phase:** [PLAN-207](../PLAN-207-COMPLETE-MODULE-REALIZATION.md)
**Spec:** SPEC-103 §6
**Owned rule:** MOD-REAL-004
**Run-route impact:** prerequisite

## Description

Resolve `use`, qualified paths, aliases, globs, and re-exports only from canonical checked module interfaces. Enforce visibility before a binding enters the importing environment.

## Dependencies

- 📝 TASK-2059 — common file/inline module units.
- 📝 TASK-2060 — checked public/private interface schema.

## Current → target

**Current files:** `crates/ash-parser/src/import_resolver.rs`, `crates/ash-typeck/src/name_binding.rs`, `crates/ash-engine/src/module_loader.rs`.

**Current state:** import resolution, binder tables, graph traversal, and Engine export lookup are separate routes with bounded visibility behavior.

**Target state:** `use` resolves module and declaration identities through interface facts. The binder consumes the same identities. The Engine never searches the filesystem or raw source to satisfy an import.

## Requirements

1. Implement one interface-driven resolver for explicit imports, aliases, groups, globs, qualified paths, and `pub use`.
2. Enforce private, `pub`, `pub(crate)`, `pub(super)`, and `pub(in path)` from canonical graph relationships.
3. Distinguish inaccessible, unresolved, ambiguous, and duplicate-binding diagnostics.
4. Preserve namespace separation and macro/notation syntax-phase boundaries.
5. Reject an import cycle deterministically before any incomplete interface enters a binding environment.
6. Replace the Engine leading-import prelude reader and import text scans in AUDIT-207 with parsed
   `use` items; any retained reader is non-authorizing and fail-closed.

## TDD Steps and evidence

1. Test equivalent file and inline trees for every import form.
2. Test visibility boundaries across sibling, parent, child, and separate crate modules.
3. Add mutation tests for aliasing, declaration/import order, path spelling, duplicate explicit/glob bindings, and false filesystem matches.
4. Add a no-fallback integration test that deletes a raw file after interfaces are constructed and proves the binder does not rediscover it.

## Completion checklist

- [ ] All supported `use` forms resolve from checked interfaces.
- [ ] All parsed visibility forms reject forbidden access before binding.
- [ ] Import diagnostics distinguish missing, inaccessible, ambiguous, and duplicate cases.
- [ ] Import scanners in AUDIT-207 have no route to binding or execution authority.
- [ ] Focused parser/typecheck/Engine tests, fmt, and clippy pass.

## Handoffs

- **Consumes:** TASK-2060 checked interfaces and canonical graph paths.
- **Produces:** resolved declaration identities and binding tables for TASK-2062.
- **Downstream owner:** TASK-2062 lowers resolved references; TASK-2064 owns system conformance.
- **Non-goals:** dynamic imports, runtime capability authority, cross-module initialization, or LSP workspace indexing.

## Files and verification

**Files:** `crates/ash-parser/src/import_resolver.rs`, `crates/ash-typeck/src/name_binding.rs`, `crates/ash-engine/src/module_loader.rs`, import/typecheck integration tests.

```text
cargo test -p ash-parser import_resolver
cargo test -p ash-typeck name_binding
cargo test -p ash-engine module_resolution
cargo clippy -p ash-parser -p ash-typeck -p ash-engine --all-targets -- -D warnings
cargo fmt --check
```
