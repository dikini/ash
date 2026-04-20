# NOTE-003: Missing `Expr::Let` in Core IR — Pure/Imperative Boundary Contamination

## Status: Open Finding

## Summary

Ash's spec defines two cleanly separated evaluation paths — pure expressions and imperative (workflow) statements. The implementation violates this boundary because the core IR has no representation for expression-level `let`-binding. The only `Let` in the system is `Workflow::Let`, which is the imperative/monadic form. This forces fn-body let-sequencing through an ad-hoc desugaring that rewrites it as `Expr::Match` — erasing the semantic distinction between pure scope extension and pattern matching with potential non-match failure.

## The Two Paths

### Pure Expressions (fn bodies, closures)

Judgment form (SPEC-004 §3.2): `E ⊢ e ⇓ v`

- Environment `E` maps variables to values. No effects, no trace, no provenance.
- `let x = e; rest` is scope extension — equivalent to `let x = e in rest`.
- The semicolon is a syntactic separator between let-bindings and the tail expression.
- Result domain: `Value | Panic`.

### Imperative Statements (workflow bodies)

Judgment form (SPEC-004 §3.1): `Γ, C, P, Ω, π ⊢wf w ⇓ out`

- Context carries capabilities `C`, policy `P`, obligations `Ω`, provenance `π`.
- `;` is monadic bind — composes two effectful computations, threading context forward.
- `let x = e; rest` evaluates `e` (possibly effectful), binds the result, and continues.
- Every step may produce trace entries, provenance records, obligation mutations.

### Boundary Rule

Pure expressions **can** be embedded into imperative statements (a workflow `let` can bind a pure expression result). Imperative statements **cannot** become pure — they carry context that the pure judgment form has no vocabulary for.

---

## The Gap

### What the spec says

SPEC-027 §2.2 defines fn bodies:

```
Body ::= Statement* Expr
Statement ::= let <pattern> = Expr ;
```

SPEC-027 §4.1 gives the evaluation rule:

```
(LET)
  E ⊢ e : v
  E[x↦v] ⊢ rest : v'
  ──────────────────────────────
  E ⊢ let x = e; rest : v'
```

This is pure expression composition. The `;` separates let-bindings that extend the environment, not effectful steps.

### What the core IR has

The core `Expr` enum (`ash-core/src/ast.rs`) contains:

```
Literal, Variable, FieldAccess, IndexAccess,
Unary, Binary, Call, Match, Constructor,
IfLet, Spawn, Split, CheckObligation, FnDef, FnApply
```

**There is no `Expr::Let`.** The only `Let` in the system is `Workflow::Let` — the imperative/monadic form with a `continuation` field.

### What the parser produces

`parse_fn_expr_body` (`parse_expr.rs:326`) produces:

```rust
Expr::Block {
    statements: Vec<BlockStmt::Let>,  // flat list of let-bindings
    tail_expr: Option<Expr>,          // final expression
    span: Span,
}
```

This is an imperative-style flat block, not nested expression composition.

### What the lowerer does

`lower_expr` (`lower.rs:1512`):

```rust
Expr::Block { .. } => Err(LoweringError::ExprNotLowerable { kind: "block" }),
```

The lowerer rejects `Expr::Block` entirely. Fn bodies with let-bindings cannot pass through the standard lowering path.

---

## The Workaround and Its Problem

### module_loader normalization

`normalize_imported_callable_expr` (`module_loader.rs:739`) converts `Expr::Block` into nested `Expr::Match`:

```
{ let x = e1; let y = e2; tail }
→ match e1 { x => match e2 { y => tail } }
```

This only runs for imported `pub fn` callables (cross-module imports). It does not run for:
- Inline `fn(x) { ... }` expressions
- Top-level `fn` definitions parsed by `parse_fn_definition`

### Why this is the wrong primitive

For irrefutable patterns (simple variable bindings like `let x = e`), `match e { x => rest }` is semantically equivalent to `let x = e in rest` because the pattern always matches. But:

1. **Refutable patterns silently diverge.** `let Some { value: x } = expr; rest` would desugar to `match expr { Some { value: x } => rest }` with no arm for `None`. At runtime, this produces a match failure, not a type error or binding failure.

2. **It erases the semantic distinction.** `match` is a discriminating construct that may fail. `let` is a binding construct that always succeeds (for irrefutable patterns) or is statically rejected (for refutable patterns). These are not the same operation.

3. **It exists in the wrong layer.** The normalization lives in the module loader (engine layer), not in the lowering pipeline (parser layer). This means the same syntax takes different code paths depending on whether the fn is imported or inline.

---

## Current State: What Actually Works

| Fn body form | Path | Works? |
|---|---|---|
| Single expression: `fn f() -> Int { x + 1 }` | parse_fn_expr_body → Expr::Block { statements: [], tail_expr: Some(x+1) } | Lowered to x+1 (block stripped by normalization) |
| Let-sequenced body (imported pub fn) | module_loader → normalize → nested Match | Works via workaround |
| Let-sequenced body (inline fn expr) | parse_fn_expr_body → Expr::Block → lower_expr | **Fails** — LoweringError |
| Let-sequenced body (top-level fn def) | parse_fn_definition → process_program_definitions → lower_expr | **Fails** — LoweringError |

---

## Proposed Resolution

### Option A: Add `Expr::Let` to core IR

Add a core `Expr::Let` variant representing pure scope extension:

```rust
// In ash-core/src/ast.rs
pub enum Expr {
    // ... existing variants ...
    Let {
        pattern: Pattern,
        expr: Box<Expr>,      // the bound expression
        body: Box<Expr>,      // continuation in the same expression
    },
}
```

Then:
1. The lowerer desugars `Expr::Block { [Let ...], tail }` into nested `CoreExpr::Let`.
2. The evaluator handles `Expr::Let` by evaluating `expr`, extending the environment, and evaluating `body`.
3. `Workflow::Let` remains the imperative/monadic form (with continuation semantics).

This gives the core IR the vocabulary to represent pure let-binding as distinct from imperative let-binding.

### Option B: Desugar at parse time

Have `parse_fn_expr_body` produce nested `Expr::Let` directly instead of `Expr::Block`. This avoids adding a new core variant but requires the surface AST to have an `Expr::Let` form (it currently doesn't).

### Option C: Desugar at lowering time

Keep `Expr::Block` in the surface AST but have `lower_expr` handle it by desugaring to nested `CoreExpr::Let` (if Option A) or nested `CoreExpr::Match` (if keeping the core minimal). This moves the normalization out of the module loader and into the proper lowering pipeline.

### Recommended

**Option A + C**: Add `CoreExpr::Let`, and have the lowerer desugar `Expr::Block` into nested `CoreExpr::Let`. This:
- Gives the core IR the right primitive
- Moves the desugaring to the correct layer
- Removes the module_loader workaround
- Preserves the semantic distinction between pure let-binding and match

---

## Secondary Finding: And/Or Short-Circuit

While investigating the pure/imperative boundary, SPEC-004 §4.6 defines `EXPR-AND-FALSE`:

```
  Γ ⊢e e1 ⇓ Bool(false)
  ────────────────────────────
  Γ ⊢e e1 and e2 ⇓ Bool(false)
```

The right operand is not evaluated. But `eval.rs` evaluates both operands eagerly before calling `eval_binary_op`. This is a separate bug — the implementation violates the spec's short-circuit semantics for boolean operators.

---

## References

- SPEC-001 §2.6 — canonical core expression forms
- SPEC-002 — surface grammar (fn_def, block_expr, if_expr, workflow if_stmt)
- SPEC-004 §3.1 — workflow judgment form
- SPEC-004 §3.2 — expression judgment form
- SPEC-004 §4.6 — expression evaluation rules (including EXPR-AND-FALSE)
- SPEC-025 — small-step semantics (LET-EVAL, LET-BIND, SEQ-STEP)
- SPEC-027 §2.2 — fn body grammar
- SPEC-027 §4.1 — fn body evaluation rules
- `crates/ash-parser/src/parse_expr.rs:326` — parse_fn_expr_body
- `crates/ash-parser/src/lower.rs:1512` — Expr::Block rejection
- `crates/ash-engine/src/module_loader.rs:739` — normalize_imported_callable_expr
- `crates/ash-core/src/ast.rs` — core Expr enum (no Let variant)
- `crates/ash-interp/src/eval.rs` — eval_expr (no Expr::Let handling)
- NOTE-001 — Workflow Computation Type (`comp T`)
