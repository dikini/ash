# TASK-1552: Typecheck Destructors

## Status: ✅ Complete

## Description

Typecheck `let` destructuring. Verify that fields exist on the record type, types match, and there are no duplicates.

## Specification Reference

- [SPEC-091: Let Destructors](../../spec/SPEC-091-LET-DESTRUCTORS.md)
- [PLAN-155: Let Destructors](../PLAN-155-LET-DESTRUCTORS.md)

## Implementation

**Already existed** — The typechecker already handles `Pattern::Record` and `Pattern::Tuple` through `check_pattern.rs`.

The typechecker:
1. Looks up the type definition for the record/tuple type
2. Verifies fields exist on the record type
3. Checks type compatibility for each bound variable
4. Reports errors for missing fields or type mismatches

No changes needed to the typechecker.

## Verification

- [x] `cargo test -p ash-typeck` — passes
- [x] `cargo test -p ash-cli --test stdlib_corpus_check` — 54/54 pass

## Dependencies

- TASK-1550 (parser)
- TASK-1551 (AST)

## Closeout Checklist

- [x] Typechecker already supports destructuring (no changes needed)
- [x] Verified by tests
