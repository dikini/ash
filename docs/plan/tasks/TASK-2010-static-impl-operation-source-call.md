# TASK-2010: Statically Resolvable Impl-Qualified Source Operation Calls

**Status:** Complete — bounded initial `time::sleep(0)` vertical slice only.  The engine now
recognizes one strict concrete descriptor after ordinary source checking, adds a non-granting row
requirement, uses existing time-provider admission/direct execution, and exposes a private
checked-CPS `Raise` inspection artifact.  General operation-call realization remains deferred.
**Phase:** Follow-up from [TASK-2000](TASK-2000-residual-act-proc-public-machinery-decision.md),
[TASK-1810](TASK-1810-impl-qualified-operation-identity-resolution.md), and
[TASK-1829](TASK-1829-operation-row-provider-admission.md)

## Description

Implement the first admitted source-call path for a statically resolvable concrete operation,
starting only with `time::sleep(0)`.  After the ordinary source checker accepts the existing
qualified call, the bounded engine path recognizes a concrete operation identity, contributes
that identity to the callable requirement row, and dispatches only through already admitted
authority.  It replaces neither the removed direct-source `invoke` form nor the ordinary-function
call semantics for unrelated qualified names.

The first slice is deliberately small: `time::sleep(0)` is the only accepted source operation
call.  Its canonical identity is `time::sleep`; the existing time provider is the only authority
route considered by this task.  A zero duration makes the positive execution observation
deterministic without making elapsed-time behavior part of the language contract.

## Authoritative References

- [TASK-2000](TASK-2000-residual-act-proc-public-machinery-decision.md#accepted-direct-source-invoke-rejection-slice): direct source `invoke` is removed; a replacement needs stable identity, signature, row, discharge, lowering, and observable behavior.
- [TASK-1810](TASK-1810-impl-qualified-operation-identity-resolution.md): identity resolution is not provider lookup or an authority grant.
- [TASK-1829](TASK-1829-operation-row-provider-admission.md): a row checks already registered provider/operation authority; it does not register a provider.
- [SPEC-097b §3.3 and §8.1](../../spec/SPEC-097b-TARGET-TYPE-SYSTEM.md): operation identities are impl-qualified and calls contribute operation row items.
- [SPEC-097b §8.8](../../spec/SPEC-097b-TARGET-TYPE-SYSTEM.md): rows do not install handler or provider frames.
- [SPEC-098c §7](../../spec/SPEC-098c-SURFACE-TO-CORE-LOWERING.md): concrete identity is `ImplType::op` and must survive lowering.
- [SPEC-098b §5](../../spec/SPEC-098b-TARGET-IR.md): an operation request lowers to `Raise`, with a local operation row and an admitted discharge route.
- [ASH-CPS-CALCULUS gated effect extension](../../spec/ASH-CPS-CALCULUS.md#gated-effect-extension): `SEM-EFFECT-LOOKUP-001` and `SEM-EFFECT-RAISE-001` are the stable effect-operation rule identities.
- [CANONICAL-CORE](../../spec/CANONICAL-CORE.md#target-types-and-effects) and [surface-to-Core handoff](../../spec/CANONICAL-CORE.md#surface-to-core-handoff): `TYPE-TARGET-ROW-001` and `LOWER-SURFACE-CORE-001` remain the canonical row and lowering obligations.

## Scope and Explicit Deferrals

### In scope

- One source spelling: `time::sleep(0)` in an ordinary function body.
- Resolution through an explicit registered concrete-operation descriptor, not by arbitrary
  qualified-function fallback.  The existing type environment continues to own the `Int -> Null`
  signature check.
- A canonical `time::sleep` requirement-row item and ordinary admission check against the existing
  time provider.
- A deterministic execution observation with an admitted time provider, and a structured missing
  discharge/admission failure when it is absent.
- A checked target-lowering inspection artifact that represents this request as `Raise` with the
  canonical identity, argument/result signature, local operation row, and source anchor.

### Explicitly deferred

- Generic identities such as `F::op`, interface-qualified calls, specialization, and binding-name
  operation calls.
- Any second operation, any dynamic provider/action string, and any compatibility restoration of
  `invoke`.
- Arbitrary `module::function` calls becoming operations.  Existing ordinary qualified function
  calls retain their own semantics and may not silently acquire provider dispatch.
- Handler syntax, handler installation, row subtraction, and multi-operation dispatch.
- Production execution through checked Core/CPS.  TASK-2004 retains that route as a private
  inspection/prototype boundary.  This task must either use the existing direct-runtime time
  provider with the same canonical identity or reject before execution; it must not claim that a
  target `Raise` inspection artifact alone changes production execution.
- General direct-runtime/Core-CPS parity; that remains TASK-2005 work.

## Requirements

1. Introduce one explicit concrete-operation descriptor for `time::sleep`, with exact module,
   operation, and existing provider-admission names.  Do not infer operation status from a string
   containing `::`; the existing type environment retains the `Int -> Null` signature contract.
2. After ordinary source checking succeeds, recognize exactly the descriptor-matching lowered
   call and attach canonical `time::sleep` to the checked callable's requirement row.  The row is
   a requirement, never an authority grant.
3. Do not route an unrecognized qualified call through a provider.  This slice does not define a
   new operation-specific diagnostic family: ordinary qualified-call lookup and signature
   diagnostics retain their existing owners.
4. At normal application admission, require the existing time provider for the canonical
   `time::sleep` row item.  Provider absence must produce the existing structured admission or
   missing-discharge diagnostic rather than an untyped function-not-found error or a hidden
   `ActEnv` capture.
5. With the provider admitted, execute `time::sleep(0)` through the existing direct-runtime time
   authority and preserve the established unit/null observable result.  No hidden authority may
   be installed merely because the type checker inferred the row.
6. Add a target inspection lowering for the supported operation call that produces `Raise`, not
   ordinary `CoreExpr::Call`/`FnApply`, and retains canonical identity, operation signature, and
   local operation row.  The entry retains its existing entry-body source origin sidecar; this
   initial CPS term does not claim per-term source-origin metadata.  Keep the legacy production
   lowerer boundary explicit because it cannot consume that artifact.
7. Preserve TASK-2000's direct-source `invoke` rejection unchanged.  This task must not create a
   provider/action-string fallback or expose `Act<T>`/`Proc<T>` vocabulary.
8. Add stable traceability evidence only when implementation and tests exist: map the resulting
   implementation/test nodes to `TYPE-TARGET-ROW-001`, `LOWER-SURFACE-CORE-001`,
   `SEM-EFFECT-RAISE-001`, and, if dispatch is exercised, `SEM-EFFECT-LOOKUP-001`.  Task planning
   alone does not constitute an implementation or test node.

## TDD Steps

1. **Discover and freeze the seam.** Locate the existing parser `Expr::Call` module qualifier,
   typechecker qualified-call lookup, time builtin/type registration, engine admission carrier,
   direct-runtime time dispatch, and private target-lowering inspection seam.  Record the chosen
   carriers in the implementation PR before changing behavior.
2. **RED: source typing and identity.** Add a focused typechecker/engine regression for a file
   containing `time::sleep(0)`.  It must initially fail until the checked result exposes the
   canonical `time::sleep` requirement-row item, its `Int -> Unit`/null result contract, and a
   source anchor.  Add negative cases for zero/multiple/non-integer arguments and unknown
   `time::missing`; ensure a non-operation qualified function remains on the ordinary call path.
3. **GREEN: bounded concrete resolution.** Implement only the registered `time::sleep`
   operation-resolution path needed by the red tests.  Run the focused typechecker and engine
   tests and confirm that source `invoke` is still rejected.
4. **RED: admission versus authority.** Add engine integration tests in two configurations:
   a row-bearing `time::sleep(0)` program with no admitted time provider must fail with a
   structured admission/missing-discharge outcome; the same program with the time provider
   admitted must execute and yield the established unit/null result.  A row-only configuration
   must not gain authority.
5. **GREEN: direct-runtime dispatch.** Reuse the existing time provider/admission route to make
   the admitted positive test pass.  Do not add a parallel `invoke` bridge or a hidden provider
   capture.
6. **RED: target inspection lowering.** Add a private checked Core/CPS inspection test that
   expects the supported source operation to lower as `Raise { op: time::sleep, ... }`, with a
   local operation row and source anchor; it must reject or remain explicitly unsupported at any
   currently private production boundary.
7. **GREEN: target artifact only.** Implement the smallest inspection lowering necessary for the
   `Raise` test.  Do not alter TASK-2004's production route solely to satisfy this step.
8. **Regression and documentation gates.** Run focused parser, typechecker, engine, interpreter,
   and target-inspection tests; then run formatting, Clippy, docs, traceability, and diff gates.
   Add the implementation/test traceability nodes only after the concrete evidence exists.

## Completion Checklist

- [x] `time::sleep(0)` is the sole descriptor-based source-operation slice in this task.
- [x] The checked callable carries canonical `time::sleep` as a non-granting requirement row, and the entry retains its body-origin sidecar.
- [x] Arbitrary qualified calls do not acquire provider-operation dispatch; existing ordinary lookup/signature diagnostics remain their owner.
- [x] A requirement row alone grants no authority; missing time-provider admission is structured and fail-closed.
- [x] The admitted existing time provider executes the zero-duration call with the established null result.
- [x] Direct source `invoke` remains rejected and no `Act<T>`/`Proc<T>` public carrier returns.
- [x] The private target inspection artifact uses `Raise`, not ordinary `Call`/`FnApply`, and does not promote Core/CPS to production execution.
- [x] Focused tests, `cargo fmt --check`, affected `cargo clippy -- -D warnings`, docs gate, traceability validation, and `git diff --check` pass.
- [x] `CHANGELOG.md`, `PLAN-INDEX.md`, and semantic traceability evidence record the implementation evidence.

## Completed Initial `time::sleep(0)` Vertical Slice

`crates/ash-engine/src/operation.rs` supplies the strict
`ConcreteOperationDescriptor { module: "time", name: "sleep", provider: "time" }`.  It matches
only the parsed/lowered `time::sleep` call shape; a `::`-qualified string is not itself evidence
that a call is an operation.  Ordinary source checking still uses the existing type-environment
binding for the one integer argument and null result.  After that check succeeds, the engine adds
exactly `CoreRowItem::Operation { path: ["time"], operation: "sleep" }` to `main`'s callable
row.  This is requirement metadata only, not an installed provider, handler, or hidden `ActEnv`
capture.

The focused engine test first admits the checked row without the time provider and observes the
structured rejection.  Installing the existing application-default time profile then permits the
normal direct-runtime execution route, where `time::sleep(0)` returns `Null`.  Thus this is an
admission-and-execution slice using existing authority, rather than a new dynamic dispatch path.
The same test retains TASK-2000's direct-source `invoke("time", "sleep", [0])` rejection.

For private inspection only, `Engine::lower_entry_to_checked_cps` recognizes this exact entry form
and emits `Raise` with capability item `time::sleep`, argument type `Int`, result type `Null`, a
single integer atom argument, and local row `{time::sleep}`.  It remains outside the production
execution boundary established by TASK-2004.  The entry-level origin sidecar is preserved; this
does not add per-term source origins, handler-frame realization, generic operation resolution, or
direct-runtime/Core-CPS parity.

The evidence is
[`task_2010_time_sleep_operation_source_call.rs`](../../../crates/ash-engine/tests/task_2010_time_sleep_operation_source_call.rs):
it covers canonical non-granting row attachment, no-provider rejection, admitted-provider `Null`
execution, private `Raise` inspection, and retained direct-source-`invoke` rejection.
