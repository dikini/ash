---
id: ref.status.removed_forms
title: Phase 201 Removed Forms
kind: status
audience: [human, agent]
authority: canonical-adjacent
status: current
stability: alpha
owner: reference-corpus
last_verified: 2026-07-08
verified_against:
  git_commit: null
  release_tag: null
  ash_version: unreleased-alpha
  specs:
    - docs/spec/SPEC-095b-TARGET-GRAMMAR.md
    - docs/spec/SPEC-096b-TARGET-EFFECT-SYSTEM.md
    - docs/spec/SPEC-097b-TARGET-TYPE-SYSTEM.md
    - docs/spec/SPEC-098c-SURFACE-TO-CORE-LOWERING.md
  tasks:
    - docs/plan/tasks/TASK-1960-deprecated-functionality-removal-plan-packet.md
    - docs/plan/tasks/TASK-1967-deprecated-functionality-removal-gates.md
    - docs/plan/tasks/TASK-1981-removed-form-authority-page.md
  code:
    - crates/ash-cli/tests/phase201_deprecated_functionality_removal_gate.rs
  tests:
    - crates/ash-cli/tests/phase201_deprecated_functionality_removal_gate.rs
  examples: []
related:
  depends_on:
    - ref.status.feature_matrix
    - ref.agents.common_confusions
  explains:
    []
  supersedes: []
  superseded_by: null
  historical_rationale:
    - docs/plan/PLAN-201-DEPRECATED-FUNCTIONALITY-REMOVAL.md
refresh_trigger:
  - Phase 201 closeout changes
  - target grammar/effect/type/lowering spec changes
  - deprecated-functionality gate changes
---

# Phase 201 Removed Forms

This page is the safe current reference for removed historical Ash forms. It is prose-only by
policy: do not add Ash code fences, fixtures, or copyable removed syntax here.

## How To Use This Page

- Treat rows in the table as removed from current productive Ash.
- Use the target replacement column when writing docs, examples, templates, or tests.
- Historical pages may mention these terms only when they are explicitly marked historical and are
  not presented as current source guidance.
- New examples, templates, `.ash` files, Rust source literals, formatter fixtures, and LSP fixtures
  must use target Ash only.

## Removed Historical Forms

| Historical term or shape | Current status | Target replacement |
| --- | --- | --- |
| Workflow declarations as a public language entry form | Removed from productive Ash | Checked target functions plus application runtime admission/reporting |
| Workflow header role, ownership, use, and direct capability clauses | Removed from productive Ash | Function contracts, provider profiles, row admission, and runtime binding metadata |
| Public Act/Proc/Workflow tower carrier syntax as source guidance | Historical only | Target effect rows, provider profiles, process/channel helpers, and application reports |
| Old observe-with and act-with statement shapes | Removed from productive Ash | Current checked target examples and provider/profile APIs |
| Direct capability/provider authority declaration forms | Removed from productive Ash | Provider profiles and admitted operation rows |
| Old callable spellings and historical higher-stratum callable arrows | Removed or reserved | Parenthesized target callable types and pure function/closure syntax |
| Workflow-scoped contract helper names | Removed from active typechecker paths | Compiler-known contract helper identities under the contract namespace |
| Runtime report fields named for workflow identity | Removed from public report schema | Application identity fields and entry metadata |
| Historical stdlib Act, Proc, Workflow modules as current APIs | Historical only | Checked stdlib target modules, process helpers, runtime helpers, and Result/domain helpers |

## Current Target Routes

| Need | Current route |
| --- | --- |
| Pure value computation | `reference/language/functions.md` |
| Runtime admission and authority | `reference/runtime/admission.md` |
| Policy profiles | `reference/runtime/policy-profiles.md` |
| Runtime reports and artifacts | `reference/runtime/README.md` and `reference/runtime/artifacts.md` |
| Process/channel behavior | Checked examples, runtime evidence, and process helper documentation |
| Domain error values | `reference/stdlib/result.md` |
| Historical context for old links | Historical pages under `reference/language/`, `reference/stdlib/`, and `reference/agents/cards/` |

## Agent Rule

When a historical term appears in retrieved context, do not copy it into new Ash source or current
examples. First find the current target route above, then cite this page if the historical term
needs to be explained.
