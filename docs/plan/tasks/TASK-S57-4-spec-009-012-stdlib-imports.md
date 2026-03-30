# TASK-S57-4: Update SPEC-009/SPEC-012 with Stdlib Import/Namespace Rules

## Status: ⬜ Pending

## Description

Update SPEC-009 (Module System) and/or SPEC-012 (Imports) with normative rules for standard library imports, resolving the mismatch between current `use result.{Result, Ok, Err}` syntax and the Rust-style `::` import syntax in current specs.

## Background

Per architectural review, 57B implementation relies heavily on:
```ash
use result.{Result, Ok, Err}
use runtime.Args
use io.Stdout
```

But current import spec (SPEC-012-IMPORTS.md:10-29) uses Rust-style `::` syntax. This mismatch must be resolved before implementation.

**Options:**
1. Update SPEC-012 to support dot-style `use module.{item}` syntax
2. Rewrite 57B to use current `::` syntax: `use result::Result`
3. Define stdlib as special case with different import rules

## Requirements

Update SPEC-009 or SPEC-012 with:

1. **Import syntax**: Resolve `use module.{item}` vs `use module::item`
2. **Stdlib namespace**: How is `result`, `runtime`, `io` resolved?
3. **Prelude**: What is auto-imported without explicit `use`?
4. **Module path resolution**: How does `runtime.Args` resolve to file?

## SPEC Sections to Update

### Option A: Update SPEC-012 to Support Dot Syntax

Update import grammar:
```
use_path ::= simple_path | dotted_import
dotted_import ::= identifier "." "{" identifier_list "}"
```

Examples:
```ash
use result.{Result, Ok, Err}  -- new dot syntax
use runtime.Args              -- simple import
```

### Option B: Use Current Syntax, Update 57B

Keep SPEC-012 as-is (Rust-style), rewrite 57B:
```ash
use result::Result
use result::Ok
use result::Err
use runtime::Args
```

### Option C: Mixed Approach

- Simple imports: `use runtime.Args` (dot as namespace separator)
- Explicit lists: `use result.{Result, Ok}` (dot + braces)
- Or: `use result::{Result, Ok}` (double colon + braces)

## Open Questions

### Q1: Dot vs Double-Colon
- Is `.` namespace separator or field access?
- Can both coexist? `use foo.bar` vs `use foo::bar`?

### Q2: Stdlib Location
- Where do `result`, `runtime`, `io` resolve?
- `std/src/result.ash` → module `result`?
- `std/src/runtime/args.ash` → module `runtime.Args`?

### Q3: Prelude
- Is `Result`/`Option` in prelude (auto-imported)?
- Or must always `use result.{Result}`?

### Q4: Nested Modules
- Can stdlib have nested modules?
- `std/src/io/stdout.ash` → `io.stdout`?

## Acceptance Criteria

- [ ] Import syntax for stdlib is normatively specified
- [ ] Module path resolution rules defined
- [ ] Examples show valid stdlib imports
- [ ] Prelude (if any) explicitly defined
- [ ] 57B tasks can be written using normative syntax

## Related

- SPEC-009: Module system
- SPEC-012: Imports
- MCE-001: Entry point (uses stdlib imports)
- 57B tasks: All stdlib usage must use normative syntax

## Est. Hours: 3-4

## Blocking

- All 57B tasks using `use result.{...}` or `use runtime.X`
