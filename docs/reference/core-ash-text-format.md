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
(if Atom Expr Expr)
(call Atom (Atom...))
(jump (label name) Atom)
(raise EffectOp (Atom...))
(handle HandlerClause Expr)
(record-discharge ContractDischarge Expr)
(trap TrapReason)
```

`let-val`, `let-prim`, `if`, `call`, `raise`, `handle`, `record-discharge`, and `trap` are the Phase 161 fixture forms. `call`, `raise`, and `handle` are direct-style Core forms; CPS continuation fields are synthesized only during Core-to-CPS lowering.

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

The initial corpus lives in `crates/ash-core/tests/fixtures/core/`:

- `let_val_jump.core`
- `let_prim_if.core`
- `call_non_tail.core`
- `raise_handle.core`
- `contract_trap.core`
