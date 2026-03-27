# TASK-328: Update Example Files to Capabilities Syntax

## Status: 🔴 Critical

## Problem

Example files still use the deprecated `authority:` syntax. TASK-322 implemented SPEC-024 compliant `capabilities:` syntax, but examples were not updated. This creates confusion for users and contradicts the specification.

## Files to Modify

| File | Current Syntax |
|------|----------------|
| `examples/code_review.ash` | `authority: [...]` |
| `examples/multi_agent_research.ash` | `authority: [...]` |
| `examples/04-real-world/customer-support.ash` | `authority: [...]` |
| `examples/04-real-world/code-review.ash` | `authority: [...]` |
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

# All examples parse correctly
find examples tests/workflows -name "*.ash" -exec cargo run --package ash-cli -- check {} \;
```

## Completion Checklist

- [ ] All 6 `.ash` files updated
- [ ] No `authority:` syntax remains (except in documentation comments)
- [ ] Example files parse without errors
- [ ] CHANGELOG.md updated

**Estimated Hours:** 3
**Priority:** Critical (spec compliance, user experience)
**Supersedes/Relates to:** TASK-322 (capabilities syntax implementation)
