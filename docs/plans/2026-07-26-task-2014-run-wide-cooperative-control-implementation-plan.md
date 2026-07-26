# TASK-2014 Run-Wide Cooperative Control Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Admit and execute the single fully typed, registry-bound `time::sleep` source-operation slice through sealed checked Core/CPS with one execution-phase-wide cooperative timeout/cancellation envelope and CLI-owned V1 terminal projection.

**Architecture:** Keep Path B closed: an Engine-owned production token is minted only after parse/check/typed lowering, exact source anchor validation, registry binding resolution, and explicit ordered provider-frame authorization. `ash-engine` privately consumes that token, constructs authorized frames from its exact resolved provider objects, and drives checked CPS with a `RunControl`, using cancellation > deadline > provider completion. `ash-interp` remains a dependency for public CPS types and validation only; it receives neither a public token handoff nor provider authority. This first live slice is intentionally `time::sleep` only: source handlers, `resume`, `done`, generic operations, residual rows/open tails, and every unimplemented lowering remain rejected at admission.

**Tech Stack:** Rust 2024; Tokio 1.42 (already workspace-provided); `ash-engine` admission/registry/private production-driver owner; `ash-interp` CPS types and validation; `ash-cli` V1 terminal envelopes; existing TASK-1993 frame lookup.

---

## Implemented narrow slice (2026-07-26)

Tasks 1–4 are implemented for the sole admitted effectful form:
`fn main() -> Null { time::sleep(<non-negative Int literal>) }`. A real checked entry seals the
exact Core/CPS `Raise`, canonical source anchor, registry-resolved `time.sleep` provider object,
and one explicit Provider instruction. The private Engine driver constructs only that one frame;
it never derives frame authority from a row, V1 inspection artifact, caller input, or ambient
provider lookup.

The same post-admission `RunControl` drives the one-frame async operation. Focused paused-time
tests prove normal return, timeout/drop, cancellation priority over expiry/drop, control binding to
one admission, and issuer/anchor/binding rejection before frame construction. The CLI forwards
`--timeout` and SIGINT only after admission, owns the V1 return/timeout/cancelled projection and
output ownership, and rejects `--trace` for this no-telemetry route.

This is not completion of the plan's general wording: no generic source handler, `resume`/`done`,
residual/open row, multi-frame/TASK-1993 production dispatch, other operation, or pending
missing-admission/malformed-Core/handler-trap taxonomy is implemented.

## Non-negotiable boundary

- The deadline starts **after** a production token is sealed. Parsing, checking, typed lowering, provider registration/binding validation, and sealing are never charged to `--timeout`.
- The deadline is absolute and run-wide. A second reduction or provider call observes only remaining time; no per-operation reset is allowed.
- One control decision has deterministic precedence: `cancelled`, then `timeout`, then provider completion. Losing provider futures are dropped only; no kill, rollback, retry, or compensation is promised.
- Rows are descriptive. Only the ordered `FrameInstallationInstructionV1` data sealed into the opaque production token can install provider frames. Preserve `HandlerChain::find_operation_frame`’s reverse (innermost-first) scan.
- `ash-engine` privately owns the production-token handoff, frame chain, resolved `CapabilityProvider` objects, and async driver. Do not add an `ash-interp` public or `pub(crate)` handoff: `pub(crate)` is not visible across the crate boundary, a public constructor would leak authority, and making `ash-interp` depend on `ash-engine` would create a dependency cycle.
- The only initially admitted effectful source program is exact typed `fn main() -> Null { time::sleep(<non-negative Int literal>) }` with an exact registered `time.sleep` binding. Existing pure/constructor admissions remain valid. All handler-bearing terms (`Raise`/`Handle` other than this producer), source handlers, `resume`, `done`, residual operations, and open tails stay closed.
- No route may call `eval_expr`, `eval_expr_async`, or the legacy direct evaluator as a fallback. The existing synchronous `eval_checked_terminal` remains an inspection/handler-free evaluator; it must not acquire provider authority.
- Missing admission and malformed/unchecked-Core remain the pending taxonomy: do not misproject them as `timeout`, `cancelled`, or a driver trap in this task.
- The narrow admitted host-operation route is no-telemetry and does not support `--trace`. It may reject that flag combination; do not implement or claim a trace session, report, or telemetry for this route.

## Working rules

1. Use `@task-development-using-tdd`, `@rust-skills`, `@superpowers:test-driven-development`, and `@superpowers:systematic-debugging` for the implementation tasks.
2. Run one serial task at a time. Later tasks depend on the opaque token, driver outcome, and route API introduced by earlier tasks.
3. Make no commits from this shared worktree. The generic skill template’s commit step is intentionally replaced by a review/status step here.
4. Before editing, inspect `git status --short`; do not overwrite unrelated work already present in the shared worktree.

## Task 1: Seal the exact `time::sleep` production admission

**Purpose:** Turn the already typed/private `checked_cps_time_sleep_raise` producer into the first narrow, production-sealable artifact. This task does not execute an operation.

**Files:**

- Modify: `crates/ash-engine/src/checked_cps_admission.rs`
- Modify: `crates/ash-engine/src/lib.rs` — `Engine::lower_entry_to_checked_cps`, `Engine::admit_entry_to_checked_cps`, `checked_cps_time_sleep_raise`, and the `CheckedCpsEntryAdmission` boundary
- Modify: `crates/ash-engine/tests/task_2010_time_sleep_operation_source_call.rs`
- Create: `crates/ash-engine/tests/task_2014_production_time_sleep_admission.rs`
- Preserve: `crates/ash-engine/tests/task_2014_checked_cps_admission.rs`

### Step 1: Write the failing production-admission tests

Add `task_2014_production_time_sleep_admission.rs` with fixtures that:

```rust
const SLEEP: &str = "fn main() -> Null { time::sleep(0) }";

#[tokio::test]
async fn exact_typed_time_sleep_with_registered_binding_seals_a_production_token() {
    // install StandardProviderProfile, parse/check SLEEP, then assert the
    // opaque production token retains the exact anchor, checked CPS Raise and
    // one Provider(time::sleep) instruction backed by time.sleep.
}

#[tokio::test]
async fn no_binding_wrong_binding_row_only_or_foreign_engine_cannot_seal_time_sleep() {
    // Every case must reject before a token exists and before provider work.
}

#[tokio::test]
async fn handler_resume_done_open_tail_and_non_time_raises_remain_closed() {
    // Keep present admission-error assertions for each unsupported form.
}
```

Assert the token is opaque: callers can inspect diagnostics/anchor/instruction summary only, cannot provide a reconstructed `CheckedCpsAdmissionV1` to an execution API. Assert the provider binding is resolved through `Engine::declared_operation_provider_binding` plus `RuntimeState::get_provider`, not synthesized from the Core row or `TIME_SLEEP_OPERATION` spelling.

### Step 2: Run the RED test

Run:

```bash
cargo test -p ash-engine --test task_2014_production_time_sleep_admission
```

Expected: compilation failure because no production `time::sleep` admission API/token exists, or runtime failure at the current handler-free `checked_cps_term_has_handler_or_raise` guard. Do not loosen that guard merely to make the test compile.

### Step 3: Add an opaque production token and a narrow sealer

In `checked_cps_admission.rs`, add an engine-constructible, non-public-constructor carrier (for example `CheckedCpsProductionAdmission`) distinct from both `CheckedCpsEntryAdmission` and public `CheckedCpsAdmissionV1`. It must retain:

```rust
// conceptual fields; choose crate-private concrete types
issuer_token: Arc<()>,
entry_id: u64,
source_anchor: SourceAnchor,
checked_core: CheckedLoweredCoreProgram,
executable: CpsTerm,
provider_bindings: Vec<ResolvedProviderBinding>,
frame_installations: Vec<FrameInstallationInstructionV1>,
```

Use the exact `CheckedLoweredCoreProgram` and lowered `CpsTerm` from one checked source entry. Keep the executable and provider objects inaccessible outside the Engine-private driver path. Add a `validate_production_time_sleep(...)` crate-private constructor that accepts only one checked concrete `time::sleep` operation, one exact resolved `time.sleep` provider binding, the same entry anchor, and exactly one explicit `Provider` instruction. It must reject absent/unregistered/mismatched providers, altered anchors, another Engine’s issuer seal, source-handler instructions, nonempty residual facts, and open tails. Do not fabricate duplicate/conflicting instructions merely to test a frame-chain condition that this one-instruction admission cannot authorize.

Do **not** expose a generic `from_v1`, `from_row`, or `from_cps` conversion. `CheckedCpsAdmissionV1::validate` remains inspection evidence and cannot become executable by rewrapping it.

### Step 4: Route only the exact typed source producer into the sealer

In `lib.rs`, preserve `checked_cps_time_sleep_raise` as the one exact producer but obtain its concrete `DeclaredConcreteOperation` and the registry binding from the already checked `Entry`, not inferred from a textual call. Add a dedicated admission method (for example `Engine::admit_production_checked_cps`) that:

1. calls `check`, proves the exact supported producer shape, validates/lower-checks Core/CPS, and creates the terminal answer continuation;
2. resolves the Engine registry binding and `RuntimeState` provider object; and
3. seals the ordered Provider instruction and source anchor into the production token.

Keep `admit_entry_to_checked_cps` and `execute_checked_cps_admission` restricted to their current pure handler-free subset until Task 3 gives the caller an explicit production driver route.

### Step 5: Prove the sealer is green and does not widen legacy coverage

Run:

```bash
cargo test -p ash-engine --test task_2014_production_time_sleep_admission
cargo test -p ash-engine --test task_2010_time_sleep_operation_source_call
cargo test -p ash-engine --test task_2014_checked_cps_admission
cargo test -p ash-engine --test task_2014_positive_checked_cps_admission
```

Expected: all pass; existing test names/expectations are updated only where they accurately describe the newly admitted exact `time::sleep` slice. The row-only and handler/open-tail regressions still reject before a token and host operation.

### Step 6: Review shared-worktree state

Run `git diff --check` and `git status --short`. Do not commit.

## Task 2: Construct the one Engine-private token-authorized provider frame

**Purpose:** Give the async driver the one provider frame the current production token can
actually authorize, without changing the existing synchronous `HandlerChain` semantics or
inventing source-handler execution. This is not yet a multi-frame chain implementation.

**Files:**

- Create: `crates/ash-engine/src/production_cps_driver.rs` — private production-token handoff, authorized frames, and dispatch support
- Modify: `crates/ash-engine/src/checked_cps_admission.rs` — private token accessors consumed only by the Engine driver
- Modify: `crates/ash-engine/src/lib.rs` — declare the private module and dispatch only from an Engine-issued token
- Create: `crates/ash-engine/tests/task_2014_authorized_provider_frame_order.rs`
- Modify: `crates/ash-core/tests/task_1993_operation_frame_lookup.rs`
- Preserve: `crates/ash-interp/tests/task_1993_frame_ordered_dispatch.rs`

### Step 1: Write the failing one-frame tests

Add Engine integration tests whose provider fixture records the selected binding. Cover:

```rust
#[tokio::test]
async fn one_sealed_time_sleep_instruction_constructs_one_matching_provider_frame() { /* … */ }

#[tokio::test]
async fn row_only_foreign_engine_altered_anchor_and_wrong_binding_reject_before_frame_construction() { /* … */ }

#[tokio::test]
async fn no_source_handler_frame_is_constructed_by_the_time_sleep_slice() { /* … */ }
```

The tests must obtain a real Engine-issued production token from the narrow admission path; neither a test helper nor a public API may synthesize one from a `CpsEffectRow`. Assert exact `EffectOp` equality includes namespace/name/argument/result signature. Row-only, foreign-Engine, altered-anchor, and wrong-binding cases reject at admission before a token/frame exists. Keep the existing TASK-1993 tests as the authority for `HandlerChain` reverse traversal; do not fabricate multiple instructions from this one-instruction token.

### Step 2: Run the RED test

Run:

```bash
cargo test -p ash-engine --test task_2014_authorized_provider_frame_order
```

Expected: FAIL because the Engine has no private authorized-frame carrier/driver dispatch path.

### Step 3: Define a narrow Engine-private one-frame handoff

In `production_cps_driver.rs`, define non-public `AuthorizedProviderFrame` data that stores the exact CPS `EffectOp` and exact resolved `Arc<dyn CapabilityProvider>` held in the sealed token. Its construction path must be private to `ash-engine` and consume only the issuing Engine's validated one-instruction token. Do not introduce an `AuthorizedFrameChain` or synthetic instruction list: ordering is not observable in a one-frame slice.

Define private dispatch support with no ambient registry lookup. It invokes the exact sealed `CapabilityProvider::execute(provider_operation, engine_values)` on that resolved handle and fails closed if conversion, arity, or action identity does not match. Do not expose a public dispatcher trait merely to cross the crate boundary, and do not wire `HandlerFrame::Provider { handler: Name }`, direct source `invoke`, or `eval_expr_async` into this support.

### Step 4: Implement minimal provider-only operation selection

Implement the smallest private Engine-driver transition required by the `time::sleep` `Term::Raise`: evaluate atomic arguments, match the one exact `AuthorizedProviderFrame`, call the exact sealed provider, convert the `Null` result, and resume the sealed continuation. Source-handler clause execution remains unsupported in this driver and must return a closed admission/driver error if somehow present; it must not call synchronous `eval_raise`.

### Step 5: Prove one-frame authority without overstating ordering

Run:

```bash
cargo test -p ash-engine --test task_2014_authorized_provider_frame_order
cargo test -p ash-core --test task_1993_operation_frame_lookup
cargo test -p ash-engine --test task_2014_production_time_sleep_admission
```

Expected: the one sealed explicit instruction creates one exact provider frame; row-only and binding/issuer/anchor mismatches reject at admission before frame construction; all handler forms remain closed. Nested/multiple-frame ordering remains deferred, while the existing TASK-1993 tests retain the generic reverse-lookup invariant.

### Step 6: Review shared-worktree state

Run `git diff --check`; do not commit.

## Task 3: Add the Engine-private async checked-CPS driver and run-wide control races

**Purpose:** Implement execution-phase control in the driver, not around parsing/admission or as a CLI timer wrapper.

**Files:**

- Modify: `crates/ash-engine/src/production_cps_driver.rs`
- Modify: `crates/ash-engine/src/lib.rs` — `Engine::execute_production_checked_cps` (new, async) and control construction API
- Modify: `crates/ash-engine/Cargo.toml` only if a currently absent Tokio feature is genuinely needed (it should not be; workspace Tokio uses `full`)
- Create: `crates/ash-engine/tests/task_2014_run_wide_control.rs`
- Create: `crates/ash-engine/tests/task_2014_async_time_sleep_execution.rs`

### Step 1: Write the failing control-race tests

Use `#[tokio::test(start_paused = true)]` where practical. The tests must observe outcomes, not wall-clock sleeps:

```rust
#[tokio::test(start_paused = true)]
async fn one_absolute_deadline_is_not_restarted_for_the_second_provider_await() { /* … */ }

#[tokio::test]
async fn cancellation_before_reduction_prevents_provider_dispatch() { /* … */ }

#[tokio::test]
async fn cancellation_while_awaiting_drops_provider_future_and_prevents_later_reduction() { /* … */ }

#[tokio::test]
async fn cancellation_wins_when_cancellation_deadline_and_completion_are_ready_together() { /* … */ }

#[tokio::test]
async fn provider_completion_before_control_resumes_checked_cps_normally() { /* … */ }
```

The fake dispatcher must expose an atomic/drop guard proving that cancellation/timeout drops its future and that no subsequent continuation is evaluated. Add an Engine integration test for `time::sleep(0)` normal return and a pending sleep under an already-expired control. Assert control construction happens only after `admit_production_checked_cps` succeeds.

### Step 2: Run the RED tests

Run:

```bash
cargo test -p ash-engine --test task_2014_run_wide_control
cargo test -p ash-engine --test task_2014_async_time_sleep_execution
```

Expected: FAIL because the synchronous evaluator exposes no `RunControl`, async terminal outcome, or provider-await race.

### Step 3: Define private control and outcome types at the Engine-driver boundary

In `production_cps_driver.rs`, introduce a private cloneable `RunControl` with an optional absolute `tokio::time::Instant` deadline and explicit cancellation observation (a local watch/oneshot based carrier is sufficient; do not add `tokio-util` just for a token). Introduce a typed result that distinguishes:

```rust
enum AsyncCpsTerminalOutcome {
    Return(CpsValue),
    Trap(TrapReason),
    TimedOut,
    Cancelled,
}
```

Malformed CPS remains a validation error before the driver begins. Validate once, then check `RunControl` before every reduction and use `tokio::select! { biased; ... }` (or an equivalent explicit precheck) so cancellation wins over an expired deadline, which wins over provider completion when simultaneously ready.

### Step 4: Implement the minimal async CPS reduction loop

Implement only term shapes reachable from sealed pure/constructor/time-sleep artifacts: terminal continuation forms, `LetVal`, `LetPrim`, `LetCont`, `Jump`/`JumpValue`, `Return`, `Trap`, and Provider `Raise`. Reuse factored pure evaluation helpers only when that preserves a control check at each reduction; do not wrap whole synchronous evaluation in `spawn_blocking` or `tokio::timeout`.

On Provider `Raise`, race the exact authorized provider future against the same absolute `RunControl`. On control win, drop that future and return the corresponding typed outcome; on provider completion, resume only the sealed continuation. A driver receiving `Handle`, a source-handler authorization, a residual row, an open tail, or any unrecognized CPS term returns a closed error rather than interpreting it.

### Step 5: Wire Engine execution after successful sealing only

Add an async Engine execution method that accepts only its opaque production token and a `RunControl`. Verify the issuer seal, anchor, and frame handoff; pass no public V1 object or raw CPS term. Convert `Return`/`Trap` to the Engine’s internal execution result while preserving `TimedOut` and `Cancelled` as typed control results for the CLI. Do not create a control object from `Engine::parse`, `check`, or the general closed `execute` route.

### Step 6: Prove control behaviour is green

Run:

```bash
cargo test -p ash-engine --test task_2014_run_wide_control
cargo test -p ash-engine --test task_2014_async_time_sleep_execution
cargo test -p ash-engine --test task_2010_time_sleep_operation_source_call
cargo test -p ash-engine --test task_2014_production_time_sleep_admission
```

Expected: deadline spans all driver progress; cancellation wins deterministic races; no provider work after an early control outcome; normal `time::sleep(0)` returns `Null` via checked CPS.

### Step 7: Review shared-worktree state

Run `git diff --check`; do not commit.

## Task 4: Give CLI sole ownership of timeout/cancellation envelopes

**Purpose:** Replace outer CLI `tokio::timeout` execution wrappers for the admitted production route with a post-admission Engine control handoff. Preserve existing non-admitted failures and terminal JSON ownership.

**Files:**

- Modify: `crates/ash-cli/src/commands/run.rs` — `run`, `run_runnable_source`, `execute_with_trace`, `run_execution_with_cancellation`
- Modify: `crates/ash-cli/src/error.rs` only if a typed Engine-control handoff needs one error variant
- Modify: `crates/ash-cli/tests/task_2008_runtime_terminal_envelope.rs`
- Create: `crates/ash-cli/tests/task_2014_run_wide_control_envelope.rs`
- Modify: `crates/ash-engine/src/lib.rs` only for the already-designed CLI-facing production execution handoff

### Step 1: Write the failing CLI ownership tests

Add integration cases that run the binary with the exact admitted source and standard time profile:

```rust
#[test]
fn timeout_starts_after_admission_and_projects_one_v1_execution_timeout() { /* … */ }

#[test]
fn cancellation_during_admitted_sleep_projects_one_v1_execution_cancelled_and_exit_130() { /* … */ }

#[test]
fn unsupported_source_still_fails_at_admission_not_as_timeout_or_cancelled() { /* … */ }

#[test]
fn output_file_owns_exactly_one_control_envelope_and_stdout_is_empty() { /* … */ }
```

For deterministic cancellation, test the private command-level cancellation future with a controllable signal/channel; do not send a real process signal. Assert exactly:

```json
{"schema_version":1,"kind":"external","boundary":"execution","outcome":"timeout"}
```

or `"cancelled"`, no trace/runtime/provider telemetry, and one output sink only. Assert a malformed/unchecked or missing-admission case retains its existing pending classification rather than being relabeled.

### Step 2: Run the RED tests

Run:

```bash
cargo test -p ash-cli --test task_2014_run_wide_control_envelope
cargo test -p ash-cli --test task_2008_runtime_terminal_envelope
```

Expected: FAIL because current `run.rs` starts broad `tokio::time::timeout` wrappers before route-specific admission and performs one-shot cancellation outside the Engine driver.

### Step 3: Separate admission from execution control in `run.rs`

Refactor the admitted `time::sleep` route into:

1. parse/check/admit production token;
2. construct one `RunControl` from `--timeout` and the one-shot cancellation receiver; then
3. await only `Engine::execute_production_checked_cps(token, control)`.

Do not create a zero-second shortcut that emits an envelope before admission. A zero timeout becomes an already-expired control **after** admission, so unsupported/unadmitted source still produces its normal pre-entry/admission outcome.

Keep the current signal helper as a signal source only; it must no longer decide the final timeout/cancellation race outside the driver for the new route. Preserve legacy bounded bootstrap/pure behavior until it is separately migrated, but do not permit it to execute an effectful source fallback.

### Step 4: Project driver outcomes at the CLI boundary only

Map Engine `Return` and `Trap` through existing `entry_terminal_observable`/`CanonicalTerminalObservable`; map typed `TimedOut`/`Cancelled` to the existing V1 external execution envelopes and cancellation exit 130. `--output` remains the sole writer when specified. Do not emit driver telemetry or turn a lower-level `CpsRunError` into a new wire form.

### Step 5: Prove CLI routing is green

Run:

```bash
cargo test -p ash-cli --test task_2014_run_wide_control_envelope
cargo test -p ash-cli --test task_2008_runtime_terminal_envelope
cargo test -p ash-cli --test task_2008_terminal_observable_projection
cargo test -p ash-engine --test task_2014_async_time_sleep_execution
```

Expected: the CLI starts control only after admission; all supported driver control outcomes emit exactly one V1 envelope at the CLI; unsupported source remains closed without a direct evaluator fallback.

### Step 6: Review shared-worktree state

Run `git diff --check`; do not commit.

## Task 5: Cross-route regression evidence, traceability, and QA

**Purpose:** Prove the narrow production slice did not widen handler/source admission or break the strict cutover, then document only facts actually implemented.

**Files:**

- Modify: `crates/ash-engine/tests/task_2004_core_cps_production_boundary.rs`
- Modify: `crates/ash-engine/tests/task_2014_checked_cps_admission.rs`
- Modify: `crates/ash-engine/tests/task_2005_time_sleep_provider_parity.rs`
- Modify: `crates/ash-cli/tests/task_2008_runtime_terminal_envelope.rs`
- Modify: `docs/plan/tasks/TASK-2014-source-handler-runtime-boundary-decision.md`
- Modify: `docs/plan/tasks/TASK-2004-core-cps-production-boundary-decision.md`
- Modify: `docs/plan/tasks/TASK-2013-source-handler-and-handle-lowering.md`
- Modify: `docs/plan/tasks/TASK-2005-direct-runtime-core-cps-semantic-parity.md`
- Modify: `docs/plan/tasks/TASK-2008-json-variant-observable-projection.md`
- Modify: `docs/plan/tasks/TASK-439-differential-conformance-harness-rust-first.md`
- Modify: `docs/plan/PLAN-INDEX.md`
- Modify: `docs/spec/SEMANTIC-TRACEABILITY.json`
- Modify: `CHANGELOG.md`
- Conditionally modify: `tests/differential/corpus/*` only if the already-approved corpus schema can represent the exact production `time::sleep` control outcome without a schema change. Otherwise record it as deferred; do not invent the TASK-439 input schema.

### Step 1: Write cross-route RED regressions

Add/extend tests for:

- `Engine::run`, `run_file`, and CLI non-trace execute only the exact sealed `time::sleep` slice through the async checked-CPS driver. `--trace` is unsupported for this no-telemetry route and may reject rather than invoking a traced execution path.
- `Engine::execute`, `execute_with_input`, application admission, handler inspection, source handlers, resume/done, arbitrary declared operations, residual rows, and open tails remain closed and never contact a provider.
- malformed/raw Core cannot enter the production driver; no timeout/cancellation envelope is emitted merely because driver input is invalid.
- TASK-2005’s old private direct-runtime/CPS comparison is not mislabeled as production evidence. Add a separate exact production observation only if its corpus schema already supports it.

### Step 2: Run RED coverage

Run:

```bash
cargo test -p ash-engine --test task_2004_core_cps_production_boundary
cargo test -p ash-engine --test task_2014_checked_cps_admission
cargo test -p ash-engine --test task_2005_time_sleep_provider_parity
cargo test -p ash-cli --test task_2008_runtime_terminal_envelope
```

Expected: at least the new route/negative assertions fail before the Task 1–4 implementation is present; after implementation, every unimplemented source form retains closed-admission evidence.

### Step 3: Update documentation and traceability truthfully

Record the implemented scope exactly: one source-to-checked-Core/CPS `time::sleep` producer, exact registry binding, ordered authorized Provider frame, run-wide control priority/drop semantics, and CLI V1 timeout/cancellation projections. Keep completion boxes unchecked for general source lowering, handlers/resume/done, residual rows/open tails, all-source route ownership, and the missing-admission/malformed-Core envelope taxonomy.

Add trace links for tests and code symbols only after their tests pass. Do not claim generic frame installation merely because the provider-only carrier preserves ordering. Add Common Changelog entries under `[Unreleased]` with `(TASK-2014)` and related task IDs as appropriate.

### Step 4: Run focused and broad verification serially

Run, serially to avoid Cargo lock contention in the shared worktree:

```bash
cargo test -p ash-engine --test task_2014_production_time_sleep_admission
cargo test -p ash-engine --test task_2014_async_time_sleep_execution
cargo test -p ash-engine --test task_2014_authorized_provider_frame_order
cargo test -p ash-engine --test task_2014_run_wide_control
cargo test -p ash-cli --test task_2014_run_wide_control_envelope
cargo test -p ash-engine --tests
cargo test -p ash-interp --tests
cargo test -p ash-cli --tests
cargo clippy -p ash-engine -p ash-interp -p ash-cli --all-targets --all-features -- -D warnings
cargo fmt --check
python3 tools/docs/validate_orientation_indexes.py --self-test
bash scripts/check-docs-gate.sh
python3 tools/docs/validate_semantic_traceability.py
git diff --check
```

Expected: all commands exit 0. If a broad legacy expectation fails because it assumed direct source evaluation, migrate only the assertion to the selected closed/admitted boundary with the test’s original parser/typechecker evidence retained.

### Step 5: Independent reviews and handoff

Use fresh subagents for a specification review (especially token authority, no row authority, and remaining closed forms) and a code-quality review (Tokio cancellation/drop behaviour, exact identity checks, and output ownership). Address blocking findings, rerun affected tests, report the changed files and command evidence, and leave the shared worktree uncommitted.

## Completion evidence matrix

| Requirement | Primary proof |
| --- | --- |
| Exact typed `time::sleep` source producer | `task_2014_production_time_sleep_admission` plus TASK-2010 regression |
| Sealed concrete provider/anchor/frame authority | Task 1 negative token/binding/anchor/row tests |
| One exact authorized `time::sleep` provider frame | Task 2 one-frame test plus Task 1 admission rejections |
| Ordered, innermost-first production provider choice | Deferred until widened real production admission can seal multiple instructions; TASK-1993 remains the generic lookup authority |
| One absolute execution-only deadline | paused-time two-await control test |
| Cancellation priority and dropped await | controlled cancellation/drop-guard test |
| No direct evaluator fallback | TASK-2004 route regressions and provider-evidence-empty negatives |
| CLI owns V1 timeout/cancelled output | Task 4 binary/stdout/`--output` tests |
| No accidental handler/resume/open-tail admission | Task 1/5 negative admissions and TASK-2013 regression |
| Accurate plans/traces/changelog | docs gates and semantic traceability validator |

## Explicitly deferred after this plan

- General source-to-Core/CPS lowering, generic declared-operation dispatch, and all imported operation execution.
- Any source handler execution, continuation/resume semantics beyond current inspection controls, `done`, residual-row realization, aliases/groups/open-tail execution, or source-handler frame installation.
- Per-provider budgets, retries, host kill, rollback, compensation, telemetry, and a public cancellation API.
- Canonical terminal taxonomy for missing admission and malformed/unchecked Core.
- TASK-439 canonical corpus input-schema migration unless separately approved.
