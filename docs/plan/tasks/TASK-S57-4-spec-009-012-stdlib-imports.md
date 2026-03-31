# TASK-S57-4: Update SPEC-009/SPEC-012 with Stdlib Import/Namespace Rules

## Status: ✅ Complete

## Description

Update SPEC-009 (Module System) and SPEC-012 (Imports) with normative rules for standard-library imports, resolving the mismatch between legacy dot-style examples such as `use result.{Result, Ok, Err}` and the canonical `::` syntax used elsewhere in the specs.

## Background

Per architectural review, early 57B task prose relied on dot-style examples such as:

```ash
use result.{Result, Ok, Err}
use runtime.Args
use io.Stdout
```

But the current module and import specs use `::` paths. This task resolves that mismatch by keeping one import/path syntax and defining how standard-library modules and the prelude are resolved.

## Resolved Design

- `::` is the only normative module and import path separator.
- The standard library is a compiler-provided root namespace, so standard-library modules are
  imported as top-level modules such as `result`, `runtime`, and `io` without a `std::` prefix.
- Standard-library module resolution follows the same file-based rules as user modules, rooted at
  `std/src/`.
- The standard prelude is defined by `std/src/prelude.ash`.
- `Option`, `Some`, `None`, `Result`, `Ok`, and `Err` are implicitly available in all modules;
  other standard-library modules and items require explicit `use` unless re-exported by the
  prelude.

## Requirements

Update SPEC-009 and SPEC-012 with:

1. **Import syntax**: Define `::` as the only normative module/import separator
2. **Stdlib namespace**: How is `result`, `runtime`, `io` resolved?
3. **Prelude**: What is auto-imported without explicit `use`?
4. **Module path resolution**: How does `runtime::Args` resolve to file?

## SPEC Sections to Update

### SPEC-009

- Define the standard-library root namespace
- Define file-based resolution rooted at `std/src/`
- Cross-reference the prelude boundary in SPEC-012

### SPEC-012

- Define `::` as the only valid import separator
- Mark dot-style stdlib imports invalid
- Define explicit standard-library import examples
- Define prelude-imported names and the boundary for explicit imports

## Examples

Valid:

```ash
use result::Result
use result::{Result, Ok, Err}
use runtime::Args
use io::Stdout
```

Invalid legacy examples:

```ash
use result.{Result, Ok, Err}
use runtime.Args
use io.Stdout
```

## Acceptance Criteria

- [x] Import syntax for stdlib is normatively specified
- [x] Module path resolution rules defined
- [x] Examples show valid stdlib imports
- [x] Prelude is explicitly defined
- [x] Legacy dot-style examples are marked invalid
- [x] 57B tasks can be written using normative syntax

## Related

- SPEC-009: Module system
- SPEC-012: Imports
- MCE-001: Entry point (uses stdlib imports)
- 57B tasks: All stdlib usage must use normative syntax

## Est. Hours: 3-4

## Blocking

- All 57B tasks using stdlib imports

## Completion Summary

SPEC-009 and SPEC-012 now define the standard library as a compiler-provided root namespace,
keep `::` as the only normative path separator, and tie implicit imports to the standard prelude
defined in `std/src/prelude.ash`.

Legacy planning and idea documents that still show dot-style imports remain follow-up cleanup
work under TASK-S57-7; this task completes the normative specification update only.
