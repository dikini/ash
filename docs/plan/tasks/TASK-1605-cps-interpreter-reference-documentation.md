---
id: task.1605
title: Add CPS interpreter reference documentation
phase: 159
status: completed
owner: hermes
created: 2026-06-19
completed: 2026-06-19
---

# TASK-1605: Add CPS interpreter reference documentation

## Description

Write canonical reference documentation for the Ash CPS IR interpreter and add it to the reference corpus.

## Requirements

- [x] Write `reference/runtime/cps-interpreter.md` with:
  - Summary of interpreter architecture
  - Entry point (`eval_term`)
  - Thin dispatcher + per-term evaluators pattern
  - Per-term evaluation details (LetVal, LetPrim, LetCont, Jump, Call, If, LetRec, Raise, Handle)
  - Primitive operations table
  - Error handling (`CpsError`)
  - Handler chain semantics
  - Example execution trace (factorial)
  - Testing approach
  - Known limitations
  - Cross-references to specs and plans
- [x] Write `reference/agents/cards/cps-interpreter.md` agent card with:
  - Retrieval tags
  - Stale-claim warnings
  - Quick facts
  - When to use this card
  - Key invariants
  - Per-term evaluator reference table
  - Common debugging patterns
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

The CPS interpreter reference targets programmers who need to understand how Ash evaluates CPS IR terms and LLM agents that may need to debug or extend the interpreter. It is self-contained but references the canonical semantics (SPEC-099b) and implementation plan (PLAN-159).
