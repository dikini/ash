---
id: plan.ash.docs-orientation-indexes
title: Docs Orientation Indexes
kind: plan
audience: [human, agent]
authority: design
status: complete
stability: alpha
owner: language
last_verified: 2026-06-29
verified_against:
  docs:
    - docs/notes/NOTE-INDEX.md
    - docs/spec/SPEC-INDEX.md
  tools:
    - tools/docs/validate_orientation_indexes.py
---

# Docs Orientation Indexes Implementation Plan

**Goal:** Improve human and agent orientation across Ash's notes/spec corpus by adding structured indexes, cross-cutting tags, machine-checkable coverage, and an independent before/after usability evaluation.

**Architecture:** Keep the indexes human-authored Markdown, but constrain the table shape enough for lint tooling to validate coverage and vocabulary. The model uses a small topic ontology for placement and unstructured tags for retrieval. Agents should use read paths first, then table metadata, then source documents.

**Tech Stack:** Markdown indexes under `docs/notes/` and `docs/spec/`; Python docs tooling under `tools/docs/`; docs gate integration through `scripts/check-docs-gate.sh`; independent baseline and post-index evaluation through subagents.

---

## Phase: 166

## Status

Complete: 6/6 tasks complete. The indexes and lint tooling are implemented, docs gate validates them, and independent agent usability evaluation reports are recorded in TASK-1707.

## Scope locks

1. The indexes are navigational metadata, not new normative authority.
2. Topics are structured placement metadata. Tags are flexible retrieval handles for cross-cutting concerns such as `grammar`, `semantics`, `references`, `diagnostics`, and `authority`.
3. The first lint tool validates coverage, link shape, required headings, topic vocabulary, and tag vocabulary. It does not infer semantic correctness of every tag.
4. This phase does not retrofit frontmatter into every historical note/spec. It creates index-level metadata and a validator that can support later frontmatter migration.
5. Agent usability is measured by independent subagents before and after indexes exist; the controller does not use current-chat knowledge as evidence.

## Task overview

| Task | Description | Status |
|------|-------------|--------|
| [TASK-1703](tasks/TASK-1703-docs-orientation-index-plan.md) | Create this Phase 166 plan and task packet | ✅ Complete |
| [TASK-1704](tasks/TASK-1704-notes-orientation-index.md) | Create `docs/notes/NOTE-INDEX.md` with topic ontology, tags, read paths, and complete note table | ✅ Complete |
| [TASK-1705](tasks/TASK-1705-specs-orientation-index.md) | Create `docs/spec/SPEC-INDEX.md` with topic ontology, tags, read paths, and complete spec table | ✅ Complete |
| [TASK-1706](tasks/TASK-1706-orientation-index-lint-tooling.md) | Add validator tooling and wire it into docs gate | ✅ Complete |
| [TASK-1707](tasks/TASK-1707-agent-usability-evaluation.md) | Record independent before/after agent discovery evaluations | ✅ Complete |
| [TASK-1708](tasks/TASK-1708-docs-orientation-index-closeout.md) | Reconcile PLAN-INDEX/CHANGELOG and run verification | ✅ Complete |

## Verification

```bash
python3 tools/docs/validate_orientation_indexes.py --self-test
bash scripts/check-docs-gate.sh
git diff --check
```

## Changelog

| Date | Change |
|------|--------|
| 2026-06-29 | Created and completed the docs orientation index phase. |
