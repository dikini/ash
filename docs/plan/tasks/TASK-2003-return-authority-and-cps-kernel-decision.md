# TASK-2003: `Return` Authority and CPS Kernel Decision

**Status:** In progress — calculus authority now aligns canonical `Return v` with recursive CPS
`Value` terminal observation. Checked projection retains literal atom and structured-trap behavior,
adds bounded constructor and one recursive typed `PureAnf` path for approved `Int` primitives,
exact `Bool`-operand `Eq`/`Ne`, and Boolean `Not`. That pure subset is also admitted by the sealed handler-free
production path;
general source/Core realization and production parity remain out of scope.
**Phase:** Follow-up from [TASK-1988](TASK-1988-semantic-implementation-deprecation-audit.md)
**Depends on:** TASK-1989

## Description

Resolve the conflict between `CANONICAL-CORE` listing `Return` in the kernel and SPEC-098b’s
statement that CPS has no direct return.

## Requirements

- Record one authoritative grammar decision with typed supersession links.
- Specify whether `Return` is a Core boundary form, CPS terminal form, derived notation, or absent.
- Align Core/CPS text parsing, validation, lowering, type/answer discipline, and conformance cases.
- Do not silently select behavior from the existing `cps::Term::Return` implementation.

## TDD Steps

1. Add contradictory grammar fixtures that expose the unresolved authority state.
2. Freeze the calculus rule and malformed counterpart cases.
3. Implement only the selected parser/lowering/checker changes.
4. Run corpus, Core/CPS, and documentation gates.

## Completion Checklist

- [x] Exactly one active canonical interpretation exists: `Return` is a terminal
  `λAsh-CPS₀` observation, while direct-style source return lowers through `Jump`.
- [ ] All affected layers and examples agree. One checked literal/atomic-let/typed-variable-let
  source-return inspection bridge and its bounded typed `PureAnf`
  extensions, the calculus, and the checked CPS prototype agree, including answer-type checking
  for their narrow source subset. The independently case-bound differential `7 - 2` corpus witness
  remains TASK-2005/TASK-439 evidence; it neither grants the differential oracle production
  authority nor broadens the primitive family.
  General parser/Core lowering/validation and production execution remain explicitly unselected or unproved;
  TASK-2004/TASK-2005 own those realization decisions.
- [ ] Terminal observables and complete answer-type behavior are tested. The checked prototype
  covers atom and recursive-value return, structured trap, malformed-terminal rejection, and
  answer type for the narrow source inspection subset, but not general end-to-end source/Core
  evidence.
- [x] Supersession, traceability, and changelog evidence are recorded for the scoped decision.

## Evidence required

The TASK-1988 packet identifies the direct conflict. A green existing `Term::Return` test is not
evidence for either semantic choice.

## Scoped decision and evidence

The active owner is `CORE-CPS-SYNTAX-001`, refined by
[λAsh-CPS Calculus](../../spec/ASH-CPS-CALCULUS.md#mathematical-syntax-and-state): `Return v` is
only the terminal observation of completed kernel evaluation. It is not a direct-style source
form, Core boundary form, or CPS call result. The previous broad SPEC-098b wording is reconciled
at [its terminal-observation note](../../spec/SPEC-098b-TARGET-IR.md#10-terminal-observation-reconciliation):
the existing “no direct return” rule remains applicable to executable CPS tails.

`ash_core::cps::Term::Return` and `ash_interp::cps::CpsTerminalOutcome::Return` now carry the
canonical CPS `Value`, not just an atom. `eval_checked_terminal` therefore observes recursively
constructed records, tuples, and constructors without collapsing a bound value through
`eval_atom`. The focused regression retains literal `Int(42)` as `Value::Atom`, retains the
distinct structured trap, rejects an unresolved terminal variable before observation, and proves a
nested tagged `Err(RuntimeError(42, "boom"))` record/tuple value. Core remains non-terminal:
direct Core `return`, including a recursive value, still rejects. The legacy `eval_checked`/
`eval_unchecked` atom-return APIs remain compatibility APIs and reject a non-atom terminal value
rather than fabricating an atom.

This aligns the kernel carrier only. It does not make CPS production execution, general source
parsing/lowering, Core validation, or complete answer-type behavior claims.

`ash_engine::Engine::lower_entry_to_checked_cps` adds one intentionally narrow inspection bridge:
the checked literal source `fn main() -> Int { do { return 42; } }`, atomic-let source
`do { let x = 41; return x; }`, and typed variable-let source
`do { let x = 41; let y = x; return y; }` materialize a checked Core/CPS prototype `Jump` to
`__answer`, never `Term::Return`. The let cases preserve a typed `LetVal` spine around the answer
jump; the variable-let case carries the already-bound atom type through the private bridge rather
than inferring a type from an unbound variable. Wrapped in the affine `__answer` continuation, the
typed variable-let bridge evaluates to the terminal observation `Return(Int 41)`. Its incompatible
`-> String` counterpart is rejected by the checked answer type. A function-tail
source `if true then 42 else 0`, represented by legacy lowering as a two-arm boolean `Expr::Match`,
is additionally converted to checked `CoreExpr::If`; both branches jump to `__answer`.
`crates/ash-engine/tests/task_2003_source_return_cps_lowering.rs` is the direct regression
evidence. The bridge accepts only literals, variables with a previously bound atom type,
variable-pattern atomic lets, and exactly two literal boolean conditional arms. The bridge derives the selected source result type and
registers `__answer : Cont<result, Unit, ∅, affine>` in the checked Core environment; a source
`fn main() -> Int { "not an integer" }` is therefore rejected. It is not a general source/Core
realization, and `Engine::execute` continues to
be a closed checked-Core/CPS admission guard. The private inspection bridge itself does not
establish direct-runtime/CPS parity. The separate sealed admission path owns the bounded `run`,
`run_file`, and zero-input bootstrap slices; general source lowering, complete answer-type
implementation, and production parity remain deferred to TASK-2004 and TASK-2005. This task
remains in progress.

For canonical bootstrap only, the same bridge has one separate structural constructor subset:
nested constructors whose fields are primitive literals or recursively supported constructors
become CPS `Value::Constructor` trees and cross the engine boundary as the matching Ash variants.
It is sufficient for the zero-input `Err { error: RuntimeError(42, "boom") }` entry control. It
rejects computed constructor fields and every unsupported expression; it does not add general ADT,
record/tuple, handler, provider, or async lowering.

The shared bounded lowering uses one typed `PureAnf` normalizer. Its leaves are typed literals or
already-bound variables. It recursively admits the approved `Int`-operand binary family `Add`,
`Sub`, `Mul`, `Div`, `Eq`, `Ne`, `Lt`, `Le`, `Gt`, and `Ge`, **plus only** `Bool` × `Bool`
`Eq` and `Ne`, together with recursive Boolean `Not`; every intermediate receives a collision-safe
internal temporary, and the final atom alone `Jump`s to `__answer`. Thus `!!(1 + 2 < 4)` carries one ordered
`Add → Lt → Not → Not` spine without exposing any temporary in the admission artifact.

The same bounded route additionally accepts an irrefutable `let` only when its pattern is a
variable and its RHS is a typed `PureAnf` expression. Its recursive RHS bindings are emitted
left-to-right as the `LetPrim` spine **before** the typed source `LetVal`; the source variable
then carries the final RHS result type into the admitted body.
Generated temporaries are collision-safe against all collected source names, including reserved
`__checked_*`-shaped names. The focused computed-let regression proves that ordering and type
carry without granting general `let` lowering. The TASK-2003/
TASK-2004/TASK-2014 contracts prove the atomic family, the nested left-to-right spine, their
terminal `Int`/`Bool` results through sealed `Engine::run`, representative `run_file`, and CLI
runnable-source cases, and the absence of a legacy evaluator reopening. The same normalizer
supplies the Boolean condition and both branches of the existing bounded Boolean `if`/`match`
forms, so a computed condition and branch expression may each retain their own ordered spine.
Apart from the exact `Bool` × `Bool` `Eq`/`Ne` pair, mixed or other non-`Int` binary operands,
`Neg`, `&&`/`||`, calls, `Raise`/`Handle`, effects, providers, frames, and every other expression
form remain fail-closed. This does not make the fragment generic ANF, general arithmetic, general
`let`, or general conditional/match lowering.

`phase202-source-int-sub-bridge-return-5` remains a second, stricter evidence plane: the
differential harness permits only the exact `fn main() -> Int { 7 - 2 }`,
`LetPrim(Sub, [Int(7), Int(2)]) → Jump(__answer, Var(result))`, and `Int(5)` tuple. Swapped
operands or `Add` reject at corpus load before either differential target executes. That
case-bound oracle evidence remains private and cannot invoke production routes; conversely, the
production family does not make arbitrary corpus cases or a direct-evaluator fallback admissible.

`Not` is therefore not a separate scope exception: its operand itself is a recursively normalized
typed `PureAnf` Boolean expression, including in a variable-let RHS, Boolean condition, or branch.
Each `!` materializes one checked `CoreExpr::LetPrim(CorePrimOp::Not)` and the matching ordered
CPS `LetPrim(PrimOp::Not)` binding. Likewise, exact typed `Bool` × `Bool` source `==`/`!=` retain
their selected Core operation, materialize exactly one matching `LetPrim(Eq|Ne, [Bool, Bool])`,
and jump only that result to `__answer`. Non-Boolean `Not`, mixed or other non-`Int`/non-`Bool`
equality operands, `Neg`, `&&`/`||`, calls, effects, handlers, providers, and frames remain
closed. This narrow rule adds no provider/frame authority, async host operation, or direct-evaluator
path.

Focused evidence is
[`task_2003_pure_anf_normalizer.rs`](../../../crates/ash-engine/tests/task_2003_pure_anf_normalizer.rs)
and
[`task_2004_2014_nested_binary_anf_production.rs`](../../../crates/ash-engine/tests/task_2004_2014_nested_binary_anf_production.rs),
which inspect the composed spines, exact Boolean `Eq`/`Ne` Core/CPS/answer-jump shape, and sealed
runtime result while retaining the negative boundary controls.

## Sealed local-call Core/CPS slice

One additional production-admitted call shape is deliberately exact rather than an extension of
`PureAnf`: a private, zero-argument `helper() -> Int { 7 }` **or** the exact ambient
`helper() -> Int { do { return 7; } }`, immediately followed by
`main() -> Int { helper() }`. The latter is a second accepted body spelling for the same sealed
recognizer, not a general `do`/source-`return` feature. The Engine obtains the retained surface
program only after confirming the Entry's same-Engine canonical parsed source anchor and parse-time
legacy Core. It rejects an Entry whose public Core was retargeted, and rejects any retained
imported type, semantic-summary, or type-function state before it recognizes the fixture.
Consequently the accepted source is source-proven local code, not an imported callable or a
mutable Entry-sidecar convention.

Both spellings construct the existing checked Core
`LetVal(helper, Lam([], Atom(7)), Call(Var(helper), []))`; normal validation and typechecking then
lower it to a CPS `LetVal` whose helper lambda jumps `7` to its explicit caller continuation.
For the ambient-`do` spelling, the explicit source `return` therefore becomes that
`Jump(cont, 7)`, never `Term::Return`; the caller tail remains
`Call(Var(helper), [], __answer)`. `Engine::run` still executes only the sealed handler-free
artifact and observes `Int(7)`. This is the first bounded local `Lam`/`Call` witness, not
general function calls, return/do lowering, local-call lowering, closure conversion, thunk
inference, recursion, parameter passing, call-result binding, or imported-call admission.
Unsupported call shapes remain closed at admission, and generic `Engine::execute` remains closed
rather than becoming a direct-evaluator fallback.

Focused evidence is
[`task_2003_local_call_core_cps_lowering.rs`](../../../crates/ash-engine/tests/task_2003_local_call_core_cps_lowering.rs),
which checks both exact Core/CPS spines (including the ambient-helper `return → Jump(cont, 7)`
and caller `__answer` tail), the sealed `run` result, forged public-Core rejection, and
rejection of a file entry that retains only a type import.
