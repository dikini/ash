# TASK-1361: Parser — `law` keyword at module scope

## Status: 📝 Planned

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

- [ ] `law` parses at module scope
- [ ] Module file contains laws alongside types, functions, impls
- [ ] Parser test passes
- [ ] No regressions

## Related

- [PLAN-136](../PLAN-136-INTERFACE-LAW-SYNTAX.md)
- [TASK-1360](TASK-1360-parser-law-in-interfaces.md)
