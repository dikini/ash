# TASK-2003: `Return` Authority and CPS Kernel Decision

**Status:** In progress — calculus authority now aligns canonical `Return v` with recursive CPS
`Value` terminal observation. Checked projection retains literal atom and structured-trap behavior,
adds bounded constructor, atomic-`Add`, and atomic-Boolean-`Not` inspection paths, and rejects
non-atoms through legacy atom-only APIs; general source/Core realization and production parity
remain out of scope.
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
  source-return inspection bridge, its bounded atomic-`Int` addition and atomic-Boolean-`Not`
  extensions, the calculus, and the checked CPS prototype agree, including answer-type checking
  for their narrow source subset.
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

The same private inspection bridge also accepts one deliberately small arithmetic form:
`Expr::Binary(Add, left, right)` only when both operands are already atoms (`Int` literals or
previously bound local variables). It materializes checked `CoreExpr::LetPrim(Add)` and its bound
result immediately `Jump`s to `__answer`; after checked Core lowering this is CPS `LetPrim(Add)`
followed by that answer jump. The literal `2 + 5` and lexical
`let x = 2; let y = 5; return x + y` regressions prove that shape. A nested operand such as
`(1 + 2) + 3` remains a deterministic type error until a separately designed ANF/source lowering
stage exists. This is neither general arithmetic nor a production Core/CPS migration.

The bridge likewise accepts unary `!` only when its operand is an atomic `Bool` literal or an
already-bound local with `Bool` type. It materializes checked `CoreExpr::LetPrim(CorePrimOp::Not)`;
checked lowering produces CPS `LetPrim(PrimOp::Not)`, whose result immediately
`Jump`s to `__answer`. The answer-continuation terminal observation is the Boolean complement.
`!!true`, `!1`, `Neg`, and every other unary or wider expression reject at this private inspection
boundary pending ANF/general lowering. This narrow rule adds no production admission, frame or
provider authority, async host operation, or direct-evaluator path.

Focused evidence for this bounded extension is 14/14 TASK-2003 source-return/CPS-lowering tests,
alongside `ash-engine` library tests (308), one focused Core task test, symbolic-operation tests
TASK-2010 (5), TASK-2011 (6), TASK-2012 (8), TASK-2015 (2), and TASK-2017 (9), plus clean
`cargo clippy`, formatting, and diff checks.
