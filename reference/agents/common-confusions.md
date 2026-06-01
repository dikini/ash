---
id: ref.agents.common_confusions
title: Agent Common Confusions
kind: guide
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
  - SPEC-075 changes
  - reference/stdlib/** changes
  - reference/tools/ashgrove/** changes
  - reference/runtime/** changes
  - reference/status/** changes
  - Phase closeout changes reference policy
---

# Agent Common Confusions

- Do not say Act is Result. Act is an opaque runtime-managed effect; Result is a value-level success/failure type.
- Do not add implicit tower lifts. Use explicit Act-to-Proc or lower-to-Workflow bridges when current sources expose them.
- Preserve the public tower order: Pure < Act < Proc < Workflow.
- Do not say a final expression in `do` is always returned. Current generalized do lowering is evidence-driven.
- Do not say `Err { error: e }` is operational bottom or that `fail e` implicitly constructs `Err { error: e }`.
- Do not say Ashgrove provides a hosted registry, global/system install roots, OS package-manager integration, arbitrary SemVer registry solving, hosted release-channel discovery, or independent stdlib updates.
- Do not say Ashgrove source-root ignores are broad user-supplied glob policy. Current ignored local-state handling is narrow and fail-closed for nonignored payload changes.
- Do not say RuntimeKernel provides remote/multi-user daemon APIs, distributed scheduling, production init integration, or hot-swapping already-running instances.
- Do not say file presence executes code or provider/resource inventory grants authority. RuntimeKernel execution requires selection, admission, and admitted grants.
- Do not treat historical examples as normative-pass without the example status table.
- Do not move `docs/`; it remains the working and historical corpus.
