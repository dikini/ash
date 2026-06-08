# TASK-1361: Parser — `law` keyword at module scope

## Status: ✅ Complete

## Description

Extend parser to accept `law` declarations at module top level.

## Requirements

1. Add `Definition::Law(LawDef)` variant to `Definition` enum in `surface.rs`
2. Add `law` to parser's keyword recognition (lexer.rs or parse_module.rs string matching)
3. Parse `law` declarations in `parse_definitions` alongside other module items
4. Same syntax as interface laws

## Files

- Modify: `crates/ash-parser/src/surface.rs` — add `Law` variant to `Definition` enum
- Modify: `crates/ash-parser/src/parse_module.rs` — add `law` parsing in `parse_definitions` loop
- Modify: `crates/ash-parser/src/lexer.rs` — add `law` as recognized keyword
- Test: `crates/ash-parser/tests/law_module_scope.rs`

## Acceptance Criteria

- [x] `law` parses at module scope
- [x] Module file contains laws alongside types, functions, impls
- [x] Parser test passes
- [x] No regressions

## Completion Notes

- Added `Definition::Law(LawDef)` and module-definition dispatch for `law`.
- Parser accepts module-scoped laws at top level and in inline modules.
- LSP exhaustive-match fallout from the new definition variant was repaired before TASK-1362.

## Related

- [PLAN-136](../PLAN-136-INTERFACE-LAW-SYNTAX.md)
- [TASK-1360](TASK-1360-parser-law-in-interfaces.md)
