# TASK-2004: Core/CPS Production-Boundary Decision

**Status:** In progress — TASK-2014 Path B now has a strict closed-admission guard at the public
Engine boundary. The guard removes direct source evaluation and direct provider execution from
that boundary. `Engine::run`/`run_file` and the bounded CLI runnable/trace helpers admit a narrow
handler-free pure subset, including one bounded typed `PureAnf` fragment over approved binary
primitives and recursive Boolean `Not`, and zero-input canonical bootstrap
admits the bounded constructor subset, via sealed checked Core/CPS artifacts; the general
production cutover remains incomplete.
The ordinary-file CLI path is now parse → check → sealed checked-CPS admission → execution: it no
longer selects the former bootstrap/direct evaluator after a source has parsed or checked.
**Phase:** Follow-up from [TASK-1988](TASK-1988-semantic-implementation-deprecation-audit.md)
**Depends on:** TASK-1989 and TASK-2003

## Description

Establish the production Core→CPS boundary and make its status impossible to misstate. TASK-2014
selects strict cutover with closed admission: checked Core/CPS becomes the sole production owner
for admitted source programs, and unsupported typed lowering rejects at admission without a
legacy direct-evaluator fallback.

## Requirements

- Trace source entry through validation, checking, lowering, execution, and terminal projection.
- Enforce the TASK-2014 Path B closed admission boundary for every production source route.
- Require a validated typed admission artifact: concrete operation identities, checked handler
  clauses, residual rows, source anchors, admitted provider bindings, and separately authorized
  frame-installation instructions. Rows alone never install frames. Require named canonical
  terminal outcomes.
- Preserve prototype/private evidence as historical evidence only; do not present it as the target
  production boundary or a compatibility fallback.

## TDD Steps

1. Add a failing boundary/reachability test for the selected architecture.
2. Add end-to-end checked input and malformed/unchecked boundary cases.
3. Implement the boundary or privacy guard.
4. Run application, Core/CPS, conformance, and docs gates.

## Completion Checklist

- [x] The prior retained-private route evidence and its limitations are recorded without claiming
  production Core/CPS execution.
- [x] Public Engine source execution and application admission select the checked-Core/CPS
  closed-admission boundary and reject rather than use the direct evaluator or direct provider
  shortcut while no valid production artifact exists.
- [x] The opt-in handler-free entry slice checks, validates, lowers, seals source provenance, and
  terminalizes the supported pure subset through the checked CPS evaluator.
- [x] `Engine::run` uses that sealed admission for the supported handler-free pure subset,
  including nested approved `Add`/`Sub`/`Mul`/`Div` and `Eq`/`Ne`/`Lt`/`Le`/`Gt`/`Ge` trees with
  typed atomic leaves and recursive Boolean `Not`, including variable-pattern lets, Boolean
  conditions, and Boolean `if`/`match` branches.
  RHS `LetPrim` bindings precede the typed source `LetVal`; unsupported lowering rejects before
  direct evaluation.
- [x] Zero-input canonical bootstrap routes its bounded constructor subset through sealed checked
  CPS and retains the terminal value with its derived exit code.
- [x] The bounded CLI helpers admit their supported checked pure entries through the sealed token;
  runnable-source coverage includes nested binary ANF, while unsupported calls, broader unary nesting,
  effects, and frames retain the closed-admission diagnostic.
- [ ] All source, bootstrap, CLI, and application-admission routes accept and execute a validated
  checked Core/CPS production artifact through the selected boundary.
- [ ] Beyond TASK-2014's implemented one-frame built-in `time::sleep` and local
  `TestClock::sleep(Int) -> Null` production admissions and their Engine-private async driver, and
  the one closed-empty `absorb_sleep` handler (`TestClock::sleep(Int) -> Int`, direct `resume(ms)`,
  identity `done`, literal `0`) admitted only through `Engine::run`/`run_file`, general/V1
  production routes, generalized provider selection, general handler execution, multi-frame
  construction, and TASK-1993 lookup preservation remain unimplemented and unroute-tested.
- [ ] Canonical terminal-envelope coverage includes return, missing admission, malformed/unchecked
  Core, handler-body trap, timeout, and cancellation. The bounded closed routes implement missing
  admission as `external/admission/rejected` (exit 1) and invalid purported checked Core/CPS as
  fixed `pre_entry_failure/entry_verification` (exit 4). The exact admitted abortive `trap_sleep`
  fixture now projects its fixed `1 / 0` as V1 `trap` (exit 5), but no general handler-body or
  continuation semantics are complete.
- [x] Task/index/changelog/traceability updates record the implemented closed-admission guard and
  bounded positive slices without claiming the general production cutover.

## Evidence required

TASK-1988 found no non-test caller of Core→CPS lowering or checked CPS evaluation; any contrary
claim needs an end-to-end executable proof.

## Current sealed local-call evidence

`Engine::run` additionally admits exactly `fn helper() -> Int { 7 }` followed by
`fn main() -> Int { helper() }`. The Engine first requires its canonical parsed Entry provenance,
including an unchanged public legacy Core and source anchor, then refuses any retained imported
type/semantic/type-function state. Its checked artifact is `Core LetVal/Lam/Call` and lowers to a
CPS lambda that jumps to its explicit continuation plus a tail `Call(helper, [], __answer)`.
Only the opaque handler-free admission token executes it to `Int(7)`; generic `execute` remains
closed. This is neither general function-call lowering nor a legacy direct-evaluator compatibility
route: parameters, inference/thunking, recursion, closures, imported callees, and every other call
shape still reject at admission.

## Current Sealed Declaration-Resolved Provider Evidence

The second real one-frame host-operation admission is local
`TestClock::sleep(Int) -> Null`, with either a literal or already-checked lexical `Int` delay.
Before checking, the Engine compares public `Entry::core` with its retained parse-time Core; after
checking, it retains the corresponding checked Core comparison as defense in depth. The declared
`Raise` argument is derived from the parse-time record, while the operation identity, canonical
anchor, explicit Engine binding, and one authorized Provider instruction remain exact. Thus public
Core/sidecar/anchor mutation and missing or mismatched binding reject before dispatch, and generic
`Engine::execute` remains closed. This is not generalized declaration/provider execution, frame
lookup, or terminal-taxonomy evidence.

## Bounded TASK-2014 V1 Admission Evidence

The V1 artifact is a private/in-memory validation seam, not the production boundary. It accepts
checked Core/CPS plus checked source facts; preserves exact operation, clause, residual-row, anchor,
and ordered explicit frame evidence; rejects row-only authority, concrete provider-identity mismatch,
wrong handler locators, unresolved open tails, and residual concrete operations lacking an explicit
provider instruction. A fully handled operation needs only its explicit source-handler instruction.

It neither proves that source lowering produced the supplied checked Core artifact nor verifies a
host-provider registry binding, constructs any ordered frame, drives async operations, changes
route selection, or projects a terminal outcome. One TASK-2014 exception is an opaque
Engine-issued, closed-empty identity handler inspection admission: one exact root `SourceHandler`
instruction gates answer-continuation terminalization of its already checked CPS term. Generic V1
evidence cannot invoke that executor, and the slice has no provider selection or frame
installation. Consequently it cannot satisfy this task's selected production-boundary requirement.
See
[TASK-2014's bounded artifact evidence](TASK-2014-source-handler-runtime-boundary-decision.md#completed-bounded-v1-admission-evidence-artifact).

Successful Engine checks also retain handler source facts only under a private same-Engine/Entry
token and the exact checked entry-body anchor. This prevents unchecked, cross-Engine same-ID, and
anchor-mutated entries from projecting those facts. One exact exception now consumes those facts:
the canonical parsed `absorb_sleep` fixture validates retained parse-time Core/source provenance,
lowers its closed-empty identity handler to checked Core/CPS, and seals one root `SourceHandler`
instruction in a distinct production token. `Engine::run`/`run_file` terminalize that token to
`Int(0)` with its one authorized engine-private checked-CPS handler installation/dispatch, but
without provider binding, provider-frame construction, or row-derived authority. It remains a
single fixture, not a generic source-to-Core/provisioning artifact.

## Current Sealed Closed-Empty Handler Evidence

The third positive TASK-2014 slice is local `absorb_sleep` over
`TestClock::sleep(Int) -> Int`, with exactly direct `resume(ms)`, identity `done`, and literal
`0`. A prior same-Engine check, immutable canonical parsed anchor and Core comparison, checked
handler facts, typed Core/CPS validation, and one exact root `SourceHandler` instruction are all
required before an opaque Engine-issued production token exists. `run` and `run_file` consume it
through checked CPS terminalization with that one authorized handler installation/dispatch;
generic `execute`, generic V1 artifacts, provider bindings, provider frames, row-derived/general/
multi-frame installation, CLI trace/runnable helpers, and all nonsealed handler shapes remain closed. Its
provenance-negative tests reject unchecked, foreign, anchor-mutated, and Core-mutated entries;
handler-name and nonidentity-`done` controls also reject before successful execution.

## Historical Retained-Private Evidence

The prior selected architecture was **retain private/prototype**. It declared
`ProductionExecutionBoundary::LegacyExpressionEvaluator`, executed `Engine::run` through the
direct expression evaluator, and evaluated admitted application bodies directly. That evidence is
historical only and is no longer an implementation-state description or a permitted fallback.

These focused behavior-plus-declaration regressions document the superseded boundary. The private
`#[cfg(test)]` counter continues to distinguish the explicit inspection bridge from the sealed
admission path and is absent from production builds and public APIs. The current positive-route
tests, rather than that historical zero-call expectation, prove that bounded `run`, `run_file`,
and zero-input bootstrap execute through sealed checked Core/CPS admission. This does not
establish a general Core/CPS refinement or route-wide live language execution.

The private prototype boundary is also checked at its Core input: `lower_core_program` accepts only a
validated Core program. The focused admission tests reject malformed raw Core before lowering and
admit a validated program. Together with TASK-2003's checked CPS terminal-projection tests, these
establish separate prototype admission and terminal behavior; they do **not** establish Core/CPS
production execution.

`ash_core::cps` and `ash_interp::cps` remain currently exported compatibility/prototype surfaces.
Their visibility, external compatibility, and checked-versus-unchecked API decision are explicitly
owned by TASK-2006; this task neither removes nor promotes those APIs.

## TASK-2014 Path B Migration Requirements

The retained-private decision above is superseded by
[TASK-2014 Path B](TASK-2014-source-handler-runtime-boundary-decision.md#selected-path-b-strict-cutover-and-closed-admission).
The selected target permits no legacy direct-evaluator fallback. This task now owns the boundary
migration and must coordinate with TASK-2013's typed lowering and TASK-2008's terminal envelope:

- introduce the explicit admission artifact before any frame is constructed; it carries concrete
  operation identities, checked clauses, residual rows, source anchors, admitted provider
  bindings, and separately authorized frame-installation instructions;
- reject missing admission and malformed/unchecked Core at the common production boundary, carrying
  their typed classifications to the CLI rather than reconstructing them from diagnostic text;
- add an async CPS host-operation/provider driver rather than invoking async providers through the
  synchronous prototype evaluator;
- preserve TASK-1993 innermost-first handler/provider lookup after authorized frame construction;
- eliminate direct evaluator execution for all admitted source programs and project return,
  missing admission, invalid checked Core/CPS, handler-body traps, timeout, and cancellation
  through the canonical terminal envelope.

Until each source form has validated typed lowering and admission evidence, it must fail closed at
admission. The existing prototype tests do not satisfy any of these migration requirements.

## Completed Strict Closed-Admission Guard

`Engine::production_execution_boundary` now declares
`ProductionExecutionBoundary::CheckedCoreCpsClosedAdmission`. `Engine::execute` and
`Engine::execute_with_input` reject with the checked-Core/CPS admission error instead of reaching
the former direct expression evaluator or its provider shortcut. `Engine::admit_application`
performs its existing request-context collection and returns a structured
`ApplicationAdmissionOutcome::Rejected` with `ApplicationFailureKind::AdmissionFailure`; it no
longer evaluates `request.body` directly.

Consequently, `execute`, `execute_with_input`, input-bearing bootstrap, and application admission
fail closed after normal parsing/checking until a validated production artifact and CPS driver are
wired. `run`, `run_file`, zero-input bootstrap, and the two bounded CLI helper paths have only the
exceptions described below. The focused regression test
[`task_2004_core_cps_production_boundary.rs`](../../../crates/ash-engine/tests/task_2004_core_cps_production_boundary.rs)
proves the declared boundary, direct source rejection, rejection of a nontrivial arithmetic source
that the old evaluator would have returned, and structured application-admission rejection.
The TASK-597 file-backed JSON-import regression similarly proves `parse`, `stringify`, and
`stringify_pretty` still parse/typecheck, then asserts the exact closed-admission diagnostic rather
than any former direct-evaluator result.

The migrated generic-source regressions retain their original parser, typechecker, import, and
where applicable declared-provider-binding evidence before asserting the exact closed-admission
outcome. This includes the string, record, JSON, list, module/import, lexical-scope, role, and
provider-binding surfaces; it is evidence that Path B did not silently retain the former evaluator,
not evidence that those richer source forms run through Core/CPS. The filesystem, HTTP, clock/time,
and logging wrapper controls additionally install their profiles, parse and check their request
shapes, then prove empty `host_boundary_evidence()` after rejection. Thus no host/provider dispatch
or host evidence is produced before authorized frames and an async CPS host driver exist. See
[`task_1936_filesystem_provider_wrappers.rs`](../../../crates/ash-engine/tests/task_1936_filesystem_provider_wrappers.rs)
and
[`task_1937_http_provider_wrappers.rs`](../../../crates/ash-engine/tests/task_1937_http_provider_wrappers.rs).

The general routes remain guards, not checked Core/CPS execution. The bounded positive slices below
do not install frames, bind a provider registry, or perform async host operations. The closed
ordinary-file CLI route now has no bootstrap/direct-evaluator escape hatch: after parse/check it
must obtain a sealed checked-CPS admission or project `external/admission/rejected` (exit 1).
Forged or unchecked purported checked Core/CPS is classified by the Engine before CLI text
conversion and projects the fixed `entry_verification` envelope (exit 4), without dispatch. The
exact admitted abortive `trap_sleep` fixture instead reaches post-admission checked-CPS division
and projects V1 `trap` (exit 5). It proves no general handler or continuation behavior.
Input-bearing bootstrap continues to transport its closed Engine error through the existing
bootstrap error path.

The legacy lexical-scope fixtures retain parser/typechecker proof for nested bindings, block
shadowing, and branch-local binders. Sequential non-shadowing atomic lets are now an admitted
`PureAnf` fragment: `let a = 10; let b = 20; let c = 30; a + b + c` returns `Int(60)` through
checked CPS. The duplicate lexical-shadowing control still rejects at checked Core validation, and
the input-bearing conditional still rejects with the missing-typed-lowering diagnostic. This is
bounded lexical evidence, not a general scope/lowering claim.

The Phase 147 coverage/mutation and Phase 148 flake/orchestration fixtures have the same status:
they retain schema, report, retry, quarantine, shard-planning, and malformed-input evidence, but
their authored Ash test bodies now reject at generic checked-Core/CPS admission. Consequently,
coverage records both authored laws as uncovered and mutation records both generated mutants as
survived; retry and shard reports classify the admission error rather than a source-test result.
The only successful Phase 148 merge control consumes explicitly synthetic successful JSON shard
envelopes. It is protocol evidence for merging envelopes, not evidence that source shard fixtures
executed under Path B.

The authored `ash test` unit/property/small-world controls likewise retain discovery, explicit
names/tags, test-library import checking, and kind-specific seed, failing-case, or world-index
metadata. Their bodies then report the exact generic closed-admission error rather than a passing
execution result; unit rows retain no property or small-world metadata. This is runner metadata
and front-end evidence, not evidence that authored test bodies execute under Path B.

The standalone `ash trace` command remains a generic `Engine::execute` route, distinct from the
bounded `run --trace` helper. A checked but unadmitted source reports the exact missing-typed-
lowering error before trace-document emission: stdout stays empty and `--output` creates no file.
This does not weaken the narrow helper admission evidence or claim standalone trace execution.

The REPL has no independent expression-admission route. Its nonempty input is wrapped as an
unannotated `fn main() { ... }` entry and sent through the same bounded `Engine::run` admission.
Consequently, a literal such as `42` rejects at checked Core-to-CPS lowering with the unresolved
synthetic `main_return` type variable instead of using a direct evaluator. Empty REPL input still
returns `Null` without source execution. This preserves the SPEC-021 failure-visibility contract;
it does not add general expression inference or a REPL compatibility fallback.

## Completed Narrow Handler-Free Positive Admission

`Engine::run` and `run_file` now use `Engine::admit_entry_to_checked_cps` for a checked,
handler-free entry. It
performs `check`, validates and type-checks the bounded Core lowering, lowers to CPS, rejects a
`Raise` or `Handle` anywhere in the resulting term, and seals the entry ID plus its exact
`EntryLoweringSidecars::entry_body_origin` in `CheckedCpsEntryAdmission`. The token keeps the
terminalized CPS term crate-private; only `Engine::execute_checked_cps_admission` can consume it.

That executor runs `eval_checked_terminal` with an empty CPS environment and handler chain, then
maps the checked terminal return into an engine value. It does not call the direct expression
evaluator, resolve a provider, construct a provider/handler frame, or drive an async host
operation. The focused positive regression
[`task_2014_positive_checked_cps_admission.rs`](../../../crates/ash-engine/tests/task_2014_positive_checked_cps_admission.rs)
proves one literal `Int` entry retains its source anchor and returns `42` through this API.

Zero-input `bootstrap_entry_source_result` uses the same sealed path after canonical entry
verification. Its bounded nested-constructor lowering can return the canonical
`Err(RuntimeError(42, "boom"))` terminal value and derive exit code `42`; unsupported nested
computation rejects as `EntryBootstrapError::Execution` with the closed-admission marker.

The CLI's non-bootstrap `run_runnable_source` helper now checks a parsed runnable entry, obtains
`CheckedCpsEntryAdmission`, and consumes it only through
`execute_checked_cps_admission`. Its trace companion `execute_with_trace` performs the same
admission/consumption sequence inside the trace session and records failed admission as
`checked_cps_admission`; neither helper calls `Engine::execute`. The focused module tests in
[`run.rs`](../../../crates/ash-cli/src/commands/run.rs) prove `fn main() -> Int { 42 }` returns
`Int(42)` on each helper. The TASK-2003/TASK-2004/TASK-2014 nested-ANF contracts additionally
prove exact left-to-right `LetPrim` spines for the approved `Add`/`Sub`/`Mul`/`Div` and
`Eq`/`Ne`/`Lt`/`Le`/`Gt`/`Ge` family: recursive binary children have typed atomic leaves, fresh
internal temporaries, and one final `Jump(__answer)`. They prove terminal values through
`Engine::run`, representative `run_file`, and CLI runnable-source cases. Calls, `Raise`/`Handle`,
  providers, frames, unary `Neg`, non-Boolean `Not`, Boolean equality, `&&`/`||`, and other
  unsupported children still reject or remain unavailable. One typed `PureAnf` normalizer admits
  recursive Boolean `Not` over its typed Boolean subexpressions at top level, variable-let RHS,
  Boolean conditions, and Boolean `if`/`match` branches. The exact `7 - 2`
differential fixture remains a separate case-bound private-oracle control, not the authority for
this production admission.

The same sealed pure entry may contain a computed `let` only with a variable pattern and an atomic
or recursively approved `PureAnf` RHS. Its newly generated RHS temporaries are reserved against all
source names and are wrapped left-to-right before the source `LetVal`; the final RHS atom carries
its checked type into the body. The focused nested-ANF contract proves this `LetPrim`-then-`LetVal`
shape and `Engine::run`/`run_file` result. It does not admit destructuring, calls, effects,
  handlers, providers, frames, `Neg`, non-Boolean `Not`, Boolean equality, `&&`/`||`, or general `let`
  lowering.

This is not a route-wide cutover: input-bearing bootstrap, `execute`, `execute_with_input`, and
application admission remain closed without their own validated production artifact. The general
CLI command/route matrix remains open beyond these helpers. The pure and constructor subsets are
intentionally bounded; nested effect `Raise`/`Handle` terms reject before a token exists, and
handler/provider semantics remain deferred.

## Current Bounded CLI Bootstrap Controls

The CLI bootstrap success and nonzero-timeout controls now use exactly
`fn main() -> Result<(), RuntimeError> { Ok { value: {} } }`. They therefore exercise the
already admitted unit `Ok` constructor return rather than a nested `match`, `do`, provider call,
or a general entry-lowering claim. The focused controls prove only that this canonical bounded
entry retains its success behavior with and without the CLI timeout wrapper. They do not make
nested match lowering, general `Result` construction, async host work, or route-wide entry
execution available.
