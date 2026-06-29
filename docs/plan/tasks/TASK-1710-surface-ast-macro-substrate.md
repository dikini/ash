# TASK-1710: Create SPEC-095c syntax-tree layers and macro boundaries

## Status: 📝 Planned

## Summary

Create SPEC-095c syntax-tree layers and macro boundaries. This is a documentation-only task in PLAN-167 and belongs to Phase 1.

## Specification Reference

- PLAN-167: `docs/plan/PLAN-167-TARGET-SURFACE-SEMANTICS-GAP-CLOSURE.md`
- Audit: `docs/audit/2026-06-29-target-spec-notes-gap-audit.md`

## Dependencies

- 📝 TASK-1709: Patch target grammar drift in SPEC-095b (planned)

## Deferral / Planned-Feature Reconciliation

| Prior item | Source | Original reason | Prereqs now? | Decision | Gate |
|---|---|---|---|---|---|
| Target surface/semantics gap | `docs/audit/2026-06-29-target-spec-notes-gap-audit.md` | Target specs are not implementation-grade for parser/macro/lowering/semantics work | Yes, audit preserved and PLAN-167 created | Implement this docs-only slice now | Docs gate plus task-specific structural assertion |

## Description

Create the companion surface spec for a source-preserving AST and future macro substrate. This task
establishes the syntax tree layers and expansion boundary, but leaves detailed notation and operator
sections to later tasks.

## Files

- `docs/spec/SPEC-095c-SURFACE-AST-MACROS-AND-NOTATION.md`
- `docs/spec/SPEC-INDEX.md`
- `CHANGELOG.md`

## Requirements

1. Define the purpose and non-goals of `SPEC-095c`.
2. Define the pipeline:
   `tokens -> CST/parsed surface AST -> macro expansion -> notation resolution -> expanded surface AST -> elaborated Core`.
3. Define source-preservation requirements: spans, delimiters, attributes, doc comments, grouping,
   raw operator tokens, macro invocations, and origin metadata.
4. Define AST layer boundaries: token/concrete syntax, parsed surface AST, expanded surface AST,
   and elaborated Core boundary.
5. Define macro expansion as syntax-to-syntax and hygiene-ready, without committing to typed macros.
6. Add `SPEC-095c` to `SPEC-INDEX.md` with topic/tags and a read path entry.

## Docs-only steps

1. Create `SPEC-095c` with frontmatter matching nearby target specs.
2. Include proposed/aspirational labels for syntax that is not implemented.
3. Keep notation declarations as section placeholders only; detailed notation is TASK-1711.
4. Update indexes and changelog.

## Verification

```yaml
strictness: clean
commands:
  - git diff --check
  - python3 tools/docs/validate_orientation_indexes.py --self-test
  - bash scripts/check-docs-gate.sh
  - python3 -c 'from pathlib import Path; p=Path("docs/spec/SPEC-095c-SURFACE-AST-MACROS-AND-NOTATION.md"); s=p.read_text(); assert "source-preserving" in s; assert "macro expansion" in s; assert "expanded surface AST" in s'
checklist:
  - [ ] SPEC-095c exists.
  - [ ] AST layers are explicit.
  - [ ] Macro expansion boundary is explicit.
  - [ ] SPEC-INDEX links SPEC-095c.
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
