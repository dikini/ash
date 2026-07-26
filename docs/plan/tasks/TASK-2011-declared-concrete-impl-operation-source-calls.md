# TASK-2011: Declared Concrete Impl-Qualified Source Operation Calls

**Status:** Complete — local declaration-backed resolver, non-granting row, and private `Raise`
inspection stage. Provider mapping/execution and broader implementation coverage remain deferred;
the target semantics of symbolic `ImplType::operation(args)` calls are already specified.
**Phase:** Follow-up from [TASK-2010](TASK-2010-static-impl-operation-source-call.md),
[TASK-1810](TASK-1810-impl-qualified-operation-identity-resolution.md), and
[TASK-1829](TASK-1829-operation-row-provider-admission.md)

## Description

Extend the named-operation source path from TASK-2010's one engine-local `time::sleep` descriptor
to calls whose concrete impl-qualified identity is proven from parsed and registered Ash
declarations.  A supported source call has the shape already represented by `Expr::Call` with a
module qualifier and must resolve to one concrete declared impl identity and one declared
interface operation signature.

This task is about **declared identity resolution**, not a generic string registry.  The source
qualifier, canonical `ImplType::operation` identity, argument/result signature, row item, and
admission key must arise from the same registered interface/impl/module-summary facts.  A spelling
such as `Any::operation`, a `module::function` string, or an engine descriptor that accepts an
open set of names is insufficient evidence and must fail closed.

The bounded local/literal fixture is an implementation seam, not an unresolved call-semantics
question. Once normal resolution proves `PosixFs::read(path)` (or another
`ImplType::operation(args)`) against its declaration, the active target contract already fixes its
identity, declared signature, row contribution, `Raise` lowering, and explicit discharge model.

## Authoritative References

- [TASK-2000](TASK-2000-residual-act-proc-public-machinery-decision.md#accepted-direct-source-invoke-rejection-slice): a source replacement for removed `invoke` needs stable identity, signature, row, discharge, lowering, and behavior.
- [TASK-2010](TASK-2010-static-impl-operation-source-call.md#completed-initial-timesleep0-vertical-slice): the completed descriptor slice is intentionally bounded and is not a declaration-backed resolver.
- [TASK-1810](TASK-1810-impl-qualified-operation-identity-resolution.md): reuse registered interface/impl facts; operation resolution neither finds providers nor grants authority.
- [TASK-1829](TASK-1829-operation-row-provider-admission.md): operation-row admission uses an already registered provider/operation; a row itself is non-granting.
- [SPEC-097b §3.3 and §8.1](../../spec/SPEC-097b-TARGET-TYPE-SYSTEM.md): concrete impl-qualified operations are row identities and their interfaces declare signatures.
- [SPEC-097b §8.8](../../spec/SPEC-097b-TARGET-TYPE-SYSTEM.md): provider/handler frames are runtime authority and are never installed by rows.
- [SPEC-098c §7](../../spec/SPEC-098c-SURFACE-TO-CORE-LOWERING.md): canonical concrete identity is `ImplType::op` and must survive lowering.
- [SPEC-098b §5](../../spec/SPEC-098b-TARGET-IR.md): an operation request is `Raise`, with a local operation row and explicit discharge layering.
- [CANONICAL-CORE](../../spec/CANONICAL-CORE.md#target-types-and-effects) and [surface-to-Core handoff](../../spec/CANONICAL-CORE.md#surface-to-core-handoff): `TYPE-TARGET-ROW-001` and `LOWER-SURFACE-CORE-001` remain authoritative.

## Scope

### In scope

- One or more local, **concrete**, parsed and registered interface/impl declarations whose exact
  existing surface form can establish a concrete impl identity and a declared operation signature.
- A source call represented as `Expr::Call { module: Some(...), func, args, .. }` that resolves to
  exactly one such declaration-backed operation.
- A typed resolved-operation carrier containing at least canonical concrete identity, declared
  argument/result types, declaration/source anchor, and the provider-admission lookup key.
- Checked callable row attachment using that carrier's canonical identity, without authority.
- Existing normal provider admission and direct-runtime execution only where a declared operation
  can be mapped to existing authority without a string fallback.
- A private target inspection lowering that emits `Raise` from the resolved-operation carrier,
  preserving identity, declared signature, local operation row, and the declaration/call anchors
  available at this boundary.

### Explicit deferrals

- The following are implementation-coverage follow-ups under the already-settled symbolic
  operation-call contract; they are not design gates or semantic ambiguities for ordinary source
  arguments or declaration-resolved `ImplType::operation(args)` calls.
- Generic `F::operation`, interface-qualified identities, specialization/monomorphization, and
  binding-name operation calls.
- Imported/re-exported declaration resolution, cross-module coherence, overlap selection, and
  multi-impl dispatch unless a later task narrows them with separate evidence.
- New parser syntax for impl identities.  This task uses only already parsed declaration and call
  carriers; if they cannot express the selected concrete case, it fails closed and records the
  syntax gap instead of widening the grammar opportunistically.
- Any name-only, provider-name-only, `module::function`, or generic descriptor registry; dynamic
  provider/action strings; and any restoration of direct-source `invoke`.
- Handler syntax/installation, residual-row subtraction, handler-frame realization, and arbitrary
  provider dispatch.
- Production Core/CPS execution and general direct-runtime/Core-CPS parity.  TASK-2004 and
  TASK-2005 retain those boundaries.

## Target APIs and Data Flow

1. **Surface/parser:** reuse `ash_core::Expr::Call` module qualifier and the existing parsed
   `InterfaceDef`/`ImplDef` declaration carriers.  Do not add a stringly typed parallel AST.
2. **Declaration registration:** reuse `ash_typeck::type_env::TypeEnv` interface/impl and
   capability-operation lookup/impl-matching APIs.  Add a single declaration-backed resolved-call
   query or carrier at this seam; it must return an ambiguity/unknown/signature diagnostic rather
   than a provider lookup.
3. **Expression checking:** update `crates/ash-typeck/src/check_expr/mod.rs` only after the
   resolver API exists.  The checker consumes the resolved carrier to check arguments and result,
   and emits the canonical operation row identity; it does not inspect runtime providers.
4. **Engine entry/admission:** thread the resolved row through `ash_engine::Entry` and existing
   `CoreRowItem::Operation` / `Engine::admit_application_with_explicit_rows` paths.  Replace no
   ordinary qualified-function path unless it is proven to name the selected declared operation.
5. **Target inspection:** have `Engine::lower_entry_to_checked_cps` (or a narrowly named helper)
   consume resolved operation data to produce `Raise`; do not pattern-match a raw source-name
   string or promote it to production execution.

## Requirements

1. A source operation is admitted only when a local registered declaration proves all of:
   concrete impl identity, associated interface, operation existence, unambiguous selection, and
   declared signature.  The resolver must not manufacture identity from provider metadata.
2. The canonical row item is exactly the resolved concrete `ImplType::operation` identity.
   Different concrete impl identities with the same operation tail remain distinct; a row does not
   install a provider, handler, grant, or hidden runtime capture.
3. The call uses the declared argument/result signature.  Unknown impl, unknown operation,
   ambiguous impl selection, and argument/result mismatch fail before admission with diagnostics
   that identify the source qualifier and canonical identity when one exists.
4. This stage proves only that the row does not grant authority: without an explicit
   declared-operation-to-provider mapping, admission rejects structurally.  Mapping a resolved
   declaration identity to an existing provider and any successful direct-runtime execution are
   separately scoped follow-up work, never direct `invoke`.
5. The private lowering inspection produces `Raise`, not `Call`/`FnApply`, from the resolved
   carrier.  Its `EffectOp` and local row agree exactly with the checked row identity and declared
   signature.  Production execution remains on TASK-2004's boundary.
6. TASK-2010's strict `time::sleep` descriptor stays a compatibility/bounded slice; it must not
   become a wildcard fallback or substitute for declared resolver evidence.
7. Add traceability implementation/test nodes only for behavior actually implemented by this task;
   map row, lowering, raise, and exercised lookup evidence to `TYPE-TARGET-ROW-001`,
   `LOWER-SURFACE-CORE-001`, `SEM-EFFECT-RAISE-001`, and `SEM-EFFECT-LOOKUP-001` respectively.

## TDD Steps

1. **Freeze a declaration fixture.** Identify one existing parseable local interface/impl pair
   with an operation that can be mapped to a deterministic existing test provider.  If no such
   pair exists, write a failing seam test and stop at the documented syntax/registration gap;
   do not replace it with a descriptor or string map.
2. **RED: declaration-backed resolution.** Add focused typechecker tests that register the fixture
   declarations and prove a qualified source call resolves to one carrier containing concrete
   identity, signature, and declaration anchor.  Add negative unknown-impl, unknown-operation,
   ambiguous-selection, and argument-mismatch cases.
3. **GREEN: resolver only.** Implement the minimal `TypeEnv` resolved-call query/carrier and
   expression-checker integration.  Run focused typechecker tests; confirm it never queries a
   provider and ordinary qualified functions still retain their ordinary path.
4. **RED: row and admission separation.** Add engine tests showing the checked call's exact
   concrete row item, structured rejection without the declared operation's provider, and no
   authority from the row alone.
5. **GREEN: declared authority mapping.** Thread only the resolved carrier through existing row
   admission.  Reuse an existing deterministic provider test double/profile; do not add dynamic
   provider/action dispatch.
6. **RED: `Raise` inspection.** Add a private checked-CPS test that checks exact resolved identity,
   declared argument/result types, local row, and available source/declaration anchors.  Assert
   that the production boundary did not invoke this inspection lowering.
7. **GREEN: inspection lowering.** Implement the minimal carrier-to-`Raise` lowerer and preserve
   TASK-2004's production boundary.
8. **Regression gates.** Confirm direct source `invoke` remains rejected and TASK-2010's
   `time::sleep(0)` evidence remains intact.  Run focused parser/typechecker/engine/interpreter
   tests, `cargo fmt --check`, affected `cargo clippy -- -D warnings`, docs/traceability gates,
   and `git diff --check`.

## Completion Checklist

- [x] A local parsed and registered `Clock<TestClock>` declaration fixture proves `TestClock::sleep(0)`.
- [x] The resolver carrier derives identity and signature from registered local interface/impl facts, never provider strings.
- [x] Unknown concrete impl, unknown declared operation, and argument mismatch fail before admission with exact focused diagnostics.
- [x] Checked rows carry exact `TestClock::sleep` identity and remain non-granting.
- [x] No provider mapping exists: explicit-row admission rejects structurally and no provider execution is claimed.
- [x] Private inspection lowers from the resolved carrier to exact `Raise`; production Core/CPS remains unpromoted.
- [x] Direct-source `invoke` remains rejected; TASK-2010 remains its strict descriptor-only slice.
- [x] Focused tests, formatting, Clippy, changelog, plan/index, traceability, docs gate, and diff checks are passing.

## Completed Local Resolver, Row, and `Raise` Stage

The accepted local fixture declares `interface Clock<T> { sleep(Int) -> Null }`, a concrete
`TestClock` type, and `impl Clock<TestClock> { sleep(milliseconds) = null }`, then calls
`TestClock::sleep(0)`.  `TypeEnv::resolve_declared_concrete_operation` searches registered local
impl schemes for the exact concrete target and method.  It returns `DeclaredConcreteOperation`
with `impl_type`, declaring interface, operation name, declared parameter types, and result type;
it does not consult providers, handlers, bindings, or descriptor strings.

`check_expr` consumes that carrier for the qualified source call.  The stage has focused,
deterministic diagnostics:

- `unknown concrete impl 'MissingClock'`;
- `concrete impl 'TestClock' has no operation 'wake'`; and
- `TestClock::sleep: argument type mismatch`.

After successful checking, the engine records the resolved carrier on `Entry` and adds exactly
`CoreRowItem::Operation { path: ["TestClock"], operation: "sleep" }` to `main`'s callable row.
The entry retains its existing body-origin sidecar.  Explicit-row admission then rejects because
this row has installed no authority.  There is deliberately no declared-operation-to-provider
metadata mapping and no successful provider execution assertion in this task.

For private inspection only, `Engine::lower_entry_to_checked_cps` lowers the stored resolved
carrier to `Raise` with capability item `TestClock::sleep`, argument types `[Int]`, result type
`Null`, argument atom `Int(0)`, and local row `{TestClock::sleep}`.  The current term contains no
per-term source anchor, and this path remains outside TASK-2004's production boundary.

The focused evidence is
[`task_2011_declared_concrete_operation_source_call.rs`](../../../crates/ash-engine/tests/task_2011_declared_concrete_operation_source_call.rs).
It does not establish broader implementation coverage for evaluated local source arguments,
generic/interface/binding resolution, imported declarations, provider execution, handler behavior,
production Core/CPS, or direct-runtime/Core-CPS parity. Those remain explicit implementation
deferrals, not undecided semantics for resolved symbolic operation calls. TASK-2000's direct
source `invoke` rejection remains independently enforced and is not reopened here.
