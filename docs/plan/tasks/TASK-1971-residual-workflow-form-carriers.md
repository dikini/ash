# TASK-1971: Residual Workflow-Form Carrier Removal

**Status:** Planned
**Phase:** [PLAN-201 Semantic Cleanup Follow-up](../PLAN-201-SEMANTIC-CLEANUP-FOLLOWUP.md)
**Source audit:** [AUDIT-201 Semantic Removal Vs Rename](../audits/AUDIT-201-semantic-removal-vs-rename.md)

## Description

Remove or confine residual workflow-form parser and lowering carriers that are not needed for
current target Ash contracts. Target function contracts should lower directly through
contract/evidence helpers rather than old declaration adapters.

## Requirements

- Identify residual workflow-form parser/lowering carriers still reachable from target contract
  paths.
- Prove current `requires` and `ensures` contract paths do not need removed workflow declaration
  adapters.
- Delete or rewrite workflow-form-only tests.
- Add absence tests or gates so removed declaration adapters cannot re-enter active lowering paths.
- Preserve current target function contract parsing, lowering, checking, and engine metadata.

## TDD Steps

1. Add or tighten tests that expose residual workflow-form carrier use in parser/lowering paths.
2. Prove target function contract events lower directly through contract/evidence helpers.
3. Remove or confine stale workflow-form carriers and rewrite affected tests.
4. Run parser, typechecker, engine metadata, Phase 201 gate, and docs/index checks.

## Completion Checklist

- [ ] Workflow-form-only parser/lowering carriers are removed or confined as private historical
      substrate.
- [ ] Target function `requires`/`ensures` paths lower without removed declaration adapters.
- [ ] Workflow-form-only tests are deleted or rewritten to target contract/evidence behavior.
- [ ] Phase 201 removal gates cover the stale carriers.
- [ ] `CHANGELOG.md`, AUDIT-201, and relevant plan evidence are updated.
