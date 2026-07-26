---
id: reference.ash.parser-to-core.workflow-first
title: Historical Parser-to-Core Lowering Contract
kind: historical-routing-note
audience: [human, agent]
authority: historical
status: superseded
stability: frozen
owner: language-semantics
last_verified: 2026-07-24
---

# Historical Parser-to-Core Lowering Contract

**Status:** Superseded historical routing page (TASK-1987)

## Current route

For the current surface-to-Core handoff, start with the
[Ash Canonical Core](../spec/CANONICAL-CORE.md#surface-to-core-handoff), specifically
`LOWER-SURFACE-CORE-001`, and then read its active target source,
[SPEC-098c: Surface-to-Core Lowering](../spec/SPEC-098c-SURFACE-TO-CORE-LOWERING.md).

This page is not a current lowering contract and must not appear in a default human or agent
reading path.

## Historical record

**Last authoritative revision:** `00dd3bcffbee64d0191d8643746fcbb93d218382`
(2026-06-03, `docs: refresh matching reference docs`). Git preserves the full workflow-first
parser/lowering mapping and its migration rationale from that revision.

The prior contract lowered parsed workflow-era forms into a workflow-oriented core. Phase 202
superseded that model: target lowering begins after expansion and resolution and produces
checked-Core-ready terms with explicit origin and boundary sidecars. The canonical core and
SPEC-098c above are the replacement authority.

The retained historical material may be used only to understand the transition; it makes no
productive lowering or semantic claim on this page.
