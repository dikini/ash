# TASK-1362: Parser — `proof` keyword in impl blocks

## Status: ✅ Complete

## Description

Extend parser to accept `proof` blocks inside `impl` declarations.

## Requirements

1. Add `ProofDef` AST node with:
   - Name identifier
   - Parameter list
   - Optional `where` constraints
   - Body: `by_definition`, `by test "..."`, or expression body
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

- [x] `proof` parses inside `impl`
- [x] Supports `by_definition`, `by test`, and expression body
- [x] Parser test passes
- [x] No regressions

## Verification

- `cargo test -p ash-parser --test task_1362_proof_keyword_impl -- --nocapture` — 6 passed
- `cargo test -p ash-parser lexer::tests::test_all_keywords -- --nocapture` — 1 passed
- `cargo test -p ash-engine --test task_568_monomorphize --no-run` — passed
- `cargo test --workspace --no-run` — passed
- `cargo clippy -p ash-parser --all-targets --all-features -- -D warnings` — passed
- `cargo check --workspace` — passed

## Completion Notes

- Added `ProofDef` / `ProofBody` surface AST and `ImplDef.proofs` storage.
- Parsed impl-scoped `proof` declarations for `by_definition`, `by test "..."`, and expression bodies.
- Added `law`, `proof`, and `by_definition` to canonical parser keyword and lexer token surfaces.
- Module-scoped `proof` remains intentionally deferred to TASK-1363.

## Related

- [PLAN-136](../PLAN-136-INTERFACE-LAW-SYNTAX.md)
- [TASK-1360](TASK-1360-parser-law-in-interfaces.md)
