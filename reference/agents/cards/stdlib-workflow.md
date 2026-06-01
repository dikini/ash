---
id: ref.agents.card.stdlib_workflow
title: Stdlib Workflow Card
kind: agent-card
audience: [agent]
authority: derivative
status: current
stability: alpha
owner: reference-corpus
last_verified: 2026-06-01
verified_against:
  git_commit: 7fc92f6
  release_tag: null
  ash_version: unreleased-alpha
  specs:
    - docs/spec/SPEC-051-WORKFLOW-SEMANTICS.md
    - docs/spec/SPEC-069-ALPHA-VISIBLE-TOWER-ALGEBRA-AND-DO-LOWERING.md
    - docs/spec/SPEC-071-REFERENCE-CORPUS-METADATA-AND-MAINTENANCE.md
    - docs/spec/SPEC-075-REFERENCE-SLICE-2-RUNTIME-TOOLCHAIN-MAINTENANCE.md
  tasks:
    - docs/plan/tasks/TASK-997-reference-stdlib-tower-pages.md
    - docs/plan/tasks/TASK-998-reference-agent-cards-and-context-pack.md
  code:
    - std/src/workflow.ash
    - std/src/lib.ash
  tests:
    - crates/ash-cli/tests/stdlib_corpus_check.rs
    - crates/ash-typeck/tests/alpha_tower_opaque_carriers.rs
  examples:
    - examples/09-phase108/01-do-workflow-unit.ash
related:
  depends_on:
    - ref.stdlib.workflow
  explains:
    - ref.runtime.kernel
    - ref.agents.common_confusions
  supersedes: []
  superseded_by: null
  historical_rationale:
    - docs/spec/SPEC-051-WORKFLOW-SEMANTICS.md
refresh_trigger:
  - reference/stdlib/workflow.md changes
  - reference/runtime/** changes
  - std/src/workflow.ash changes
  - std/src/lib.ash changes
  - docs/spec/SPEC-051-WORKFLOW-SEMANTICS.md changes
  - docs/spec/SPEC-069-ALPHA-VISIBLE-TOWER-ALGEBRA-AND-DO-LOWERING.md changes
---

# Stdlib Workflow Card

canonical_page: ref.stdlib.workflow
canonical_page_path: ../../stdlib/workflow.md
dependency_order: stdlib-tower-3
warning: Read the canonical page first. This card is derivative and must not redefine Workflow or RuntimeKernel semantics.

## Use

Retrieve the canonical page first, then use this card for search tags, stale-claim warnings, and edit preflight.

## Retrieval tags

- ash
- reference-slice-2
- stdlib-workflow
- Workflow
- workflow-carrier
- runtime-admission
- no-implicit-lifts
- Pure-Act-Proc-Workflow

## Must check before editing

- ../../stdlib/workflow.md
- ../../stdlib/act.md
- ../../stdlib/proc.md
- ../../runtime/README.md
- ../../runtime/kernel.md
- ../../status/runtime-kernel.md
- ../../../std/src/workflow.ash
- ../../../std/src/lib.ash
- ../../../docs/spec/SPEC-051-WORKFLOW-SEMANTICS.md
- ../../../docs/spec/SPEC-069-ALPHA-VISIBLE-TOWER-ALGEBRA-AND-DO-LOWERING.md

## Forbidden stale claims

- Workflow silently accepts raw Act or Proc binds.
- Workflow contract operations are ordinary public stdlib functions in `std/src/workflow.ash`.
- Reference-only workflow algebra examples are runnable-current examples without corpus evidence.
- Workflow stdlib docs replace RuntimeKernel admission and artifact docs.
- The current tower order is anything other than `Pure < Act < Proc < Workflow`.
- Agent cards are normative specs.
