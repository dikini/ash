# TASK-1563: Add regression tests for parser blockers

## Status: 📝 Planned

## Description

Add comprehensive regression tests for all three parser blockers to prevent future regressions.

## Specification Reference

- [SPEC-092: Parser Blocker Resolution](../../spec/SPEC-092-PARSER-BLOCKER-RESOLUTION.md)
- [PLAN-156: Parser Blocker Resolution](../PLAN-156-PARSER-BLOCKER-RESOLUTION.md)

## Tests to Add

### B1: `if`/`else` with `match`

```rust
#[test]
fn if_else_with_match_parses() {
    let mut input = test_input("if n <= 0 then [] else match list { Nil => [] }");
    let result = parse_fn_expr(&mut input);
    assert!(result.is_ok());
}
```

### B2: Variant patterns with record payloads

```rust
#[test]
fn variant_record_pattern_parses() {
    let mut input = test_input("Cons { head: h, tail: rest }");
    let result = pattern(&mut input);
    assert!(matches!(result, Ok(Pattern::Variant { .. })));
}
```

### B3: List literal patterns

```rust
#[test]
fn list_pattern_parses() {
    let mut input = test_input("[h, ..rest]");
    let result = pattern(&mut input);
    assert!(matches!(result, Ok(Pattern::List { .. })));
}
```

## Files to Modify

- `crates/ash-parser/src/parse_expr/tests.rs` — Add `if`/`else`/`match` tests
- `crates/ash-parser/src/parse_pattern.rs` — Add variant and list pattern tests
- `crates/ash-parser/tests/` — Add end-to-end `.ash` file tests

## Verification

- [ ] All new tests pass
- [ ] All existing tests pass
- [ ] `cargo test -p ash-parser` passes

## Closeout Checklist

- [ ] Tests added for all three blockers
- [ ] Tests pass
- [ ] Committed to branch
