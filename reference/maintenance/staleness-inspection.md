---
id: ref.maintenance.staleness
title: Reference Staleness Inspection
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
    - tools/reference/check_staleness.py
  tests:
    - check_staleness maintenance path audit
  examples:
    []
related:
  depends_on:
    - ref.maintenance.metadata
  explains:
    - ref.maintenance.refresh
    - ref.maintenance.stale_triage
  supersedes: []
  superseded_by: null
  historical_rationale:
    - docs/design/DESIGN-043-REFERENCE-SLICE-2-RUNTIME-TOOLCHAIN-MAINTENANCE.md
refresh_trigger:
  - docs/spec/SPEC-075-REFERENCE-SLICE-2-RUNTIME-TOOLCHAIN-MAINTENANCE.md changes
  - tools/reference/check_staleness.py changes
  - reference/maintenance/** changes
---

# Reference Staleness Inspection

## Summary

Staleness inspection compares the page's verification baseline with the current repository state. It produces a derived inspection state. It does not automatically rewrite page content or declared status.

## Status

This procedure is current for Reference Slice 2. It is path-based and deterministic. It is not a full semantic staleness detector.

## Procedure

For a page, read `verified_against.git_commit` from frontmatter. Call that value `<baseline>`. Then run:

```bash
git diff --name-only <baseline>..HEAD
```

Compare the changed paths to:

- `verified_against.specs`;
- `verified_against.tasks`;
- `verified_against.code`;
- path-like entries in `verified_against.tests`;
- `verified_against.examples`;
- path-like entries in `refresh_trigger`.

If the baseline is `unknown`, inspect the page manually before calling it current. After inspection, update `last_verified`, `verified_against.git_commit`, and evidence lists only when the page's claims were actually checked.

## Derived States

| Derived state | Meaning | Frontmatter effect |
| --- | --- | --- |
| `no-relevant-changes` | No changed path intersects the page evidence or path-like refresh triggers. | Do not change `status` just for this result. |
| `needs-inspection` | At least one changed path intersects evidence or triggers. The page may still be correct. | Do not write `status: needs-inspection`; inspect evidence first. |
| `stale` | Inspection found a contradiction between the page and current evidence. | Set `status: stale` or fix the page and reverify. |
| `partial` | Inspection found the page accurate but incomplete or more caveated than its title implies. | Use `status: partial` until the gap is resolved. |
| `superseded` | A newer page or authority replaces this page. | Use `status: superseded` and set `related.superseded_by`. |

`needs-inspection` is derived, not a SPEC-071 status value.

## Tooling

Run the deterministic checker for a page group:

```bash
python3 tools/reference/check_staleness.py --path reference/maintenance
```

The checker reports the same derived states from changed Git paths. Human or agent inspection is still required before declaring `stale`, `partial`, or `superseded`.

## Authority and Traceability

The model follows [SPEC-075](../../docs/spec/SPEC-075-REFERENCE-SLICE-2-RUNTIME-TOOLCHAIN-MAINTENANCE.md) section 5 and preserves the SPEC-071 status vocabulary.

## Agent Notes

Do not treat any changed file as automatic proof that a page is stale. Treat relevant changes as a queue for evidence review.
