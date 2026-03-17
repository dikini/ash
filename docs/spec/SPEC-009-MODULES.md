# SPEC-009: Module System

## Status: Draft

## 1. Overview

Ash supports a Rust-style module system for organizing code into logical units with visibility control. This enables:

- **Code organization**: Split large workflows across multiple files
- **Encapsulation**: Hide implementation details with visibility modifiers
- **Reusability**: Share capabilities and policies across workflows
- **Namespace management**: Avoid naming conflicts

## 2. Module Declaration

### 2.1 File-Based Modules

A module declaration `mod name;` looks for the module content in external files:

```
mod foo;      -- Looks for foo.ash or foo/mod.ash
mod bar;      -- Looks for bar.ash or bar/mod.ash
```

Resolution follows Rust's rules:
1. `name.ash` (module file)
2. `name/mod.ash` (module directory with mod.ash)

### 2.2 Inline Modules

A module declaration with a body defines the module inline:

```
mod foo {
    capability read: observe() returns string;
    
    workflow helper {
        action read_file {
            effect: operational;
            body: || -> file:read("data.txt");
        }
    }
}
```

## 3. Visibility

### 3.1 Visibility Modifiers

Definitions can have visibility modifiers:

```
visibility ::= "pub"
             | "pub(crate)"
             | "pub(super)"
             | "pub(self)"
             | "pub(in" module_path ")"
             | <empty>  -- private (inherited)
```

### 3.2 Visibility Levels

| Modifier | Meaning | Accessible From |
|----------|---------|-----------------|
| (none) | Private | Same module only |
| `pub` | Public | Anywhere |
| `pub(crate)` | Crate-public | Same crate/package |
| `pub(super)` | Parent-public | Parent module and its descendants |
| `pub(self)` | Self-public | Same as private (explicit) |
| `pub(in path)` | Restricted | Specified module path and descendants |

### 3.3 Examples

```
-- Private capability (default)
capability internal_log: observe(msg: string);

-- Public capability
pub capability http_get: observe(url: string) returns json;

-- Crate-public policy
pub(crate) policy budget_limit:
    when cost > 1000
    then require_approval(role: manager);

-- Parent-visible workflow
pub(super) workflow helper {
    action validate {
        effect: evaluative;
        body: || -> true;
    }
}
```

## 4. Module Tree

### 4.1 Root Module

Every Ash program has a root module (the entry point file). All other modules are descendants of the root.

### 4.2 Module Paths

Module paths use `::` as separator:

```
module_path ::= "crate" | "super" | "self"
              | IDENTIFIER ("::" IDENTIFIER)*
```

### 4.3 Path Resolution

```
crate::foo::bar    -- Absolute path from root
super::baz         -- Parent module
self::helper       -- Current module (redundant but valid)
foo::bar           -- Relative path from current module
```

## 5. Item References

### 5.1 Referencing Items

Items from other modules are referenced by path:

```
workflow main {
    action use_helper {
        effect: operational;
        body: || -> crate::utils::helper();
    }
}
```

### 5.2 Name Resolution Order

1. Current module (`self`)
2. Parent modules (for `super`)
3. Root module (for `crate`)
4. Imported names (future: `use` statements)

## 6. Module Graph

### 6.1 Graph Structure

The module system builds a directed graph:
- Nodes: Modules (file-based or inline)
- Edges: Parent-child relationships from `mod` declarations

### 6.2 Cycle Detection

Module graphs must be acyclic. Circular dependencies are errors:

```
-- foo.ash
mod bar;  -- ERROR: circular if bar.ash contains "mod foo;"

-- bar.ash
mod foo;  -- creates cycle
```

## 7. Visibility Checking

### 7.1 Access Rules

An item is accessible from module M if:
1. The item is in M (same module)
2. The item is `pub` (anywhere)
3. The item is `pub(crate)` and M is in the same crate
4. The item is `pub(super)` and M is the parent or descendant
5. The item is `pub(in path)` and M is in the specified path

### 7.2 Type Checking Phase

Visibility checking occurs during type checking (ash-typeck):
- After name resolution
- Before type inference
- Reports visibility violations as errors

## 8. Grammar Extension

### 8.1 Surface Grammar

```
program         ::= module_item*

module_item     ::= visibility? definition
                  | visibility? module_decl

definition      ::= capability_def | policy_def | role_def
                  | memory_def | datatype_def | workflow_def

module_decl     ::= "mod" IDENTIFIER ";"           -- File-based
                  | "mod" IDENTIFIER "{" module_item* "}"  -- Inline

visibility      ::= "pub" ( "(" visibility_rest ")" )?
visibility_rest ::= "crate" | "super" | "self" | "in" module_path
```

## 9. Implementation Notes

### 9.1 Module Resolution

Module resolution happens in two phases:
1. **Discovery**: Parse root, find `mod` declarations, recursively discover files
2. **Loading**: Parse discovered files, build module graph

### 9.2 Error Handling

Common errors:
- Module not found (file doesn't exist)
- Visibility violation (accessing private item)
- Circular dependency
- Duplicate module name

### 9.3 Future Extensions

Not in current scope:
- `use` statements for importing
- `pub use` for re-exports
- External crate dependencies
- Binary module compilation
