# TASK-184: Audit Spec Hardening Readiness

## Status: 📝 Planned

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

- [ ] readiness audit created
- [ ] Rust-convergence gate documented
- [ ] Lean-readiness assessment documented
- [ ] `CHANGELOG.md` updated

## Non-goals

- No implementation changes
- No Lean code changes

## Dependencies

- Depends on: TASK-177, TASK-178, TASK-179, TASK-180, TASK-181, TASK-182, TASK-183
- Blocks: TASK-164 through TASK-176
