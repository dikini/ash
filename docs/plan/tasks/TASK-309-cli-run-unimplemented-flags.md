# TASK-309: Implement ash run --dry-run, --timeout, --capability

## Status: 🔴 Critical - API Mismatch

## Problem

`ash run` advertises several flags that are not actually implemented:

| Flag | Status | Issue |
|------|--------|-------|
| `--dry-run` | ❌ No-op | Still executes workflow |
| `--timeout` | ❌ Ignored | Never read by run() |
| `--capability` | ❌ Ignored | Never read by run() |

This is misleading API design and a SPEC-005 contract gap.

## Root Cause

`crates/ash-cli/src/commands/run.rs`:
- Lines 52-61: Args defined but never used
- Line 79-80: Only `args.trace` is checked
- No timeout implementation
- No dry-run check
- No capability granting

## Files to Modify

- `crates/ash-cli/src/commands/run.rs` - Implement flags
- May need engine changes for timeout support

## Implementation

1. **--dry-run**: Validate workflow but don't execute (parse + type check only)
2. **--timeout**: Add timeout to workflow execution
3. **--capability**: Grant capabilities to engine builder

## Verification

```bash
# Dry run should not execute
$ ash run --dry-run workflow.ash
[check output only, no execution]

# Timeout should interrupt
$ ash run --timeout 5 slow.ash
[exits after 5 seconds if not complete]

# Capability should grant
$ ash run --capability http workflow.ash
[workflow has http capability]
```

## Completion Checklist

- [ ] `--dry-run` validates without executing
- [ ] `--timeout` interrupts execution
- [ ] `--capability` grants to workflow
- [ ] Tests verify each flag
- [ ] CHANGELOG.md updated

**Estimated Hours:** 8
**Priority:** Critical (API contract gap)
