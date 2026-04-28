# Legacy Act Block Migration

Phase 105 keeps legacy expression-level Act blocks as a temporary compatibility carrier:

```ash
act {
    x = read(path);
    ret x;
}
```

Prefer the new typed-do statement forms:

```ash
act {
    x <- read(path);
    return x
}
```

or the explicit target form:

```ash
do:Act {
    x <- read(path);
    return x
}
```

Migration rules:

- `ret expr;` becomes final `return expr` with no trailing semicolon.
- `x = effectful_expr;` becomes `x <- effectful_expr;` when the RHS has type `Act<T>`.
- `x = pure_expr;` becomes `let x = pure_expr;` when the RHS is pure.
- A value of type `Act<T>` inside `do:Proc` must be wrapped with `proc::from_act(...)`; no implicit lift occurs.
