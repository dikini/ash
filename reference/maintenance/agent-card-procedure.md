---
id: ref.maintenance.agent_cards
title: Reference Agent Card Procedure
kind: guide
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
    - docs/plan/tasks/TASK-998-reference-agent-cards-and-context-pack.md
  code:
    - tools/reference/check_frontmatter.py
  tests:
    - check_frontmatter full reference validation
  examples:
    []
related:
  depends_on:
    - ref.agents.index
    - ref.maintenance.metadata
  explains:
    - ref.agents.context_pack
    - ref.agents.common_confusions
  supersedes: []
  superseded_by: null
  historical_rationale:
    - docs/design/DESIGN-043-REFERENCE-SLICE-2-RUNTIME-TOOLCHAIN-MAINTENANCE.md
refresh_trigger:
  - docs/spec/SPEC-071-REFERENCE-CORPUS-METADATA-AND-MAINTENANCE.md changes
  - docs/spec/SPEC-075-REFERENCE-SLICE-2-RUNTIME-TOOLCHAIN-MAINTENANCE.md changes
  - reference/agents/** changes
  - reference/maintenance/** changes
  - tools/reference/check_frontmatter.py changes
---

# Reference Agent Card Procedure

## Summary

Agent cards are derivative artifacts. They compress reference pages for retrieval and editing context, but they must not introduce independent semantic claims.

## Procedure

1. Choose one canonical reference page for the card's main subject.
2. Set frontmatter `kind: agent-card`, `authority: derivative`, and appropriate evidence.
3. Include body fields `canonical_page` and `canonical_page_path`.
4. Add retrieval tags and common-confusion warnings from the canonical page.
5. Add `must_check_before_editing` links for specs, code, tests, and status pages that affect implementation-sensitive claims.
6. Re-run `python3 tools/reference/check_frontmatter.py`.

## Refresh Rules

Refresh an agent card when:

- its canonical reference page changes;
- a linked status page changes a limitation or maturity claim;
- `reference/agents/common-confusions.md` changes relevant warnings;
- SPEC-071 or SPEC-075 changes agent-derivative policy.

## Agent Notes

If a card and canonical page disagree, fix the card or mark it stale. Do not use the card as authority over the reference page.
