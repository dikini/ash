# TASK-328: Update Example Files to Capabilities Syntax

## Status: 🔴 Critical

## Problem

Example and workflow fixture files still use the deprecated `authority:` syntax. TASK-322 implemented SPEC-024 compliant `capabilities:` syntax, but several user-facing `.ash` files were not updated. This creates confusion for users and contradicts the specification.

## Files to Modify

| File | Current Syntax |
|------|----------------|
| `examples/code_review.ash` | `authority: [...]` |
| `examples/multi_agent_research.ash` | `authority: [...]` |
| `examples/03-policies/01-role-based.ash` | `authority: [...]` |
| `examples/03-policies/02-time-based.ash` | `authority: [...]` |
| `examples/04-real-world/customer-support.ash` | `authority: [...]` |
| `examples/04-real-world/code-review.ash` | `authority: [...]` |
| `examples/workflows/40_tdd_workflow.ash` | `authority: [...]` |
| `examples/workflows/40a_tdd_concrete_example.ash` | `authority: [...]` |
| `tests/workflows/code_review.ash` | `authority: [...]` |
| `tests/workflows/multi_agent_research.ash` | `authority: [...]` |

## Transformation Pattern

**Before:**
```ash
role drafter {
  authority: [read_code, create_pr, respond_to_comments],
  obligations: [ensure_tests_pass]
}
```

**After:**
```ash
role drafter {
  capabilities: [read_code, create_pr, respond_to_comments]
  obligations: [ensure_tests_pass]
}
```

Note: Remove trailing commas inside capability lists (Ash syntax convention).

## Verification

```bash
# No authority: should remain
grep -r "authority:" --include="*.ash" .
# Expected: No output (or only in comments if documenting the change)

# Parser validation covers the migrated files
cargo test --package ash-parser --quiet
```

Note: `ash check` is not sufficient verification for this task because the current CLI path does not accept every module-shaped example surface. Validation must use parser coverage that exercises the full source form used by these files.

If the current parser tests do not already load these files directly, add or extend a regression test in `ash-parser` that parses each migrated file through the module/program parser.

## Completion Checklist

- [ ] All affected example/workflow `.ash` files updated
- [ ] No `authority:` syntax remains (except in documentation comments)
- [ ] Parser validation covers the migrated files
- [ ] CHANGELOG.md updated

**Estimated Hours:** 3
**Priority:** Critical (spec compliance, user experience)
**Supersedes/Relates to:** TASK-322 (capabilities syntax implementation)
