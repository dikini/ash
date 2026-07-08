---
id: ref.agents.card.proc
title: Historical Proc Card
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
    - ref.language.proc
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

# Proc Card

canonical_page: ref.language.proc
canonical_page_path: ../../language/processes-proc.md
dependency_order: 3
warning: Historical after Phase 201; use process/channel helper and runtime guidance for current target Ash.

## Use

This card is retained only for old links. Retrieve current guidance from
`examples/11-process-channel-helpers/process_channel_helpers.ash`, `reference/runtime/README.md`,
and checked target examples instead.

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
- Do not promote Proc tower pages as authoritative target-source guidance.
