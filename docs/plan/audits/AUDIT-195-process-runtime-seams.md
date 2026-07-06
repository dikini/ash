# AUDIT-195: Process Runtime Seams

**Task:** [TASK-1903](../tasks/TASK-1903-process-runtime-seam-audit.md)
**Phase:** [PLAN-195](../PLAN-195-PROCESS-AND-CONCURRENCY-MODEL.md)
**Date:** 2026-07-06

## Purpose

Map the live seams that Phase 195 must use or harden before adding structured process execution.
The audit treats `Act`, `Proc`, and `Workflow` as deprecated development forms and historical
reference vocabulary only. New process work must route through ambient computations, process row
facts, Core/CPS carriers, runtime frames, and trace/evidence records.

## Current Carriers

| Seam | Current state | Phase 195 owner |
|------|---------------|-----------------|
| Surface row syntax | `ash_parser::surface::ComputationRowItem::{Channel, Process}` preserves `channel ...` and `proc`/`process ...` row items. | TASK-1905 must keep these as row facts, not tower forms. |
| Source row to Core | `ash-engine` lowers surface channel/process row items into `CoreRowItem::{Channel, Process}`. | TASK-1905 must add missing operation vocabulary and preservation tests. |
| Core row model | `CoreRowItem::Channel { path, mode, payload_type }` and `CoreRowItem::Process { operation }` already exist. | TASK-1905 should harden process operation identity and unsupported-tail diagnostics. |
| Core raised operations | `CoreEffectOp::{Channel, Process}` exists for raised operation kinds. | TASK-1907/TASK-1908 own runtime semantics for supported operations. |
| Core to CPS lowering | `core_ash_lower` maps channel/process row items to `EffectItemKind::{Channel, Process}`. | TASK-1905 owns row preservation tests; TASK-1911 owns cross-boundary fixtures. |
| CPS rows | `EffectItemKind::{Channel, Process}` exists. | TASK-1905 should verify no new `Proc` IR node is introduced. |
| Row admission | `RowAdmissionRequirement::Process` exists but currently checks as unsupported; channel rows also fail closed as unsupported. | TASK-1905/TASK-1908 own fail-closed behavior until concrete discharge is implemented. |
| Runtime process state | Legacy `ProcessId`, `ProcessHandle`, `ProcessTerminalState`, process children, and `proc::await`/`proc::join` helpers exist in `ash-interp`/`ash-core`. | TASK-1907/TASK-1909 must reuse or replace these without adding public `Proc` types. |
| Runtime channels | Existing mailbox/send/receive behavior is workflow-era and not yet typed-channel endpoint ownership. | TASK-1908 owns bounded channel runtime semantics. |
| Contract/evidence | Phase 194 added contract discharge records and runtime monitor evidence carriers. | TASK-1910 owns process/channel trace evidence integration. |

## Authority And Handler Risks

- Process rows currently fail closed in admission, which prevents accidental authority grants but
  also means runtime process operations cannot be admitted as supported Phase 195 behavior yet.
- Spawn must not copy provider/handler authority by ambient runtime state alone. TASK-1907 must
  define exactly which admitted frames and discharges are inherited by a child process.
- Channel send/receive must not bypass operation/resource/role/policy discharge. TASK-1908 must
  route endpoint ownership and message movement through explicit runtime facts.
- Runtime monitor evidence must remain observation metadata. TASK-1910 must prove monitor evidence
  does not discharge process, channel, operation, or policy rows by mention.

## Ownership And Sendability Risks

- Existing `Value::ProcessHandle` and `ProcessHandle` support affine consumption paths, but there is
  no general sendability classifier for arbitrary Ash values.
- Closure values, live handler frames, borrowed resources, unstable observer state, and runtime
  provider handles are not safe to cross process/channel boundaries by default.
- TASK-1906 must introduce a fail-closed validation boundary before TASK-1907/TASK-1908 allow any
  value movement across process boundaries.

## Failure And Cancellation Risks

- Existing runtime failures distinguish operational failures and consumed process handles, but
  cancellation is not yet a first-class Phase 195 outcome.
- TASK-1909 must keep cancellation, child failure, join failure, contract violation, and predicate
  fault separate in diagnostics and trace facts.
- Joining a failed child must preserve the child failure identity rather than collapsing it into a
  generic runtime error.

## Trace And Monitor Risks

- NOTE-035 describes process/channel trace facts, but current runtime evidence does not yet emit a
  stable Phase 195 process event alphabet.
- TASK-1910 must define a bounded event set for spawn, start, complete, fail, cancel, join, send,
  receive, and close.
- Workflow/normative ledger facts remain out of scope for Phase 195 except as historical or future
  interpretation references.

## Required Follow-Up Ownership

1. TASK-1904: reconcile specs that still describe `Act`, `Proc`, and `Workflow` as active target
   profiles.
2. TASK-1905: harden process/channel row and Core/CPS carrier preservation.
3. TASK-1906: add sendability and ownership-transfer validation.
4. TASK-1907: implement bounded spawn/join/await semantics over ambient computations.
5. TASK-1908: implement bounded typed-channel semantics.
6. TASK-1909: add cancellation and failure propagation diagnostics.
7. TASK-1910: emit process/channel trace facts and runtime monitor evidence.
8. TASK-1911: add cross-boundary fixtures proving parser/typecheck/Core/CPS/runtime/CLI alignment.

## Audit Decision

Proceed with Phase 195 as a hardening and reconciliation phase over existing partial carriers.
Do not introduce new `Act`, `Proc`, or `Workflow` surface forms, Core terms, IR nodes, public stdlib
types, or runtime entry paths.
