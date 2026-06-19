# TASK-1618: Add reference documentation for expanded CPS IR

## Status: 📝 Planned

## Description

Add reference documentation for the expanded CPS IR: records, tuples, constructor tags, match dispatch, and mutual recursion desugaring. This documents the lowering contract for frontend developers.

## Specification Reference

- [PLAN-160](../PLAN-160-CPS-IR-RUNTIME-EXPANSION.md)
- [SPEC-098b: Target IR](../../spec/SPEC-098b-TARGET-IR.md)
- [SPEC-099c: Expanded Operational Semantics](../../spec/SPEC-099c-CPS-IR-EXPANDED-OPERATIONAL-SEMANTICS.md) (from TASK-1617)

## Dependencies

- ✅ TASK-1617: Expanded operational semantics (must be complete)

## Requirements

### Functional Requirements

1. Create/update reference documentation:
   - `reference/language/ir/records.md` — Record construction and field access
   - `reference/language/ir/tuples.md` — Tuple construction and element access
   - `reference/language/ir/constructors.md` — Sum type constructors and tags
   - `reference/language/ir/pattern-matching.md` — Match dispatch lowering
   - `reference/language/ir/mutual-recursion.md` — Mutual recursion desugaring
2. Each page must include:
   - Lowering rule (what frontend produces)
   - Illustrative `.cps` syntax (for human readers — actual fixtures use serde-lexpr serialization)
   - Runtime semantics (how interpreter executes it)
   - Cross-references to specs

**Note on `.cps` syntax examples:** The reference docs use hand-written S-expression syntax for readability. The actual `.cps` fixture files are generated from Rust structs via `serde_lexpr` (see TASK-1616). The syntax examples are approximate — the exact serde-lexpr output may differ.

### Content Outline

**records.md:**
```markdown
# Records in CPS IR

## Lowering Rule

```ash
type Point = { x: Int, y: Int };
let p = Point { x: 1, y: 2 };
```

Lowers to:
```lisp
(letval p (record ((x 1) (y 2)))
  ...)
```

## Field Access

```ash
p.x
```

Lowers to:
```lisp
(letprim x_val (record_get x p)
  ...)
```
```

**tuples.md:**
```markdown
# Tuples in CPS IR

## Lowering Rule

```ash
let t = (1, 2, 3);
```

Lowers to:
```lisp
(letval t (tuple (1 2 3))
  ...)
```

## Element Access

```ash
t.1
```

Lowers to:
```lisp
(letprim second (tuple_get 1 t)
  ...)
```
```

**constructors.md:**
```markdown
# Sum Type Constructors in CPS IR

## Lowering Rule

```ash
type Shape = Circle { radius: Float } | Rect { width: Float, height: Float };
let s = Circle { radius: 5.0 };
```

Lowers to:
```lisp
(letval s (tuple ((constructor "Circle") 5.0))
  ...)
```

## Constructor Tag

The tag `ConstructorName("Circle")` is an inert atom used for discrimination.
```

**pattern-matching.md:**
```markdown
# Pattern Matching in CPS IR

## Lowering Rule

```ash
match s with
  Circle(r) -> ...
  Rect(w, h) -> ...
```

Lowers to:
```lisp
(match s
  ("Circle" (letprim r (tuple_get 1 s) ...))
  ("Rect" (letprim w (tuple_get 1 s)
             (letprim h (tuple_get 2 s) ...)))
  (default (trap MatchFailure)))
```
```

**mutual-recursion.md:**
```markdown
# Mutual Recursion in CPS IR

## Lowering Rule

```ash
letrec even(n) = if n == 0 then true else odd(n - 1)
       odd(n)  = if n == 0 then false else even(n - 1)
```

Lowers to:
```lisp
(letrec pair
  (tuple
    (lam [n] k ... (tuple_get 1 pair) ...)
    (lam [n] k ... (tuple_get 0 pair) ...))
  (letprim even (tuple_get 0 pair)
    ...))
```

## Why This Works

The placeholder/backfill mechanism (see SPEC-099b §5.1) ensures that when `pair` is backfilled with the actual tuple, the lambdas (which capture `pair` by reference) see the updated value.
```

## Dispatch

```yaml
agent: hermes
reasoning: medium
max_turns: 15
toolsets: [terminal, file]
```

## Verification

```yaml
strictness: clean
commands:
  - test -f reference/language/ir/records.md
  - test -f reference/language/ir/tuples.md
  - test -f reference/language/ir/constructors.md
  - test -f reference/language/ir/pattern-matching.md
  - test -f reference/language/ir/mutual-recursion.md
  - git diff --check
checklist:
  - [ ] All reference pages exist
  - [ ] Each page has lowering rule, example, and runtime semantics
  - [ ] Cross-references to specs are correct
  - [ ] No modification to existing reference pages
  - [ ] CHANGELOG.md entry staged
```

## Dependencies for Next Task

- Provides documentation for TASK-1619 (closeout)

## Notes

- Reference pages should be concise but complete. They are for frontend developers, not end users.
- The `.cps` syntax examples are illustrative only. The actual serde-lexpr format is determined by serde's representation.
- If `reference/language/ir/` directory doesn't exist, create it.
