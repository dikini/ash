# NOTE-012: Mutual Recursion and CPS Translation Design (Target-Aware Notes)

This note translates the Gemini discussion examples into the project’s three concrete syntactic surfaces:

- **Ash surface sketch** (human-friendly feature sketch)
- **Core text** (validator/typecheck/lower fixture form, `.core`)
- **CPS IR** (serializer output shape, `*.core.cps.golden` style)

It is scoped to the currently implemented Core/Lowering subset.

## 1) Even/Odd Mutual Recursion

### Ash surface sketch

```ash
fn even(n: Int): Bool {
  if n == 0 {
    true
  } else {
    odd(n - 1)
  }
}

fn odd(n: Int): Bool {
  if n == 0 {
    false
  } else {
    even(n - 1)
  }
}

even(4)
```

### Core

`let-rec` is encoded as a recursive aggregate so both functions can be captured mutually.

```core
(let-rec pair : (tuple (fn (Int) -> Bool {}) (fn (Int) -> Bool {}))
  (tuple
    (lam ((n : Int)) : {}
      (let-prim n_is_zero eq (n (lit-int 0))
        (if n_is_zero
          (lit-bool true)
          (let-prim n1 sub (n (lit-int 1))
            (let-prim odd_fn (tuple-get-1 pair)
              (call odd_fn ((n1)))))))
    (lam ((n : Int)) : {}
      (let-prim n_is_zero eq (n (lit-int 0))
        (if n_is_zero
          (lit-bool false)
          (let-prim n1 sub (n (lit-int 1))
            (let-prim even_fn (tuple-get-0 pair)
              (call even_fn ((n1))))))))
  (let-prim start_fn (tuple-get-0 pair)
    (call start_fn ((lit-int 4)))))
```

### CPS (conceptual, constructor-style)

```lisp
(LetRec (name . "pair")
  (value . (Tuple
    (Lam (params "n") (cont . "__k_even")
      (body
        (LetPrim (name . "n_is_zero") (op . Eq)
          (args (Var . "n") (Int . 0))
          (body
            (If (cond Var . "n_is_zero")
              (then_branch
                (Jump (cont Var . "__k_even") (arg Bool . true) (row (items))))
              (else_branch
                (LetPrim (name . "n1") (op . Sub)
                  (args (Var . "n") (Int . 1))
                  (body
                    (LetPrim (name . "odd_fn") (op . TupleGet 1)
                      (args (Var . "pair"))
                      (body
                        (Call (func Var . "odd_fn") (args (Var . "n1")) (cont Var . "__k_even") (row (items))))))))
              (row (items)))))
    (Lam ...)
    ))
  (body
    (LetPrim (name . "start_fn") (op . TupleGet 0)
      (args (Var . "pair"))
      (body (Call (func Var . "start_fn") (args (Int . 4)) (cont Label . "halt") (row (items))))))))
```

In this project, recursion stays local to the value representation; the second function is resolved via projections from the shared recursive aggregate.

## 2) Direct recursion and why CPS makes recursive flow first-class

### Ash surface sketch

```ash
fn sum(n: Int): Int {
  if n == 0 {
    0
  } else {
    n + sum(n - 1)
  }
}

sum(3)
```

### Core

```core
(let-val sum : (fn (Int) -> Int {})
  (lam ((n : Int)) : {}
    (let-prim n_is_zero eq (n (lit-int 0))
      (if n_is_zero
        (lit-int 0)
        (let-prim n1 sub (n (lit-int 1))
          (let-prim rec (call sum ((n1)))
            (let-prim out add (n rec)
              out))))))
  (call sum ((lit-int 3))))
```

### CPS intuition

```lisp
(LetVal (name . "sum")
  (value .
    (Lam (params "n") (cont . "__k_sum")
      (body
        (If (cond Var . "n_is_zero")
          (then_branch (Jump (cont Var . "__k_sum") (arg Int . 0) (row (items)))
          (else_branch
            (LetPrim ...
              (Call (func Var . "sum") (args (Var . "n1")) (cont Label . "__k_sum") (row (items))))))))
  (body
    (Call (func Var . "sum") (args (Int . 3)) (cont Label . "halt") (row (items)))))
```

Conceptually, both direct and mutual recursion become ordinary continuation-passing flow; tail-position concerns are handled by the CPS shape rather than ad-hoc flow analyses.

## 3) Letrec mutual recursion in CPS runtime terms

In the conversation, the key problem is the circular capture constraint:

- `even` needs to capture `odd`
- `odd` needs to capture `even`

Both must see each other in the same recursive environment.

### Core pattern (prototype-consistent)

```core
(let-rec pair : (tuple (fn () -> Unit {}) (fn () -> Unit {}))
  (tuple
    (lam () : {}
      (call (tuple-get-1 pair) ()))
    (lam () : {}
      (call (tuple-get-0 pair) ())))
  (let-prim a (tuple-get-0 pair)
    (let-prim b (tuple-get-1 pair)
      (trap "use-some-operation-here"))))
```

### Two-stage initialization (same idea, regardless of recursion depth)

1. Allocate placeholders for recursive names in a shared environment frame.
2. Evaluate each recursive RHS under that shared frame so each RHS captures references to all names.
3. Replace each placeholder with the evaluated closure.
4. Evaluate body under the frame.

This is exactly the “placeholder + overwrite” pattern already used by the project’s recursive environment design.

## 4) TCO and CPS (from the design discussion)

### Pros/cons of mutual recursion in practice

| Style | Mutual recursion | Single-function recursion |
|---|---|---|
| Code model | Alternating states/phases become explicit | Monolithic state-machine in one function |
| Readability | High when state partition is natural | Easier to start, harder at scale |
| Optimizer burden | One recursive call discipline in CPS | Similar, but still recursive-tail reasoning needed |
| Debugging | More call-path jumps | Fewer conceptual entities |
| Extension | Pairs well with effect handlers/continuations | Simpler one-function mental model |

### TCO path

In direct style, TCO is a control-flow property that needs analysis.

In Core-CPS, **every call is in continuation-passing shape**, so tail-call reasoning becomes structural at IR level; the hard work shifts to backend runtime policy (trampoline vs native strategies), not recursive shape detection.

### Runtime tradeoff to remember

CPS is not “free lunch”:

- you typically avoid recursive stack growth by trampoline or continuation stack strategy,
- `Call/Jump`-dense control needs stronger runtime discipline,
- but recursive and mutually recursive patterns are much easier to compile consistently.

## 5) JIT-relevant note (from the same thread)

For direct-style JITs, the current architecture is simpler at the backend but has explicit TCO analysis obligations.

For CPS-first JITs, the allocator pressure moves to continuation escape analysis: many continuation values can become heap-resident because they are passed around structurally.

Practical takeaway for this implementation slice:

- keep CPS IR for clarity and effect accounting,
- avoid adding extra escape-heavy continuation allocation without a clear strategy,
- and preserve row/handler metadata exactly where calls/raises/resume are introduced.

## 6) Current-project relevance

This note intentionally maps the conversation to concrete syntax already supported by the available target surfaces:

- **Core** forms for recursive aggregates and projection-based mutual references,
- **CPS** constructor shape (especially `LetRec`, `LetCont`, `If`, `Call`, `Jump`, `Handle`) as produced by lowering,
- and a surface sketch for intent only (useful when validating whether recursion examples are representable at the boundary).

If you want, I can now add a strict “target matrix” for each example row by row: exact Core parser-ready form, exact `.core.cps.golden` form, and a short invariance check for the row effects expected at every call/handle boundary.
