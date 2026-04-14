# TASK-556: Parse fn Expressions and Named Local Functions

**Phase:** 80
**Spec:** SPEC-031 §8.2, §8.4
**Depends on:** TASK-555
**Estimate:** 5 hours

## Description

Parse `fn(params) { body }` as an expression and `fn name(params) { body }` in workflow bodies and block statements. Named local functions desugar to `let name = fn(params) { body }`.

## Requirements

### 1. Anonymous fn Expression in parse_expr

Extend `crates/ash-parser/src/parse_expr.rs` to recognize `fn(params) [-> type] { body }` as an expression. Produce `Expr::FnDef { params, return_type, body }`.

This is permissive -- the parser does not reject `fn` expressions at module scope. That validation happens during lowering.

### 2. Named Local fn in Workflow Bodies

Extend `crates/ash-parser/src/parse_workflow.rs` to recognize `fn name(params) { body }` as a workflow statement. Desugar to `Workflow::Let { pattern: Pattern::Name("name"), expr: Expr::FnDef { params, body }, continuation, span }`.

No new `Workflow` variant needed -- reuses `Workflow::Let`.

### 3. Named Local fn in Block Statements

Extend the block parser to recognize `fn name(params) { body }` in block statement position. Desugar to `BlockStmt::Let { pattern: Pattern::Name("name"), expr: Expr::FnDef { ... }, span }`.

No new `BlockStmt` variant needed -- reuses `BlockStmt::Let`.

### 4. Post-Parse Validation

During lowering (`lower.rs`), when encountering `Expr::FnDef` at module top-level, emit a lowering error: "fn expressions are not valid at module scope; use `pub fn` instead".

### 5. Update Test

Change `parse_fn_rejects_nested_fn` to expect parse success + lowering rejection (not parse failure).

## TDD Steps

1. Test: `fn(x) { x + 1 }` parses as `Expr::FnDef`
2. Test: `fn(x: Int) -> Int { x + 1 }` parses with types
3. Test: `fn helper(x) { x + 1 }` in workflow body desugars to `Workflow::Let { expr: FnDef }`
4. Test: `fn helper(x) { x + 1 }` in block desugars to `BlockStmt::Let { expr: FnDef }`
5. Test: `fn(x) { x }` at module scope -> lowering error
6. Test: update `parse_fn_rejects_nested_fn` to expect parse-success + lower-reject
7. Verify `cargo test --all` passes

## Completion Checklist

- [ ] `fn(params) { body }` parses as expression
- [ ] Named `fn name(params) { body }` in workflow body -> `Workflow::Let`
- [ ] Named `fn name(params) { body }` in block -> `BlockStmt::Let`
- [ ] Post-parse validation rejects FnDef at module scope
- [ ] `parse_fn_rejects_nested_fn` test updated
- [ ] `cargo test --all` passes
- [ ] `cargo clippy` clean
- [ ] CHANGELOG.md updated
