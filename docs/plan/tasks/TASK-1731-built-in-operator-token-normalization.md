# TASK-1731: Preserve raw built-in infix operator tokens

## Status: ✅ Complete

## Summary

Preserve raw token spelling for built-in infix expressions alongside semantic `BinaryOp` information,
so diagnostics, formatting, and notation elaboration can distinguish source spelling without breaking
existing typechecking/lowering consumers.

## Specification Reference

- PLAN-169: `docs/plan/PLAN-169-SURFACE-EXPANSION-AND-NOTATION-ELABORATION.md`
- SPEC-095c §4 and §8: parsed surface shape and operator sections
- SPEC-098c §10: notation/operator-section erasure

## Dependencies

- 📝 TASK-1730: Notation declaration parser and AST

## Deferral / Planned-Feature Reconciliation

| Prior item | Source | Original reason | Prereqs now? | Decision | Gate |
|---|---|---|---|---|---|
| Raw operator spelling for built-in infix expressions | Phase 168 lowering inventory | Phase 168 only carried raw tokens for sections | Yes | Add optional raw token metadata to built-in infix nodes | Existing binary-op consumers pass and focused raw-token tests pass |

## Files

- `crates/ash-parser/src/surface.rs`
- `crates/ash-parser/src/parse_expr.rs`
- `crates/ash-parser/src/lower.rs`
- downstream exhaustive consumers as needed
- `crates/ash-parser/tests/task_1731_builtin_operator_token_preservation.rs`

## Requirements

1. Preserve the semantic `BinaryOp` enum for existing consumers.
2. Add raw operator-token metadata to built-in binary expressions or an equivalent source-origin sidecar.
3. Keep `lower_expr` behavior unchanged for existing built-in operators.
4. Add tests proving raw spelling survives parsing while semantic lowering still produces the same Core
   operator.
5. Run workspace check because `Expr::Binary` shape changes may affect downstream pattern matches.

## Current state

`Expr::OperatorSection` has `RawOperatorToken`, but ordinary `Expr::Binary` primarily exposes the
semantic `BinaryOp`.

## Target state

Built-in binary expressions carry enough raw token metadata for later notation diagnostics without
breaking current lowering/typechecking.

## TDD steps

1. Add parser tests that inspect raw spelling for several built-in operators.
2. Add a lowering non-regression test for an operator whose raw token is now preserved.
3. Extend carriers conservatively.
4. Fix downstream exhaustive matches without changing semantics.

## Verification

```yaml
strictness: clean
commands:
  - cargo test -p ash-parser --test task_1731_builtin_operator_token_preservation
  - cargo test -p ash-parser
  - cargo check --workspace
  - cargo fmt --check
  - git diff --check
checklist:
  - [x] Built-in binary semantics are unchanged.
  - [x] Raw token spelling is available to expansion diagnostics.
  - [x] Downstream exhaustive matches are updated deliberately.
```

## Implementation evidence

Implemented in Phase 169 final diff. Verified with:

- `cargo test -p ash-parser --test task_1731_builtin_operator_token_preservation`
- `cargo test -p ash-parser`
- `cargo check --workspace`

## Dispatch

```yaml
agent: hermes
reasoning: medium
max_turns: 25
toolsets: [terminal, file]
```

## Dependencies for next task

Provides raw operator-token inputs for local notation-table conflict diagnostics.
