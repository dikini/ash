# TASK-747: Do-Block Surface AST and Parser Substrate

## Status: 📝 Planned

## Description

Add parser and surface AST substrate for generalized `do:K { ... }` blocks without lowering them to `unit`/`bind` calls. This task is a substrate task: it must preserve enough surface information for later typed elaboration.

## Specification Reference

- [SPEC-054](../../spec/SPEC-054-GENERALIZED-TYPED-DO-NOTATION.md) §§5, 9, 13
- [SPEC-002](../../spec/SPEC-002-SURFACE.md)

## Dependencies

- ✅ TASK-746: spec/plan packet.
- 📝 Phase 104 closeout, unless user authorizes isolated parser-only work.

## Requirements

1. Add a target-carrying surface do-block node.
2. Parse `do:Act { ... }` and `do:Proc { ... }` as expressions.
3. Parse statement forms `let x = expr;`, `x <- expr;`, and final `return expr`.
4. Preserve target, binder, statement, and return spans.
5. Keep legacy `Expr::ActBlock` parsing intact until TASK-750 migrates it.
6. Do not lower `DoBlock` in this task.

## TDD Steps

### Step 1: Add failing parser tests

**Files:**

- Modify: `crates/ash-parser/src/parse_expr.rs`
- Modify: `crates/ash-parser/src/surface.rs`
- Test: existing parser expression tests in `crates/ash-parser/src/parse_expr.rs`

Add tests covering:

- `do:Act { return 1 }` parses as a do block with target `Act`.
- `do:Proc { x <- proc::unit(1); return x }` parses with one bind and one return.
- `do:Act { let x = 1; return x }` parses with pure let.
- `do:Act { return 1; }` is rejected or classified as the planned trailing-semicolon diagnostic carrier.
- legacy `act { ret 1; }` still parses as before.

### Step 2: Add surface carriers

Add a surface shape equivalent to:

```rust
pub struct DoTarget {
    pub name: Name,
    pub args: Vec<Type>,
    pub span: Span,
}

pub enum DoStmt {
    Let { name: Name, value: Box<Expr>, span: Span },
    Bind { name: Name, value: Box<Expr>, span: Span },
    Return { value: Box<Expr>, span: Span },
}

pub enum Expr {
    DoBlock { target: DoTarget, stmts: Vec<DoStmt>, span: Span },
    // existing variants...
}
```

Final field names may differ, but the AST must preserve all semantic information.

### Step 3: Implement parser

Add a parser for `do:target { ... }` before generic expression parsing can consume `do` as an identifier-like form. Reuse existing span utilities and whitespace/comment handling.

### Step 4: Verify parser substrate

Run:

```bash
cargo test -p ash-parser do_block -- --nocapture
cargo test -p ash-parser act_block -- --nocapture
cargo fmt --check
```

Expected: new do parser tests pass and existing act parser tests still pass.

## Verification Steps

- [ ] `cargo test -p ash-parser do_block -- --nocapture` passes.
- [ ] Existing `act_block` parser/lowering tests still pass.
- [ ] `cargo fmt --check` passes.
- [ ] No parser lowering of `DoBlock` happens in this task.
- [ ] Independent review confirms Phase 104 code paths were not touched.

## Dependencies for Next Task

Required by:

- TASK-748: target kind/dictionary resolution.
- TASK-749: typed do elaboration.
