# TASK-1388 Audit: Stdlib Law/Proof Readiness

## Status: ✅ Complete

## Test results

Phase 136 law tests: PASS (0 filtered failures; all targets compiled and ran).
Phase 1373 stdlib algebra integration: PASS.

## Law-body syntax

The law parser (`parse_law_definition` in `crates/ash-parser/src/parse_module.rs:1513-1541`) parses the law proposition as a **full expression** via `expr()`. This means:
- Closures like `fn(x) => x`, `fn(x) => g(f(x))`, and `fn(f) => fn(g) => fn(x) => f(g(x))` are all parseable as law proposition bodies.
- The existing Semigroup/Monoid laws already use method calls like `eq.equiv(...)` in law bodies, proving the parser handles complex expressions.

**Conclusion:** All planned law-body forms (Functor, Applicative, Monad) are syntactically supported.

## Proof-body semantics

The typechecker proof-validation code (`crates/ash-typeck/src/type_env/surface_types_laws_and_prelude.rs:1927-1929`) shows:

```rust
match &proof.body {
    ProofBody::ByDefinition | ProofBody::ByTest { .. } => {}
    ProofBody::Expr(expr) => checker.visit_expr(expr),
}
```

**`by_definition` is syntactically accepted, NOT semantically validated.** The match arm is empty — it accepts the proof without checking definitional equality. Similarly, `by test` just records the delegation.

**`ProofBody::Expr`** gets fuel-checked (step budget) but does not validate the expression against the law proposition.

## Proof policy decision

For Option/Result proofs in TASK-1393:
- Use **`by test "..."`** as the safe baseline for all proofs.
- Do NOT use `by_definition` since it is not semantically validated — it would overclaim.
- `ProofBody::Expr` could be used if the expression typechecks, but it provides no semantic proof guarantee at this stage.

## Recommended law-body forms

| Interface | Laws | Body form |
|---|---|---|
| Semigroup | `associativity` | Already exists: `eq.equiv(append(append(a, b), c), append(a, append(b, c)))` |
| Monoid | `left_identity`, `right_identity` | Already exists: `eq.equiv(append(empty(), a), a)` |
| Functor | `identity`, `composition` | `eq.equiv(map(value, fn(x) => x), value)` etc. |
| Applicative | `identity`, `homomorphism`, `interchange`, `composition` | `eq.equiv(apply(pure(fn(x) => x), value), value)` etc. |
| Monad | `left_identity`, `right_identity`, `associativity` | `eq.equiv(bind(unit(a), f), f(a))` etc. |

## Downstream task patches

No changes to planned code shapes are needed. The parser supports all required forms. The only policy change is: **all Option/Result proofs must use `by test`**, not `by_definition`.
