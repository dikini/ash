---
id: ref.agents.card.stdlib_act
title: Stdlib Act Card
kind: agent-card
audience: [agent]
authority: derivative
status: current
stability: alpha
owner: reference-corpus
last_verified: 2026-06-01
verified_against:
  git_commit: 01bafb4
  release_tag: null
  ash_version: unreleased-alpha
  specs:
    - docs/spec/SPEC-047-ACT-MONAD.md
    - docs/spec/SPEC-069-ALPHA-VISIBLE-TOWER-ALGEBRA-AND-DO-LOWERING.md
    - docs/spec/SPEC-071-REFERENCE-CORPUS-METADATA-AND-MAINTENANCE.md
    - docs/spec/SPEC-075-REFERENCE-SLICE-2-RUNTIME-TOOLCHAIN-MAINTENANCE.md
  tasks:
    - docs/plan/tasks/TASK-997-reference-stdlib-tower-pages.md
    - docs/plan/tasks/TASK-998-reference-agent-cards-and-context-pack.md
    - docs/plan/tasks/TASK-999-reference-slice-2-closeout.md
  code:
    - std/src/act.ash
    - std/src/lib.ash
  tests:
    - crates/ash-cli/tests/stdlib_corpus_check.rs
    - crates/ash-typeck/tests/alpha_tower_opaque_carriers.rs
  examples:
    - examples/07-phase105/01-do-act.ash
related:
  depends_on:
    - ref.stdlib.act
  explains:
    - ref.agents.common_confusions
  supersedes: []
  superseded_by: null
  historical_rationale:
    - docs/spec/SPEC-047-ACT-MONAD.md
refresh_trigger:
  - reference/stdlib/act.md changes
  - std/src/act.ash changes
  - std/src/lib.ash changes
  - docs/spec/SPEC-047-ACT-MONAD.md changes
  - docs/spec/SPEC-069-ALPHA-VISIBLE-TOWER-ALGEBRA-AND-DO-LOWERING.md changes
---

# Stdlib Act Card

canonical_page: ref.stdlib.act
canonical_page_path: ../../stdlib/act.md
dependency_order: stdlib-tower-1
warning: Read the canonical page first. This card is derivative and must not redefine Act semantics.

## Use

Retrieve the canonical page first, then use this card for search tags, stale-claim warnings, and edit preflight.

## Retrieval tags

- ash
- reference-slice-2
- stdlib-act
- Act
- effect-carrier
- no-implicit-lifts
- Pure-Act-Proc-Workflow

## Must check before editing

- ../../stdlib/act.md
- ../../stdlib/proc.md
- ../../stdlib/workflow.md
- ../../stdlib/result.md
- ../../status/alpha-limitations.md
- ../../../std/src/act.ash
- ../../../std/src/lib.ash
- ../../../docs/spec/SPEC-047-ACT-MONAD.md
- ../../../docs/spec/SPEC-069-ALPHA-VISIBLE-TOWER-ALGEBRA-AND-DO-LOWERING.md

## Forbidden stale claims

- Act is Result.
- `ActEnv` is source-denotable or user-constructible.
- Act implicitly lifts to Proc or Workflow.
- The current tower order is anything other than `Pure < Act < Proc < Workflow`.
- `Act<Result<A, E>>` is the same thing as `Result<A, E>`.
- Agent cards are normative specs.
