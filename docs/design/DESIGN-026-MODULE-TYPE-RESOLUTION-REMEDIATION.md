# DESIGN-026: Module Type Resolution Remediation

## Status: Draft (v2 -- revised after independent review)

## Overview

Remediate three concrete bugs in module type resolution and module-file checking, without changing baseline module/import semantics (SPEC-009, SPEC-012).

## Problem Statement

### Bug 1: Sibling type cross-references fail during Engine::check()

When a module file defines multiple `pub type` declarations that reference each other, `ash check` reports "Unbound variable" for types defined in the same file.

**Failing code path** (the real path, confirmed by source inspection):

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

**Key observation**: `resolve_type` already has an `ast_types` fallback (line 973): if a type name exists in `ast_types` (even without full `TypeInfo`), resolution succeeds with `None` info. This means pre-declaring type names before full registration would fix the cross-reference issue.

### Bug 2: `ash check` rejects non-workflow module files

`ash check` calls `engine.parse_file()` which requires a `workflow ... { }` declaration. Files containing only `pub type`, `pub fn`, and `pub use` fail at the workflow parse step.

SPEC-009 §4.1a already defines that non-entry modules are valid `ModuleFile`s and do not require a workflow. The `ash check` command should support this model.

### Bug 3: `pub fn` export parsing silently drops failures

`parse_supported_pub_fn_callable(snippet)` (module_loader.rs:603-604) returns `Option`, swallowing both parse errors and unsupported constructs. When a `pub fn` in a stdlib file fails to parse, it is silently omitted from exports with no diagnostic.

This is a separate bug from the three originally identified, discovered during independent review. It affects TASK-542 (end-to-end stdlib validation) because `prompt.ash` exports may be silently dropped.

## Design Decisions

### D1: Pre-declare type names in TypeEnv before full registration

Add `TypeEnv::declare_type_name(name: &str)` that inserts a placeholder into `ast_types` without converting or validating. Then in `Engine::check()`, call `declare_type_name` for all imported types before the full `register_type` loop.

**Why this layer**: The failure occurs in `TypeEnv::register_type` → `convert_type_def` → `resolve_type`. The module loader already parses types correctly; only the registration ordering is wrong. Fixing the registration path is the minimal correct fix.

**Why not two-pass in the module loader**: The module loader doesn't validate type expressions. `parse_public_type_defs` calls `parse_type_def` (syntax only) then `convert_type_def` (no-op for the module loader's CoreTypeDef path). The loader's `convert_type_def` is a different function from typeck's `convert_type_def` -- it doesn't call `resolve_type` at all.

### D2: `pub mod` loading preserves baseline semantics

`pub mod <name>;` should make the child module's public items available for qualified access (`llm::types::Role`). It should NOT implicitly flatten exports into the parent. Explicit `pub use types::Role;` in `mod.ash` is still required for `llm::Role` to work.

This preserves SPEC-009 module-tree resolution and SPEC-012 explicit re-export semantics. The change is purely about making `pub mod` actually load the child module for qualified path resolution, instead of being silently ignored.

### D3: Module-file check aligned with SPEC-009 ModuleFile model

SPEC-009 §4.1a defines the `ModuleFile` parse model: all `.ash` files parse as `ModuleFile`; entry-point loading then promotes to `Program` if a workflow is present. `ash check` should follow this model:

1. Parse file as `ModuleFile` (types, fns, mod declarations, optionally a workflow).
2. If a workflow is present, type-check it (existing behavior).
3. If no workflow, validate module-level declarations (parse correctness, type cross-references).
4. Report per-declaration errors.

This is not a fallback model. It is the canonical SPEC-009 file-level parse model.

### D4: Error reporting for silently-dropped pub fn exports

Change `parse_supported_pub_fn_callable` from silent `Option` return to `Result`, logging a warning when a `pub fn` snippet fails to parse. This ensures stdlib authors are informed when their function definitions are malformed.

## Affected Components

| Component | Change | Risk |
|-----------|--------|------|
| `ash-typeck/type_env.rs` | Add `declare_type_name()` | Low -- additive API |
| `ash-engine/lib.rs` | Pre-declare before register loop | Low -- reorders existing registration |
| `ash-engine/module_loader.rs` | Load child modules on `pub mod`, warn on fn parse failures | Medium -- changes collect_module_exports |
| `ash-cli/commands/check.rs` | ModuleFile-based check path | Low -- new path, existing untouched |

## Out of Scope

- Type inference for `pub fn` bodies (deferred to future phase)
- `pub(crate)` / `pub(super)` visibility enforcement (deferred)
- Cycle detection in module graph (deferred: SPEC-009 §6.2 specifies the requirement but this phase does not implement it)
- Binary module compilation, package registry integration
- Ordinary `use` inside module files (current loader only handles `pub use` for export merging; `use` in non-workflow files is a separate concern)
