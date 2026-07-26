# TASK-2026: Sealed `forward_sleep` Handler-Provider Production Slice

**Status:** Complete — the exact same-Engine admission, ordered two-frame checked-CPS driver,
normal return, provenance/binding controls, and cooperative timeout/cancellation envelope are
implemented and covered by focused regression tests.
**Phase:** TASK-1988 implementation follow-up; production continuation of
[TASK-2024](TASK-2024-handler-local-effect-row-propagation.md)
**Depends on:** [TASK-1993](TASK-1993-verus-frame-ordered-dispatch-pilot.md),
[TASK-2013](TASK-2013-source-handler-and-handle-lowering.md),
[TASK-2014](TASK-2014-source-handler-runtime-boundary-decision.md), and TASK-2024

## Description

Promote exactly TASK-2024's local `forward_sleep` inspection fixture into one sealed Path-B
checked-Core/CPS production route.  The fixture has an outer explicit provider frame for
`TestClock::wake` and an inner explicit source-handler frame for `TestClock::sleep`; it proves
that an authorized ordered instruction list, rather than a row, constructs both frames and that
TASK-1993 innermost-first lookup dispatches the two operations correctly.

This is one exact Engine-only production slice.  It neither makes nonempty rows generally
executable nor implements general source handlers, residual rows, continuations, provider
selection, or CLI handler execution.

## Exact bounded source and outcome

The admitted source is this locally declared fixture, modulo ordinary whitespace only:

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

handler forward_sleep(comp: () -> { TestClock::sleep } Int) -> Int {
    on comp {
        TestClock::sleep(ms, resume) => TestClock::wake(ms),
        done(value) => value,
    }
}

fn main() -> Int { handle TestClock::sleep(0) with forward_sleep }
```

The canonical checked Core/CPS shape is a `Handle` whose local row is exactly
`{TestClock::wake}`, whose handled body is `Raise(TestClock::sleep, [Int(0)])`, and whose sole
clause body is `Raise(TestClock::wake, [Var(ms)])`.  `done` is identity and `resume` is unused.
The exact registered `TestClock::wake(Int) -> Int` provider echoes its one integer argument;
successful `Engine::run` therefore returns `Int(0)` and records one `wake(0)` dispatch.

## Admission and execution contract

1. **Opaque same-Engine token.** Only the issuing `Engine` may mint and consume a new private
   production admission/token.  It seals the canonical parsed source, exact parsed legacy Core,
   checked Core/CPS, source anchor, checked handler facts, resolved concrete `sleep`/`wake`
   identities, and the exact registered `wake` provider binding.  A foreign Engine, unchecked
   entry, altered source anchor, mutated public legacy Core, forged sidecar, absent binding, or
   mismatched provider rejects before a frame or provider await exists.
2. **Explicit ordered instructions.** The token contains exactly two separately authorized frame
   instructions in installation order: outer `Provider(TestClock::wake, resolved-binding)`, then
   inner `SourceHandler(TestClock::sleep, forward_sleep)`.  Neither `{TestClock::wake}` nor any
   other row installs a frame.  Missing, reordered, duplicate, extra, stale, or mismatched
   instructions reject at admission.
3. **Private checked-CPS driver.** The Engine constructs frames only from those sealed
   instructions and reverse-scans the ordered frame list for every `Raise`, preserving TASK-1993
   innermost-first lookup.  `sleep(0)` selects the inner handler, whose exact clause raises
   `wake(ms)`; that raise selects the outer echo provider.  The identity `done` returns the
   provider result.  No legacy direct evaluator, ambient registry lookup, or public frame API is
   permitted.
4. **Control envelope.** Only the admitted `wake` provider await is governed by the existing
   run-wide asynchronous timeout/cancellation envelope.  It observes cancellation before timeout
   before normal completion; a winning timeout or cancellation drops the in-flight await
   cooperatively and projects the existing canonical terminal envelope.  This promises no host
   kill, rollback, compensation, telemetry, or trace.

## Explicit exclusions

- All source other than the exact fixture: changed handler/operation names, literal `wake(0)`,
  different `done` or `resume`, parameters, alternate literals, multiple clauses, nested
  handlers, residual/open rows, imports, generics, recursion, aliases/groups, and arbitrary
  operation expressions.
- General handler/provider frame chains, arbitrary instruction validation, continuation
  invocation or residual/multi-shot semantics, provider discovery/inference, host I/O, and
  generic Core/CPS loading.
- `Engine::execute`, `Engine::execute_with_input`, generic V1 admission/inspection execution,
  direct-evaluator fallback, CLI runnable/trace handler routes, and all trace/monitor authority.
  No rejected route may reach the provider.

## TDD steps

1. **RED — exact production admission.** Add an Engine integration test that parses/checks the
   fixture, requires the exact checked Core/CPS `Handle`/two-`Raise` structure, admits only the
   same Engine's exact `wake` binding, and proves the sealed instruction order is Provider outer,
   SourceHandler inner.
2. **RED — execution and lookup.** Require `Engine::run` to return `Int(0)`, dispatch
   `wake(0)` exactly once, and retain a TASK-1993 outermost-first mutation sentinel.  Require
   generic execution entrypoints to remain closed.
3. **GREEN — smallest private bridge.** Implement only this exact nonempty-row lowering/admission
   and a distinct opaque token/driver path.  Construct frames exclusively from its two sealed
   instructions; reverse-scan them for both raises.
4. **RED/GREEN — boundaries and control.** Prove missing/mismatched/foreign bindings, changed
   provenance/Core/anchor, altered rows/operations/clauses, and malformed/reordered/extra
   instructions reject before dispatch.  Prove timeout and cancellation terminalization around
   `wake`, with cancellation winning a tie and no provider completion after the winning control.
5. **Regression and evidence.** Preserve TASK-2000 `invoke` rejection, TASK-1993 lookup
   controls, TASK-2013 closed-empty handler controls, TASK-2014 one-frame provider/handler
   production controls, and TASK-2024 structural-row controls.  Update task records,
   PLAN-INDEX, CHANGELOG, and semantic traceability after implementation.

## Completion checklist

- [x] The exact local fixture alone lowers to the checked `Handle { wake }`, `Raise(sleep)`, and
  `Raise(wake)` production candidate.
- [x] A same-Engine opaque token seals canonical source/Core/anchor/provenance, checked handler
  facts, concrete identities, exact `wake` binding, and exactly two ordered instructions.
- [x] The private driver installs only outer Provider then inner SourceHandler, reverse-scans
  innermost-first, returns `Int(0)`, and dispatches `wake(0)` once.
- [x] Rows never install frames; the current exact-source, imported-state, provenance, binding,
  and instruction-authority gates reject before dispatch.  Broader negative coverage remains a
  completion requirement below.
- [x] Timeout/cancellation use the canonical terminal envelope around the `wake` await;
  cancellation wins a timeout tie and both are cooperative drops only.
- [x] Generic execute/input, V1, direct evaluation, CLI runnable/trace, and monitor routes remain
  closed.
- [x] Focused and affected engine tests, formatting, strict Clippy, relevant full tests,
  orientation/traceability validators, docs gate, and `git diff --check` pass; CHANGELOG,
  PLAN-INDEX, TASK-2013/TASK-2014/TASK-2024, and semantic traceability are updated.

## Completion evidence

`Engine::admit_production_forward_sleep` now admits only the canonical locally declared,
row-annotated `forward_sleep` source shown above.  It requires retained parsed-Core and
source-anchor provenance, same-Engine checked handler facts, no retained imported state, and one
exact Engine-registered `TestClock::wake(Int) -> Int` binding.  Its opaque admission seals exactly
two installation instructions in installation order: outer
`Provider(TestClock::wake, binding)`, then inner
`SourceHandler(TestClock::sleep, forward_sleep)`.  The row describes the checked local residual;
it supplies neither instruction nor frame authority.

The private checked-CPS driver validates the sealed root `Handle`, its literal `sleep(0)` raise,
and its `wake(ms)` clause raise.  It reverse-scans the sealed ordered instruction list, selecting
the inner handler for `sleep` and the outer provider for `wake`; the provider result is the
identity `done` result. The focused Engine suite proves `Int(0)`, one `wake(0)` dispatch, a
nonzero `wake` result through identity `done`, closed generic `execute`/`execute_with_input`,
missing/mismatched/extra-row/mislabeled bindings, extra-local source rejection, and foreign or
mutated public provenance rejection before dispatch.

The admitted provider await reuses the existing run-control envelope. Paused-time tests prove
timeout and cancellation terminalization, cooperative dropping of the pending `wake` future, and
cancellation priority when cancellation and deadline expiry are both observable. `Engine::run`
and `run_file` use this same sealed route after a same-Engine binding registration; generic V1,
direct evaluation, CLI runnable/trace handler routes, generic handler/provider chains, and all
nonexact source forms remain closed.

TASK-2014's later bounded frame-order witness is a successor, not a reinterpretation of this
task: it adds exactly outer Provider(`wake`), inner Provider(`wake`), then
SourceHandler(`sleep`) and observes the inner `Int(73)`. It remains separate production evidence
for TASK-1993; it adds neither arbitrary chains nor direct-runtime↔checked-CPS parity.

## Authoritative references

- [SPEC-099b](../../spec/SPEC-099b-TARGET-OPERATIONAL-SEMANTICS.md): operation frames and the
  rule that rows never install frames.
- [TASK-1993](TASK-1993-verus-frame-ordered-dispatch-pilot.md): innermost-first lookup and its
  outermost-first mutation sentinel.
- [TASK-2013](TASK-2013-source-handler-and-handle-lowering.md): typed source handler facts and
  non-synthesis of provider/frame authority.
- [TASK-2014](TASK-2014-source-handler-runtime-boundary-decision.md): strict Path-B closed
  admission, Engine-private ownership, and canonical control envelope.
- [TASK-2024](TASK-2024-handler-local-effect-row-propagation.md): the exact nonproduction
  `forward_sleep` Core/CPS row and operation-identity fixture promoted here.
