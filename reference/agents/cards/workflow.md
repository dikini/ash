---
id: ref.agents.card.workflow
title: Historical Workflow Card
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
    - ref.language.workflow
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

# Workflow Card

canonical_page: ref.language.workflow
canonical_page_path: ../../language/workflows.md
dependency_order: 4
warning: Historical after Phase 201; use application runtime reports over checked target functions.

## Use

This card is retained only for old links. Retrieve current guidance from `docs/API.md`,
`docs/TUTORIAL.md`, `reference/runtime/README.md`, and checked target examples instead.

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
- Do not promote Workflow tower pages as authoritative target-source guidance.
