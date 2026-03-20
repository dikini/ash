# TASK-212: Design Control-Link Retention Policy

## Status: ✅ Complete

## Description

Design the long-term runtime policy for retaining, exposing, and eventually cleaning up
`ControlLink` lifecycle state after terminal supervision operations.

## Specification Reference

- SPEC-004: Operational Semantics
- SPEC-021: Runtime Observable Behavior

## Reference Contract

- `docs/plan/2026-03-20-runtime-boundary-steering-brief.md`
- `docs/plan/tasks/TASK-206-align-runtime-admission-rejection-and-commitment-visibility.md`

## Requirements

### Functional Requirements

1. Define whether terminated control targets remain observable as tombstones, are eagerly removed,
   or transition through a bounded retention lifecycle
2. Define the runtime cleanup trigger and ownership model for retained supervision state
3. Preserve the current reusable supervision contract for live links
4. Document how later cleanup interacts with runtime visibility, diagnostics, and trace/provenance

## Deliverables

- A design note or spec/reference update freezing the retention policy
- Follow-up implementation task(s) if runtime cleanup or tombstone compaction is required

## Non-goals

- No change to the current `TASK-206` runtime behavior
- No new runtime/reasoner interaction semantics

## Completion Checklist

- [x] canonical retention policy frozen in spec/reference text
- [x] ownership and cleanup boundary documented
- [x] runtime visibility and diagnostics interaction documented
- [x] stale design references aligned
- [x] `CHANGELOG.md` updated

## Outcome

`TASK-212` freezes the current long-term safe baseline:

- live `ControlLink` authority remains reusable while the target is valid
- `kill` creates a retained tombstone inside the owning `RuntimeState`
- later control attempts in that same runtime state continue to fail as explicit terminal-control
  failures rather than silently degrading into `NotFound`
- cleanup is currently owned by whole-`RuntimeState` teardown rather than background per-link
  scavenging

No new implementation task was opened in this pass because the frozen policy keeps the current
runtime behavior and bounds cleanup by runtime-state lifetime. If long-lived embedders later need
in-process tombstone compaction, that should be introduced as an explicit runtime-maintenance
feature rather than hidden cleanup.
