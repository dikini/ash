# TASK-1560: Fix `if`/`else` with `match` in `else` branch

## Status: 📝 Planned

## Description

Fix the parser so that `if` expressions can have `match` in the `else` branch.

## Specification Reference

- [SPEC-092: Parser Blocker Resolution](../../spec/SPEC-092-PARSER-BLOCKER-RESOLUTION.md)
- [PLAN-156: Parser Blocker Resolution](../PLAN-156-PARSER-BLOCKER-RESOLUTION.md)

## Problem

```ash
-- FAILS:
if n <= 0 then []
else match list {        -- parse error
    Nil => [],
    Cons { head: h, tail: rest } => [h]
}
```

## Root Cause

The `parse_fn_if_expr` function in `parse_module/fn_defs.rs` parses `else` branches using `parse_fn_block_or_expr`. After parsing the `then` branch (`[]`), the parser state may not correctly position for parsing `match` in the `else` branch.

## Files to Modify

- `crates/ash-parser/src/parse_module/fn_defs.rs` — Fix `parse_fn_if_expr`

## Verification

- [ ] Parser test: `if n <= 0 then [] else match list { Nil => [] }` parses correctly
- [ ] End-to-end test: `ash check` on `.ash` file with `if`/`else`/`match`
- [ ] All existing parser tests pass

## Closeout Checklist

- [ ] Fix implemented
- [ ] Tests added
- [ ] Tests pass
- [ ] Committed to branch
