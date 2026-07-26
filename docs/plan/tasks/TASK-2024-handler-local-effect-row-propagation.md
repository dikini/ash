# TASK-2024: Nonproduction Handler-Local Effect-Row Propagation

**Status:** Complete — one declaration-resolved, nonproduction `forward_sleep` control now
preserves exactly `{TestClock::wake}` as the private Core/CPS `Handle.row`; it remains an
inspection-only artifact with no continuation residual, runtime, provider, admission, or
production Core/CPS authority.
**Phase:** TASK-1988 implementation follow-up; bounded continuation of
[TASK-2013](TASK-2013-source-handler-and-handle-lowering.md)
**Depends on:** TASK-2011 declared concrete-operation resolution, TASK-2013 checked handler
declarations and private Core/CPS inspection, TASK-2004 retained-private Core/CPS boundary, and
[SPEC-099](../../spec/SPEC-099-CORE-LANGUAGE.md#handlers) local `Handle.row` semantics

## Description

Extend the existing private checked-handler inspection bridge by one exact, local-row control:
the handled computation raises `TestClock::sleep`, while the handler clause raises the distinct
declared `TestClock::wake` operation.  The resulting Core `Handle` must retain `wake` in its
**local residual body row**, while its handled body retains `sleep` and its clause body retains
`wake` as exact operation-typed `Raise` artifacts.

The control is deliberately nonresumptive.  It proves handler-local effects are not silently
dropped during the existing typed inspection bridge; it does not claim continuation residual
semantics, resume execution, multi-shot behavior, source-handler runtime execution, or provider
admission.

## Exact bounded source control

The test fixture must use the following declaration-backed shape (with ordinary formatting
variation only):

```ash
interface Clock<T> {
    sleep(Int) -> Int
    wake(Int) -> Int
}
type TestClock = SystemClock(Int);
impl Clock<TestClock> {
    sleep(milliseconds) = milliseconds
    wake(milliseconds) = milliseconds
}

handler forward_sleep(comp: Int) -> Int {
    on comp {
        TestClock::sleep(ms, resume) => TestClock::wake(ms),
        done(value) => value,
    }
}

fn main() -> Int { handle TestClock::sleep(0) with forward_sleep }
```

`resume` is present only because canonical source clause syntax requires its binder.  This task
does not invoke it, assign it a residual row, or infer any multiplicity property beyond the
already-established unused affine binder fact.

## Authoritative References

- [SPEC-095b §4.3](../../spec/SPEC-095b-TARGET-GRAMMAR.md#43-handler-expressions): canonical
  source handler clause syntax.
- [SPEC-097b §8.8](../../spec/SPEC-097b-TARGET-TYPE-SYSTEM.md#88-handler-typing): handler typing
  boundary; this task does not claim its general continuation rules.
- [SPEC-098c §6](../../spec/SPEC-098c-SURFACE-TO-CORE-LOWERING.md#6-handlers-and-provider-boundaries):
  handler lowering without provider-boundary synthesis.
- [SPEC-099](../../spec/SPEC-099-CORE-LANGUAGE.md#handlers): `Handle.row` is the local residual
  body row and excludes the outer continuation row.
- [SPEC-098b §5](../../spec/SPEC-098b-TARGET-IR.md#5-target-ir): `Raise.row`/`Handle.row` local
  row contracts.
- [TASK-2013](TASK-2013-source-handler-and-handle-lowering.md#completed-typed-core-handler-raise-inspection-stage):
  current empty-row private inspection bridge.
- [TASK-2014](TASK-2014-source-handler-runtime-boundary-decision.md): source handler runtime
  boundary remains undecided and unimplemented.

## Scope

### In scope

- One parser-supported, locally declared `Clock<TestClock>` fixture containing exactly
  `sleep(Int) -> Int` and `wake(Int) -> Int` operation facts.
- One checked handler `forward_sleep` whose sole handled clause resolves `TestClock::sleep`, whose
  clause body resolves `TestClock::wake(ms)`, and whose `done` body is identity.
- Exact typed-Core inspection evidence:

  ```text
  Handle {
    row = { TestClock::wake },
    body = Raise(TestClock::sleep, [Int(0)]),
    clause = sleep(ms, resume) => Raise(TestClock::wake, [Var(ms)])
  }
  ```

  The operation signatures and source-declared argument/result types must be retained on both
  `Raise` forms. `Handle.row` must contain exactly `wake`, not `sleep`, not an empty row, and not
  an inferred provider capability.
- Exact checked-CPS inspection evidence that preserves a `Handle` with local row `{wake}`, a body
  `Raise(TestClock.sleep, Int(0))`, and a clause `Raise(TestClock.wake, Var(ms))`.
- Negative evidence that changing the clause operation/body identity or omitting the handler-local
  operation does not create a successful residual-row artifact.

### Explicit exclusions

- General residual-row subtraction/composition, arbitrary handler bodies or operations, multiple
  clauses, nested handlers, imports/generics, provider selection, and handler inference.
- Any source `resume(...)` semantics beyond the existing direct unused-binder static fact:
  no source continuation residual-row claim, invocation lowering, answer transformation,
  affine/multi-shot expansion, capture semantics, or runtime continuation execution.
- Engine/CLI/bootstrap direct execution, admission records, provider or handler frame creation,
  time/host I/O, cancellation/timeout behavior, traces/monitors, or terminal-envelope behavior.
- Promotion of Core/CPS to production, direct-runtime/CPS parity, generic Core loading, or a
  change to TASK-2004/TASK-2014 boundary decisions.

## Requirements and invariants

1. **Declared symbolic identities only.** Both `TestClock::sleep` and `TestClock::wake` resolve
   through local declaration facts; no string dispatch, operation-name heuristic, `invoke`, or
   provider metadata supplies their identity.
2. **Exact local-row ownership.** The handler removes only its handled `sleep` operation from the
   local handled computation and records the clause's `wake` effect in `Handle.row`.  The row is
   exactly `{TestClock::wake}` and is not the total continuation row or a grant of authority.
3. **Exact Core carriers.** The handled body lowers as `Raise(sleep, Int(0))`; the clause body
   lowers as `Raise(wake, Var(ms))`. Neither may become an ordinary call or a synthetic provider
   frame.
4. **Exact CPS carriers.** Existing checked Core-to-CPS lowering preserves the `Handle` local row
   and both operation identities. The evidence is structural inspection only and does not execute
   the term in production.
5. **No continuation overclaim.** The source `resume` binder remains unused. Success must not be
   documented or tested as proving continuation residual rows, continuation invocation, or
   multi-shot behavior.
6. **No authority synthesis.** A row names a requirement only. It must not install a handler or
   provider frame, pass admission, dispatch a provider, perform host I/O, or make a source handler
   executable.
7. **Fail closed.** Unsupported clause forms, operation identities, or row shapes reject before
   any production evaluator/engine/CLI route; the existing empty-row TASK-2013 fixture remains a
   control rather than being reinterpreted as general row support.

## TDD steps

1. **RED — exact source/type facts.** Add a focused typechecker test for `forward_sleep`; require
   checked declarations to preserve both declared operations and expose the handler-local `wake`
   effect. The test must fail before row propagation exists.
2. **RED — Core/CPS shape.** Add a focused inspection test requiring exact Core `Handle.row`,
   `Raise(sleep)`, `Raise(wake)`, and their checked-CPS counterparts. Assert no provider/frame
   carrier is created and no production route calls this bridge.
3. **GREEN — minimal typed bridge extension.** Propagate only this validated clause-body operation
   effect into the local `Handle.row`, and lower only the exact operation expression as the clause
   `Raise`. Do not generalize handler-body evaluation or resume semantics.
4. **Negative controls.** Reject a changed/unresolved clause operation, a body that does not match
   the bounded `wake(ms)` shape, and any attempt to treat the row as runtime admission/dispatch.
5. **Regression and documentation.** Keep TASK-2000 `invoke` rejection, TASK-2011/2012 symbolic
   operation evidence, TASK-2013 empty-row/resume checks, TASK-1993 frame-order controls, and
   TASK-2004 private-boundary controls green. Update this task, TASK-2013, PLAN index,
   `CHANGELOG.md`, and semantic traceability only after implementation; run focused tests,
   `cargo fmt --check`, relevant Clippy, docs/traceability gates, and `git diff --check`.

## Completion checklist

- [x] The exact `forward_sleep` source fixture parses and checks from declaration-backed `sleep`
  and `wake` facts.
- [x] Typed Core is exactly `Handle.row = {TestClock::wake}` with body `Raise(sleep, Int(0))` and
  clause `Raise(wake, Var(ms))`.
- [x] Checked CPS structurally preserves the same local `Handle` row and both `Raise` identities.
- [x] Negative shape/identity controls reject fail closed.
- [x] No continuation residual/multi-shot/runtime/provider/admission/production claim is added.
- [x] Existing handler, symbolic-operation, frame-order, and private-boundary controls remain
  green.
- [x] Tests, formatting, Clippy, changelog, plan/index, traceability, docs gate, and diff checks
  pass after implementation.

## Completion evidence

The private `forward_sleep` inspection fixture now carries exactly two locally declared symbolic
operation identities: the handled expression is `Raise(TestClock::sleep, [Int(0)])`, and the
clause body is `Raise(TestClock::wake, [Var(ms)])`.  The checked Core `Handle` owns the closed
local row `{TestClock::wake}`; after existing checked Core-to-CPS lowering, the CPS `Handle` has
exactly one local capability item, `cap:TestClock.wake`, while its body and clause preserve the
`TestClock.sleep` and `TestClock.wake` `Raise` carriers respectively.

The focused suite has three passing controls:

1. the exact declaration-backed `sleep` → `wake(ms)` fixture, Core/CPS structure, and local row;
2. a distinct declared clause operation (`other`) rejected at the private lowering boundary; and
3. a `wake(0)` clause payload rejected rather than becoming a residual-row artifact.

This does not establish a continuation residual, resume invocation, answer transformation,
multi-shot behavior, frame/admission/provider construction, host time/I/O, tracing/monitoring,
direct-runtime execution, or production Core/CPS authority.  Rows remain requirements, never
authority grants.
