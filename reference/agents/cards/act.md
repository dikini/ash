---
id: ref.agents.card.act
title: Historical Act Card
kind: agent-card
audience: [agent]
authority: derivative
status: superseded
stability: alpha
owner: reference-corpus
last_verified: 2026-05-23
verified_against:
  git_commit: ff1f98f
  specs:
    - docs/spec/SPEC-071-REFERENCE-CORPUS-METADATA-AND-MAINTENANCE.md
  tasks:
    - docs/plan/tasks/TASK-950-agent-concept-cards-and-context-pack-index.md
  code:
    []
  tests:
    []
  examples:
    []
related:
  depends_on:
    - ref.language.act
  explains:
    []
  supersedes: []
  superseded_by: null
  historical_rationale:
    []
refresh_trigger:
  - Canonical page changes
  - SPEC-069 or SPEC-071 changes
---

# Act Card

canonical_page: ref.language.act
canonical_page_path: ../../language/effects-act.md
dependency_order: 2
warning: Historical after Phase 201; use runtime admission/provider-profile guidance for current target Ash.

## Use

This card is retained only for old links. Retrieve current guidance from
`reference/runtime/admission.md`, `reference/runtime/README.md`, `docs/TUTORIAL.md`, and checked
examples instead.

## Retrieval tags

- ash
- phase124-reference-pilot
- historical-tower

## Warnings

Do not use this card as current source guidance.

## Must check before editing

- ../../status/feature-matrix.md
- ../../status/known-limitations.md
- ../../../docs/spec/SPEC-069-ALPHA-VISIBLE-TOWER-ALGEBRA-AND-DO-LOWERING.md

## Forbidden stale claims

- The whole reference corpus is complete.
- Agent cards are normative specs.
- Do not promote Act tower pages as authoritative target-source guidance.
