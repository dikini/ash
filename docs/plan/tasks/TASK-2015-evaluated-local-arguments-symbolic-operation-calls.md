# TASK-2015: Evaluated Local Arguments for Symbolic Operation Calls

**Status:** Complete — bounded checked-local argument transport, exact binding/admission, direct
provider dispatch, and private `Raise` inspection. Arbitrary expressions, imports, generic
selection, and production Core/CPS execution remain deferred.
**Phase:** Implementation follow-up from
[TASK-2011](TASK-2011-declared-concrete-impl-operation-source-calls.md) and
[TASK-2012](TASK-2012-declared-operation-provider-binding.md)

## Description

Implement the existing declaration-resolved symbolic operation-call contract for evaluated local
arguments, starting with `TestClock::sleep(delay)` where `delay` is a checked local `Int` value.
This task extends the completed local literal fixture `TestClock::sleep(0)`; it does not define
new operation-call semantics. The active target specifications already determine that a resolved
`ImplType::operation(args)` call has its declaration-derived identity and signature, contributes
that exact operation row, lowers as `Raise`, and is discharged only by an explicit matching
handler/provider frame or host binding.

The implementation must carry the evaluated checked argument through the existing resolved
operation carrier, exact provider binding, row admission, direct-runtime dispatch, and private
checked Core/CPS inspection. It must not restore string `invoke`, infer a provider from text, or
make an ordinary qualified function into an operation.

## Authoritative References

- [SPEC-097b §3.3 and §8.1](../../spec/SPEC-097b-TARGET-TYPE-SYSTEM.md): an
  impl-type-qualified operation has an interface-declared signature and contributes its exact row
  identity after resolution.
- [SPEC-097b §8.8](../../spec/SPEC-097b-TARGET-TYPE-SYSTEM.md): rows are requirements and do not
  install provider/handler authority.
- [SPEC-098c §7](../../spec/SPEC-098c-SURFACE-TO-CORE-LOWERING.md): canonical concrete operation
  identity survives source-to-Core lowering.
- [SPEC-098b §5](../../spec/SPEC-098b-TARGET-IR.md): operation requests lower as `Raise` with
  typed argument atoms, a local row, and an explicit discharge route.
- [TASK-2011](TASK-2011-declared-concrete-impl-operation-source-calls.md): declaration-backed
  symbolic resolution, exact row attachment, and `Raise` inspection.
- [TASK-2012](TASK-2012-declared-operation-provider-binding.md): explicit typed provider binding
  and direct-runtime dispatch.

## Scope

### In scope

- A declaration-resolved local concrete operation call with a checked local argument, initially
  `let delay: Int = 0; TestClock::sleep(delay)`.
- Reuse of the existing `DeclaredConcreteOperation` resolution and exact
  `DeclaredOperationProviderBinding`; normal expression evaluation supplies the argument value.
- Exact declared-signature checking for the local argument and ordinary source diagnostics for
  unbound or mismatched local values.
- Non-granting row attachment and explicit-row admission identical to the literal slice.
- Direct-runtime dispatch of the evaluated argument through the explicitly registered provider.
- Private inspected Core/CPS `Raise` evidence whose atom/value corresponds to the evaluated local
  argument, while retaining TASK-2004's production boundary.

### Explicit deferrals

- Generic/interface/binding-name symbolic calls, imports/re-exports/cross-module coherence,
  overload/specialization selection, and multi-provider policy.
- Arbitrary expression arguments beyond the narrowly evidenced local-value bridge, if they require
  a separate source/Core evaluation carrier.
- Source handler installation/resumption, residual-row realization, production Core/CPS execution,
  and broad direct-runtime/Core-CPS parity.
- Every stringly invocation/provider fallback, including direct source `invoke`; no compatibility
  layer is permitted.

These deferrals are implementation scope boundaries, not undecided semantics for
`ImplType::operation(args)` or an ordinary resolvable argument such as `PosixFs::read(path)`.

## Requirements

1. A checked local argument compatible with the declaration signature resolves through the same
   canonical `DeclaredConcreteOperation` as the corresponding literal call. The implementation
   must never derive identity from the local variable name or a provider string.
2. The callable row remains exactly the resolved `ImplType::operation` identity and remains
   non-granting. A local argument neither selects nor installs authority.
3. Admission and execution use only the already explicit typed binding. The installed bound
   provider receives the evaluated argument value once; absence/mismatch failures preserve the
   existing structured boundaries.
4. Private Core/CPS inspection emits `Raise` with the resolved identity, declared types, local
   operation row, and the evaluated argument atom/value. This remains nonproduction unless
   TASK-2004's boundary changes separately.
5. Direct-source `invoke` stays rejected, and no raw qualified-name, provider-name, operation-tail,
   or row-text fallback is introduced.
6. Add semantic traceability implementation/test nodes only after concrete code and focused
   evidence exist, mapping the realized behavior to `TYPE-TARGET-ROW-001`,
   `LOWER-SURFACE-CORE-001`, `SEM-EFFECT-RAISE-001`, and `SEM-EFFECT-LOOKUP-001` where dispatch is
   exercised.

## TDD Steps

1. **Freeze the current literal seam.** Identify the `TestClock::sleep(0)` resolved carrier,
   binding/admission route, argument representation, direct provider dispatch, and private
   lowering path. Record the selected extension point without changing resolution semantics.
2. **RED: checked local argument.** Add focused source/typechecker/engine tests for
   `let delay: Int = 0; TestClock::sleep(delay)`. Assert the same resolved concrete identity and
   declared signature as the literal fixture; add an unbound-local and declared-type-mismatch
   negative case.
3. **GREEN: carry the evaluated local value.** Implement only the checked local-value transport
   necessary to satisfy those tests. Confirm no provider lookup occurs during resolution and no
   string fallback enters the path.
4. **RED: authority-neutral admission and dispatch.** Add engine tests proving the local-argument
   row rejects without the exact binding/provider, then invokes the bound deterministic provider
   exactly once with `0` and returns its declared `Null` result.
5. **GREEN: exact bound execution.** Extend the existing binding execution path to consume the
   evaluated argument without changing binding key selection or row policy.
6. **RED/GREEN: private `Raise` inspection.** Assert the checked target carries exact identity,
   declared argument/result types, local row, and evaluated argument atom. Keep the production
   bridge non-invoked under TASK-2004's retained boundary.
7. **Regression/documentation gates.** Preserve TASK-2000 `invoke` rejection and TASK-2011/2012
   literal/binding evidence. Run focused tests, formatting, affected Clippy, docs/traceability
   gates, and `git diff --check`. Update `CHANGELOG.md`, `PLAN-INDEX.md`, this task record, and
   semantic traceability only for behavior actually implemented and tested.

## Completion Checklist

- [x] `TestClock::sleep(delay)` with a checked local `Int` resolves to the same declaration-backed
  concrete operation as `TestClock::sleep(0)`.
- [x] The exact non-granting operation row and explicit provider binding are preserved.
- [x] Missing binding/provider and type/local-value failures remain structured and fail closed.
- [x] The bound provider receives the evaluated local value once and returns the declared result.
- [x] Private `Raise` inspection carries the resolved identity, declared types, local row, and
  evaluated argument; no production Core/CPS promotion is claimed.
- [x] Direct source `invoke` and all string/name inference fallbacks remain rejected.
- [x] Focused tests, formatting, Clippy, docs/traceability checks, and `git diff --check` pass.
- [x] `CHANGELOG.md`, `PLAN-INDEX.md`, and traceability evidence are updated for the implemented
  slice.

## Completed Evaluated-Local Argument Slice

The accepted source shape is a lexical entry spine containing ordinary variable `let` bindings and
a tail declaration-resolved call.  In the exercised fixture, `let delay = 0;
TestClock::sleep(delay)` retains the same checked `DeclaredConcreteOperation` identity and
`TestClock::sleep` requirement row as the literal form.  Its local argument is evaluated to
`Int(0)` before the already explicit `DeclaredOperationProviderBinding` is consulted; the binding
is still keyed by the resolved identity and signature, so neither `delay` nor provider text
selects authority.  Rechecking the same entry is idempotent for that canonical requirement: it
does not append a duplicate `TestClock::sleep` row item.

Without that exact registration, admission/execution remains reject-before-dispatch.  With the
registered `clock-host.sleep` binding, the deterministic provider receives exactly one argument
vector, `[Int(0)]`, and returns its declared `Null` result.  A local `String` argument is rejected
during checking with the existing declared-operation argument-mismatch diagnostic.  The private
inspection bridge lowers the accepted fixture to `Raise(TestClock::sleep, [Int(0)],
{TestClock::sleep})`, retaining declared `[Int] -> Null` types.  A conditional initializer is
not silently evaluated there: it fails closed at the inspection boundary because this slice only
implements literal and earlier-local-value transport.

This is not arbitrary expression evaluation, imported/cross-module resolution, generic or
multi-provider selection, source handler execution, or production Core/CPS execution.  It does
not restore direct source `invoke` or any string/name/row inference fallback.

Focused evidence is
[`task_2012_declared_operation_provider_binding.rs`](../../../crates/ash-engine/tests/task_2012_declared_operation_provider_binding.rs),
which records exact row retention, one direct provider dispatch with `Int(0)`, and a checked
non-`Int` rejection, plus repeated-check row deduplication, and
[`task_2015_evaluated_local_operation_cps_lowering.rs`](../../../crates/ash-engine/tests/task_2015_evaluated_local_operation_cps_lowering.rs),
which freezes the private exact `Raise` artifact and conditional-initializer fail-closed boundary.
The semantic trace maps the realized row, lowering, `Raise`, and exercised explicit lookup facts
to `TYPE-TARGET-ROW-001`, `LOWER-SURFACE-CORE-001`, `SEM-EFFECT-RAISE-001`, and
`SEM-EFFECT-LOOKUP-001`; those links are evidence for this narrow bridge, not a production-CPS
claim.

## No Design Gate

No language-design decision is required before implementation. The semantic contract for
declaration-resolved symbolic calls is already fixed by the cited target specifications. This task
owns only the Rust bridge that evaluates a normal checked local argument and carries its value
through that established contract.
