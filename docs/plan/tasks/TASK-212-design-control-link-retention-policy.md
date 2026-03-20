# TASK-212: Design Control-Link Retention Policy

## Status: 📝 Planned

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
