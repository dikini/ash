# SPEC-030: Module Type Resolution

## Status: Draft (v2 -- revised after independent review)

## 1. Overview

This spec defines normative behavior for three fixes: sibling type cross-reference resolution during engine type checking, `pub mod` child module loading for qualified path resolution, and module-file checking aligned with the SPEC-009 `ModuleFile` model. It does not change baseline module/import semantics.

## 2. Definitions

**Sibling types**: Two or more `pub type` definitions imported from the same module file.

**Pre-declaration**: Inserting a type name into the `TypeEnv`'s `ast_types` map without full conversion/validation, so that subsequent registrations can resolve the name.

**Module file**: An `.ash` source file parsed as a `ModuleFile` per SPEC-009 §4.1a. May contain `pub type`, `pub fn`, `pub use`, `pub mod`, and optionally a `workflow` declaration.

## 3. Sibling Type Cross-Reference Resolution

### 3.1 Pre-declaration Before Registration

When `Engine::check()` registers imported type definitions into `TypeEnv`, it MUST:

1. **Pre-declare**: For each imported type definition, insert its name into `TypeEnv::ast_types` (via `declare_type_name`) without calling `register_type`.
2. **Register**: For each imported type definition, call `register_type` which performs full conversion and validation.
3. Pre-declaration MUST happen before any registration.

### 3.2 Ordering Independence

After pre-declaration, the order of full `register_type` calls MUST NOT affect whether sibling type resolution succeeds. Given imported types `Role` and `Message` where `Message` references `Role`:

```ash
pub type Role = System | User | Assistant | Tool { tool_call_id: String };
pub type Message = Message { role: Role, content: String, ... };
```

Both MUST register successfully regardless of which is processed first.

### 3.3 Error Reporting

If a type expression references a name not present in the environment after pre-declaration, the error MUST include:

- The type being registered when the error occurred
- The unbound name
- The file (or "imported types") context

### 3.4 Scope

This section applies to the `Engine::check()` type registration path (`lib.rs:442-451`). It does NOT change the module loader's type parsing, which already works correctly.

### 3.5 Conformance Tests

| ID | Test | Requirement |
|----|------|-------------|
| ST-1 | Two imported types with forward reference register without error | §3.2 |
| ST-2 | All 11 SPEC-029 types imported from `llm/types.ash` register without error | §3.2 |
| ST-3 | Reference to truly unbound type produces descriptive error | §3.3 |
| ST-4 | Self-referential type (`pub type Tree = Tree { children: List<Tree> }`) registers | §3.2 |
| ST-5 | Generic reference (`List<Role>`, `Option<Message>`) resolves with builtin+imported types | §3.2 |

## 4. `pub mod` Child Module Loading

### 4.1 Behavior

When `collect_module_exports` encounters a line matching `pub mod <name>;`, it MUST:

1. Resolve `<name>` to a file path using SPEC-009 §2.1 resolution rules.
2. Recursively call `collect_module_exports` on the resolved path.
3. Store the submodule's exports under the child module name for qualified path resolution.

### 4.2 No Implicit Parent Export

`pub mod <name>;` does NOT merge the child module's exports into the parent's export table. To re-export a child item at the parent level, explicit `pub use` is required per SPEC-012.

This preserves baseline semantics:
- `llm::types::Role` resolves (child module loaded via `pub mod types;`)
- `llm::Role` requires `pub use types::Role;` in `mod.ash` (explicit re-export)

### 4.3 Unknown Module

If the resolved path does not exist, the loader MUST report an error with the module name and the searched paths.

### 4.4 Conformance Tests

| ID | Test | Requirement |
|----|------|-------------|
| ST-6 | `pub mod types;` makes `types.ash` items available for qualified import | §4.1 |
| ST-7 | `use llm::types::Role` resolves via `pub mod types;` in `llm/mod.ash` | §4.1, §4.2 |
| ST-8 | `use llm::Role` fails unless `pub use types::Role;` exists | §4.2 |
| ST-9 | `pub mod nonexistent;` reports module-not-found error | §4.3 |

## 5. Module-File Checking

### 5.1 Check Model

`ash check` MUST follow the SPEC-009 §4.1a `ModuleFile` model:

1. Parse the file as a `ModuleFile` containing type/fn/mod/use declarations and optionally a workflow.
2. If the file contains a workflow, type-check it (existing behavior).
3. If the file does not contain a workflow, validate module-level declarations:
   - All `pub type` snippets parse without error.
   - All `pub fn` snippets parse without error.
   - All `pub use` snippets parse and resolve.
   - All `pub mod` targets resolve to existing files.
   - Sibling type cross-references resolve (per §3).
4. If both workflow checking and module-level validation are applicable, report all errors from both.

### 5.2 Output Format

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

### 5.3 pub fn Parse Failure Diagnostics

When a `pub fn` snippet fails to parse or is unsupported, `parse_supported_pub_fn_callable` MUST produce a diagnostic warning instead of silently returning `None`. The warning MUST include the function name (if extractable) and the reason.

### 5.4 Conformance Tests

| ID | Test | Requirement |
|----|------|-------------|
| ST-10 | `ash check std/src/llm/types.ash` succeeds with type count | §5.1 |
| ST-11 | `ash check` on file with only `pub type X = X { a: Int };` succeeds | §5.1 |
| ST-12 | `ash check` on file with invalid type reports specific error | §5.2 |
| ST-13 | `pub fn` parse failure produces warning, not silent drop | §5.3 |

## 6. Compatibility

### 6.1 Backward Compatibility

- Existing workflow files and import paths continue to work unchanged.
- `pub mod` loading is additive: files without `pub mod` are unaffected.
- `TypeEnv::declare_type_name` is a new method; existing `register_type` is unchanged.

### 6.2 Baseline Semantics Preserved

- SPEC-009 module-tree resolution: unchanged.
- SPEC-012 explicit re-export for parent-level access: unchanged.
- `pub mod` loads child for qualified access only; no implicit flattening.

### 6.3 Out of Scope (Deferred)

- Cycle detection in module graph (SPEC-009 §6.2 specifies; this phase does not implement)
- Ordinary `use` inside module files (loader only handles `pub use` for exports)
- `pub(crate)` / `pub(super)` visibility enforcement
- Type inference for `pub fn` bodies

## 7. Reference

- SPEC-009 §4.1a: ModuleFile parse model
- SPEC-009 §2.1: File-based module resolution
- SPEC-012 §2.1-2.5: Import syntax and re-export
- DESIGN-026: Implementation design (D1-D4)
