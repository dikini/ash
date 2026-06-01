---
id: ref.agents.context_pack
title: Pilot Context Pack Index
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
    - ref.language.functions
    - ref.language.act
    - ref.language.proc
    - ref.language.workflow
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
  - reference/tools/** changes
  - reference/runtime/** changes
  - Phase closeout changes reference policy
---

# Pilot Context Pack Index

Dependency order for agent retrieval:

1. [Functions card](cards/functions.md) and [functions reference](../language/functions.md)
2. [Act card](cards/act.md) and [Act reference](../language/effects-act.md)
3. [Proc card](cards/proc.md) and [Proc reference](../language/processes-proc.md)
4. [Workflow card](cards/workflow.md) and [Workflow reference](../language/workflows.md)
5. [Generalized do card](cards/generalized-do.md) and [generalized do reference](../language/generalized-do.md)
6. [Stdlib Act card](cards/stdlib-act.md) and [Act stdlib](../stdlib/act.md)
7. [Stdlib Proc card](cards/stdlib-proc.md) and [Proc stdlib](../stdlib/proc.md)
8. [Stdlib Workflow card](cards/stdlib-workflow.md) and [Workflow stdlib](../stdlib/workflow.md)
9. [Stdlib Result card](cards/stdlib-result.md) and [Result stdlib](../stdlib/result.md)
10. [Ash CLI card](cards/ash-cli.md) and [CLI reference](../tools/cli.md)
11. [Ashgrove card](cards/ashgrove.md), [Ashgrove reference](../tools/ashgrove.md), and the relevant Ashgrove procedure page.
12. [RuntimeKernel card](cards/runtime-kernel.md), [RuntimeKernel reference](../runtime/kernel.md), and the relevant runtime subpage.

Always read the canonical page named by the card's body-level `canonical_page` and `canonical_page_path` fields before editing. Cards are derivative retrieval aids.

Retrieval tags: `ash`, `reference-pilot`, `reference-slice-2`, `pure-act-proc-workflow`, `stdlib-tower`, `tools-runtime`, `generalized-do`, `capability-provider`, `no-implicit-lifts`, `ashgrove`, `runtime-kernel`.
