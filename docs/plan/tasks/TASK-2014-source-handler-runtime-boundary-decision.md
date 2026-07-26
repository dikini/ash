# TASK-2014: Source-Handler Runtime Boundary Decision

**Status:** In progress — Path B is selected. Public Engine routes use a strict closed-admission
guard rather than direct evaluation. In addition to the bounded handler-free/constructor controls,
two exact checked one-frame provider slices now seal Engine-owned production tokens: built-in
`time::sleep` and local declaration-resolved `TestClock::sleep(Int) -> Null` with either a
literal or an already-checked lexical `Int` argument. Each installs its one authorized resolved-
provider frame privately and runs through the async checked-CPS driver. Separately, the exact local
closed-empty `absorb_sleep` handler over `TestClock::sleep(Int) -> Int` runs through an opaque
same-Engine checked-CPS token with one root `SourceHandler` instruction authorizing one private
handler installation/dispatch, but no provider binding/provider frame or frame chain.
General source lowering, handlers beyond that fixture, multi-frame dispatch, and full-route
terminal coverage remain open.
**Phase:** Follow-up from [TASK-2004](TASK-2004-core-cps-production-boundary-decision.md) and
[TASK-2013](TASK-2013-source-handler-and-handle-lowering.md)
**Depends on:** TASK-2004 boundary migration, TASK-2013 typed lowering, TASK-1993 frame-order
evidence, and TASK-2008 canonical terminal projection

## Description

Establish the production execution boundary for admitted source programs, including source
handlers. This task records the approved strict cutover; it does not turn the existing inspection
artifacts into production execution evidence.

## Decision History

Before this decision, TASK-2004 retained
`ProductionExecutionBoundary::LegacyExpressionEvaluator` and TASK-2013's checked Core/CPS bridge
was prototype-only. That retained-private evidence remains historically accurate for the current
implementation, but its architecture choice is superseded by the selected Path B below. It is not
a fallback permitted by the target design.

## Selected Path B: Strict Cutover and Closed Admission

Checked Core/CPS is the **sole production execution owner** for every admitted source program.
There is no legacy direct-evaluator fallback. A source program that lacks validated typed lowering
must reject at admission until its lowering is implemented; parser acceptance, a checked row, a
handler marker, or a private inspection artifact is not admission.

The production admission artifact must carry, at minimum:

- concrete declared operation identities;
- checked handler clauses;
- normalized residual rows;
- source anchors;
- admitted provider bindings; and
- separately authorized frame-installation instructions.

Rows describe effects only. A row, including a residual row, never installs a provider or handler
frame. Admission must validate frame-installation instructions against the artifact's concrete
operation and provider/handler facts before the Core/CPS driver constructs a frame.

Handler and provider lookup must preserve TASK-1993's innermost-first operation-frame rule. The
new admission artifact authorizes frame construction; it neither changes that lookup rule nor lets
row membership bypass it.

All selected production routes — source run, bootstrap entry, CLI, and application admission —
must share the closed admission boundary. They must project return, missing admission,
malformed/unchecked Core, handler-body trap, timeout, and cancellation through the canonical
terminal envelope. No route may fall back to the direct evaluator after failed or absent typed
lowering.

## Approved Engine-Private Production Driver Ownership (2026-07-26)

The sealed provider-frame handoff, resolved provider objects, ordered authorized-frame chain, and
async checked-CPS production driver are private `ash-engine` implementation details. They consume
only the issuing Engine's production token; `ash-interp` supplies public CPS types and validation
but is not an authority-bearing execution owner.

This ownership is required by the crate graph and the sealed-admission boundary. An
`ash-interp` `pub(crate)` constructor is not callable from `ash-engine`; making that constructor
public would let callers reconstruct authority from row/CPS-shaped data; and making `ash-interp`
depend on `ash-engine` would introduce a dependency cycle. The Engine driver therefore retains the
exact `Arc<dyn CapabilityProvider>` selected during admission, builds frames only from its sealed
instructions, and performs the TASK-1993 reverse (innermost-first) lookup itself. No ambient
registry lookup or public frame-handoff API is permitted.

## Narrow One-Instruction Production Slices and Deferred Chain Evidence

The current real production admissions are intentionally narrower than the end-state frame model.
Each seals exactly one checked Provider instruction with one exact resolved provider binding:
built-in `time::sleep`, or the locally declared `TestClock::sleep(Int) -> Null` identity with a
literal or already-checked lexical `Int` argument. The Engine-private driver constructs and
dispatches only that matching frame. This is sufficient to prove that the token—not a row—
authorizes that frame.

The third production admission is intentionally distinct: exact local closed-empty
`absorb_sleep`, which has `TestClock::sleep(Int) -> Int`, direct `resume(ms)`, identity `done`,
and literal `0`. Its private token seals canonical parsed source/Core provenance and one explicit
root `SourceHandler` instruction after a prior same-Engine `check`; it authorizes exactly one
engine-private checked-CPS handler installation/dispatch, but no provider binding, provider frame,
row-derived installation, or frame chain. `Engine::run`/`run_file` alone consume this handler token. Generic
`Engine::execute`, generic V1 evidence, CLI trace/runnable helpers, and all other handler source
forms remain closed.

The actual negative evidence stays at admission: a row alone, foreign Engine, altered anchor, or
wrong/absent binding rejects before a token or frame exists. The current one-instruction artifacts
cannot honestly exercise nested or multi-frame lookup, nor can they supply real stale, duplicate,
or conflicting instructions. Such controls must not fabricate authority solely for a test.

TASK-1993 remains the generic innermost-first lookup requirement. Production evidence for ordered
multiple instructions, nested frames, and stale/duplicate/conflicting instruction handling is
deferred until a widened, real production admission can validate and seal more than one ordered
instruction. This deferral does not relax the end-state requirement that rows never install frames
and authorized lookup is innermost-first.

## Approved Run-Wide Cooperative Control (2026-07-26)

For the future Engine-private async checked-CPS host-operation driver, `ash run --timeout` is one run-wide
deadline covering the post-admission execution phase, not a timeout reset for each operation.
The Engine must pass that deadline and a cancellation signal through one private cooperative
control envelope. Its driver observes it before each reduction and races provider awaits against it; when
multiple outcomes are observed together, cancellation wins over an expired deadline, which wins
over normal completion. Cancellation is drop-only cooperation, not a guarantee of host kill,
rollback, or compensation.

The executable production token must seal the exact checked Core/CPS, source anchor, actual
registry-resolved provider binding, and ordered separately authorized frame-installation
instructions. Rows remain non-authorizing. Frame construction is authorized only by that token
and preserves TASK-1993 innermost-first lookup. The CLI alone projects the existing V1 return,
trap, timeout, and cancellation terminal forms; this decision adds no telemetry. Missing-admission
and malformed/unchecked-Core terminal taxonomy is intentionally still pending.

The narrow admitted host-operation route is no-telemetry and does not support `--trace`.
Consequently, no trace session, report, or telemetry may be inferred from its terminal envelope;
an implementation may reject that flag combination rather than attempting a traced route.

The approved implementation sequence, data flow, constraints, and test matrix are recorded in
[`2026-07-26-task-2014-run-wide-cooperative-control-design.md`](../../plans/2026-07-26-task-2014-run-wide-cooperative-control-design.md).

## Completed Engine-Private Single-Provider-Raise Production Slices

`Engine::admit_production_checked_cps` admits only the fully typed direct source form
`fn main() -> Null { time::sleep(<non-negative Int literal>) }`. It derives the exact operation
from the retained typechecker fact, validates the canonical checked source anchor, resolves the
registered `time.sleep` provider, and seals one explicit Provider frame-installation instruction
with the checked Core/CPS artifact. A row, a public V1 artifact, a foreign Engine, a mutated
anchor, or an absent/mismatched binding cannot mint that token.

Only the issuing Engine can create `ProductionRunControl` for its token. The private
`production_cps_driver` validates checked CPS, constructs one frame from the sealed resolved
provider object, and dispatches only the exact `time::sleep` `Raise`. It checks the same absolute
deadline/cancellation envelope before driver progress and races the provider await with it, with
`cancelled` taking priority over `timeout`, then normal completion. A winning control outcome
drops the in-flight await cooperatively; it promises no host kill, rollback, or compensation.

For this exact admitted route, `ash run` forwards `--timeout` and SIGINT into that Engine control
after admission and the CLI projects `return`, `external/execution/timeout`, or
`external/execution/cancelled` through the existing V1 envelope. The route is deliberately
no-telemetry and rejects `--trace`. Focused Engine evidence covers normal return, issuer/control
binding, deadline overflow, timeout/drop, cancellation priority/drop, and no cross-admission
control reuse; the CLI terminal suite covers JSON/text return, timeout, cancellation, output
ownership, and trace rejection.

This completes neither general operation dispatch nor source-handler execution. The one sealed
instruction cannot establish ordered multi-frame lookup, stale/duplicate/conflicting instruction
handling, `Handle`/`resume`/`done`, residual/open-row realization, generic malformed-Core or
missing-admission terminal taxonomy, or execution for any other source form.

The same admission/driver shape now has one separately sealed declaration-resolved slice:
`fn main() -> Null { TestClock::sleep(7) }` and a prior checked lexical `Int` delay for that exact
local `Clock<TestClock>::sleep(Int) -> Null` declaration. Admission retains the typechecker's
concrete operation identity, validates the canonical source anchor, requires the exact explicit
Engine-registered provider binding, lowers one checked `Raise`, and seals one separately
authorized `Provider` instruction. The Engine-private driver returns `Null` only after dispatching
that selected provider. Missing or mismatched bindings, a forged anchor, a forged public operation
sidecar, or a mutated public legacy Core/argument reject before dispatch. The Engine compares
`Entry::core` with its parse-time retained Core before invoking `check`, then retains the post-check
Core comparison as a second defense; declared-Raise arguments come from the parse-time record, not
the mutable public field. Generic `Engine::execute` remains closed, so this is not a direct-
evaluator fallback. The existing built-in `time::sleep` route remains compatible.

Focused evidence is
[`task_2014_declared_operation_production_admission.rs`](../../../crates/ash-engine/tests/task_2014_declared_operation_production_admission.rs).
It does not admit generic or imported declarations, `PosixFs`, handlers, multi-frame chains, row-
derived frames, or broader terminal taxonomy.

## Completed Bounded V1 Admission-Evidence Artifact

`ash_engine::checked_cps_admission::CheckedCpsAdmissionV1` is an engine-owned, in-memory V1
validation artifact. It combines an independently supplied `CheckedLoweredCoreProgram` with a
selected checked handler/application projection. The sealed artifact retains exact declared
operation identities, checked handler-clause facts, residual-row descriptors, a supplied source
anchor, and caller-supplied frame-installation instructions in their input order.

The validator proves only the bounded authorization contract:

- a fully handled concrete operation admits with an explicit matching source-handler instruction;
- an unhandled concrete residual operation requires an explicit matching provider binding;
- provider instruction identity compares the impl type, interface, operation name, parameter
  types, and result type exactly;
- source-handler instructions must name a checked clause and the exact checked Core `Handle`
  locator; and
- an unexpanded residual open tail rejects until resolver-attested concrete expansion exists.

Rows, including residual rows, are descriptive evidence only. They never synthesize a handler or
provider instruction. The artifact preserves instruction order for a later TASK-1993
innermost-first frame construction step; it does not construct frames or perform lookup itself.
Focused evidence is
[`task_2014_checked_cps_admission.rs`](../../../crates/ash-engine/tests/task_2014_checked_cps_admission.rs).

This is **not production admission or a cutover**. It has no source-to-Core provenance bridge, no
registry-admitted host-provider binding, no frame construction or execution, no async CPS
host-operation driver, and no terminal-envelope projection. The public route guard below now
rejects before execution, but does not consume this artifact or turn it into a successful
admission.

## Completed Entry-Owned Checked Handler Source Facts

After a successful `Engine::check`, the Engine retains its checked type result behind a private
Engine/Entry provenance token and the exact checked entry-body source anchor. Only
`Engine::checked_source_facts_for_handler` may project that retained result into
`CheckedSourceFactsV1`; it requires the same Engine-owned `Entry`, a prior successful check, and
an unchanged `EntryLoweringSidecars::entry_body_origin`. A same numeric entry ID from another
Engine, an unchecked entry, or a mutated public anchor rejects rather than reusing those facts.

The focused `absorb_sleep` source-handler fixture now parses and checks far enough to retain this
source evidence. It remains a deliberately unlowered parse/check boundary: `Engine::run` rejects
it at checked Core/CPS admission, so no placeholder handler result can execute. The facts do not
prove a source-to-Core `Handle`/`Raise` artifact, authorize V1 admission or frame installation,
bind a provider, perform lookup, drive an async operation, or project a terminal envelope.

## Completed Narrow Handler Inspection Admission

`Engine::admit_checked_handler_inspection` is a bridge for one pre-checked, Engine-owned entry.
It reuses the private owner token and exact checked entry-body anchor before building facts, so
unchecked entries, entries from another Engine (even at the same numeric ID), and anchor mutation
reject. The bridge invokes TASK-2013's closed-empty identity Core inspection subset, validates and
typechecks/lowers the resulting root `CoreExpr::Handle`, and seals a `CheckedCpsAdmissionV1` whose
source anchor is exactly the entry anchor.

The public result is instead an opaque `CheckedHandlerInspectionAdmission` that wraps V1. Only the
issuing Engine can create its private execution seal; it retains the exact source anchor and one
exact root `SourceHandler` instruction for the selected checked operation. A generic public V1,
even when reconstructed from public checked facts, is not an executable parameter, and a foreign
Engine rejects the opaque admission.

`Engine::execute_checked_handler_inspection` accepts only that opaque admission. It adds the exact
private answer `LetCont` around the already-checked CPS term and returns the closed-empty identity
`echo_sleep` handler's `Int(0)` result without a provider. The instruction is an execution gate,
not an instruction to construct a runtime frame: this slice performs no ordered frame installation
or TASK-1993 operational dispatch, provider/residual handling, generic handler execution, async
host operation, timeout/cancellation handling, public-route integration, or canonical terminal-
envelope projection. General source/Core provenance and production admission remain open except
for the separately sealed `absorb_sleep` production slice below.

## Completed Sealed Closed-Empty Source-Handler Production Slice

`Engine::admit_production_checked_handler` admits only the local `absorb_sleep` fixture over
`TestClock::sleep(Int) -> Int`: exactly one clause, direct `resume(ms)`, identity `done`, and a
literal `0` operation argument. It first requires a successful check issued by the same Engine,
then compares the public entry against its canonical parsed source anchor and retained parsed
legacy Core before reusing checked handler facts. The inspection lowering/type-check validates the
root `Handle`/`Raise`, and the resulting opaque production token retains the exact anchor, sealed
handler name, and one root `SourceHandler` instruction. A row never installs a frame.

`Engine::execute_production_checked_handler` is the only consumer of that token; `Engine::run`
and `run_file` route this source shape to it. It terminalizes the already-checked CPS term with its
one authorized engine-private checked-CPS handler installation/dispatch, without the legacy
evaluator, a provider binding, a provider frame, a row-derived/general/multi-frame installation,
or generic V1 execution. Unchecked entries, a foreign Engine, a forged source anchor, a mutated public legacy
Core, a different handler name, or a nonidentity `done` reject before successful production
execution. The route returns `Int(0)` for its one fixture.

This is not general handler execution: it adds no other operation, handler, literal/lexical
argument shape, residual/open row, continuation form, frame chain, TASK-1993 lookup evidence,
async control, CLI trace/runnable route, or handler terminal-envelope taxonomy. Focused evidence
is [`task_2014_handler_production_admission.rs`](../../../crates/ash-engine/tests/task_2014_handler_production_admission.rs).

## Completed Narrow Handler-Free Entry Admission

`CheckedCpsEntryAdmission` is intentionally separate from handler-specific
`CheckedCpsAdmissionV1`: a pure entry has neither checked handler/application facts nor any frame
authorization to validate. `Engine::admit_entry_to_checked_cps` checks a mutable entry, validates
and type-checks its bounded Core lowering, lowers it to CPS, and seals the entry ID and exact entry
body source anchor with a terminal answer continuation. It rejects any nested CPS `Raise` or
`Handle`, so no operation, handler, provider, residual row, or frame authority can enter this
positive slice.

`Engine::execute_checked_cps_admission` is the sole consumer of that token. It invokes the checked
CPS terminal evaluator with empty environment and handler chain; it uses neither the direct
expression evaluator nor direct providers, frames, or async host operations. `Engine::run` routes
the supported pure subset through this owner: the focused literal test receives `Int(42)`, and the
approved binary `Add`/`Sub`/`Mul`/`Div` plus `Eq`/`Ne`/`Lt`/`Le`/`Gt`/`Ge` family reaches exact
left-to-right nested `LetPrim` spines with typed atomic leaves, fresh internal temporaries, and a
final `Jump(__answer)`. The same typed `PureAnf` normalizer recursively admits Boolean `Not` over
its typed Boolean subexpressions. A computed `let` is admitted only for a variable pattern with an
atomic or recursively approved `PureAnf` RHS: that RHS's collision-safe `LetPrim` spine precedes
the typed source `LetVal`, whose final atom carries the source binding type into the body. Boolean
`if`/`match` conditions and branches use the same normalizer. Calls, `Raise`/`Handle`, provider/frame
forms, unary `Neg`, non-Boolean `Not`, Boolean equality, `&&`/`||`, and other unsupported children
reject before direct evaluation. `run_file` has
the same bounded handler-free route, and
zero-input canonical bootstrap can project its bounded constructor return. The CLI
`run_runnable_source` and `execute_with_trace` helpers now also admit the same checked pure entry
before terminal execution; representative primitive runnable-source cases are covered, and the
trace helper still records admission failure in its trace session. Input bootstrap, the remaining
CLI route matrix, and application admission remain closed. This does not make source handlers
executable.

## Current State and Explicit Gap

The selected architecture is partially implemented through the two exact one-frame provider slices
and one closed-empty root-source-handler slice
above. `Engine::execute` and `Engine::execute_with_input` still select
`ProductionExecutionBoundary::CheckedCoreCpsClosedAdmission` and reject without calling the
legacy expression evaluator or direct provider path. `Engine::admit_application` likewise
returns a structured admission rejection rather than evaluating its body. Thus existing public
source execution is closed after parsing/checking unless it is one of the sealed positive slices.
`Engine::run`/`run_file` additionally admit the bounded handler-free pure subset, and zero-input
bootstrap additionally admits the bounded nested-constructor return subset; its capability-input
route remains closed through `execute_with_input`. None broadens the remaining closed routes.

The async driver is deliberately restricted to its exact sealed provider awaits. Completed
TASK-2026 adds one exact two-instruction composition: a canonical local `forward_sleep` token
installs outer `Provider(TestClock::wake)` then inner `SourceHandler(TestClock::sleep)` only
through explicit Engine-issued instructions and reverse-scans innermost-first. It requires
same-Engine source/Core/anchor provenance and an exact registered `wake` binding; its local row
grants no frame authority. Focused paused-time evidence proves normal return, timeout,
cancellation, cancellation priority over an expired deadline, and cooperative dropping of the
pending `wake` await. The separately sealed handler terminalization is otherwise restricted to
`absorb_sleep`'s direct resume and identity done semantics, and has no CLI route. General
source-handler lowering, continuation/resume behavior, `done`/residual-row realization, and
arbitrary multi-frame dispatch remain incomplete. The CLI envelope now covers return, timeout,
and cancellation for the one-frame provider route, but missing admission, malformed/unchecked
Core, and handler-body trap still lack the required canonical taxonomy and route coverage.

Consequently, current accepted source forms must continue to fail closed wherever they lack
validated typed lowering. This is an implementation-state statement, not a retained legacy
execution policy.

The TASK-446 lexical-scope regression keeps parser/typechecker coverage for nested bindings,
block shadowing, and independent `if`-branch binders. Its sequential non-shadowing atomic-let
control is now admitted by `PureAnf` and returns `Int(60)` for
`let a = 10; let b = 20; let c = 30; a + b + c`. Only the duplicate lexical-shadowing control
remains closed at checked Core validation; input-bearing conditionals remain closed for missing
typed lowering. This is deliberately not a general lexical-scope execution claim.

The TASK-597 JSON-import regression makes this rule observable for a legacy stdlib call surface:
representative `parse` inputs, both stringify imports, their combined import, and malformed JSON
all parse and typecheck, then receive the same exact missing-typed-lowering admission error.
The malformed case deliberately demonstrates that admission precedes JSON host dispatch.

The broader strict-cutover migration follows the same evidence rule: tests retain parse, check,
import, and declared binding/profile setup evidence, then assert the exact closed-admission error
instead of a former direct-evaluator result. Provider-wrapper controls additionally prove that
rejection leaves `host_boundary_evidence()` empty, so source admission performs neither provider
dispatch nor host-side evidence emission. This is deliberately negative boundary evidence pending
an artifact-authorized frame installation and async CPS host-operation driver; it is not a general
provider, frame, or runtime implementation. The ambient `do` handler-prewalk and tuple-ADT
legacy-pattern regressions restore source typechecking only. Their richer source forms remain
closed until validated typed lowering can feed this task's artifact.

The legacy CLI bootstrap success fixtures use only the canonical bounded
`Result<(), RuntimeError>` unit-`Ok` entry. They are distinct from the exact production
`fn main() -> Null { time::sleep(<non-negative Int literal>) }` slice, which is admitted and
controlled after sealing a checked-CPS token. The legacy generic outer signal race remains a
command-level control for other routes; it is not evidence about the Engine's post-admission
deadline/cancellation/provider race. The narrow `time::sleep` route is no-telemetry and does not
support `--trace`; it must not be described as a traced bootstrap route. Top-level Boolean literal
exhaustiveness and the reserved-`handle`/unregistered-`Proc` TASK-786 controls are front-end
constraints only and do not expand the admitted source set.

## Remaining Delivery Sequence

1. Complete typed source-to-Core lowering for each additional admitted source subset and reject
   every other source form at the shared admission boundary.
2. Widen private frame construction only when a real admission seals validated ordered multiple
   instructions; preserve TASK-1993 innermost-first lookup and add real multi-frame evidence.
3. Route all production entry points exclusively through checked Core/CPS; remove the direct
   evaluator as an execution fallback for admitted source programs.
4. Extend canonical terminal-envelope projection and differential evidence for return, missing
   admission, malformed/unchecked Core, handler-body trap, timeout, and cancellation.

## TDD Steps

1. Add failing route-level tests proving that every admitted source route reaches checked Core/CPS
   and that unsupported lowering rejects at admission without direct-evaluator fallback.
2. Add failing admission-artifact tests covering concrete identity, clauses, residual rows,
   anchors, provider bindings, and frame-install authorization; prove that rows alone install no
   frame.
3. Add failing Engine-owned one-frame, async host-operation, continuation, and handler-body
   failure tests. Keep row/issuer/anchor/binding negatives at admission; defer fabricated
   multi-frame ordering controls until a real multi-instruction artifact exists. Prove a public
   `ash-interp` handoff cannot be constructed or used as authority.
4. Add canonical terminal-envelope tests for each required terminal outcome across all production
   entry routes.
5. Implement the cutover incrementally, keeping unsupported source forms closed until their typed
   lowering and tests exist.

## Completion Checklist

- [x] Path B strict cutover with closed admission is selected.
- [ ] The bounded V1 artifact validates checked facts and separately authorizes frame
  installation; production admission still lacks source-to-Core provenance and registry-admitted
  provider bindings.
- [x] Public Engine source execution and application admission fail closed without a direct-
  evaluator fallback while no validated production artifact exists.
- [x] A bounded handler-free pure entry can be admitted as an anchor-bound sealed token and run by
  the checked CPS evaluator without providers or frames.
- [ ] Every admitted source route is owned by checked Core/CPS and executes its validated artifact.
- [x] The current one-frame provider slices construct only their exact token-authorized provider
  frame; row/issuer/anchor/binding rejections occur at admission. A separate closed-empty
  `absorb_sleep` token authorizes one root `SourceHandler` instruction and its one private
  checked-CPS handler installation/dispatch, but no provider binding/provider frame, row-derived
  installation, or frame chain; `run`/`run_file` terminalize it.
- [ ] TASK-1993 innermost-first handler/provider lookup is preserved through Engine-private
  authorized multiple-frame construction once a real production admission seals ordered multiple
  instructions.
- [x] Async host-operation/provider execution supports the one exact admitted `time::sleep`
  provider binding with one post-admission absolute deadline and cooperative cancellation.
- [ ] General handler lowering, continuation/resume, `done`, and residual-row semantics are
  realized for admitted forms.
- [ ] Return, missing admission, malformed/unchecked Core, handler-body trap, timeout, and
  cancellation use the canonical terminal envelope across all required production routes. The
  exact `time::sleep` route currently projects return, timeout, and cancellation only.
- [ ] TASK-2004, TASK-2013, TASK-2005, TASK-2008, TASK-439, plan index, changelog, traceability,
  tests, and docs gates record the implemented cutover evidence.

## Explicit Non-Goals

- Retaining a legacy direct-evaluator fallback for an admitted or rejected source program.
- Treating the sealed `absorb_sleep` fixture as generic source-handler behavior, or treating the
  inspection admission as its production token.
- Synthesizing provider/handler frames from rows or adding a provider solely because a handler
  clause matches an operation.
