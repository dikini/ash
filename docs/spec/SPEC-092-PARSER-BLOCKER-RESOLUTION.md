# SPEC-092: Parser Blocker Resolution for List Migration

**Status:** Implemented MVP (Phase 156)
**Date:** 2026-06-17
**Amends:** [SPEC-089](SPEC-089-LIST-BUILTIN-TO-STDLIB.md) (List Builtin to Stdlib)
**Plan:** [PLAN-156](../plan/PLAN-156-PARSER-BLOCKER-RESOLUTION.md)
**Blocked Phase:** [PLAN-153](../plan/PLAN-153-LIST-BUILTIN-TO-STDLIB.md)

## 1. Summary

Phase 153 (List Builtin to Stdlib) is blocked on three parser issues that prevent writing idiomatic list operations in pure Ash. This spec defines the fixes needed in the parser to unblock Phase 153.

## 2. Blockers

### B1: `if`/`else` with `match` in `else` branch

**Problem:** The parser fails when `else` is followed by `match` in an `if` expression.

```ash
-- FAILS:
if n <= 0 then []
else match list {        -- parse error
    Nil => [],
    Cons { head: h, tail: rest } => [h]
}

-- WORKS:
if n == 0 then 1 else 2   -- simple if/else works
```

**Root Cause:** The `parse_fn_if_expr` function parses `else` branches using `parse_fn_block_or_expr`. After parsing the `then` branch (`[]`), the parser state may not correctly position for parsing `match` in the `else` branch.

**Fix:** Ensure `parse_fn_if_expr` correctly handles `else` followed by `match` by properly managing parser state between branches.

### B2: Variant patterns with record payloads

**Problem:** The parser fails on variant patterns with record payloads like `Cons { head: h, tail: rest }`.

```ash
-- FAILS:
match list {
    Nil => [],
    Cons { head: h, tail: rest } => [h]   -- parse error
}

-- WORKS:
match list {
    Nil => [],
    x => x   -- variable pattern works
}
```

**Root Cause:** The `parse_variant_pattern` function calls `parse_variant_fields` for record payloads. The field parsing may fail due to incorrect handling of nested patterns or whitespace.

**Fix:** Verify `parse_variant_fields` correctly handles `field: pattern` syntax and ensure `parse_variant_pattern` backtracks correctly on failure.

### B3: List literal patterns in `match`

**Problem:** The parser fails on list patterns like `[h, ..rest]` or even `[h]`.

```ash
-- FAILS:
match list {
    [] => [],           -- empty list pattern fails
    [h, ..rest] => [h]  -- list pattern with rest fails
}

-- WORKS:
match list {
    Nil => [],
    Cons { head: h, tail: rest } => [h]   -- variant pattern (when fixed)
}
```

**Root Cause:** The `parse_list_pattern` function exists but may not be reached in the `match` context. The `pattern()` function's `alt` combinator may consume input before reaching `parse_list_pattern`.

**Fix:** Ensure `parse_list_pattern` is correctly ordered in the `alt` chain and that `[]` is not consumed by earlier parsers (like `parse_variant_pattern` for unit variants).

## 3. Acceptance Criteria

### C92-1: `if`/`else` with `match`

```ash
pub fn take(n: Int, list: List<Int>) -> List<Int> {
    if n <= 0 then []
    else match list {
        Nil => [],
        Cons { head: h, tail: rest } => [h, ..take(n - 1, rest)]
    }
}
```

Must parse and typecheck successfully.

### C92-2: Variant patterns with record payloads

```ash
match list {
    Nil => [],
    Cons { head: h, tail: rest } => [h]
}
```

Must parse and typecheck successfully.

### C92-3: List literal patterns

```ash
match list {
    [] => [],
    [h, ..rest] => [h],
    [h] => [h]
}
```

Must parse and typecheck successfully.

### C92-4: All existing tests pass

`cargo test -p ash-parser` and `cargo test -p ash-cli --test stdlib_corpus_check` must pass.

## 4. Files to Modify

| File | Change |
|------|--------|
| `crates/ash-parser/src/parse_module/fn_defs.rs` | Fix `parse_fn_if_expr` for `else match` |
| `crates/ash-parser/src/parse_pattern.rs` | Fix `parse_variant_pattern` and `parse_variant_fields` |
| `crates/ash-parser/src/parse_pattern.rs` | Fix `parse_list_pattern` ordering in `alt` chain |

## 5. Verification Strategy

1. Add parser tests for each blocker scenario
2. Add end-to-end tests with `ash check` on `.ash` files
3. Verify all existing parser tests still pass
4. Verify stdlib corpus check passes

## 6. Relationship to Other Specs

| Spec | Relationship |
|------|-------------|
| SPEC-089 | Unblocks: List Builtin to Stdlib |
| SPEC-031 | Consistent: if/match expressions in fn bodies |

## 7. Closeout Criteria

- [ ] C92-1 through C92-4 all pass
- [ ] Parser tests added for each blocker
- [ ] PLAN-156 and PLAN-INDEX updated
- [ ] Phase 153 unblocked (can proceed with list.ash implementation)
