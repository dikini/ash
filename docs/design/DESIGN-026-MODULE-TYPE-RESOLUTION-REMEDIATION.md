# DESIGN-026: Module Type Resolution Remediation

## Status: Draft (v3 -- revised after independent review)

## Overview

Remediate four concrete bugs in module type resolution and module-file checking, without changing baseline module/import semantics (SPEC-009, SPEC-012).

## Problem Statement

### Bug 1: Sibling type cross-references fail during Engine::check()

When a module file defines multiple `pub type` declarations that reference each other, `ash check` reports "Unbound variable" for types defined in the same file.

**Failing code path** (confirmed by source inspection):

```
Engine::check()                            -- lib.rs:442-451
  for imported_type in imported_type_defs {
      type_env.register_type(&imported_type)  -- one-by-one registration
  }

TypeEnv::register_type()                   -- type_env.rs:484-505
  convert_type_def(def, self)              -- resolves field types against current env
    type_expr_to_type(field_type, .., type_env)
      type_env.resolve_type("Role")        -- Role not yet registered → UnboundVariable

TypeEnv::resolve_type()                    -- type_env.rs:960-975
  self.type_info.get(name)                 -- not found
  self.ast_types.contains_key(name)        -- not found
  → TypeError::UnboundVariable(name)
```

The module loader (`module_loader.rs:523-529`) parses and converts each `pub type` snippet correctly into a `CoreTypeDef`. The failure occurs later, during `Engine::check()`, when `TypeEnv::register_type` validates field types against an environment that doesn't yet contain sibling types.

### Bug 2: `ash check` rejects non-workflow module files

`ash check` calls `engine.parse_file()` which requires a `workflow ... { }` declaration. Files containing only `pub type`, `pub fn`, and `pub use` fail at the workflow parse step.

SPEC-009 §4.1a already defines that non-entry modules are valid `ModuleFile`s and do not require a workflow. The `ash check` command should support this model.

The current CLI path is:

```
check.rs:check_file()
  engine.parse_file(path)       -- always expects a workflow
  engine.check(&workflow)        -- type-checks the workflow
```

There is no `ModuleFile` parse/check API exposed by the engine. TASK-541 must add one.

### Bug 3: `pub mod` declarations are silently ignored

`collect_module_exports` does not process `pub mod <name>;` lines. However, the import resolution picture is more nuanced than originally described:

**How `use llm::types::Role` resolves today**: `resolve_in_root` (module_loader.rs:751-762) walks filesystem segments directly: `root/llm/types.ash` or `root/llm/types/mod.ash`. It does NOT consult parent module exports. So `use llm::types::Role` already works without `pub mod` processing.

**What `pub mod` loading actually fixes**: When `collect_module_exports` is called on `llm/mod.ash` to get `llm`'s exports, it currently ignores `pub mod types;`. This means:
- Child module exports are not available for `pub use` re-export merging (the `merge_use_exports` path can't re-export what was never collected).
- `collect_module_exports("llm/mod.ash")` produces incomplete exports when child modules contain items referenced by `pub use` in the parent.

So the fix is: `collect_module_exports` must recurse into `pub mod` children so that `pub use types::Role;` in `mod.ash` can find `Role` from the child. The qualified path resolution (`llm::types::Role`) already works via filesystem.

### Bug 4: `pub fn` export parsing silently drops failures

`parse_supported_pub_fn_callable(snippet)` (module_loader.rs:603-604) returns `Option`, swallowing both parse errors and unsupported constructs. When a `pub fn` in a stdlib file fails to parse, it is silently omitted from exports with no diagnostic.

## Design Decisions

### D1: Pre-declare type names, then register with upgrade semantics

Add `TypeEnv::declare_type_name(name: &str)` that inserts a placeholder into `ast_types` without converting or validating. Then modify `register_type()` to allow upgrading a placeholder entry (instead of rejecting duplicates outright).

**Required change to `register_type`**: Current code at type_env.rs:487-489 rejects any duplicate in `ast_types`:

```rust
if self.ast_types.contains_key(&type_name) {
    return Err(TypeEnvError::DuplicateType(type_name));
}
```

This must be changed to allow replacement when the existing entry is a placeholder (a minimal `TypeDef` inserted by `declare_type_name`). Two implementation options:

**Option A (recommended)**: Add a `placeholder: bool` flag to `TypeDef` (or use a sentinel body like `TypeBody::Struct(vec![])` with a naming convention). `register_type` checks if the existing entry is a placeholder and replaces it. Non-placeholder duplicates still error.

**Option B**: Use a separate `declared_names: HashSet<TypeName>` alongside `ast_types`. `register_type` checks `ast_types` for duplicates but not `declared_names`. `resolve_type` checks `declared_names` as a third fallback.

Option A is simpler because it reuses the existing `resolve_type` fallback path (line 973: `ast_types.contains_key(name)`) without adding a new store.

**Flow**:
1. `Engine::check()`: call `declare_type_name(name)` for each imported type (inserts placeholder into `ast_types`).
2. `Engine::check()`: call `register_type(def)` for each imported type (replaces placeholder with full definition).

**Why this layer**: The failure occurs in `TypeEnv::register_type` → `convert_type_def` → `resolve_type`. The module loader already parses types correctly; only the registration ordering is wrong.

### D2: `pub mod` loading enables re-export completeness

`pub mod <name>;` causes `collect_module_exports` to recursively load the child and store its exports. This enables `pub use` re-exports in the parent to find items from the child.

**Impact on import resolution**: Qualified path resolution (`llm::types::Role`) is handled by `resolve_in_root` walking filesystem segments -- it already works. `pub mod` loading fixes the export-collection path so that `pub use types::Role;` in `mod.ash` can find `Role` from the child module's exports.

It does NOT implicitly flatten child exports into the parent. Explicit `pub use` is still required for parent-level access (SPEC-012 semantics preserved).

### D3: Engine exposes ModuleFile check API for CLI

The CLI cannot implement module-file checking using the current engine API. Required changes:

1. **Engine**: Add `Engine::check_module_file(path: &Path)` that:
   - Reads the file source
   - Extracts `pub type` definitions via `collect_public_type_defs_from_source`
   - Pre-declares all type names into a `TypeEnv`
   - Registers all type definitions
   - Validates `pub fn` parse success (with diagnostic warnings)
   - Returns `ModuleFileCheckResult` (type count, fn count, warnings, errors)

2. **Engine**: Promote `collect_public_type_defs_from_source` from `pub(crate)` to `pub` (or expose via a public wrapper), so the CLI can use it.

3. **CLI**: `check_file()` detects whether the file is a workflow file or a module file. For module files, calls `engine.check_module_file(path)`. For workflow files, uses existing `parse_file` + `check` path.

### D4: Error reporting for silently-dropped pub fn exports

Change `parse_supported_pub_fn_callable` from silent `Option` return to `Result`, logging a warning when a `pub fn` snippet fails to parse. This ensures stdlib authors are informed when their function definitions are malformed.

## Affected Components

| Component | Change | Risk |
|-----------|--------|------|
| `ash-typeck/type_env.rs` | Add `declare_type_name()`, modify `register_type()` for placeholder upgrade | Medium -- changes existing rejection behavior |
| `ash-engine/lib.rs` | Pre-declare before register loop, add `check_module_file()` | Low -- additive API + reordered registration |
| `ash-engine/module_loader.rs` | `pub mod` recursive loading, `pub fn` diagnostics, expose `collect_public_type_defs_from_source` as `pub` | Medium -- changes collect_module_exports |
| `ash-cli/commands/check.rs` | ModuleFile detection and check path via engine API | Low -- new path, existing untouched |

## Out of Scope

- Type inference for `pub fn` bodies (deferred to future phase)
- `pub(crate)` / `pub(super)` visibility enforcement (deferred)
- Cycle detection in module graph (deferred: SPEC-009 §6.2 specifies the requirement but this phase does not implement it)
- Binary module compilation, package registry integration
- Ordinary `use` inside module files (current loader only handles `pub use` for export merging; `use` in non-workflow files is a separate concern)
- Changes to `resolve_in_root` / `resolve_module_path` (filesystem path resolution already works correctly)
