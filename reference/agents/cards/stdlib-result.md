---
id: ref.agents.card.stdlib_result
title: Stdlib Result Card
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
    - docs/spec/SPEC-050-OPERATIONAL-BOTTOM-AND-SCOPED-HANDLING.md
    - docs/spec/SPEC-069-ALPHA-VISIBLE-TOWER-ALGEBRA-AND-DO-LOWERING.md
    - docs/spec/SPEC-071-REFERENCE-CORPUS-METADATA-AND-MAINTENANCE.md
    - docs/spec/SPEC-075-REFERENCE-SLICE-2-RUNTIME-TOOLCHAIN-MAINTENANCE.md
  tasks:
    - docs/plan/tasks/TASK-997-reference-stdlib-tower-pages.md
    - docs/plan/tasks/TASK-998-reference-agent-cards-and-context-pack.md
    - docs/plan/tasks/TASK-999-reference-slice-2-closeout.md
  code:
    - std/src/result.ash
    - std/src/lib.ash
  tests:
    - crates/ash-cli/tests/stdlib_corpus_check.rs
    - crates/ash-typeck/tests/alpha_generalized_do_full_bind_lowering.rs
  examples:
    - tests/std/result.ash
related:
  depends_on:
    - ref.stdlib.result
  explains:
    - ref.stdlib.act
    - ref.agents.common_confusions
  supersedes: []
  superseded_by: null
  historical_rationale:
    - docs/spec/SPEC-050-OPERATIONAL-BOTTOM-AND-SCOPED-HANDLING.md
refresh_trigger:
  - reference/stdlib/result.md changes
  - std/src/result.ash changes
  - std/src/lib.ash changes
  - tests/std/result.ash changes
  - docs/spec/SPEC-050-OPERATIONAL-BOTTOM-AND-SCOPED-HANDLING.md changes
---

# Stdlib Result Card

canonical_page: ref.stdlib.result
canonical_page_path: ../../stdlib/result.md
dependency_order: stdlib-domain-values
warning: Read the canonical page first. This card is derivative and must not redefine Result or operational-bottom semantics.

## Use

Retrieve the canonical page first, then use this card for search tags, stale-claim warnings, and edit preflight.

## Retrieval tags

- ash
- reference-slice-2
- stdlib-result
- Result
- Ok
- Err
- domain-failure
- operational-bottom

## Must check before editing

- ../../stdlib/result.md
- ../../stdlib/act.md
- ../../status/alpha-limitations.md
- ../../../std/src/result.ash
- ../../../std/src/lib.ash
- ../../../tests/std/result.ash
- ../../../docs/spec/SPEC-050-OPERATIONAL-BOTTOM-AND-SCOPED-HANDLING.md
- ../../../docs/spec/SPEC-069-ALPHA-VISIBLE-TOWER-ALGEBRA-AND-DO-LOWERING.md

## Forbidden stale claims

- Result is Act.
- `Err { error: e }` is operational bottom.
- `fail e` implicitly constructs `Err { error: e }`.
- Result helpers grant capabilities, run processes, or admit workflows.
- `Act<Result<A, E>>` can be collapsed to `Result<A, E>`.
- Agent cards are normative specs.
