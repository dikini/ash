# TASK-2012: Declared Operation-to-Provider Binding

**Status:** Complete — bounded local binding, explicit-row admission, and direct provider
execution slice. Generic/imported/multi-provider implementation coverage, handler UX, and
production CPS realization remain deferred; the symbolic operation-call contract is not pending a
design choice.
**Phase:** Follow-up from [TASK-2011](TASK-2011-declared-concrete-impl-operation-source-calls.md)
and [TASK-1829](TASK-1829-operation-row-provider-admission.md)

## Description

Bind a resolved local `DeclaredConcreteOperation` to one admitted provider operation through
explicit typed metadata, then use that binding for source admission and existing direct-runtime
execution.  This closes the authority gap intentionally left by TASK-2011: a declaration-backed
row may name `TestClock::sleep`, but it does not identify a provider or grant authority.

The binding is a first-class typed relation from canonical concrete operation identity plus
declared signature to a provider identity plus `ProviderOperationMetadata`.  It must be registered
or validated at a controlled engine/runtime boundary.  It must never be inferred merely because
the impl type, provider name, operation tail, required-row string, or source qualifier happens to
match textually.

This is the authority implementation boundary for an already-specified resolved source call, not
an alternative call syntax or a decision about whether ordinary arguments are resolvable. A normal
call such as `PosixFs::read(path)` is resolved and typed from its declaration before this boundary;
the binding then selects only explicitly registered authority for its exact canonical identity.

## Authoritative References

- [TASK-2011](TASK-2011-declared-concrete-impl-operation-source-calls.md#completed-local-resolver-row-and-raise-stage): local declaration resolution, row attachment, and private `Raise` exist, but provider mapping/execution do not.
- [TASK-1829](TASK-1829-operation-row-provider-admission.md): rows check already registered authority and never register it.
- [SPEC-097b §8.8](../../spec/SPEC-097b-TARGET-TYPE-SYSTEM.md): rows do not install frames; matching handler/provider frames discharge in innermost-to-outermost order.
- [SPEC-098b §5 and §runtime dispatch](../../spec/SPEC-098b-TARGET-IR.md): `Raise` is discharged by an admitted provider/handler frame; missing authority and unhandled effect are distinct.
- [ASH-CPS-CALCULUS gated effect extension](../../spec/ASH-CPS-CALCULUS.md#gated-effect-extension): `SEM-EFFECT-LOOKUP-001`, `SEM-EFFECT-RAISE-001`, and `SEM-EFFECT-MISSDISCHARGE-001`.
- [CANONICAL-CORE](../../spec/CANONICAL-CORE.md#target-types-and-effects): `TYPE-TARGET-ROW-001` remains authoritative for non-granting rows.

## Scope

### In scope

- One deterministic local declared operation fixture, initially the completed
  `TestClock::sleep(Int) -> Null` shape from TASK-2011.
- A typed `DeclaredOperationProviderBinding`-style carrier that names the exact concrete identity,
  signature, provider identity, provider operation, and validated `ProviderOperationMetadata`.
- Registration/validation against existing `CapabilityProvider`, `ProviderAuthoringMetadata`, and
  `ProviderOperationMetadata` APIs, plus existing standard-profile/provider installation.
- Admission that requires the exact registered binding and an installed matching provider before
  direct runtime runs the call.
- Existing direct-runtime execution through the bound provider operation, with its normal result,
  error, provenance, and policy behavior.
- A private CPS/handler-frame inspection that proves a user handler matching the exact operation
  wins over an outer provider frame, consistent with existing innermost-to-outermost lookup.

### Explicit deferrals

- These are implementation-coverage extensions of the settled `ImplType::operation(args)`
  contract, not unresolved semantics or compatibility requirements for the removed string
  `invoke` form.
- String inference from `TestClock`, `sleep`, `clock.sleep`, provider metadata `required_row`, or
  any matching namespace/tail.  A name-only compatibility fallback is prohibited.
- Wildcard/generic descriptors, automatic binding synthesis, generic/interface/binding-name calls,
  imported declarations, cross-module binding selection, and multi-provider selection policy.
- Handler syntax, handler registration UX, multi-operation handlers, residual-row transformations,
  and production promotion of checked Core/CPS.
- Direct-source `invoke`, hidden `ActEnv` capture, and any public `Act<T>`/`Proc<T>` restoration.
- General direct-runtime/Core-CPS parity; TASK-2005 remains its owner.

## Target APIs and Data Flow

1. **Resolved source fact:** consume `ash_typeck::DeclaredConcreteOperation` from `Entry`; do not
   re-resolve a raw `Expr::Call` string at admission or execution.
2. **Binding metadata:** introduce a typed engine-owned carrier adjacent to entry/admission data,
   referencing the exact concrete identity and declared signature, a provider identity, and a
   provider operation.  Validate it against `ash_core::capability::ProviderAuthoringMetadata` and
   `ProviderOperationMetadata` at registration/install time.
3. **Admission:** extend the existing explicit-row admission seam so it resolves the typed binding,
   then checks the provider is installed and exposes that operation.  Preserve structured distinct
   outcomes for missing binding, missing provider, and missing provider operation.
4. **Direct runtime:** dispatch only through the selected installed provider operation, carrying
   existing host policy/provenance metadata.  It must not reconstruct provider/action strings or
   call the removed source `invoke` path.
5. **Frame priority:** reuse `ash_core::cps::HandlerChain::find_operation_frame` / the checked CPS
   runtime semantics to verify an inner user handler wins over an outer provider frame for the
   same typed bound operation; provider frames persist as specified after resume.

## Requirements

1. A binding is valid only when all identity and signature fields match the resolved concrete
   declaration and the registered provider operation metadata.  Registration rejects unknown
   declarations, duplicate/conflicting bindings, signature mismatch, unknown provider operation,
   and provider metadata that does not declare the selected operation.
2. Source admission uses the resolved declaration-backed identity to look up exactly one typed
   binding.  A row remains non-granting: no binding is synthesized from the row, and no provider
   is installed as a side effect of checking/admission.
3. Absence is diagnostic and structured: distinguish missing declared-operation binding, missing
   installed provider, and missing provider operation.  Do not collapse these into an untyped
   unknown-function error or an `UnhandledEffect` runtime trap.
4. With a valid binding and admitted provider, `TestClock::sleep(0)` executes through the existing
   provider operation and yields its declared `Null` outcome.  Errors remain provider/runtime
   outcomes with existing policy and provenance behavior.
5. Preserve the existing private `Raise` identity/row and TASK-1993 innermost-first
   handler/provider-frame semantics.  This task does not install frames or promote checked CPS:
   pre-execution missing binding remains distinct from the existing checked-CPS unhandled-effect
   outcome.
6. Preserve TASK-2011's unknown-impl, unknown-operation, and argument-mismatch diagnostics, and
   preserve TASK-2000 direct-source `invoke` rejection.

## TDD Steps

1. **RED: typed metadata registration.** Add focused engine tests that register the local
   `TestClock::sleep` declaration and a deterministic test provider.  Require a binding object;
   prove that a matching provider/operation name without that object cannot bind.  Add duplicate,
   signature-mismatch, missing-metadata-operation, and unknown-declaration negatives.
2. **GREEN: validated binding carrier.** Implement the minimal typed carrier and registration
   validation against existing provider metadata.  Keep it engine-owned and declaration-backed.
3. **RED: admission distinctions.** Add three source-entry tests for no binding, binding but no
   installed provider, and installed provider missing the operation.  Assert distinct structured
   diagnostics before runtime execution.
4. **GREEN: admission lookup.** Thread only the resolved `DeclaredConcreteOperation` through the
   binding/admission seam; prove the row alone neither chooses nor installs authority.
5. **RED: direct execution.** Add an admitted deterministic provider case for
   `TestClock::sleep(0)` returning `Null`, and an operation-error case preserving the existing
   runtime error/provenance shape.  Keep direct `invoke` rejected.
6. **GREEN: bound dispatch.** Route the call through the selected provider operation without a
   provider/action string fallback.
7. **RED/GREEN: handler priority inspection.** Build a checked CPS chain with a bound outer
   provider frame and matching inner handler; assert inner handler selection and provider-frame
   persistence rules using existing frame lookup/runtime APIs.
8. **Verification.** Run focused engine/interpreter/core/typechecker tests, format, Clippy, docs
   and traceability gates, and `git diff --check`.  Add changelog and trace implementation/test
   nodes only after concrete behavior exists.

## Completion Checklist

- [x] Registry binding is keyed by the resolved concrete declaration identity and validates provider metadata plus exact required-row identity.
- [x] No raw source name, row item, required-row string, or descriptor fallback creates a binding; host selection must call explicit registration.
- [x] Missing binding rejects at explicit-row admission and direct execution before provider invocation; provider metadata operation/row mismatch rejects at registration.
- [x] An explicit bound provider executes `TestClock::sleep(0)` once through existing direct runtime and returns `Null`.
- [x] Rows remain non-granting and source `invoke` remains rejected.
- [x] Existing TASK-1993 frame lookup/dispatch preserves innermost handler/provider priority; this binding slice does not add handler installation or CPS production dispatch.
- [x] Generic/interface/binding/imported/multi-provider behavior and Core/CPS production promotion remain explicitly deferred.
- [x] Focused tests, formatting, Clippy, changelog, plan/index, traceability, docs gate, and diff checks pass.

## Completed Binding, Admission, and Direct-Execution Slice

`Engine::register_declared_operation_provider_binding` accepts a resolved
`DeclaredConcreteOperation` plus host-selected provider and provider-operation names.  The engine
stores the result under the resolved declaration identity, not under a source string.  Registration
first finds the installed provider, validates its `ProviderAuthoringMetadata`, verifies provider
name agreement and the selected `ProviderOperationMetadata`, and requires that metadata to list
the exact declared required row `TestClock.sleep`.  Metadata declaring `grants_authority` is
rejected.  A conflicting second binding for the same declared identity is rejected rather than
making registration order observable.  Thus provider/operation strings are validated host
configuration inputs, not an inference path from the qualifier, operation tail, or row text.

At `admit_application_with_explicit_rows`, a checked declared operation with no registry binding
rejects as `CapabilityAdmissionFailure` with the note
`missing declared-operation binding for 'TestClock.sleep'`; an unrelated provider with matching
text cannot satisfy the row.  `Engine::execute` repeats the missing-binding guard before provider
invocation.  With the exact registered binding, it invokes only the bound provider operation with
the checked literal arguments.  The focused counting provider demonstrates one `sleep` invocation
and `Null` result.  Provider operation-metadata mismatch and required-row mismatch both reject at
registration.

This does not install authority merely because the row exists: the binding registry is a separate
host configuration boundary.  It also does not reintroduce source `invoke`; the focused regression
continues to reject `invoke("clock-host", "sleep", [0])`.  Existing TASK-1993 `HandlerChain`
lookup and CPS dispatch retain the common innermost-first order for handlers and provider frames.
TASK-2012 neither changes that ordering nor adds source handler installation, and its direct
runtime binding dispatch is not a promotion of checked CPS to production.

The evidence is
[`task_2012_declared_operation_provider_binding.rs`](../../../crates/ash-engine/tests/task_2012_declared_operation_provider_binding.rs):
it covers exact bound execution, unbound rejection despite an unrelated provider, metadata
operation/row validation, and retained source-`invoke` rejection.  It does not establish generic,
interface-qualified, binding-name, imported, cross-module, automatic, or multi-provider binding
selection; handler UX/frame installation; per-term trace provenance; or general direct-runtime /
Core-CPS parity. Those are implementation-coverage deferrals, not open semantic decisions for a
declaration-resolved symbolic call or its ordinary evaluated arguments.
