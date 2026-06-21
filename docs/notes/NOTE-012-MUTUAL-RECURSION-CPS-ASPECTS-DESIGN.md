# Design Exploration: Mutual Recursion, CPS, and JIT Tradeoffs

## Source

- Origin: `~/Downloads/[Gemini Conversation] Mutual Recursion_ Pros and Cons`
- Conversation topic: mutual recursion and its interaction with Continuation-Passing Style (CPS), tail-call optimization (TCO), JIT strategy, and `letrec` mutual recursion in interpreters.

## 1) Purpose and Design Question

The conversation was an exploration of whether, for a language design like Ash with advanced control-flow features, **mutual recursion** should be introduced directly in current direct style semantics or through/alongside a **CPS** direction.

Primary concerns discussed:

- Semantic clarity of mutually recursive definitions.
- Tail recursion and tail-call semantics.
- Runtime cost and compiler/JIT complexity.
- Correct handling of recursive bindings (`letrec`) in CPS.

---

## 2) Mutual Recursion: Pros and Cons

### Pros

- **Natural modeling of alternating state/domain layers**
  - Useful when the domain has paired states, nested alternations, or mutually dependent roles.
  - Examples from the discussion: parser-like structures, language grammars, and explicit state machines.
- **Better decomposition**
  - Can split complex logic into smaller responsibilities.
  - Each function handles one “mode/state,” improving readability when used correctly.
- **Separation of concern**
  - Encodes domain alternation explicitly rather than forcing all logic into one function.

### Cons

- **Stack behavior risk**
  - Calls alternate across functions and can still create deep call chains.
  - If recursion is unbounded, stack overflow is a practical risk.
- **Tail-call implementation complexity**
  - In direct style, mutual calls across functions often increase compiler complexity for tail optimization.
- **Debugging complexity**
  - Execution path is less linear and harder to trace mentally.
- **Tighter coupling**
  - Mutually recursive functions can become difficult to evolve in isolation.

### Quick comparison

| Concern | Mutual recursion | Standard recursion |
|---|---|---|
| Shape | Multiple alternating function definitions | Single recursive definition |
| Best fit | Alternating states / grammar-like control | Self-recursive patterns |
| Cognitive load | Higher (inter-function control transfer) | Moderate |
| Tail-call handling (direct style) | More complex | Simpler |

---

## 3) CPS and TCO Complexity

The discussion emphasized a strong shift in where complexity lives:

### In direct style:

- TCO is implemented as an optimization pass/pattern that must prove call position.
- Mutual recursion can be expensive to optimize because of call-stack frame compatibility concerns.

### In CPS:

- Functions never return normally; control is transferred via explicit continuations.
- Every call is structurally in tail position.
- So TCO becomes less a compile-time “special case” and more a semantic property of the IR.

### Remaining complexity in CPS:

- Runtime needs to avoid native stack overflow by design.
- Typical strategies discussed:
  - **Trampoline loop**
  - **Heap-allocated / safe stack techniques** for continuations.

This shifts complexity from tail-position analysis toward runtime execution strategy.

---

## 4) Direct Style Interpreter vs CPS JIT Path

The thread compared a JIT-ready direct-style path with a CPS-first path:

- **Direct style + JIT**
  - Easier alignment with native CALL/RET.
  - TCO is harder: JIT must actively transform tail positions to efficient jumps/stack rewrites.
- **CPS IR + JIT**
  - More convenient around advanced control features (continuations/effects/coroutines).
  - Harder engineering effort around continuation allocation and escape analysis.
  - Can lose some native return-prediction efficiency if continuation flow is not transformed into low-overhead jumps.

### Recommendation implied by conversation

- If first priority is “high-performance JIT immediately,” direct style may remain simpler.
- If language design strongly favors first-class control constructs, CPS is structurally cleaner and more expressive; then invest in escape analysis and runtime strategy.

---

## 5) CPS `letrec` Mutual Recursion (critical for Ash-style control features)

Conversation identified `letrec` as the major blocker.

### Why mutual recursion is hard in CPS

Closures capture their environment at creation time. For `f` and `g` to reference each other, they must capture a common environment containing placeholders first, then later be replaced by real closures.

### Safe implementation sketch from discussion

1. **Pre-allocate all recursive names** in environment as mutable placeholders (`Uninitialized`, `Thunk`, or dedicated cell type).
2. Evaluate all binding bodies **in the same extended environment** so each closure can refer to all placeholders.
3. Once all closure values are computed, **mutate each placeholder** to the real closure (“tie the knot”).
4. Evaluate `letrec` body with this now-complete environment.

This is a classic two-phase/two-pass binding strategy and matches mutable-environment approaches already used in many CPS implementations.

### Extra constraint noted

- `letrec` in Scheme-like semantics expects RHSs to be function-like definitions (lambdas/closures), not arbitrary self-referential eager expressions.

---

## 6) Ash-specific exploration notes

For Ash’s planned/ongoing CPS direction, this conversation supports:

- Prioritizing a **correct shared recursive binding strategy** before broadening `letrec` support.
- Treating `letrec` multi-binding as a batch process (reserve → evaluate all → write-back → execute body).
- Expecting this to be a necessary prerequisite for reliable mutual recursion in features that depend on continuations/coroutines/effects.

## 7) Actionable next steps

1. Finalize `letrec` binder representation for multi-binding in CPS.
2. Add a regression with two or more mutually recursive closures in shared env to validate:
   - both closures visible during capture,
   - both callable without forcing,
   - no uninitialized capture errors at force/entry points.
3. Evaluate two runtime variants incrementally:
   - trampoline-only baseline,
   - selective native/jit optimizations once continuation escape behavior is stable.

## 8) Concrete cross-layer translations

To make the discussion usable for implementation, the same example can be read in three layers:

- target Ash surface (conceptual),
- Core (`.core` text), and
- CPS IR (informal lowered form used in docs).

The Core/CPS snippets are structurally faithful and intentionally schematic where projector
spelling is implementation-specific.

### 8.1 Even/Odd mutual recursion

Surface (conceptual):

```ash
letrec even(n: Int) -> Bool =
  if n == 0 then true else odd(n - 1)

letrec odd(n: Int) -> Bool =
  if n == 0 then false else even(n - 1)
```

Core (conceptual):

```core
(let-rec pair : ( (fn (Int) -> Bool {}) (fn (Int) -> Bool {}) )
  (tuple
    (lam ((n : Int)) : {}
      (let-prim n_is_zero eq (n (lit-int 0))
        (if n_is_zero
          (lit-bool true)
          (let-prim n_1 sub (n (lit-int 1))
            (let-prim odd_fn (pair.1 pair)
              (call odd_fn ((n_1)))))))
    (lam ((n : Int)) : {}
      (let-prim n_is_zero eq (n (lit-int 0))
        (if n_is_zero
          (lit-bool false)
          (let-prim n_1 sub (n (lit-int 1))
            (let-prim even_fn (pair.0 pair)
              (call even_fn ((n_1)))))))))
  (let-prim even_fn (pair.0 pair)
    (call even_fn ((lit-int 4)))))
```

CPS (informal):

```lisp
(LetRec (name . "pair")
  (Tuple
    [Lam [n] [
      (If (Eq n 0)
          (Jump (cont Var . "k") (arg . true))
          (Call (tuple-get 1 pair) ((Sub n 1)) (cont Var . "k")))]
    [Lam [n] [
      (If (Eq n 0)
          (Jump (cont Var . "k") (arg . false))
          (Call (tuple-get 0 pair) ((Sub n 1)) (cont Var . "k")))]])
  (LetPrim even_fn (tuple-get 0 pair)
    (Call even_fn ((lit-int 4)) (cont Label . "__exit"))))
```

Interpretation:

- The direct-style call graph alternates between two functions.
- CPS makes the alternation explicit through `Call` edges and continuation jumps.

### 8.2 TCO intuition in CPS

- In recursive direct style, stack-safety depends on compiler proof that each recursive branch is in tail position.
- In CPS, the same recursion is represented as repeated `Call ... <cont>` transfers, so tail behavior is explicit.
- The remaining optimization work is in runtime scheduling (trampoline/stack strategy), not in detecting tail spots.

---

## Notes

- The source file appears to contain non-textual/trailing binary suffix noise after the coherent conversation content.
- This design note preserves the conversational content up to the mutual-recursion letrec algorithm section, where the signal is complete and internally consistent.
