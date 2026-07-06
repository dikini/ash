# PLAN-195: Process And Concurrency Model

**Status:** ✅ Complete; all Phase 195 tasks finished
**Audit:** [AUDIT-195: Process Runtime Seams](audits/AUDIT-195-process-runtime-seams.md)
**Depends on:** Phase 182 Core Computation Model Conformance, Phase 183 Operation And Authority
Model, Phase 184 Handler / Provider Semantics, and Phase 194 Contract And Evidence System.
**Specs/notes:** `SPEC-096b`, `SPEC-097b`, `SPEC-098b`, `SPEC-098c`, `SPEC-099b`,
`SPEC-100`, `PLAN-182`, `PLAN-183`, `PLAN-184`, `PLAN-194`, `NOTE-020`, `NOTE-021`, and
`NOTE-035`.

## Goal

Add structured execution beyond single computations: spawn/join/await, channels, cancellation,
failure propagation, and sendability/ownership checks across process boundaries.

## Architecture

Process execution is a runtime profile over ambient row-bearing Ash computations, not a separate
semantic foundation and not a revival of the deprecated `Act`, `Proc`, or `Workflow` tower forms.
Those names may remain only as legacy documentation references when explaining historical designs;
new development must not add surface syntax, Core terms, IR nodes, public stdlib types, or runtime
entry paths named `Act`, `Proc`, or `Workflow`. Process behavior is represented as ordinary
computation-row facts, Core/CPS process-event carriers, runtime frames, and trace/evidence events.
Authority remains governed by operation/resource/role/policy discharge and handler/provider frames;
process rows are requirements and lifecycle facts, never authority grants.

## Scope

- Reconcile target docs so deprecated `Act`/`Proc`/`Workflow` forms are historical references only.
- Add row/Core/CPS carriers for process lifecycle, channel, cancellation, and transfer facts.
- Validate sendability and ownership before values cross process or channel boundaries.
- Implement bounded spawn, join, await, channel send/receive/close, cancellation, and failure
  propagation.
- Emit process/channel trace facts and runtime monitor evidence for later temporal contracts.
- Prove process execution does not bypass operation authority, handler/provider discharge, or
  contract/evidence checks.

## Non-Goals

- No separate workflow semantic foundation.
- No new `Act`, `Proc`, or `Workflow` surface, Core, IR, stdlib, or runtime form.
- No distributed actor runtime or remote process transport.
- No scheduler fairness proof or preemptive runtime overhaul.
- No generalized temporal-logic source syntax beyond trace/evidence carriers.
- No authority granted by mentioning process rows.

## Design Locks

1. The old `Act`, `Proc`, and `Workflow` forms are deprecated for development and exist only as
   legacy reference vocabulary in historical docs.
2. Process runtime-profile facts classify runtime organization and lifecycle behavior for ambient
   computations without introducing `Proc` as a denotable type or IR form.
3. Process rows are requirement/lifecycle facts and must be admitted or checked fail-closed.
4. Spawned computations inherit only explicit runtime context allowed by admission and handler
   rules; they do not receive ambient authority by default.
5. Values crossing process or channel boundaries must be sendable or moved with a proven ownership
   transfer.
6. Borrowed resources, non-sendable closures, unstable observers, and live handler frames cannot
   cross process boundaries without explicit supported carriers.
7. Cancellation is observable and structured; it is not silently mapped to ordinary failure.
8. Child failure, join failure, cancellation, and contract traps remain distinguishable in
   diagnostics and trace facts.
9. Runtime monitor evidence records process and channel facts without acquiring process or
   operation authority.

## Task Overview

| Task | Description | Estimate | Depends on | Status |
|------|-------------|----------|------------|--------|
| [TASK-1902](tasks/TASK-1902-process-concurrency-plan-packet.md) | Create the Phase 195 plan and task packet | 2h | Phase 194 | ✅ Complete |
| [TASK-1903](tasks/TASK-1903-process-runtime-seam-audit.md) | Audit computation, handler/provider, row admission, runtime, and trace seams for process execution | 4h | TASK-1902 | ✅ Complete |
| [TASK-1904](tasks/TASK-1904-deprecated-tower-vocabulary-spec-reconciliation.md) | Reconcile deprecated `Act`/`Proc`/`Workflow` vocabulary in target specs and notes | 6h | TASK-1903 | ✅ Complete |
| [TASK-1905](tasks/TASK-1905-process-row-and-core-carriers.md) | Add process row facts and Core/CPS carriers | 10h | TASK-1904 | ✅ Complete |
| [TASK-1906](tasks/TASK-1906-sendability-ownership-validation.md) | Validate sendability and ownership transfer across process boundaries | 12h | TASK-1905 | ✅ Complete |
| [TASK-1907](tasks/TASK-1907-spawn-join-await-runtime-semantics.md) | Implement bounded spawn, join, and await runtime semantics | 12h | TASK-1906 | ✅ Complete |
| [TASK-1908](tasks/TASK-1908-channel-runtime-semantics.md) | Implement bounded typed channel runtime semantics | 12h | TASK-1906 | ✅ Complete |
| [TASK-1909](tasks/TASK-1909-cancellation-and-failure-propagation.md) | Model cancellation and failure propagation diagnostics | 10h | TASK-1907, TASK-1908 | ✅ Complete |
| [TASK-1910](tasks/TASK-1910-process-trace-and-monitor-evidence.md) | Emit process/channel/cancellation trace facts and monitor evidence | 8h | TASK-1909 | ✅ Complete |
| [TASK-1911](tasks/TASK-1911-process-concurrency-cross-boundary-fixtures.md) | Add parser/typecheck/Core/CPS/runtime/CLI fixtures | 8h | TASK-1905 through TASK-1910 | ✅ Complete |
| [TASK-1912](tasks/TASK-1912-process-concurrency-closeout.md) | Close out Phase 195 with docs, changelog, gates, and review remediation | 6h | TASK-1903 through TASK-1911 | ✅ Complete |

Estimated implementation effort after the plan packet: 88 hours.

## Required Test Families

### Process Profile Tests

Prove that process runtime-profile facts are preserved over ordinary computations and do not
introduce `Act`, `Proc`, or `Workflow` surface forms, Core terms, IR nodes, public stdlib types, or
workflow lowering paths.

### Sendability And Ownership Tests

Reject non-sendable closures, borrowed resources, live handler frames, and unstable observer state
across spawn/channel boundaries. Accept owned sendable values and explicit move-only transfers.

### Runtime Semantics Tests

Cover spawn, join, await, channel send/receive/close, cancellation, child failure, join failure, and
handler/provider interaction.

### Trace And Monitor Evidence Tests

Assert that process/channel/cancellation events produce stable trace facts and runtime monitor
evidence records without granting authority or discharging unrelated rows.

## Verification Gates

Each implementation task must run focused tests for touched crates plus:

```bash
cargo fmt --check
cargo clippy --all-targets --all-features
```

Phase closeout must run:

```bash
cargo fmt --check
cargo test --all
cargo clippy --all-targets --all-features
python3 tools/docs/validate_orientation_indexes.py --self-test
bash scripts/check-docs-gate.sh
git diff --check
```

## Stale-Claim Sweep

Before closeout, search live normative docs and code comments for stale claims:

```text
Proc.*separate semantic foundation
Act.*new surface
Proc.*new surface
Workflow.*new surface
Act.*Core
Proc.*Core
Workflow.*Core
process row grants authority
spawn.*inherits all authority
channel send.*no ownership check
workflow runtime required for process
cancellation.*ordinary failure
child failure.*contract violation
monitor.*acquires authority
```

Changelog/history may mention old wording as historical context. Live guidance must route through
ambient computations, process runtime-profile facts, authority-neutral row requirements, explicit
sendability/ownership validation, and structured trace evidence. `Act`, `Proc`, and `Workflow` may
appear only when clearly marked as deprecated or historical.

## Acceptance Criteria

- [x] Phase 195 plan and task files exist and are indexed.
- [x] `Act`, `Proc`, and `Workflow` are documented as deprecated development forms and legacy
      reference vocabulary only.
- [x] No Phase 195 task introduces `Act`, `Proc`, or `Workflow` surface syntax, Core terms, IR
      nodes, public stdlib types, or runtime entry paths.
- [x] Process, channel, cancellation, and transfer facts have row/Core/CPS carriers.
- [x] Sendability and ownership transfer checks fail closed across process boundaries.
- [x] Spawn/join/await semantics preserve handler/provider and contract boundaries.
- [x] Channel send/receive/close semantics preserve type and ownership boundaries.
- [x] Cancellation and failure propagation have distinct diagnostics and trace facts.
- [x] Runtime monitor evidence records process behavior without granting authority.
- [x] Cross-boundary fixtures cover parser, typecheck, Core/CPS, runtime, and CLI.
- [x] Closeout verification gates pass.
