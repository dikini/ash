# SPEC-012: Import System (use statements)

## Status: Draft

## 1. Overview

The import system enables bringing items from other modules into scope via `use` statements. This complements the module system (SPEC-009) by allowing convenient access to items without fully qualified paths.

## 2. Import Syntax

### 2.1 Basic Imports

```
use crate::foo::bar;           -- Import specific item
use crate::foo::bar as baz;    -- Import with alias
use result::Result;            -- Import a standard-library item
use runtime::Args;             -- Import from a standard-library root module
```

Import paths use `::` as the only valid separator between path segments.
Dot-separated forms such as `use runtime.Args;` and `use result.{Result, Ok, Err};`
are invalid.

### 2.2 Glob Imports

```
use crate::foo::*;             -- Import all public items
```

### 2.3 Nested Imports

```
use crate::foo::{bar, baz};    -- Import multiple items
use crate::foo::{bar as b, baz}; -- Mix aliases
```

### 2.4 Self and Super

```
use self::foo;                 -- Import from current module
use super::bar;                -- Import from parent module
```

### 2.5 Standard-Library Imports

Standard-library modules are imported from the compiler-provided root namespace using
ordinary `use` declarations. They do not require a `std::` prefix.

Valid examples:

```
use result::Result;
use result::{Result, Ok, Err};
use runtime::Args;
use io::Stdout;
```

Legacy dot-style examples are invalid:

```
use result.{Result, Ok, Err};
use runtime.Args;
use io.Stdout;
```

Standard-library module resolution follows SPEC-009 file-based rules rooted at `std/src/`.
For example, `use result::Result;` resolves `result` from `std/src/result.ash` or
`std/src/result/mod.ash`.

### 2.6 Prelude

Every module implicitly imports the standard prelude defined by `std/src/prelude.ash`.
That prelude must make the following names available in all modules:

- `Option`
- `Some`
- `None`
- `Result`
- `Ok`
- `Err`

Additional items re-exported by the prelude are also implicitly available when
present in `std/src/prelude.ash`. No other standard-library modules or bindings are imported
implicitly unless they are re-exported by the prelude; access to other standard-library items
requires an explicit `use` declaration.

## 3. Import Resolution

### 3.1 Resolution Algorithm

1. Resolve the path in the `use` statement relative to current module
     - `crate`, `self`, and `super` use ordinary module-path resolution
     - A leading identifier that names a standard-library root module resolves against the
         compiler-provided standard-library namespace
     - If a top-level user module root name collides with a standard-library root module name,
         the program is ill-formed and the import must be rejected
2. Verify target item exists and is visible
3. Add item to current module's scope with given name (or alias)

### 3.2 Shadowing Rules

- Imports can shadow previous imports (last wins)
- Local definitions shadow imports
- Glob imports have lowest priority

```
use crate::foo::bar;           -- First import
use crate::baz::bar as bar;    -- Shadows previous

capability bar: observe();     -- Error: name conflict
```

### 3.3 Unused Import Warnings

Unused imports should generate warnings:

```
use crate::foo::bar;           -- Warning: unused import
```

## 4. Re-exports

### 4.1 pub use

Re-export items to make them available through another module:

```
-- foo.ash
pub use crate::internal::helper;  -- Re-export helper as foo::helper

-- main.ash
use crate::foo::helper;           -- Works via re-export
```

### 4.2 Use Chains

Re-exports can form chains:

```
-- a.ash: pub use crate::b::item;
-- b.ash: pub use crate::c::item;
-- c.ash: pub capability item: observe();

-- main.ash
use crate::a::item;  -- Resolves through a -> b -> c
```

## 5. Visibility and Imports

### 5.1 Import Visibility

Imports are private by default:

```
use crate::foo::bar;           -- Private import (only this module)
pub use crate::foo::bar;       -- Public re-export
```

### 5.2 Accessing Private Imports

Private imports are only accessible within the module where declared:

```
-- foo.ash
use crate::util::helper;       -- Private import

pub workflow public_workflow {
    action a {
        effect: operational;
        body: || -> helper();  -- OK: can use private import
    }
}

-- bar.ash
use crate::foo::helper;        -- ERROR: helper not public
```

## 6. Grammar Extension

### 6.1 Import Statement

```
import_stmt     ::= visibility? "use" import_path ";"

import_path     ::= simple_path
                  | simple_path "as" IDENTIFIER
                  | simple_path "::" "*"
                  | simple_path "::" "{" import_list "}"

import_list     ::= import_item ("," import_item)* ","?

import_item     ::= simple_path
                  | simple_path "as" IDENTIFIER

simple_path     ::= "crate" | "self" | "super" | IDENTIFIER
                  | simple_path "::" IDENTIFIER
```

The grammar above intentionally excludes dot-separated import forms. Conforming
implementations must reject `use foo.bar;` and `use foo.{bar, baz};`.

## 7. Implementation Notes

### 7.1 Import Collection

During parsing/AST construction:

1. Collect all `use` statements
2. Store with their visibility and target module

### 7.2 Import Resolution Phase

After module resolution but before type checking:

1. Resolve each import path to actual item
2. Build name binding table per module
3. Verify visibility constraints
4. Detect conflicts and cycles

### 7.3 Name Resolution Integration

The name resolver should check in order:

1. Local definitions (let bindings, parameters)
2. Current module definitions
3. Prelude-imported names and explicit imported names (with shadowing order)

Qualified `super::...` and `crate::...` paths resolve explicitly and are not part of
unqualified fallback lookup.

## 8. Error Messages

### 8.1 Common Errors

```
use crate::foo::bar;
-- ERROR: cannot find `bar` in `foo`
-- HELP: `foo` has these public items: baz, qux

use crate::foo::bar;
use crate::baz::bar;
-- WARNING: `bar` is shadowed by a later import

use crate::internal::secret;
-- ERROR: `secret` is private
-- HELP: consider making it `pub` or `pub(crate)`
```

### 8.2 Cycle Detection

```
-- a.ash: use crate::b::item;
-- b.ash: use crate::a::item;
-- ERROR: import cycle detected: a -> b -> a
```

## 9. Examples

### 9.1 Complete Example

```
-- utils.ash
pub capability log: observe(msg: String);
pub workflow helpers {
    action format {
        effect: epistemic;
        body: |input| -> input;
    }
}

-- main.ash
use crate::utils::log;
use crate::utils::helpers as h;

workflow main {
    action run {
        effect: operational;
        body: || -> {
            log("Starting...");
            h.format("done")
        };
    }
}
```

## 10. External Crate Imports

Dependencies declared in crate root metadata can be imported using the `external::` prefix:

### 10.1 Syntax

```
use external::<alias>::<path>;
```

Where `<alias>` is the dependency alias declared in a `dependency` statement.

### 10.2 Examples

```ash
-- Assuming: dependency util from "../util/main.ash";
use external::util::sanitize::normalize;
use external::util::helpers::*;

-- Usage in workflow
workflow process {
    action sanitize_input {
        effect: operational;
        body: |input| -> normalize(input);
    }
}
```

### 10.3 Resolution

External crate imports are resolved by:

1. Looking up the alias in the declared dependencies
2. Loading the dependency's crate root from the specified path
3. Resolving the path within the dependency's module tree
4. Verifying visibility (only `pub` and `pub(crate)` items are accessible)

## 11. Future Extensions

- Import groups with visibility: `pub(crate) use crate::foo::{a, b}`
- Restricted use: `use crate::foo::bar as private_bar;` (private alias)
- Version constraints in dependency declarations
