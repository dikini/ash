---
id: ref.agents.index
title: Agent Reference Guide
kind: agent-pack
audience: [agent]
authority: derivative
status: current
stability: alpha
owner: reference-corpus
last_verified: 2026-06-01
verified_against:
  git_commit: 7fc92f6
  specs:
    - docs/spec/SPEC-071-REFERENCE-CORPUS-METADATA-AND-MAINTENANCE.md
    - docs/spec/SPEC-075-REFERENCE-SLICE-2-RUNTIME-TOOLCHAIN-MAINTENANCE.md
  tasks:
    - docs/plan/tasks/TASK-950-agent-concept-cards-and-context-pack-index.md
    - docs/plan/tasks/TASK-998-reference-agent-cards-and-context-pack.md
  code:
    []
  tests:
    []
  examples:
    []
related:
  depends_on:
    - ref.index
  explains:
    - ref.agents.context_pack
    - ref.agents.common_confusions
  supersedes: []
  superseded_by: null
  historical_rationale:
    []
refresh_trigger:
  - SPEC-071 changes
  - SPEC-075 changes
  - reference/agents/** changes
  - Phase closeout changes reference policy
---

# Agent Reference Guide

Agent pages are derivatives. They help retrieval and editing, but semantic claims come from the linked reference pages.

Use order: read [context-pack index](context-pack-index.md), then [common confusions](common-confusions.md), then the card for the concept being edited.

## Cards

- [Functions](cards/functions.md)
- [Stdlib Result](cards/stdlib-result.md)
- [Ash CLI](cards/ash-cli.md)
- [Ashgrove](cards/ashgrove.md)
- [RuntimeKernel](cards/runtime-kernel.md)

## Historical Cards

These cards are retained for old links only after Phase 201:

- [Historical Act language](cards/act.md)
- [Historical Proc language](cards/proc.md)
- [Historical Workflow language](cards/workflow.md)
- [Historical generalized do](cards/generalized-do.md)
- [Historical Stdlib Act](cards/stdlib-act.md)
- [Historical Stdlib Proc](cards/stdlib-proc.md)
- [Historical Stdlib Workflow](cards/stdlib-workflow.md)
