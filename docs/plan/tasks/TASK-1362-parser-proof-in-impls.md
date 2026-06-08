# TASK-1362: Parser — `proof` keyword in impl blocks

## Status: 📝 Planned

## Description

Extend parser to accept `proof` blocks inside `impl` declarations.

## Requirements

1. Add `ProofDef` AST node with:
   - Name identifier
   - Parameter list
   - Optional `where` constraints
   - Body: `by_definition`, `by_test`, or block expression
2. Add `proofs: Vec<ProofDef>` field to `ImplDef`
3. Add `proof` parsing rule to `parse_module.rs` (inside `parse_impl_definition`)
4. Parse `proof` inside `impl { ... }`
5. Add `by_definition` keyword to lexer (`crates/ash-parser/src/lexer.rs`)

## Files

- Modify: `crates/ash-parser/src/surface.rs` — add `ProofDef` struct and `proofs` field to `ImplDef`
- Modify: `crates/ash-parser/src/parse_module.rs` — add `proof` parsing inside impl bodies
- Modify: `crates/ash-parser/src/lexer.rs` — add `by_definition` as keyword token
- Test: `crates/ash-parser/tests/proof_syntax.rs`

## Acceptance Criteria

- [ ] `proof` parses inside `impl`
- [ ] Supports `by_definition`, `by test`, and block body
- [ ] Parser test passes
- [ ] No regressions

## Related

- [PLAN-136](../PLAN-136-INTERFACE-LAW-SYNTAX.md)
- [TASK-1360](TASK-1360-parser-law-in-interfaces.md)
