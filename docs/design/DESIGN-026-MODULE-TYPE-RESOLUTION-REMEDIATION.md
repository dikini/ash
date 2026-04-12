# DESIGN-026: Module Type Resolution Remediation

## Status: Draft

## Overview

Remediate the module loader and type checker so that `pub type` definitions within a single module file can reference each other, and so that `ash check` can verify non-workflow module files. This design addresses three concrete bugs discovered during TASK-524-528 implementation.

## Problem Statement

The module loader (`crates/ash-engine/src/module_loader.rs`) extracts `pub type` snippets from source files individually via `extract_semicolon_snippets`, then calls `parse_public_type_defs` on each snippet in isolation. The parser's type-def converter validates type expressions against an empty environment, so a type like `Message { role: Role, ... }` fails with "Unbound variable: Role" even though `Role` is defined earlier in the same file.

Additionally, `ash check` only handles workflow files (files containing `workflow ... { }`). It has no code path for verifying stdlib module files that contain only `pub type`, `pub fn`, and `pub use` declarations.

## Root Cause Analysis

### RC1: Snippet-level type conversion without accumulation

`collect_module_exports` (line 318) loops over extracted snippets:

```rust
for snippet in extract_semicolon_snippets(&source, |trimmed| trimmed.starts_with("pub type ")) {
    for type_def in parse_public_type_defs(&snippet)? {  // Each call gets empty type context
        insert_type_export(&mut exports, &type_def)?;
    }
}
```

`parse_public_type_defs` calls `parse_type_def` then `convert_type_def`. The `convert_type_def` function in the module loader does no type-env lookup -- it just converts the parsed AST to `CoreTypeDef` without validating type references. The "Unbound variable" error comes from a different path: when the engine's `check()` method later tries to register these types via `TypeEnv::register_type`, the type checker validates field types against the TypeEnv which doesn't yet contain the sibling types.

### RC2: No module-file entry point in ash check

`ash check` calls `engine.parse_file(path)` which calls `load_ordinary_file` then `parse_workflow_source_with_imports`. The latter requires a `workflow ... { }` declaration. Files containing only module-level declarations (types, fns) fail at the workflow_def parse step with a generic "Parsing Error: ContextError".

### RC3: Module submodules not loaded transitively

`pub mod types;` declarations in `mod.ash` are ignored by the module loader. The loader only processes `pub use` for re-exports. To make `use llm::Role` work, `mod.ash` needs explicit `pub use types::Role;` for every export. This is fragile and not what SPEC-009 prescribes.

## Design Decisions

### D1: Two-pass type collection

Process `pub type` snippets in two passes:

1. **Pass 1 (Register)**: Parse all `pub type` snippets, convert to `CoreTypeDef` without type-expression validation, and register their names in a temporary type name set.
2. **Pass 2 (Validate)**: Re-validate type expressions in field types against the accumulated name set.

This allows forward references within a single file. The parser already handles the syntax correctly; only the validation step needs the accumulated context.

### D2: Module-file check command

Add a `check-module` path in `ash check` that detects when a file has no `workflow` declaration and instead validates it as a module file:

1. Extract all `pub type` snippets and verify they parse.
2. Extract all `pub fn` snippets and verify they parse.
3. Extract all `pub use` snippets and verify they resolve.
4. Report parse/type errors per snippet.

This does NOT require full type checking of fn bodies (which requires execution). It validates structural correctness only.

### D3: Transitive `pub mod` loading

Extend `collect_module_exports` to process `pub mod name;` declarations:

1. Extract lines matching `pub mod <name>;`.
2. Resolve the submodule path using the existing `resolve_module_path`.
3. Recursively call `collect_module_exports` on the submodule.
4. Merge submodule exports into the parent's export table.

This makes `mod.ash` with `pub mod types;` work as SPEC-009 intends: all `pub` items from `types.ash` become available through the parent module.

### D4: Preserve existing behavior

All changes are additive. Existing workflow files and their import paths continue to work. The new paths only activate for module files that lack a `workflow` declaration, or for `pub mod` lines that were previously ignored.

## Affected Components

| Component | Change | Risk |
|-----------|--------|------|
| `module_loader.rs` | Two-pass type collection, `pub mod` processing | Medium -- core import path |
| `ash-typeck` | Type expression validation accepts deferred names | Low -- additive API |
| `ash-cli check` | Module-file detection and validation | Low -- new path, existing untouched |
| `ash-engine lib.rs` | `parse_module_file` entry point | Low -- new method |

## Dependency Graph

```
D1 (two-pass types)
 └─► D3 (pub mod loading) -- both change collect_module_exports
D2 (check-module) ──► depends on D1 and D3 being correct
```

Recommended implementation order: D1 → D3 → D2.

## Out of Scope

- Full type inference for `pub fn` bodies (deferred to future phase)
- `pub(crate)` / `pub(super)` visibility enforcement (deferred)
- Binary module compilation
- Package registry integration
- Cycle detection in module graph (SPEC-009 §6.2)
