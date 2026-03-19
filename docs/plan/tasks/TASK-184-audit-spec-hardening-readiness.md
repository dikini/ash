# TASK-184: Audit Spec Hardening Readiness

## Status: ✅ Complete

## Description

Re-audit the hardened spec set and explicitly decide whether Rust convergence can proceed
mechanically and whether Lean formalization has a stable starting point.

## Specification Reference

- All hardened contracts from TASK-177 through TASK-183

## Requirements

### Functional Requirements

1. Audit whether the specs are unambiguous enough for Rust convergence
2. Audit whether the specs are structured enough for Lean formalization
3. Audit whether the IR contract is execution-model-neutral
4. Make this audit the explicit gate before Rust convergence resumes
5. Confirm that no canonical `catch` construct remains in the hardened language definition

## TDD Evidence

### Red

Before this task, the repository had a hardened spec set but no explicit readiness gate tying the
canonical corpus back to Rust convergence and Lean formalization. The plan index still showed TASK-184
as planned, so the gate itself was not yet materially closed.

### Green

The readiness gate is now explicit:

- [docs/audit/2026-03-19-spec-hardening-readiness-review.md](../../audit/2026-03-19-spec-hardening-readiness-review.md)
  records the audit and gate conclusion
- the audit says Rust convergence may resume under TASK-164 through TASK-176
- the audit says Lean formalization has a stable starting corpus
- no canonical `catch` construct remains in the hardened language definition

## Files

- Create: `docs/audit/2026-03-19-spec-hardening-readiness-review.md`
- Modify: `docs/plan/PLAN-INDEX.md`
- Modify: `CHANGELOG.md`

## TDD Steps

### Step 1: Write the failing checklist (Red)

Check for remaining ambiguity classes relevant to:
- Rust implementation,
- Lean formalization,
- interpreter/JIT neutrality.

### Step 2: Verify RED

Expected failure conditions:
- at least one ambiguity class remains before hardening is complete.

### Step 3: Implement the minimal audit (Green)

Write only the readiness audit and gating conclusion.

### Step 4: Verify GREEN

Expected pass conditions:
- the audit explicitly says whether Rust convergence may proceed.

### Step 5: Commit

```bash
git add docs/audit/2026-03-19-spec-hardening-readiness-review.md docs/plan/PLAN-INDEX.md CHANGELOG.md
git commit -m "docs: audit spec hardening readiness"
```

## Completion Checklist

- [x] readiness audit created
- [x] Rust-convergence gate documented
- [x] Lean-readiness assessment documented
- [x] `CHANGELOG.md` updated

## Non-goals

- No implementation changes
- No Lean code changes

## Dependencies

- Depends on: TASK-177, TASK-178, TASK-179, TASK-180, TASK-181, TASK-182, TASK-183, TASK-185
- Blocks: TASK-164 through TASK-176
