# Core Ash Text Format

`.core is a fixture/debug format`, not surface Ash. It is a strict, small, S-expression-like spelling of SPEC-099 Core Ash nodes for golden tests, parser tests, serializer tests, and Core-to-CPS lowering fixtures.

The format intentionally avoids surface sugar. A `.core` file names Core AST forms directly, uses lowercase node names, and keeps all intermediate structure explicit.

## Atoms

```text
x
(lit-int 1)
(lit-string "text")
(lit-bool true)
(lit-unit)
(label exit)
```

Bare names are Core variable atoms. Labels are only valid in continuation positions such as `jump`.

## Types And Rows

```text
Int
String
Bool
Unit
(fn (Int String) -> Unit {cap console.write})
(cont Unit Unit {} affine)
{}
{fail}
{cap console.write}
```

Rows are requirement rows. They record what a term requires; they are not authority grants.

## Expressions

```text
(let-val name : Type Value Expr)
(let-prim name prim-op (Atom...) Expr)
(let-call name Atom (Atom...) Expr)
(if Atom Expr Expr)
(call Atom (Atom...))
(jump (label name) Atom)
(raise EffectOp (Atom...))
(handle HandlerClause Expr)
(record-discharge ContractDischarge Expr)
(trap TrapReason)
```

`let-val`, `let-prim`, `let-call`, `if`, `call`, `raise`, `handle`, `record-discharge`, and `trap` are the Phase 161 fixture forms. `let-call` binds the result of a non-tail direct-style call; CPS lowering introduces `LetCont` for it. `call`, `raise`, and `handle` are direct-style Core forms; CPS continuation fields are synthesized only during Core-to-CPS lowering.

## Values

```text
(lit-int 1)
(lam ((x : Int)) : {} Expr)
(record (field Atom)...)
(tuple Atom...)
(discharge-marker ContractDischarge)
```

Value forms map to `CoreValue` and do not imply surface-language syntax.

## Effects And Traps

```text
(cap console.read : (String) -> Unit)
(channel inbox send : Message -> Unit)
(proc spawn : (Command) -> ProcessHandle)
(fail Error)
(contract requires-positive dynamic)
(contract-violation requires-positive)
(unhandled-effect EffectOp)
(panic "message")
(non-exhaustive-match)
```

Only capability, channel, process, and failure operations are raised operations. Contract violations are trap reasons, not effect row items and not raised operations.

## Fixtures

The Phase 161 corpus lives in `crates/ash-core/tests/fixtures/core/`. These files are hand-authored Core fixture text, not examples of source Ash syntax:

| Core fixture | CPS golden | Purpose |
| --- | --- | --- |
| `let_val_jump.core` | `let_val_jump.core.cps.golden` | Minimal `let-val` followed by a continuation `jump`. |
| `let_prim_if.core` | `let_prim_if.core.cps.golden` | Pure primitive binding and local branch rows on `if`. |
| `call_non_tail.core` | `call_non_tail.core.cps.golden` | Lambda value and direct-style tail `call`. |
| `let_call.core` | `let_call.core.cps.golden` | Non-tail direct-style call lowered through CPS `LetCont`. |
| `raise_handle.core` | `raise_handle.core.cps.golden` | Capability `raise`, affine handler resume, and local handler row. |
| `contract_trap.core` | `contract_trap.core.cps.golden` | Dynamic contract `record-discharge` plus contract-violation `trap`. |

`invalid_duplicate_row.core` is intentionally invalid. It parses as Core text, then fails validation because duplicate row items are rejected before lowering.

## Implementation Boundaries

The text parser accepts only the Phase 161 fixture/debug subset. It is intentionally stricter than SPEC-099 as a whole and does not accept surface Ash constructs such as `workflow`, `do`, `handle ... with`, typeclass constraints, laws, properties, comprehensions, or imports.

The Core text serializer produces one canonical spelling for committed fixture forms. Stable diffs are the goal; human-friendly source syntax is not.

Core-to-CPS lowering behavior is documented in [`core-ash-lowering.md`](core-ash-lowering.md).
