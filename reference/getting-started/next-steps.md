---
id: ref.getting_started.next_steps
title: Next Steps
kind: guide
audience: [human, agent]
authority: canonical-adjacent
status: current
stability: alpha
owner: reference-corpus
last_verified: 2026-06-01
verified_against:
  git_commit: 598a8f6
  release_tag: null
  ash_version: unreleased-alpha
  specs:
    - docs/spec/SPEC-071-REFERENCE-CORPUS-METADATA-AND-MAINTENANCE.md
    - docs/spec/SPEC-075-REFERENCE-SLICE-2-RUNTIME-TOOLCHAIN-MAINTENANCE.md
  tasks:
    - docs/plan/tasks/TASK-994-reference-getting-started-journey.md
  code:
    []
  tests:
    - check_frontmatter full reference validation
  examples:
    []
related:
  depends_on:
    - ref.getting_started.index
  explains:
    - ref.language.functions
    - ref.language.act
    - ref.language.proc
    - ref.language.workflow
    - ref.tools.index
    - ref.runtime.index
  supersedes: []
  superseded_by: null
  historical_rationale:
    - docs/design/DESIGN-043-REFERENCE-SLICE-2-RUNTIME-TOOLCHAIN-MAINTENANCE.md
refresh_trigger:
  - docs/spec/SPEC-075-REFERENCE-SLICE-2-RUNTIME-TOOLCHAIN-MAINTENANCE.md changes
  - reference/getting-started/** changes
  - reference/tools/** changes
  - reference/runtime/** changes
---

# Next Steps

After the getting-started path, use the subsystem pages for detail:

- Language concepts: [Pure functions](../language/functions.md), [Act](../language/effects-act.md), [Proc](../language/processes-proc.md), and [Workflow](../language/workflows.md).
- Tooling: [Tools index](../tools/README.md), [CLI tools](../tools/cli.md), and [Ashgrove](../tools/ashgrove.md).
- Runtime: [Runtime index](../runtime/README.md), [RuntimeKernel](../runtime/kernel.md), and [daemon](../runtime/daemon.md).
- Corpus freshness: [Reference maintenance](../maintenance/README.md) and [verification evidence](../status/verification-evidence.md).

Ashgrove toolchain pages are now expanded in [Tools](../tools/README.md). RuntimeKernel and stdlib API pages are still being expanded by later Phase 130 tasks. Treat pages marked `draft` or `partial` as link targets with honest limitations, not complete subsystem manuals.
