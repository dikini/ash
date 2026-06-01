---
id: ref.maintenance.metadata
title: Reference Maintenance Metadata
kind: reference
audience: [human, agent]
authority: canonical-adjacent
status: current
stability: alpha
owner: reference-corpus
last_verified: 2026-06-01
verified_against:
  git_commit: 4fa1eba
  release_tag: null
  ash_version: unreleased-alpha
  specs:
    - docs/spec/SPEC-071-REFERENCE-CORPUS-METADATA-AND-MAINTENANCE.md
    - docs/spec/SPEC-075-REFERENCE-SLICE-2-RUNTIME-TOOLCHAIN-MAINTENANCE.md
  tasks:
    - docs/plan/tasks/TASK-993-reference-maintenance-metadata-and-staleness.md
  code:
    - tools/reference/check_frontmatter.py
    - tools/reference/check_staleness.py
  tests:
    - check_frontmatter full reference validation
    - check_staleness maintenance path audit
  examples:
    []
related:
  depends_on:
    - ref.meta
    - ref.maintenance.index
  explains:
    - ref.status.reference_maintenance
  supersedes: []
  superseded_by: null
  historical_rationale:
    - docs/design/DESIGN-043-REFERENCE-SLICE-2-RUNTIME-TOOLCHAIN-MAINTENANCE.md
refresh_trigger:
  - docs/spec/SPEC-071-REFERENCE-CORPUS-METADATA-AND-MAINTENANCE.md changes
  - docs/spec/SPEC-075-REFERENCE-SLICE-2-RUNTIME-TOOLCHAIN-MAINTENANCE.md changes
  - tools/reference/check_frontmatter.py changes
  - tools/reference/check_staleness.py changes
  - reference/maintenance/** changes
---

# Reference Maintenance Metadata

## Summary

Slice 2 keeps SPEC-071 as the frontmatter contract and tightens how maintainers interpret it. `verified_against.git_commit` is the primary freshness anchor before Alpha releases exist. `release_tag` and `ash_version` are advisory fields under `verified_against`; they do not replace the commit baseline.

## Status

This page is current for Phase 130 TASK-993. It defines metadata semantics for reference authors and for deterministic tooling. It does not add new SPEC-071 status values.

## Concept

Reference metadata has three different jobs:

| Layer | Stored where | Meaning |
| --- | --- | --- |
| Declared status | `status` | Human-maintained lifecycle state such as `current`, `partial`, `draft`, `stale`, or `superseded`. |
| Verification baseline | `last_verified` and `verified_against` | The date, commit, and evidence checked before the page claimed freshness. |
| Derived inspection state | Tool or audit output | A computed result from comparing changes since the baseline against evidence and refresh triggers. |

`needs-inspection` is a derived inspection state. It is not a `status` value and must not be written as `status: needs-inspection` unless a later spec extends SPEC-071.

## Field Semantics

`verified_against.git_commit` is the commit at which the page's claims were checked. Use a non-`unknown` short or full commit after closeout. Do not update it for copy edits unless the evidence was rechecked.

`verified_against.release_tag` is optional advisory release metadata. Use `null` when there is no release tag for the checked state.

`verified_against.ash_version` is optional advisory version metadata. Use `unreleased-alpha` until a more precise release/version line exists.

Evidence lists support traceability and staleness inspection:

| List | Contents |
| --- | --- |
| `specs` | Repo-relative specs or design-promoted specs backing normative claims. |
| `tasks` | Task, plan, closeout, or audit files backing current status. |
| `code` | Repo-relative implementation or tooling paths whose changes may affect the page. |
| `tests` | Test paths or command strings used as evidence. |
| `examples` | Cited examples, fixtures, or example-status pages. |

`refresh_trigger` lists concrete paths, path globs, or precise semantic changes that require inspection. Prefer entries such as `crates/ashgrove/src/** changes` over broad phrases such as `tooling changes`.

## Declared Status

Use the SPEC-071 lifecycle values:

| Status | Use |
| --- | --- |
| `current` | Evidence supports the page's current claims. |
| `partial` | The page is accurate but intentionally incomplete or heavily caveated. |
| `draft` | The page is a skeleton or early draft. |
| `stale` | Inspection found a contradiction with current evidence. |
| `superseded` | A newer page replaces this one. |
| `generated` | A generator owns the artifact. |
| `unknown` | Temporary state for incomplete inventory only. |

Do not use `needs-inspection` as declared status. A page can remain declared `current` while a diff audit reports `needs-inspection`; the status changes only after inspection finds stale, partial, or superseded content.

## Implementation Notes

The validator [check_frontmatter.py](../../tools/reference/check_frontmatter.py) checks SPEC-071 structure and evidence paths. The staleness checker [check_staleness.py](../../tools/reference/check_staleness.py) reads the same controlled frontmatter subset and reports derived inspection states from Git path changes.

## Authority and Traceability

This page derives from [SPEC-071](../../docs/spec/SPEC-071-REFERENCE-CORPUS-METADATA-AND-MAINTENANCE.md), [SPEC-075](../../docs/spec/SPEC-075-REFERENCE-SLICE-2-RUNTIME-TOOLCHAIN-MAINTENANCE.md), and [DESIGN-043](../../docs/design/DESIGN-043-REFERENCE-SLICE-2-RUNTIME-TOOLCHAIN-MAINTENANCE.md).

## Agent Notes

When editing a reference page, do not advance `last_verified` or `verified_against.git_commit` until every listed evidence source and applicable refresh trigger has been checked.
