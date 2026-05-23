---
id: ref.agents.card.proc
title: Proc Card
kind: agent-card
audience: [agent]
authority: derivative
status: current
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
warning: Proc requires explicit tower crossings from Act and to Workflow.

## Use

Retrieve the canonical page first, then use this card for search tags, stale-claim warnings, and edit preflight.

## Retrieval tags

- ash
- phase124-reference-pilot
- pure-act-proc-workflow

## Warnings

Check the canonical page before editing. Do not use this card as independent authority.

## Must check before editing

- ../../status/feature-matrix.md
- ../../status/known-limitations.md
- ../../../docs/spec/SPEC-069-ALPHA-VISIBLE-TOWER-ALGEBRA-AND-DO-LOWERING.md

## Forbidden stale claims

- The whole reference corpus is complete.
- Agent cards are normative specs.
- Historical examples are automatically current executable evidence.
