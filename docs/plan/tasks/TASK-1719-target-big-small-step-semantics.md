# TASK-1719: Add target Core big-step and Core/CPS small-step semantics

## Status: 📝 Planned

## Summary

Add target Core big-step and Core/CPS small-step semantics. This is a documentation-only task in PLAN-167 and belongs to Phase 3.

## Specification Reference

- PLAN-167: `docs/plan/PLAN-167-TARGET-SURFACE-SEMANTICS-GAP-CLOSURE.md`
- Audit: `docs/audit/2026-06-29-target-spec-notes-gap-audit.md`

## Dependencies

- 📝 TASK-1718: Rewrite SPEC-099b scope and preserve Phase 159 interpreter semantics as context (planned)

## Deferral / Planned-Feature Reconciliation

| Prior item | Source | Original reason | Prereqs now? | Decision | Gate |
|---|---|---|---|---|---|
| Target surface/semantics gap | `docs/audit/2026-06-29-target-spec-notes-gap-audit.md` | Target specs are not implementation-grade for parser/macro/lowering/semantics work | Yes, audit preserved and PLAN-167 created | Implement this docs-only slice now | Docs gate plus task-specific structural assertion |

## Description

Add the main target operational rules: Core big-step behavior and Core/CPS small-step behavior for
control, handlers, provider frames, traps, force, and row/accounting observations.

## Files

- `docs/spec/SPEC-099b-TARGET-OPERATIONAL-SEMANTICS.md`
- `CHANGELOG.md`

## Requirements

1. Define Core big-step judgments for checked Core terms at the level needed by lowering/type specs.
2. Define Core/CPS small-step transition shape for control and handler/provider frames.
3. Reconcile provider-frame dispatch with `SPEC-098b`.
4. Define structured trap propagation generically, leaving contract-specific payload cases to
   TASK-1720 if needed.
5. Define lazy/memo force steps at the operational level, including where latent rows become
   observable.

## Docs-only steps

1. Patch `SPEC-099b` with rules and examples.
2. Use meta-notation clearly marked as not Ash surface syntax.
3. Cross-reference `SPEC-098b`, `SPEC-098c`, `SPEC-100`, and `SPEC-101` where relevant.
4. Update changelog.

## Verification

```yaml
strictness: clean
commands:
  - git diff --check
  - python3 tools/docs/validate_orientation_indexes.py --self-test
  - bash scripts/check-docs-gate.sh
  - python3 -c 'from pathlib import Path; s=Path("docs/spec/SPEC-099b-TARGET-OPERATIONAL-SEMANTICS.md").read_text(); assert "small-step" in s; assert "big-step" in s; assert "provider" in s'
checklist:
  - [ ] Core big-step semantics specified.
  - [ ] Core/CPS small-step semantics specified.
  - [ ] Provider-frame dispatch reconciled.
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
