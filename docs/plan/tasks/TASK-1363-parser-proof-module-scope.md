# TASK-1363: Parser — `proof` keyword at module scope

## Status: ✅ Complete

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

- [x] `proof` parses at module scope
- [x] Module file contains proofs alongside laws, types, functions
- [x] Parser test passes
- [x] No regressions

## Verification

- `cargo test -p ash-parser --test task_1363_proof_keyword_module_scope -- --nocapture` — 3 passed
- `cargo test -p ash-lsp-core` — 49 unit tests, 1 integration test, and doctests passed
- `cargo clippy -p ash-parser -p ash-lsp-core --all-targets --all-features -- -D warnings` — passed
- `cargo check --workspace` — passed

## Completion Notes

- Added `Definition::Proof(ProofDef)` for module-scoped proof declarations.
- Reused the TASK-1362 proof parser at top-level and inline-module definition dispatch points.
- Updated LSP exhaustive matches for proof completion, hover, goto, and symbol traversal compatibility.

## Related

- [PLAN-136](../PLAN-136-INTERFACE-LAW-SYNTAX.md)
- [TASK-1362](TASK-1362-parser-proof-in-impls.md)
