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
    - ref.runtime.kernel
    - ref.runtime.admission
  supersedes: []
  superseded_by: null
  historical_rationale:
    - docs/design/DESIGN-043-REFERENCE-SLICE-2-RUNTIME-TOOLCHAIN-MAINTENANCE.md
refresh_trigger:
  - docs/spec/SPEC-070-ALPHA-RUNTIME-KERNEL-AND-OS-SURFACE.md changes
  - reference/language/** changes
  - reference/getting-started/** changes
---

# What Is Ash?

Ash is a programming language for writing pure transformations and checked effectful functions, with explicit process and application-runtime boundaries.

The Alpha reading model is:

| Concern | Reader phrase | Start here |
| --- | --- | --- |
| Functions | Transform values with functions | [Functions and pure code](../language/functions.md) |
| Effects | Declare required effects in rows | [Function boundaries](../language/functions/boundaries.md) |
| Processes | Use checked process and channel helpers | [RuntimeKernel](../runtime/kernel.md) |
| Applications | Admit and report an application entry | [Runtime admission](../runtime/admission.md) |

## What To Remember

Pure code transforms values without capability dispatch. A checked function's effect row records required effects. Process/channel helpers express process-capable work, and RuntimeKernel admission is the boundary for a selected application entry.

The boundaries are explicit. Ash does not infer authority from a function body, silently add an effect row, or treat a file as executing merely because it exists.

## Current Scope

This page is an orientation page, not the full language reference. The linked language pages own concept detail, examples, caveats, and traceability.
