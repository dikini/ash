# SPEC-030: Module Type Resolution

## Status: Draft (v3 -- revised after independent review)

> **Target-state override:** [SPEC-103](SPEC-103-MODULE-REALIZATION-AND-OPERATIONAL-SEMANTICS.md)
> supersedes this document's §§4–6 for child-module loading, module-file checking, and
> compatibility/source-collection paths. Section 3's two-pass sibling type-registration invariant
> remains in force.

## 1. Overview

This spec defines normative behavior for four fixes: sibling type cross-reference resolution during engine type checking, `pub mod` child module loading for export completeness, module-file checking aligned with the SPEC-009 `ModuleFile` model, and `pub fn` parse failure diagnostics. It does not change baseline module/import semantics.

## 2. Definitions

**Sibling types**: Two or more `pub type` definitions imported from the same module file.

**Pre-declaration**: Inserting a type name into the `TypeEnv`'s `ast_types` map as a placeholder, without full conversion/validation, so that subsequent registrations can resolve the name.

**Placeholder**: A minimal `TypeDef` inserted by `declare_type_name`, identifiable as a placeholder by the `register_type` upgrade path.

**Module file**: An `.ash` source file parsed as a `ModuleFile` per SPEC-009 §4.1a. May contain `pub type`, `pub fn`, `pub use`, `pub mod`, and optionally a `workflow` declaration.

## 3. Sibling Type Cross-Reference Resolution

### 3.1 Pre-declaration Before Registration

When `Engine::check()` registers imported type definitions into `TypeEnv`, it MUST:

1. **Pre-declare**: For each imported type definition, call `declare_type_name(name)` to insert a placeholder into `ast_types`.
2. **Register**: For each imported type definition, call `register_type(def)` which replaces the placeholder with a full conversion and validation.
3. Pre-declaration MUST happen before any registration.

### 3.2 Placeholder Upgrade in register_type

`register_type` MUST allow replacing a placeholder entry in `ast_types` with a full type definition. The duplicate-rejection guard (type_env.rs:487-489) MUST be modified to:

1. Check if the existing `ast_types` entry is a placeholder.
2. If it is a placeholder: allow replacement (this is the upgrade path from pre-declaration to full registration).
3. If it is not a placeholder: reject with `DuplicateType` error (existing behavior).

### 3.3 Ordering Independence

After pre-declaration, the order of full `register_type` calls MUST NOT affect whether sibling type resolution succeeds. Given imported types `Role` and `Message` where `Message` references `Role`:

```ash
pub type Role = System | User | Assistant | Tool;
pub type Message = Message { role: Role, content: String, ... };
```

Both MUST register successfully regardless of which is processed first.

### 3.4 Error Reporting

If a type expression references a name not present in the environment after pre-declaration, the error MUST include:

- The type being registered when the error occurred
- The unbound name
- The file (or "imported types") context

### 3.5 Scope

This section applies to the `Engine::check()` type registration path (`lib.rs:442-451`). It does NOT change the module loader's type parsing, which already works correctly.

### 3.6 Conformance Tests

| ID | Test | Requirement |
|----|------|-------------|
| ST-1 | Two imported types with forward reference register without error | §3.3 |
| ST-2 | All 11 SPEC-029 types imported from `llm/types.ash` register without error | §3.3 |
| ST-3 | Reference to truly unbound type produces descriptive error | §3.4 |
| ST-4 | Self-referential type (`pub type Tree = Tree { children: List<Tree> }`) registers | §3.3 |
| ST-5 | Generic reference (`List<Role>`, `Option<Message>`) resolves with builtin+imported types | §3.3 |

## 4. `pub mod` Child Module Loading

### 4.1 Purpose

`pub mod` loading fixes the export-collection path in `collect_module_exports`. When a parent module contains `pub mod <name>;`, the loader MUST recursively collect the child module's exports. This enables `pub use` re-exports in the parent to find items from the child.

### 4.2 How Import Resolution Works

Qualified path resolution (`use llm::types::Role`) is handled by `resolve_in_root` walking filesystem segments (`root/llm/types.ash`). This already works correctly and is NOT changed by `pub mod` loading. The fix targets `collect_module_exports` so that `pub use types::Role;` in `mod.ash` can find `Role` from the child's exports.

### 4.3 Behavior

When `collect_module_exports` encounters a line matching `pub mod <name>;`, it MUST:

1. Resolve `<name>` to a file path relative to the current module's directory (same resolution as `resolve_in_root`).
2. Recursively call `collect_module_exports` on the resolved path.
3. Store the submodule's exports under the child module name in the parent's export table (for `pub use` re-export lookup).

### 4.4 No Implicit Parent Export

`pub mod <name>;` does NOT merge the child module's exports into the parent's flattened export table. To re-export a child item at the parent level, explicit `pub use` is required per SPEC-012.

### 4.5 Unknown Module

If the resolved path does not exist, the loader MUST report an error with the module name and the searched paths.

### 4.6 Conformance Tests

| ID | Test | Requirement |
|----|------|-------------|
| ST-6 | `pub mod types;` makes child exports available for `pub use types::Role;` | §4.3 |
| ST-7 | `use llm::types::Role` resolves via filesystem (unchanged) | §4.2 |
| ST-8 | `use llm::Role` fails unless `pub use types::Role;` exists in parent | §4.4 |
| ST-9 | `pub mod nonexistent;` reports module-not-found error | §4.5 |

## 5. Module-File Checking

### 5.1 Engine API

The engine MUST expose a public `check_module_file(path: &Path) -> ModuleFileCheckResult` API that:

1. Reads the file source.
2. Extracts `pub type` definitions via `collect_public_type_defs_from_source` (promoted to `pub`).
3. Pre-declares all type names into a `TypeEnv` (per §3.1).
4. Registers all type definitions (per §3.1).
5. Checks `pub fn` snippets parse successfully (per §5.3).
6. Returns `ModuleFileCheckResult` containing type count, function count, warnings, and errors.

### 5.2 CLI Integration

`ash check` MUST follow the SPEC-009 §4.1a `ModuleFile` model:

1. Attempt workflow parse via `engine.parse_file(path)`.
2. If parse succeeds: type-check the workflow (existing behavior).
3. If parse fails due to missing workflow: call `engine.check_module_file(path)`.
4. Report results per §5.4.

### 5.3 pub fn Parse Failure Diagnostics

When a `pub fn` snippet fails to parse or is unsupported, `parse_supported_pub_fn_callable` MUST produce a diagnostic warning instead of silently returning `None`. The warning MUST include the function name (if extractable) and the reason.

### 5.4 Output Format

For module files without a workflow:

```
[OK] path/to/module.ash: OK (module: 11 types, 4 functions, 2 re-exports)
```

On failure:

```
[FAIL] path/to/module.ash: FAILED
  Error: type 'Message': unbound type 'NoSuchType'
  Error: pub fn 'broken': parse error at line 42
```

### 5.5 Conformance Tests

| ID | Test | Requirement |
|----|------|-------------|
| ST-10 | `ash check std/src/llm/types.ash` succeeds with type count | §5.1, §5.2 |
| ST-11 | `ash check` on file with only `pub type X = X { a: Int };` succeeds | §5.2 |
| ST-12 | `ash check` on file with invalid type reports specific error | §5.4 |
| ST-13 | `pub fn` parse failure produces warning, not silent drop | §5.3 |

## 6. Compatibility

### 6.1 Backward Compatibility

- Existing workflow files and import paths continue to work unchanged.
- `pub mod` loading is additive: files without `pub mod` are unaffected.
- `TypeEnv::declare_type_name` is a new method.
- `register_type` placeholder-upgrade is additive: non-placeholder duplicates still error.
- `check_module_file` is a new engine API; existing `parse_file` + `check` path is untouched.
- `collect_public_type_defs_from_source` promotion from `pub(crate)` to `pub` is safe (it reads and parses, no mutation).

### 6.2 Baseline Semantics Preserved

- SPEC-009 module-tree resolution: unchanged.
- SPEC-012 explicit re-export for parent-level access: unchanged.
- `pub mod` loads child exports for re-export completeness; no implicit flattening.
- Filesystem-based path resolution (`resolve_in_root`): unchanged.

### 6.3 Out of Scope (Deferred)

- Cycle detection in module graph (SPEC-009 §6.2 specifies; this phase does not implement)
- Ordinary `use` inside module files (loader only handles `pub use` for exports)
- `pub(crate)` / `pub(super)` visibility enforcement
- Type inference for `pub fn` bodies
- Changes to `resolve_in_root` / `resolve_module_path` (already works)

## 7. Reference

- SPEC-009 §4.1a: ModuleFile parse model
- SPEC-009 §2.1: File-based module resolution
- SPEC-012 §2.1-2.5: Import syntax and re-export
- DESIGN-026: Implementation design (D1-D4)
