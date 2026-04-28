# Phase 105: Generalized Typed Do-Notation Examples

Phase 105 introduces explicit typed do-notation for computation constructors:

```ash
do:K {
    let x = pure_expr;
    y <- computation_expr;
    return result_expr
}
```

The MVP supports compiler-known `Act` and `Proc` targets. It intentionally does not implement user-defined `Monad<M>`, `do:Result<_, E>`, `Option`/`List` targets, pattern binds, or implicit tower lifts.

## Files

- `01-do-act.ash` shows explicit `do:Act` sequencing.
- `02-act-sugar.ash` shows new-form `act { ... }` sugar for `do:Act`.
- `03-do-proc-from-act.ash` shows explicit `proc::from_act(...)` when embedding Act work in Proc.
- `04-legacy-act-migration.md` documents the temporary legacy `act { x = ...; ret ...; }` compatibility form and its migration.

## Key rules

- Use `let x = expr;` for pure lexical bindings.
- Use `x <- expr;` only when `expr` has the current block target type, such as `Act<T>` in `do:Act` or `Proc<T>` in `do:Proc`.
- End the block with `return expr` and no trailing semicolon.
- `do:Proc` does not implicitly lift `Act<T>`; use `proc::from_act(...)` explicitly.
- Ordinary `proc::par`, `proc::await`, `proc::join`, and `proc::from_act` operations remain ordinary qualified names, not implicit imports from `do:Proc`.
