---
id: task.1606
title: Add CPS operational semantics reference documentation
phase: 159
status: completed
owner: hermes
created: 2026-06-19
completed: 2026-06-19
---

# TASK-1606: Add CPS operational semantics reference documentation

## Description

Write canonical reference documentation for the Ash CPS IR operational semantics (big-step) and add it to the reference corpus.

## Requirements

- [x] Write `reference/language/cps-operational-semantics.md` with:
  - Summary of big-step vs small-step semantics
  - Evaluation judgment form `⟨t, η, χ⟩ ⇓ r`
  - Core term rules (LetVal, LetPrim, LetCont, Jump, Call)
  - Conditional rules (If)
  - Handler rules (Raise, Handle, chain lookup, shallow removal, provider persistence, resume, one-shot)
  - Recursion rules (LetRec)
  - Administrative rules (RecordDischarge, Trap)
  - Row-checking rules
  - Worked example: factorial evaluation trace
  - Explicit deferrals (small-step, legacy lowering, Lean differential, bytecode, JIT, mutual recursion, full row polymorphism, effect aliases, full discharge)
  - Cross-references to specs and plans
- [x] Write `reference/agents/cards/cps-operational-semantics.md` agent card with:
  - Retrieval tags
  - Stale-claim warnings
  - Quick facts
  - When to use this card
  - Key invariants
  - Rule reference table
  - Common debugging patterns
  - Related cards
  - Edit preflight
  - Future work section
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

The operational semantics reference targets programmers who need to understand how Ash CPS IR terms evaluate and LLM agents that may need to reason about evaluation behavior. It is self-contained but references the canonical spec (SPEC-099b) and implementation plan (PLAN-159).

Big-step semantics is the current reference. Small-step semantics is explicitly deferred and noted as future work.
