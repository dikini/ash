# TASK-1835: Reconcile stale target-Ash specs and notes

## Description

Clean up stale target-Ash documentation found after Phase 179 closeout. The immediate problem is that some specs and notes still read as if the target model is `effect` declarations, `capability binding` authority, or `WorkflowForm`, even though the current target direction is interface/impl-qualified operation identities, provider/handler admission, computation rows as requirements, and ambient workflow facts.

## Requirements

- Add target reconciliation fences to NOTE-009-era capability/resource specs that remain useful as historical/current-state context.
- Add target reconciliation fences to protocol and operational-semantics docs that still present legacy capability binding or lookup vocabulary as unfenced guidance.
- Update target convergence notes to replace stale `effect declaration` target wording with interface/impl operation wording.
- Fence WorkflowForm-era sections of SPEC-056 so future implementation work does not revive `WorkflowForm`.
- Update orientation indexes when spec/note routing or status changes.
- Add a CHANGELOG entry under `[Unreleased]`.

## Completion criteria

- [x] `SPEC-052` and `SPEC-053` clearly distinguish historical/current-state capability-binding vocabulary from target-state provider/operation vocabulary.
- [x] `NOTE-011` and `SPEC-004` route legacy capability-binding/lookup vocabulary to target provider/handler admission and operation-identity terminology.
- [x] `NOTE-015`, `NOTE-018`, and `NOTE-019` align with NOTE-022/023/025 on operation declaration and identity.
- [x] `SPEC-056` no longer contains unqualified target-looking WorkflowForm mandates in the high-risk sections identified by review.
- [x] `SPEC-INDEX.md` and `NOTE-INDEX.md` are updated for target-Ash routing and status.
- [x] `CHANGELOG.md` includes this task.
- [x] Stale-term scan is reviewed and remaining hits are either historical/current-state or explicitly deferred.
- [x] `python3 tools/docs/validate_orientation_indexes.py --self-test`, `bash scripts/check-docs-gate.sh`, and `git diff --check` pass.

## Evidence

- Reconciled `NOTE-015`, `NOTE-018`, and `NOTE-019` main-body target operation wording to use interface operation declarations, impl/type-qualified operation identities, and provider/handler admission.
- Added target reconciliation notices to `SPEC-052` and `SPEC-053` so their NOTE-009-era capability-binding vocabulary is historical/current-state context.
- Added target reconciliation notices to `NOTE-011` and `SPEC-004` so protocol and operational-semantics readers start from provider/handler admission and target operational semantics for target-Ash work.
- Fenced high-risk `WorkflowForm` sections in `SPEC-056` as historical MVP material and routed target workflow work to ambient rows/Core/CPS/sidecar facts.
- Updated `SPEC-INDEX.md` and `NOTE-INDEX.md` read paths/status rows for target-Ash cleanup routing.
- Reviewed stale-term scans. Remaining hits are in explicitly superseded/historical notes, current-state compatibility specs, review artifacts, or strikethrough gap-register history.
- Verification passed:
  - `python3 tools/docs/validate_orientation_indexes.py --self-test`
  - `bash scripts/check-docs-gate.sh`
  - `git diff --check`

## Depends on

- PLAN-180.
