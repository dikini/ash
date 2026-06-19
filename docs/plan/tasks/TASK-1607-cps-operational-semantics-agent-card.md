---
id: task.1607
title: Add CPS operational semantics agent card
phase: 159
status: completed
owner: hermes
created: 2026-06-19
completed: 2026-06-19
---

# TASK-1607: Add CPS operational semantics agent card

## Description

Write the agent card for the Ash CPS operational semantics reference page.

## Requirements

- [x] Write `reference/agents/cards/cps-operational-semantics.md` with:
  - Retrieval tags for agent search
  - Stale-claim warnings (big-step only, no small-step yet)
  - Quick facts with location and cross-references
  - When to use this card guidance
  - Key invariants (8 items)
  - Rule reference table
  - Common debugging patterns
  - Related cards
  - Edit preflight
  - Future work section
- [x] Update `reference/INDEX.md` agent derivatives section
- [x] Follow `ash-documentation-style-guide`
- [x] Include YAML frontmatter with `id`, `kind`, `authority`, `canonical_page`, `verified_against`, `refresh_trigger`

## Verification

- [x] Agent card renders correctly
- [x] `reference/INDEX.md` links to new card
- [x] Frontmatter is complete and valid
- [x] Cross-references to canonical page are accurate

## Notes

The operational semantics agent card is a derivative of `reference/language/cps-operational-semantics.md`. It provides quick lookup for agents working with evaluation rules, debugging interpreter behavior, or planning semantics extensions.
