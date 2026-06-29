# TASK-1712: Specify operator sections as callable sugar

## Status: 📝 Planned

## Summary

Specify operator sections as callable sugar. This is a documentation-only task in PLAN-167 and belongs to Phase 1.

## Specification Reference

- PLAN-167: `docs/plan/PLAN-167-TARGET-SURFACE-SEMANTICS-GAP-CLOSURE.md`
- Audit: `docs/audit/2026-06-29-target-spec-notes-gap-audit.md`

## Dependencies

- 📝 TASK-1711: Specify prefix/infix/suffix/mixfix notation declarations (planned)

## Deferral / Planned-Feature Reconciliation

| Prior item | Source | Original reason | Prereqs now? | Decision | Gate |
|---|---|---|---|---|---|
| Target surface/semantics gap | `docs/audit/2026-06-29-target-spec-notes-gap-audit.md` | Target specs are not implementation-grade for parser/macro/lowering/semantics work | Yes, audit preserved and PLAN-167 created | Implement this docs-only slice now | Docs gate plus task-specific structural assertion |

## Description

Add binary infix operator sections to `SPEC-095c` and state their callable typing/row behavior.
Operator sections are part of the notation substrate and must preserve source shape before
expansion.

## Files

- `docs/spec/SPEC-095c-SURFACE-AST-MACROS-AND-NOTATION.md`
- `docs/spec/SPEC-097b-TARGET-TYPE-SYSTEM.md` if a forward typing reference is useful
- `CHANGELOG.md`

## Requirements

1. Define full infix use, left section, right section, and bare operator value:
   - `a <op> b`
   - `(a <op>)`
   - `(<op> b)`
   - `(<op>)`
2. Define `Expr::OperatorSection` shape and `OperatorSectionKind::{Bare, Left, Right}` in spec
   prose or pseudocode.
3. State desugaring to callable values or eta-expanded closures.
4. State typing and row preservation:
   if `op : (A, B) -> {r} C`, then sections produce callable values with row `{r}`.
5. Limit initial section scope to binary infix operators; defer partial application of arbitrary
   mixfix patterns.
6. State predicate admissibility after expansion.

## Docs-only steps

1. Patch `SPEC-095c` operator-section section.
2. Add examples for left, right, and bare sections.
3. Add explicit non-goals for generalized mixfix sections and binder-introducing sections.
4. Update changelog.

## Verification

```yaml
strictness: clean
commands:
  - git diff --check
  - python3 tools/docs/validate_orientation_indexes.py --self-test
  - bash scripts/check-docs-gate.sh
  - python3 -c 'from pathlib import Path; s=Path("docs/spec/SPEC-095c-SURFACE-AST-MACROS-AND-NOTATION.md").read_text(); assert "OperatorSection" in s; assert "Left" in s and "Right" in s and "Bare" in s; assert "binary infix" in s'
checklist:
  - [ ] Operator section forms are specified.
  - [ ] Section typing preserves rows.
  - [ ] Generalized mixfix sections are explicitly deferred.
```


## Dispatch

```yaml
agent: hermes
reasoning: high
max_turns: 20
toolsets: [terminal, file]
```

## Dependencies for next task

This task produces a reviewed documentation slice that the next PLAN-167 task consumes.

## Notes

- This task is documentation-only. Do not add Rust implementation gates.
- Use actual Ash target syntax from existing specs. Mark proposed or illustrative syntax explicitly.
- If an independent review finds blockers, leave the task planned/in progress until those blockers are fixed.
