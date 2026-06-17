# TASK-1562: Fix list literal patterns in `match`

## Status: 📝 Planned

## Description

Fix the parser so that list literal patterns like `[h, ..rest]` work in `match` expressions.

## Specification Reference

- [SPEC-092: Parser Blocker Resolution](../../spec/SPEC-092-PARSER-BLOCKER-RESOLUTION.md)
- [PLAN-156: Parser Blocker Resolution](../PLAN-156-PARSER-BLOCKER-RESOLUTION.md)

## Problem

```ash
-- FAILS:
match list {
    [] => [],           -- empty list pattern fails
    [h, ..rest] => [h]  -- list pattern with rest fails
}
```

## Root Cause

The `parse_list_pattern` function exists but may not be reached in the `match` context. The `pattern()` function's `alt` combinator order may need adjustment.

## Files to Modify

- `crates/ash-parser/src/parse_pattern.rs` — Fix `parse_list_pattern` ordering in `alt` chain

## Verification

- [ ] Parser test: `[]` parses correctly as empty list pattern
- [ ] Parser test: `[h, ..rest]` parses correctly as list pattern with rest
- [ ] End-to-end test: `ash check` on `.ash` file with list patterns
- [ ] All existing parser tests pass

## Closeout Checklist

- [ ] Fix implemented
- [ ] Tests added
- [ ] Tests pass
- [ ] Committed to branch
