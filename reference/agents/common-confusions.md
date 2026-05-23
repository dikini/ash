---
id: ref.agents.common_confusions
title: Agent Common Confusions
kind: guide
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
    - ref.language.act
    - ref.language.generalized_do
  explains:
    []
  supersedes: []
  superseded_by: null
  historical_rationale:
    []
refresh_trigger:
  - SPEC-071 changes
  - Phase closeout changes reference policy
---

# Agent Common Confusions

- Do not say Act is Result. Act is an opaque runtime-managed effect; Result is a value-level success/failure type.
- Do not add implicit tower lifts. Use explicit Act-to-Proc or lower-to-Workflow bridges when current sources expose them.
- Do not say a final expression in `do` is always returned. Current generalized do lowering is evidence-driven.
- Do not treat historical examples as normative-pass without the example status table.
- Do not move `docs/`; it remains the working and historical corpus.
