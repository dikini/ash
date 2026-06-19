---
id: task.1604
title: Add CPS IR reference documentation
phase: 159
status: completed
owner: hermes
created: 2026-06-19
completed: 2026-06-19
---

# TASK-1604: Add CPS IR reference documentation

## Description

Write canonical reference documentation for the Ash CPS IR (Continuation Passing Style Intermediate Representation) and add it to the reference corpus.

## Requirements

- [x] Write `reference/language/cps-ir.md` with:
  - Summary of CPS IR purpose and design
  - Core concepts (values vs terms, atoms, values, terms, continuation references, effect rows)
  - Serialization formats (JSON and S-expressions)
  - Example: factorial in CPS
  - Common patterns (identity, conditional, effect raising/handling)
  - Runtime environment and handler chain
  - Known limitations
  - Cross-references to specs and plans
- [x] Write `reference/agents/cards/cps-ir.md` agent card with:
  - Retrieval tags
  - Stale-claim warnings
  - Quick facts
  - When to use this card
  - Key invariants
  - Common patterns
  - Related cards
  - Edit preflight
- [x] Update `reference/INDEX.md` with new pages
- [x] Follow `ash-documentation-style-guide` (Rust Book voice, precise but approachable)
- [x] Include YAML frontmatter with `id`, `kind`, `authority`, `verified_against`, `refresh_trigger`

## Verification

- [x] Reference page renders correctly
- [x] Agent card renders correctly
- [x] `reference/INDEX.md` links to new pages
- [x] Frontmatter is complete and valid
- [x] Cross-references to specs/plans are accurate

## Notes

The CPS IR reference targets programmers who need a deeper understanding of Ash's intermediate representation and LLM agents that may need to work with the IR. It is self-contained but references the canonical specs (SPEC-098b, SPEC-099b) and implementation plan (PLAN-159).
