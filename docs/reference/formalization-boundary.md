---
id: reference.ash.formalization-boundary.workflow-first
title: Historical Formalization Boundary and Proof Targets
kind: historical-routing-note
audience: [human, agent]
authority: historical
status: superseded
stability: frozen
owner: language-semantics
last_verified: 2026-07-24
---

# Historical Formalization Boundary and Proof Targets

**Status:** Superseded historical routing page (TASK-1987)

## Current route

For current Ash language semantics, proof work, and conformance, start with the
[Ash Canonical Core](../spec/CANONICAL-CORE.md). It owns the active target vocabulary,
Core/CPS syntax, lowering handoff, operational semantics, observable projection, and
implementation-conformance routes.

This page is not a current semantic, theorem, or proof-authority source. It must not appear in a
default human or agent reading path.

## Historical record

**Last authoritative revision:** `00dd3bcffbee64d0191d8643746fcbb93d218382`
(2026-06-03, `docs: refresh matching reference docs`). Git preserves the full workflow-first and
Lean-facing rationale from that revision.

That historical boundary selected workflow-era specifications and proof targets. Phase 202
superseded those selections because the current target language is function-first and its semantic
pivot is Core/CPS. The replacement is the active canonical core, particularly its
[`SEM-TARGET-CORE-CPS-001`](../spec/CANONICAL-CORE.md#operational-semantics) rule and linked
canonical sources.

The old material may be consulted only to understand the migration rationale; it does not retain
productive semantic claims here.
