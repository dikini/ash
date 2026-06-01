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
  git_commit: 01bafb4
  release_tag: null
  ash_version: unreleased-alpha
  specs:
    - docs/spec/SPEC-071-REFERENCE-CORPUS-METADATA-AND-MAINTENANCE.md
    - docs/spec/SPEC-075-REFERENCE-SLICE-2-RUNTIME-TOOLCHAIN-MAINTENANCE.md
  tasks:
    - docs/plan/tasks/TASK-994-reference-getting-started-journey.md
    - docs/plan/tasks/TASK-999-reference-slice-2-closeout.md
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
  - reference/stdlib/** changes
---

# Next Steps

After the getting-started path, use the subsystem pages for detail:

- Language concepts: [Pure functions](../language/functions.md), [Act](../language/effects-act.md), [Proc](../language/processes-proc.md), and [Workflow](../language/workflows.md).
- Stdlib tower APIs: [stdlib tower](../stdlib/README.md), [Act stdlib](../stdlib/act.md), [Proc stdlib](../stdlib/proc.md), [Workflow stdlib](../stdlib/workflow.md), and [Result stdlib](../stdlib/result.md).
- Tooling: [Tools index](../tools/README.md), [CLI tools](../tools/cli.md), and [Ashgrove](../tools/ashgrove.md).
- Runtime: [Runtime index](../runtime/README.md), [RuntimeKernel](../runtime/kernel.md), and [daemon](../runtime/daemon.md).
- Corpus freshness: [Reference maintenance](../maintenance/README.md) and [verification evidence](../status/verification-evidence.md).

Reference Slice 2 now includes the getting-started path, toolchain pages, RuntimeKernel pages, stdlib tower pages, maintenance procedures, status evidence, and agent cards. Treat pages marked `partial` as honest Alpha boundaries, not as a signal that later Phase 130 subsystem pages still need to be created.
