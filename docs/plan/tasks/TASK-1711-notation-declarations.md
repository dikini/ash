# TASK-1711: Specify prefix/infix/suffix/mixfix notation declarations

## Status: 📝 Planned

## Summary

Specify prefix/infix/suffix/mixfix notation declarations. This is a documentation-only task in PLAN-167 and belongs to Phase 1.

## Specification Reference

- PLAN-167: `docs/plan/PLAN-167-TARGET-SURFACE-SEMANTICS-GAP-CLOSURE.md`
- Audit: `docs/audit/2026-06-29-target-spec-notes-gap-audit.md`

## Dependencies

- 📝 TASK-1710: Create SPEC-095c syntax-tree layers and macro boundaries (planned)

## Deferral / Planned-Feature Reconciliation

| Prior item | Source | Original reason | Prereqs now? | Decision | Gate |
|---|---|---|---|---|---|
| Target surface/semantics gap | `docs/audit/2026-06-29-target-spec-notes-gap-audit.md` | Target specs are not implementation-grade for parser/macro/lowering/semantics work | Yes, audit preserved and PLAN-167 created | Implement this docs-only slice now | Docs gate plus task-specific structural assertion |

## Description

Add the user-defined notation model to `SPEC-095c`. Notation is source-level sugar over callable
values and must be erased before Core lowering.

## Files

- `docs/spec/SPEC-095c-SURFACE-AST-MACROS-AND-NOTATION.md`
- `docs/spec/SPEC-097b-TARGET-TYPE-SYSTEM.md` if a short forward reference is needed
- `CHANGELOG.md`

## Requirements

1. Define notation declaration shape for prefix, infix, suffix/postfix, and mixfix notation.
2. Specify precedence and associativity for infix notation.
3. Specify import/export and active-notation-table assumptions at a high level.
4. Define AST nodes before expansion: `Prefix`, `Infix`, `Suffix`, `Mixfix`, and `Paren`.
5. State the invariant: notation is gone before Core.
6. State that notation expansion must not hide authority or row requirements.
7. Include worked examples and desugarings to ordinary callable syntax.

## Docs-only steps

1. Extend `SPEC-095c` notation sections.
2. Keep binder-introducing mixfix out of initial notation scope or explicitly mark it future macro territory.
3. Add any needed cross-reference from `SPEC-097b` without moving full typing rules yet.
4. Update changelog.

## Verification

```yaml
strictness: clean
commands:
  - git diff --check
  - python3 tools/docs/validate_orientation_indexes.py --self-test
  - bash scripts/check-docs-gate.sh
  - python3 -c 'from pathlib import Path; s=Path("docs/spec/SPEC-095c-SURFACE-AST-MACROS-AND-NOTATION.md").read_text(); assert "prefix" in s and "infix" in s and "suffix" in s and "mixfix" in s; assert "gone before Core" in s'
checklist:
  - [ ] All four notation categories are specified.
  - [ ] Notation expands to callable syntax.
  - [ ] Authority/row preservation is stated.
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
