# TASK-1709: Patch target grammar drift in SPEC-095b

## Status: 📝 Planned

## Summary

Patch target grammar drift in SPEC-095b. This is a documentation-only task in PLAN-167 and belongs to Phase 1.

## Specification Reference

- PLAN-167: `docs/plan/PLAN-167-TARGET-SURFACE-SEMANTICS-GAP-CLOSURE.md`
- Audit: `docs/audit/2026-06-29-target-spec-notes-gap-audit.md`

## Dependencies

None.

## Deferral / Planned-Feature Reconciliation

| Prior item | Source | Original reason | Prereqs now? | Decision | Gate |
|---|---|---|---|---|---|
| Target surface/semantics gap | `docs/audit/2026-06-29-target-spec-notes-gap-audit.md` | Target specs are not implementation-grade for parser/macro/lowering/semantics work | Yes, audit preserved and PLAN-167 created | Implement this docs-only slice now | Docs gate plus task-specific structural assertion |

## Description

Patch the immediate target grammar drift identified by the audit. This task keeps `SPEC-095b` as
the grammar owner while moving macro/notation detail out to the planned companion `SPEC-095c`.

## Files

- `docs/spec/SPEC-095b-TARGET-GRAMMAR.md`
- `docs/spec/SPEC-INDEX.md`
- `CHANGELOG.md`

## Requirements

1. Remove, quarantine, or explicitly mark obsolete the inline `handle effect_item with { ... }`
   form in `do_stmt` and `workflow_stmt`.
2. Remove or rewrite the example that handles `requires { ... }` as if contract failure were a
   resumable handler case.
3. Reconcile `trace_contract_effect` with `SPEC-096b`: either add it to `contract_effect` or state
   that trace surface spelling is deferred to `SPEC-095c` / lowering work.
4. Replace the closed “No new operators” wording with a forward-compatible statement: no new
   built-in operators in this spec, but user-defined notation and operator sections are reserved
   for `SPEC-095c`.
5. Add cross-references from `SPEC-095b` to the Phase 167 audit and planned `SPEC-095c`.

## Docs-only steps

1. Inspect the live `SPEC-095b` insertion points before editing.
2. Patch only the drift called out above; do not design notation in this task.
3. Add a changelog row to `SPEC-095b` if the spec has a changelog section.
4. Update `SPEC-INDEX.md` if its read path or summary mentions closed operators or inline handlers.
5. Update `CHANGELOG.md` under `[Unreleased]`.

## Verification

```yaml
strictness: clean
commands:
  - git diff --check
  - python3 tools/docs/validate_orientation_indexes.py --self-test
  - bash scripts/check-docs-gate.sh
  - python3 -c 'from pathlib import Path; s=Path("docs/spec/SPEC-095b-TARGET-GRAMMAR.md").read_text(); assert "No new operators" not in s; assert "requires ->" not in s'
checklist:
  - [ ] Inline contract-handler syntax is no longer live target syntax.
  - [ ] Trace contract syntax is reconciled or explicitly deferred.
  - [ ] Operator future is open and points to SPEC-095c.
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
