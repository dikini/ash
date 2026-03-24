# TASK-223: Canonicalize Touched Role Docs and Examples

## Status: ✅ Complete

## Description

Bring the touched role docs/examples into one of two honest states: canonical-surface-aligned or
explicitly historical/reference-only.

## Specification Reference

- SPEC-002: Surface Language
- SPEC-017: Capability Integration
- SPEC-018: Capability Runtime Verification Matrix

## Requirements

### Functional Requirements

1. Touched user-facing examples must not overclaim canonical-surface compliance
2. Touched docs must either match canonical syntax or clearly present themselves as historical/reference material
3. Local inconsistencies uncovered by the blocker review must be fixed
4. Approval-role examples must remain explicit flat named-role examples

## Files

- Modify: `docs/TUTORIAL.md`
- Modify: `docs/SHARO_CORE_LANGUAGE.md`
- Modify: `docs/book/SUMMARY.md`
- Modify: `docs/book/appendix-a.md`
- Modify: `examples/03-policies/01-role-based.ash`
- Modify: `examples/03-policies/README.md`
- Modify: `examples/04-real-world/code-review.ash`
- Modify: `examples/04-real-world/customer-support.ash`
- Modify: `examples/code_review.ash`
- Modify: `examples/multi_agent_research.ash`
- Modify: `examples/workflows/40_tdd_workflow.ash`
- Modify: `CHANGELOG.md`

## TDD Steps

1. ✅ Write the failing checklist of touched files that still overclaim canonicality or contain local inconsistencies
2. ✅ Verify RED with focused repository audits
3. ✅ Update the files to the honest canonical/reference state and fix the adjacent local issues
4. ✅ Verify GREEN with fresh focused audits
5. ☐ Commit

## Completion Checklist

- [x] touched user-facing examples no longer overclaim canonical-surface compliance
- [x] touched docs are either canonical enough for the current role contract or clearly framed as historical/reference material
- [x] the undefined `reviewer` reference in `examples/multi_agent_research.ash` is fixed
- [x] approval-role examples remain explicit flat named-role references
- [x] focused repository audits passed
- [x] content review found no remaining blocker or important issues
- [x] `CHANGELOG.md` updated

## Notes

- This task intentionally leaves broader example modernization out of scope. It only brings the touched role docs/examples into an honest canonical-or-reference state for the current convergence branch.
- Closeout status for this task is revalidated in [2026-03-23-role-convergence-closeout-audit.md](../../audit/2026-03-23-role-convergence-closeout-audit.md).

## Non-goals

- No repository-wide example modernization beyond the touched files
- No new governance features
- No reintroduction of role hierarchy
