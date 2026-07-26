# TASK-2014 Run-Wide Cooperative Control Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Make every successfully admitted production Core/CPS run observe one run-wide deadline and cancellation signal while preserving closed admission, authorized-frame-only dispatch, and CLI-owned V1 terminal projections.

**Architecture:** Admission finishes before execution control begins. A sealed production token carries the exact checked Core/CPS artifact, source anchor, registry-resolved provider binding, and ordered, separately authorized frame-installation instructions. `ash-engine` privately owns the token handoff, authorized-frame construction, and async checked-CPS production driver; it observes control before every reduction and races provider awaits against it. `ash-interp` remains the public CPS type/validation dependency, not an authority-bearing execution boundary. No row creates a frame and no direct evaluator is available as a fallback.

**Tech Stack:** Rust 2024, Tokio, `ash-engine` admission/registry/private production driver, `ash-interp` CPS types and validation, `ash-cli`, checked-CPS admission artifacts.

---

## Approved Decision

The user approved this design on 2026-07-26.

1. `ash run --timeout N` applies one deadline to the **entire execution phase** of an admitted run. Parsing, checking, and admission occur before that phase. The deadline is not restarted for a CPS reduction or a provider call.
2. The Engine creates a single cooperative run-control envelope containing an optional deadline and a cancellation signal, and passes it to the async checked-CPS driver.
3. The driver observes the envelope before every CPS reduction and races every provider await against it. If more than one outcome is observable at one decision point, priority is: cancellation, then expired deadline, then normal provider completion.
4. Cancellation is cooperative and drop-only: cancellation stops further driver progress and drops an in-flight future. It does not promise host rollback, a provider kill, or compensation.
5. The executable production token must seal the exact checked Core/CPS, source anchor, actual registry-resolved provider binding, and ordered separately authorized frame-installation instructions. It is the only source allowed to install frames. Rows and residual rows remain descriptive and never install frames.
6. Frame construction preserves TASK-1993’s innermost-first handler/provider lookup. The new control envelope does not alter operation identity or lookup order.
7. The CLI, rather than the lower-level driver, projects outcomes to the existing versioned V1 terminal forms: return, trap, and `external/execution/timeout` or `external/execution/cancelled`. This design adds no telemetry contract.
8. The existing taxonomy for missing admission and malformed/unchecked Core remains explicitly pending. This plan does not silently classify either outcome as a timeout, cancellation, or execution trap.
9. The sealed provider-frame handoff and async production driver are private `ash-engine` implementation details. They consume only an Engine-issued production token and its exact resolved provider objects. `ash-interp` does not construct or receive a public authority-bearing handoff: placing one there would either make the token reconstructible, leak provider authority through a public API, or require an invalid `ash-interp` → `ash-engine` dependency cycle.
10. The narrow production route is no-telemetry and does not support `--trace`. It may reject that flag combination; it must not promise a trace session, report, or telemetry while projecting the V1 terminal envelope.

## Why This Scope

`--timeout` is a user-visible maximum execution duration, not a per-operation retry budget. A run-wide deadline makes a finite maximum meaningful across a sequence of reductions and awaits. A single control envelope also gives cancellation and timeout one deterministic race rule at every host boundary.

The boundary remains strict: a source form without validated typed lowering is rejected at admission, before a run-control object is created and before host activity begins. The design therefore does not reopen the removed direct-evaluator path.

## Rejected Alternatives

- **Reset a timeout for every provider operation.** Rejected because a source program can execute indefinitely through many individually timely operations; it changes `--timeout` from a run limit into an implicit per-operation policy.
- **Hierarchical run and provider budgets now.** Deferred. A later extension may add an explicitly specified provider-local budget beneath the run deadline, but it requires an approved provider-policy surface and an outcome-precedence rule beyond this task.
- **Use rows to select or install a provider/handler frame.** Rejected by the selected admission model and SPEC-099b: rows describe requirements only.
- **Fall back to direct evaluation if typed lowering, token sealing, or async dispatch is unavailable.** Rejected by TASK-2014 Path B closed admission.

## Preconditions and Current Gaps

This plan is a design and implementation sequence. The exact one-frame `time::sleep` slice now
has a production-sealed token, resolved provider binding, private async driver, run-wide control,
and CLI return/timeout/cancellation projection. The following broader capabilities still do not
exist:

- production source-to-Core/CPS provenance for general admitted forms;
- a production-sealed token for more than the one exact `time::sleep` provider binding and one
  executable frame instruction;
- generic handler lowering, continuation/resume, `done`, and residual-row realization;
- an async checked-CPS driver for providers other than the sealed `time::sleep` slice;
- a complete canonical terminal taxonomy for missing admission and malformed/unchecked Core; or
- full route and differential conformance coverage.

The bounded handler-free and closed-empty inspection tokens documented by TASK-2014 are controls only. They must not be widened into production authority merely by adding timeout plumbing.

## Required Data Flow

```text
source
  -> parse/check/typed lowering
  -> validate + seal production admission token
       { exact checked Core/CPS, source anchor,
         registry-resolved provider bindings,
         ordered authorized frame instructions }
  -> create one execution-only RunControl { deadline?, cancellation }
  -> `ash-engine` private async checked-CPS driver
       -> control check before each reduction
       -> construct only token-authorized frames
       -> TASK-1993 innermost-first lookup
       -> race provider await with RunControl
  -> terminal outcome
  -> CLI V1 terminal projection
```

If parsing, checking, typed lowering, validation, or sealing fails, the flow stops at admission. It must not construct a frame, contact a provider, begin an async run, or invoke the direct evaluator.

## Implementation Tasks

### Task 1: Define the execution-only control contract

**Files:**
- Modify: `crates/ash-engine/src/lib.rs`
- Modify or create: `crates/ash-engine/src/checked_cps_admission.rs`
- Test: `crates/ash-engine/tests/task_2014_*`

1. Add RED tests proving that a deadline is created only after successful admission and is shared by all reductions/awaits in a single run.
2. Define an Engine-private or opaque control carrier with an optional absolute deadline and a cancellation observation mechanism. Do not put control in row metadata or provider identity.
3. Specify a stable internal terminal classification that preserves the three-way priority: cancellation, expired deadline, normal completion.
4. Run the focused Engine tests and prove they fail before the driver uses the carrier.

### Task 2: Seal executable production admission authority

**Files:**
- Modify: `crates/ash-engine/src/checked_cps_admission.rs`
- Modify: `crates/ash-engine/src/lib.rs`
- Test: `crates/ash-engine/tests/task_2014_checked_cps_admission.rs`
- Test: `crates/ash-engine/tests/task_2014_*`

1. Add RED tests that a production token cannot be reconstructed from public V1 inspection data, a foreign Engine, an altered anchor, an unmatched provider binding, or a row alone.
2. Extend the production-only sealing path—not the generic inspection carrier—to retain exact checked Core/CPS, anchor, registry-resolved provider binding, and ordered authorized frame instructions.
3. Validate that instructions are concrete and correspond to checked operation/handler/provider facts before sealing. Preserve their order exactly.
4. Add tests that token construction rejects missing/unchecked/malformed Core and never calls a provider or installs a frame on rejection.

### Task 3: Implement authorized ordered frame construction

The first implementation slice is deliberately narrower than this end-state task: the current
production admission admits exactly one checked `time::sleep` Provider instruction. It may
construct and dispatch that one exact frame only. It must not fabricate extra instructions to
test chain order, stale/duplicate/conflicting instruction rejection, or nested lookup. Those are
future RED cases after a real production admission can seal multiple ordered instructions.

**Files:**
- Modify: `crates/ash-engine/src/lib.rs`
- Create or modify: `crates/ash-engine/src/production_cps_driver.rs`
- Test: `crates/ash-core/tests/task_1993_operation_frame_lookup.rs`
- Test: `crates/ash-engine/tests/task_2014_*`

1. Add RED integration tests that the one sealed `time::sleep` Provider instruction constructs one exact Engine-private provider frame. Keep row-only, foreign-Engine, altered-anchor, and mismatched-binding rejection in the admission tests, where those facts are real.
2. Build that runtime frame privately in `ash-engine`, exclusively from the sealed instruction and exact registry-resolved provider object; never derive it from a row/residual row or expose a public handoff constructor.
3. Preserve exact operation identity across sealing, construction, and dispatch.
4. Defer nested/multiple-frame chain construction, stale/duplicate/conflicting-instruction controls, and production TASK-1993 lookup evidence until a widened, real production admission supports ordered multiple instructions.

### Task 4: Add the async checked-CPS host-operation driver

**Files:**
- Modify: `crates/ash-engine/src/production_cps_driver.rs`
- Modify: `crates/ash-engine/src/lib.rs`
- Test: `crates/ash-engine/tests/task_2014_*`

1. Write RED tests for control observation before an ordinary reduction, while awaiting a provider, and after a provider completes.
2. Add an Engine-private async driver that consumes only an executable sealed token plus RunControl. It may use `ash-interp` public CPS types and validation, but it must not export an authority-bearing handoff. Keep the existing synchronous bounded evaluator restricted to its documented controls.
3. At each control decision point, implement cancellation > expired deadline > provider completion. Use a single absolute deadline so elapsed time is not reset by subsequent operations.
4. Drop an in-flight provider future when cancellation or expiry wins; document the no-rollback/no-kill guarantee and test that no later CPS reduction occurs.
5. Add handler-body trap tests proving a trap returns through the driver rather than a legacy evaluator or a timeout/cancellation projection.

### Task 5: Route production execution and CLI projection

**Files:**
- Modify: `crates/ash-engine/src/lib.rs`
- Modify: `crates/ash-cli/src/commands/run.rs`
- Modify: `crates/ash-cli/src/error.rs` only if the existing V1 projection needs a typed handoff
- Test: `crates/ash-cli/tests/task_2008_runtime_terminal_envelope.rs`
- Test: `crates/ash-cli/tests/task_2008_terminal_observable_projection.rs`
- Test: `crates/ash-engine/tests/task_2004_*` and `task_2014_*`

1. Add RED route tests showing all admitted production routes call the async checked-CPS driver and no route falls back to direct evaluation.
2. Make `--timeout` construct one execution-only control only after admission. Preserve existing command parsing and configuration errors as pre-entry concerns.
3. Project return/trap/timeout/cancelled exclusively at the CLI boundary through the existing V1 envelope; keep stdout and `--output` ownership exclusive.
4. Keep missing admission and malformed/unchecked Core unclassified by this change until the pending canonical taxonomy is selected and implemented.

### Task 6: Verify conformance evidence and documentation

**Files:**
- Modify: `docs/plan/tasks/TASK-2014-source-handler-runtime-boundary-decision.md`
- Modify: `docs/plan/tasks/TASK-2004-core-cps-production-boundary-decision.md`
- Modify: `docs/plan/tasks/TASK-2013-source-handler-and-handle-lowering.md`
- Modify: `docs/plan/tasks/TASK-2005-direct-runtime-core-cps-semantic-parity.md`
- Modify: `docs/plan/tasks/TASK-2008-json-variant-observable-projection.md`
- Modify: `docs/plan/tasks/TASK-439-differential-conformance-harness-rust-first.md`
- Modify: `docs/plan/PLAN-INDEX.md`, `docs/spec/SEMANTIC-TRACEABILITY.json`, and `CHANGELOG.md`

1. Add a differential case for a permitted provider operation whose deadline/cancellation outcome is compared at the canonical terminal boundary, once the input schema is approved.
2. Record only implemented facts in traceability; leave unimplemented source-to-Core, generic handler, and pending terminal-taxonomy requirements deferred.
3. Update the relevant task completion checklists and changelog in the same implementation change.
4. Run the documentation/trace gates plus focused Rust, CLI, and Engine tests.

## Test Matrix

| Case | Required evidence | Expected result |
|---|---|---|
| Typed lowering absent | Every production route | Admission rejects before RunControl, frame installation, provider dispatch, or direct evaluation |
| Deadline before first reduction | Async driver | `timeout` terminal classification; no reduction executes |
| Deadline across two operations | Async driver | One absolute deadline expires across the combined run; second operation receives only remaining time |
| Cancellation before/while provider await | Async driver | `cancelled` wins, await future is dropped, no subsequent reduction |
| Cancellation and expiry simultaneously observable | Async driver | `cancelled` wins deterministically |
| Provider completes before control | Async driver | Normal completion proceeds to next checked reduction |
| One admitted `time::sleep` provider frame | Engine-private driver | One sealed instruction installs one exact frame; rows never install one |
| Nested handler/provider frames | Future widened production admission + Engine-private driver | Only sealed ordered instructions install frames; lookup is innermost-first |
| Row-only provider requirement | Admission + runtime | Reject or no-frame outcome; never implicit provider installation |
| Handler-body failure | Driver + CLI | Canonical trap projection, not direct evaluator fallback or control outcome |
| CLI timeout/cancellation | `ash run --format json` | Existing V1 `external/execution/timeout|cancelled` projection, no telemetry |
| Missing admission / malformed Core | Boundary tests | Explicitly remain pending taxonomy; must not be mislabeled as timeout/cancelled |

## Out of Scope

- Per-provider or hierarchical time budgets, retries, and provider-specific cancellation policy.
- Killing host processes, rolling back provider side effects, or compensation semantics.
- A telemetry schema or execution-progress stream.
- Reopening legacy direct evaluation for any source form.
- Treating rows as frame authority.
- Adding trace/telemetry support to the narrow admitted host-operation route.
- Completing generic handlers, arbitrary source-to-Core lowering, continuation/resume semantics, `done`, residual-row realization, or the pending missing-admission/malformed-Core terminal taxonomy.

## Trace and Specification References

- [TASK-2014](../plan/tasks/TASK-2014-source-handler-runtime-boundary-decision.md) — Path B, admission artifact, and delivery sequence.
- [TASK-2004](../plan/tasks/TASK-2004-core-cps-production-boundary-decision.md) — strict production-boundary migration.
- [TASK-2013](../plan/tasks/TASK-2013-source-handler-and-handle-lowering.md) — typed handler facts and source lowering prerequisite.
- [TASK-1993](../plan/tasks/TASK-1993-verus-frame-ordered-dispatch-pilot.md) — innermost-first handler/provider lookup.
- [TASK-2008](../plan/tasks/TASK-2008-json-variant-observable-projection.md) — existing versioned terminal envelope.
- [TASK-2005](../plan/tasks/TASK-2005-direct-runtime-core-cps-semantic-parity.md) and [TASK-439](../plan/tasks/TASK-439-differential-conformance-harness-rust-first.md) — parity and corpus evidence boundaries.
- [SPEC-099b](../spec/SPEC-099b-TARGET-OPERATIONAL-SEMANTICS.md#handler-and-provider-frames) — rows do not install frames and lookup is innermost-first.
- [SEMANTIC-TRACEABILITY](../spec/SEMANTIC-TRACEABILITY.json) — `REQ-CORE-CPS-PRODUCTION-CUTOVER-001`, `REQ-SOURCE-HANDLER-ADMISSION-001`, `REQ-SOURCE-HANDLER-FRAME-ADMISSION-001`, and `REQ-SOURCE-HANDLER-TERMINAL-ENVELOPE-001` remain deferred until implementation evidence exists.

## Verification Commands

```bash
cargo test -p ash-engine --test task_2014_checked_cps_admission
cargo test -p ash-engine --test task_2014_positive_checked_cps_admission
cargo test -p ash-cli --test task_2008_runtime_terminal_envelope
cargo test -p ash-cli --test task_2008_terminal_observable_projection
cargo clippy -p ash-engine -p ash-cli --all-targets --all-features -- -D warnings
cargo fmt --check
python3 tools/docs/validate_orientation_indexes.py --self-test
bash scripts/check-docs-gate.sh
```
