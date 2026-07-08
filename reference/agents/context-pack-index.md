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
    - ref.runtime.kernel
    - ref.runtime.admission
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
2. [RuntimeKernel card](cards/runtime-kernel.md), [RuntimeKernel reference](../runtime/kernel.md), and the relevant runtime subpage.
3. [Runtime admission](../runtime/admission.md) and [runtime policy profiles](../runtime/policy-profiles.md)
4. [Stdlib Result card](cards/stdlib-result.md) and [Result stdlib](../stdlib/result.md)
5. [Ash CLI card](cards/ash-cli.md) and [CLI reference](../tools/cli.md)
6. [Ashgrove card](cards/ashgrove.md), [Ashgrove reference](../tools/ashgrove.md), and the relevant Ashgrove procedure page.
7. [Phase 201 removed forms](../status/removed-forms.md) when retrieved context contains older
   workflow/tower/capability forms.

Always read the canonical page named by the card's body-level `canonical_page` and `canonical_page_path` fields before editing. Cards are derivative retrieval aids.

Historical-only cards retained for old links: [Act](cards/act.md), [Proc](cards/proc.md),
[Workflow](cards/workflow.md), [Generalized do](cards/generalized-do.md),
[Stdlib Act](cards/stdlib-act.md), [Stdlib Proc](cards/stdlib-proc.md), and
[Stdlib Workflow](cards/stdlib-workflow.md). Do not use them as current productive source
guidance.

Retrieval tags: `ash`, `reference-pilot`, `reference-slice-2`, `target-functions`, `tools-runtime`, `capability-provider`, `ashgrove`, `runtime-kernel`.
