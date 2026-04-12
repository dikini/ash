# SPEC-030: Module Type Resolution

## Status: Draft

## 1. Overview

This spec normatively defines the behavior of module-level type resolution within a single source file, `pub mod` transitive export loading, and the `ash check` command's handling of non-workflow module files. It amends SPEC-009 (Module System) and SPEC-012 (Import System) with concrete implementation requirements.

## 2. Definitions

**Module file**: An `.ash` source file containing `pub type`, `pub fn`, `pub use`, `pub mod`, and/or `workflow` declarations. A file need not contain a `workflow` to be a valid module.

**Type snippet**: A semicolon-terminated `pub type` declaration extractable from source by the module loader.

**Type collection**: The process of parsing all type snippets from a module file and accumulating their names into a type environment before validating cross-references.

## 3. Intra-Module Type Resolution

### 3.1 Two-Pass Collection

When a module file contains multiple `pub type` declarations, the module loader MUST process them in two passes:

1. **Registration pass**: Parse each `pub type` snippet into a `CoreTypeDef`. Record each type's name in a module-local type name set. Do NOT validate that field type expressions reference known types.

2. **Validation pass**: For each registered type definition, validate that all type expressions in fields and variant payloads reference either:
   - A builtin type (`Int`, `String`, `Bool`, `Float`, `Null`)
   - A type in the module-local name set (registered in pass 1)
   - A fully-qualified imported type (via `use`)
   - A standard-library type (`List`, `Option`, `Map`, `Result`)

### 3.2 Ordering Independence

After two-pass collection, the order of `pub type` declarations in a source file MUST NOT affect whether the file parses successfully. Given:

```ash
pub type Message = Message { role: Role, content: String };
pub type Role = System | User | Assistant | Tool { tool_call_id: String };
```

Both definitions MUST resolve regardless of declaration order.

### 3.3 Error Reporting

If a type expression references an unbound name after both passes complete, the error MUST include:

- The referencing type name
- The unbound name
- The file and (approximate) line number

### 3.4 Conformance Tests

| ID | Test | Requirement |
|----|------|-------------|
| ST-1 | `pub type A = A { x: B }; pub type B = B { y: Int };` parses | §3.2 |
| ST-2 | `pub type A = A { x: NoSuchType };` reports unbound | §3.3 |
| ST-3 | All 11 SPEC-029 types in `std/src/llm/types.ash` parse as a module | §3.1 |

## 4. Transitive Submodule Export

### 4.1 `pub mod` Processing

When `collect_module_exports` encounters a line matching `pub mod <name>;`, it MUST:

1. Resolve `<name>` to a file path using the existing resolution rules (SPEC-009 §2.1):
   - `<name>.ash` in the current directory
   - `<name>/mod.ash` in the current directory
2. Recursively call `collect_module_exports` on the resolved path.
3. Merge all `pub` items from the submodule into the parent module's export table.

### 4.2 Visibility Filtering

Only items declared `pub` in the submodule are merged. Items with `pub(crate)` or no visibility modifier are excluded from the parent's exports.

### 4.3 Cycle Detection

If a `pub mod` resolution leads to a file already being processed (cycle), the loader MUST report an error. Cycles are detected by maintaining a visited-path set during recursive resolution.

### 4.4 Conformance Tests

| ID | Test | Requirement |
|----|------|-------------|
| ST-4 | `mod.ash` with `pub mod types;` makes `types.ash` exports available | §4.1 |
| ST-5 | `use llm::Role` resolves through `llm/mod.ash` → `llm/types.ash` | §4.1 |
| ST-6 | Circular `pub mod` reports error, does not stack overflow | §4.3 |

## 5. Module File Checking

### 5.1 `ash check` Module Detection

When `ash check <path>` is invoked on a file, the command MUST:

1. Attempt to parse the file as a workflow module (existing behavior).
2. If step 1 fails with a parse error, attempt to parse the file as a module file.
3. If step 2 also fails, report the parse error from step 1 (original behavior).
4. If step 2 succeeds, validate the module file's contents.

### 5.2 Module File Validation

Module file validation checks:

1. All `pub type` snippets parse without error (via `parse_type_def`).
2. All `pub fn` snippets parse without error (via `parse_fn_definition`).
3. All `pub use` snippets parse and their targets resolve.
4. All `pub mod` targets resolve to existing files.
5. No duplicate type definitions.
6. No duplicate callable definitions.

### 5.3 Output Format

For module files, `ash check` outputs:

```
[OK] path/to/module.ash: OK (module: 11 types, 4 functions, 2 re-exports)
```

or on failure:

```
[FAIL] path/to/module.ash: FAILED
  Error: type definition 'Message': unbound type 'NoSuchType'
  Error: pub fn 'broken': parse error at line 42
```

### 5.4 Conformance Tests

| ID | Test | Requirement |
|----|------|-------------|
| ST-7 | `ash check std/src/llm/types.ash` succeeds | §5.1 |
| ST-8 | `ash check std/src/llm/mod.ash` succeeds | §5.1 |
| ST-9 | File with only `pub type X = X { a: Int };` passes `ash check` | §5.2 |
| ST-10 | File with invalid type reports specific error | §5.3 |

## 6. Compatibility Requirements

### 6.1 Backward Compatibility

- Existing workflow files MUST continue to parse and check without changes.
- Existing `use` import paths MUST continue to resolve.
- The `pub mod` processing is additive: files without `pub mod` are unaffected.

### 6.2 Forward Compatibility

- This spec does not define `pub(crate)` or `pub(super)` visibility enforcement.
- This spec does not define type inference for `pub fn` bodies.
- This spec does not define incremental or cached module loading.

## 7. Reference

- SPEC-009: Module System (file-based modules, resolution rules)
- SPEC-012: Import System (use statements, path resolution)
- SPEC-027: Pure Functions (fn syntax and body grammar)
- DESIGN-026: Module Type Resolution Remediation (implementation design)
