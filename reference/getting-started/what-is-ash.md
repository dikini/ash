---
id: ref.getting_started.what_is_ash
title: What Is Ash?
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
    - docs/spec/SPEC-069-ALPHA-VISIBLE-TOWER-ALGEBRA-AND-DO-LOWERING.md
    - docs/spec/SPEC-070-ALPHA-RUNTIME-KERNEL-AND-OS-SURFACE.md
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
  supersedes: []
  superseded_by: null
  historical_rationale:
    - docs/design/DESIGN-043-REFERENCE-SLICE-2-RUNTIME-TOOLCHAIN-MAINTENANCE.md
refresh_trigger:
  - docs/spec/SPEC-069-ALPHA-VISIBLE-TOWER-ALGEBRA-AND-DO-LOWERING.md changes
  - docs/spec/SPEC-070-ALPHA-RUNTIME-KERNEL-AND-OS-SURFACE.md changes
  - reference/language/** changes
  - reference/getting-started/** changes
---

# What Is Ash?

Ash is a programming language for writing pure transformations, effectful actions, process-structured work, and workflow-level orchestration with explicit boundaries between those layers.

The Alpha reading model is:

| Layer | Reader phrase | Start here |
| --- | --- | --- |
| Pure | Transform with Pure | [Functions and pure code](../language/functions.md) |
| Act | Effect with Act | [Act effects](../language/effects-act.md) |
| Proc | Effect with Proc | [Proc processes](../language/processes-proc.md) |
| Workflow | Orchestrate with Workflow | [Workflow boundaries](../language/workflows.md) |

## What To Remember

Pure code transforms values without capability dispatch. `Act` represents sequential effectful computation. `Proc` represents process-capable computation. `Workflow` is the runtime admission and orchestration boundary.

The layers are explicit. Ash does not silently lift `Act` into `Proc`, flatten nested computations, or treat a file as executing merely because it exists.

## Current Scope

This page is an orientation page, not the full language reference. The linked language pages own concept detail, examples, caveats, and traceability.
