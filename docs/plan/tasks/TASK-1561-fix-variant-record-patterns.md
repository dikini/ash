# TASK-1561: Fix variant patterns with record payloads

## Status: 📝 Planned

## Description

Fix the parser so that variant patterns with record payloads like `Cons { head: h, tail: rest }` work in `match` expressions.

## Specification Reference

- [SPEC-092: Parser Blocker Resolution](../../spec/SPEC-092-PARSER-BLOCKER-RESOLUTION.md)
- [PLAN-156: Parser Blocker Resolution](../PLAN-156-PARSER-BLOCKER-RESOLUTION.md)

## Problem

```ash
-- FAILS:
match list {
    Nil => [],
    Cons { head: h, tail: rest } => [h]   -- parse error
}
```

## Root Cause

The `parse_variant_pattern` function calls `parse_variant_fields` for record payloads. The field parsing requires `field: pattern` syntax. The issue may be in backtracking or whitespace handling.

## Files to Modify

- `crates/ash-parser/src/parse_pattern.rs` — Fix `parse_variant_pattern` and `parse_variant_fields`

## Verification

- [ ] Parser test: `Cons { head: h, tail: rest }` parses correctly as pattern
- [ ] End-to-end test: `ash check` on `.ash` file with variant patterns
- [ ] All existing parser tests pass

## Closeout Checklist

- [ ] Fix implemented
- [ ] Tests added
- [ ] Tests pass
- [ ] Committed to branch
