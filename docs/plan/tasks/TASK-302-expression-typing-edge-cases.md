# TASK-302: Expression Typing Edge Cases

## Status: 📝 Planned

## Description

Add comprehensive edge case tests for the expression typing fixes implemented in TASK-291. The unary/binary operator type checking was fixed to return proper errors instead of fresh type variables, but we need exhaustive edge case coverage.

## Fixes from TASK-291

- **Unary `!` (Not)**: Now requires `Bool` type
- **Unary `-` (Neg)**: Now requires `Int` type
- **Binary arithmetic**: Both operands must be `Int`
- **Binary logical**: Both operands must be `Bool`
- **Binary comparison**: Operands must have matching types

## Missing Edge Cases

### Nested Unary Operators
- `!!true` → should pass (Bool)
- `--5` → should pass (Int)
- `!-5` → should error (negation returns Int, ! expects Bool)
- `-!true` → should error (! returns Bool, negation expects Int)

### Chained Binary Operations
- `1 + 2 + 3` → should pass (Int)
- `"a" + "b" + "c"` → should pass (String concatenation)
- `1 + "a"` → should error (type mismatch)

### Mixed Operations
- `1 + 2 * 3` → should pass (Int)
- `true && false || true` → should pass (Bool)
- `1 < 2 && 3 > 4` → should pass (Bool comparison, then Bool logical)
- `1 + 2 < 3 * 4` → should pass (Int arithmetic, then comparison)

### Error Recovery
- `1 + "a" + 2` → should error at `"a"`, but continue checking
- Type errors in subexpressions shouldn't prevent checking outer expressions

## Test Requirements

1. **Property Tests**: Use `proptest` to generate random expressions and verify:
   - Well-typed expressions don't produce type errors
   - Ill-typed expressions produce specific error types

2. **Specific Error Types**: Verify exact error variants:
   - `ExpectedBool { actual, span }`
   - `ExpectedNumeric { actual, span }`
   - `OperatorTypeMismatch { operator, left, right, span }`

3. **Span Accuracy**: Error spans should point to the problematic operator/operand

## Files to Modify

- `crates/ash-typeck/tests/expression_typing_soundness_test.rs` - Add edge cases

## Completion Checklist

- [ ] Nested unary operator tests
- [ ] Chained binary operation tests
- [ ] Mixed operation precedence tests
- [ ] Error recovery tests
- [ ] Property-based tests for expression typing
- [ ] All new tests pass
- [ ] `cargo clippy` clean
