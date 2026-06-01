---
id: ref.agents.card.stdlib_proc
title: Stdlib Proc Card
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
    - docs/spec/SPEC-048-PROC-LIBRARY.md
    - docs/spec/SPEC-069-ALPHA-VISIBLE-TOWER-ALGEBRA-AND-DO-LOWERING.md
    - docs/spec/SPEC-071-REFERENCE-CORPUS-METADATA-AND-MAINTENANCE.md
    - docs/spec/SPEC-075-REFERENCE-SLICE-2-RUNTIME-TOOLCHAIN-MAINTENANCE.md
  tasks:
    - docs/plan/tasks/TASK-997-reference-stdlib-tower-pages.md
    - docs/plan/tasks/TASK-998-reference-agent-cards-and-context-pack.md
  code:
    - std/src/proc.ash
    - std/src/lib.ash
  tests:
    - crates/ash-cli/tests/stdlib_corpus_check.rs
    - crates/ash-engine/tests/task_718_proc_stdlib.rs
    - crates/ash-engine/tests/task_719_proc_from_act_stdlib.rs
    - crates/ash-typeck/tests/alpha_tower_opaque_carriers.rs
  examples:
    - examples/07-phase105/03-do-proc-from-act.ash
related:
  depends_on:
    - ref.stdlib.proc
  explains:
    - ref.agents.common_confusions
  supersedes: []
  superseded_by: null
  historical_rationale:
    - docs/spec/SPEC-048-PROC-LIBRARY.md
refresh_trigger:
  - reference/stdlib/proc.md changes
  - std/src/proc.ash changes
  - std/src/lib.ash changes
  - docs/spec/SPEC-048-PROC-LIBRARY.md changes
  - docs/spec/SPEC-069-ALPHA-VISIBLE-TOWER-ALGEBRA-AND-DO-LOWERING.md changes
---

# Stdlib Proc Card

canonical_page: ref.stdlib.proc
canonical_page_path: ../../stdlib/proc.md
dependency_order: stdlib-tower-2
warning: Read the canonical page first. This card is derivative and must not redefine Proc semantics.

## Use

Retrieve the canonical page first, then use this card for search tags, stale-claim warnings, and edit preflight.

## Retrieval tags

- ash
- reference-slice-2
- stdlib-proc
- Proc
- process-carrier
- proc-from-act
- no-implicit-lifts
- Pure-Act-Proc-Workflow

## Must check before editing

- ../../stdlib/proc.md
- ../../stdlib/act.md
- ../../stdlib/workflow.md
- ../../status/known-limitations.md
- ../../../std/src/proc.ash
- ../../../std/src/lib.ash
- ../../../docs/spec/SPEC-048-PROC-LIBRARY.md
- ../../../docs/spec/SPEC-069-ALPHA-VISIBLE-TOWER-ALGEBRA-AND-DO-LOWERING.md
- ../../../crates/ash-engine/tests/task_719_proc_from_act_stdlib.rs

## Forbidden stale claims

- Proc silently accepts raw Act binds in `do:Proc`.
- Proc implicitly lifts to Workflow.
- `P<A>` is user-constructible.
- `proc::from_act` exposes a process handle eagerly.
- The current tower order is anything other than `Pure < Act < Proc < Workflow`.
- Agent cards are normative specs.
