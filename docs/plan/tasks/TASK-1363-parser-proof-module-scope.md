# TASK-1363: Parser — `proof` keyword at module scope

## Status: 📝 Planned

## Description

Extend parser to accept module-scoped `proof` blocks.

## Requirements

1. Add `Definition::Proof(ProofDef)` variant to `Definition` enum in `surface.rs`
2. Add `proof` to parser's keyword recognition
3. Parse `proof` declarations in `parse_definitions` alongside other module items

## Files

- Modify: `crates/ash-parser/src/surface.rs` — add `Proof` variant to `Definition` enum
- Modify: `crates/ash-parser/src/parse_module.rs` — add `proof` parsing in `parse_definitions` loop
- Modify: `crates/ash-parser/src/lexer.rs` — add `proof` as recognized keyword
- Test: `crates/ash-parser/tests/proof_module_scope.rs`

## Acceptance Criteria

- [ ] `proof` parses at module scope
- [ ] Module file contains proofs alongside laws, types, functions
- [ ] Parser test passes
- [ ] No regressions

## Related

- [PLAN-136](../PLAN-136-INTERFACE-LAW-SYNTAX.md)
- [TASK-1362](TASK-1362-parser-proof-in-impls.md)
