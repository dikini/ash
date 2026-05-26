# TASK-557: Closure Syntax |params| => body

**Superseded syntax note (2026-05-26):** [SPEC-072](../../spec/SPEC-072-TOWER-CALLABLE-TYPE-AND-CLOSURE-SYNTAX.md) supersedes this task's pure closure spelling. New pure closure work must use `|params| -> body`; `|params| => body` is reserved for future Proc closures in closure-literal context. This file is preserved as historical Phase 80 planning context, not current implementation guidance.

**Phase:** 80
**Spec:** SPEC-031 §4.3, §8.3
**Depends on:** TASK-556
**Estimate:** 2 hours

## Description

Parse `|params| => expr` closure syntax as sugar for `fn(params) { expr }`.

## Requirements

### 1. Parse Closure Syntax

In `crates/ash-parser/src/parse_expr.rs`, add recognition of `|params| => expr`:

- `|x| => x + 1` -> `Expr::FnDef { params: [("x", None)], body: Expr::BinOp(x + 1) }`
- `|x, y| => x + y` -> `Expr::FnDef { params: [("x", None), ("y", None)], body: ... }`

Immediately desugar during parsing -- no new surface AST node.

### 2. Ambiguity Handling

`|` is currently not used as a prefix operator in expressions, so there should be no ambiguity. Verify this does not conflict with existing syntax.

## TDD Steps

1. Test: `|x| => x + 1` parses and produces `Expr::FnDef`
2. Test: `|x, y| => x + y` parses with two params
3. Test: closure in call position: `apply(|x| => x * 2, 5)` parses
4. Test: closure in let: `let f = |x| => x + 1;` parses
5. Verify `cargo test --all` passes

## Completion Checklist

- [ ] `|params| => expr` parses and desugars to `Expr::FnDef`
- [ ] No ambiguity with existing syntax
- [ ] Tests for closure syntax
- [ ] `cargo test --all` passes
- [ ] `cargo clippy` clean
- [ ] CHANGELOG.md updated
