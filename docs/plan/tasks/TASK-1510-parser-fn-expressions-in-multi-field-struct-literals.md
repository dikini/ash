# TASK-1510: Parser supports `fn` expressions and closures in multi-field struct literals

## Status: ✅ Complete

## Description

The Ash parser must accept anonymous `fn` expressions and closure shorthand as field values in struct literals with one or more fields. This unblocks QuickCheck combinator patterns that construct `Strategy<T>` values with function-valued `gen` and `shrink` fields (TASK-1502).

## Root Cause

The live RED suite narrowed the remaining parser blockers to two concrete parser issues:

1. `parse_constructor_fields` consumed the closing `}` while probing for a trailing comma, then the caller tried to consume `}` a second time.
2. Anonymous function expression annotations used `parse_simple_type_name`, which accepted only a bare identifier and left generic suffixes such as `<Int>` unconsumed in return annotations like `-> List<Int>`.

## Verified Working Cases

- ✅ Single-field struct with `fn` expression: `Box { value: fn(x: Int) -> Int { x + 1 } }`
- ✅ `fn` expressions in `let` bindings: `let f = fn(x) -> x + 1;`
- ✅ Returning `fn` expressions from functions
- ✅ Function types as parameters and in type aliases
- ✅ Closure shorthand `|x| -> x + 1` in single-field structs
- ✅ Multi-field struct literals with `fn` expressions and closures
- ✅ Trailing-comma struct literals with `fn` expression fields
- ✅ Generic anonymous function annotations such as `-> List<Int>`

## Previously Failing Cases

- ✅ Generic struct with `fn` expressions: `Strategy<T> { gen: fn(ctx: GenContext) -> T { ... }, shrink: fn(x: T) -> List<T> { [] } }`
- ✅ Trailing-comma record constructor: `Pair { first: fn(...) { ... }, second: fn(...) { ... }, }`
- ✅ Combinator patterns such as `map`, `one_of`, `with_shrink`, `append_shrink`, and `prepend_shrink`.

## Impact

- **TASK-1502 unblocked at parser/check level**: Combinators (`map`, `one_of`, `with_shrink`) can now parse/check when authored as ordinary Ash functions constructing `Strategy<T>` values.
- **Remaining TASK-1502 work**: Replace runner-side/builtin combinator MVPs with ordinary stdlib implementations and verify final-surface behavior.

## Files Changed

- `docs/plan/tasks/TASK-1510-parser-fn-expressions-in-multi-field-struct-literals.md` (this file)
- `docs/plan/PLAN-151-QUICKCHECK-V1-ORDINARY-STRATEGY-SEMANTICS.md` (TASK-1510 status updated)
- `docs/plan/PLAN-INDEX.md` (TASK-1510 status and Phase 151 counts updated)
- `reference/language/functions/local-and-anonymous.md` (removed known-limitation wording)
- `CHANGELOG.md` (entry updated for TASK-1510 fix)
- `crates/ash-parser/src/parse_expr.rs` (trailing-comma probe and generic anonymous-fn annotation parsing fixed)
- `crates/ash-parser/tests/task_1510_fn_expr_struct_literal_regression.rs` (12 parser tests now passing)
- `crates/ash-engine/tests/task_1510_fn_expr_struct_literal_integration.rs` (15 engine checks now passing)

## Test Results

```
Parser regression tests: 12 passed, 0 failed
Engine integration tests: 15 passed, 0 failed
cargo test -p ash-parser: passed
cargo test -p ash-engine: blocked by unrelated TASK-786 import-visibility failures in task_786_import_visibility_summary_rules
```

## Next Steps

1. Implement TASK-1502 combinators as ordinary Ash functions now that the parser/check blocker is fixed.
2. Close Phase 151 after status/reference/changelog reconciliation and broad verification.

## Reproduction

### Supported case (two fields with `fn` expressions)
```ash
type Pair = Pair { first: (Int) -> Int, second: (Int) -> Int };

fn make_pair() -> Pair {
    Pair {
        first: fn(x: Int) -> Int { 42 },
        second: fn(x: Int) -> Int { 43 }
    }
}
```

### Supported case (two fields with closures)
```ash
type Pair = Pair { first: (Int) -> Int, second: (Int) -> Int };

fn make_pair() -> Pair {
    Pair {
        first: |x: Int| -> 42,
        second: |x: Int| -> 43
    }
}
```

### Supported case (single field with `fn` expression)
```ash
type Box = Box { value: (Int) -> Int };

fn make_box() -> Box {
    Box { value: fn(x: Int) -> Int { 42 } }
}
```

### Supported case (two fields, simple values)
```ash
type Pair = Pair { first: Int, second: Int };

fn make_pair() -> Pair {
    Pair { first: 42, second: 43 }
}
```

## Root Cause Analysis

The parser did not need a special field-expression parser. The remaining failures came from delimiter and annotation parsing:

- `parse_constructor_fields` used `literal_str("}").parse_next(input).is_ok()` to detect a trailing comma before a closing brace. That probe consumed the brace, so the subsequent mandatory `}` parse failed. The fix checks `input.input.starts_with('}')` without consuming.
- `parse_simple_type_name` accepted only the leading identifier in anonymous `fn` parameter/return annotations. `-> List<Int>` therefore parsed `List` and left `<Int>` in the stream, causing the surrounding function/constructor parser to fail. The fix consumes balanced generic suffixes into the preserved annotation name.

## Impact

This bug no longer blocks parser/check admission for ordinary Ash QuickCheck combinator definitions. `Strategy<T>` constructors with function-valued `gen` and `shrink` fields now parse and check in the focused TASK-1510 integration suite.

## Requirements

- [x] Fix parser to handle `fn` expressions in multi-field struct literals
- [x] Fix parser to handle closure shorthand in multi-field struct literals
- [x] Add parser regression tests for the failing cases
- [x] Add integration tests verifying the fixed behavior end-to-end
- [x] Confirm relevant specs already describe supported anonymous function/closure syntax
- [x] Update reference docs to show examples of `fn` expressions in struct literals
- [ ] Verify all existing tests still pass (TASK-1510 focused suites and ash-parser pass; full ash-engine currently blocked by unrelated TASK-786 import-visibility failures)
- [x] Update CHANGELOG.md

## TDD Steps

1. **RED**: Parser regression tests reproduced 2 failing cases in `task_1510_fn_expr_struct_literal_regression`.
2. **RED**: Engine integration tests reproduced 3 parse/check failures in `task_1510_fn_expr_struct_literal_integration`.
3. **GREEN**: Fixed delimiter probing and generic anonymous-fn annotation parsing.
4. **GREEN**: Verified all focused parser and engine tests pass.
5. **REFACTOR**: Removed stale unused test helpers and formatted the workspace.

## Completion Checklist

- [x] Parser regression tests added to `crates/ash-parser/tests/`
- [x] Integration tests added to `crates/ash-engine/tests/`
- [x] Parser fix implemented and verified
- [x] All parser tests pass: `cargo test -p ash-parser`
- [ ] All engine tests pass: `cargo test -p ash-engine` (blocked by unrelated TASK-786 import-visibility failures)
- [ ] Full workspace tests pass: `cargo test --workspace`
- [x] Specs checked: SPEC-027, SPEC-031, SPEC-072 need no syntax-limit update for this fix
- [x] Reference docs updated: `ref.language.functions.local-and-anonymous.md`
- [x] CHANGELOG.md updated with entry for this fix
- [x] PLAN-151 status updated
- [x] Task file status marked complete

## Related

- [SPEC-027-PURE-FUNCTIONS.md](../spec/SPEC-027-PURE-FUNCTIONS.md)
- [SPEC-031-FIRST-CLASS-FUNCTIONS.md](../spec/SPEC-031-FIRST-CLASS-FUNCTIONS.md)
- [SPEC-072-TOWER-CALLABLE-TYPE-AND-CLOSURE-SYNTAX.md](../spec/SPEC-072-TOWER-CALLABLE-TYPE-AND-CLOSURE-SYNTAX.md)
- [ref.language.functions.local-and-anonymous.md](../../reference/language/functions/local-and-anonymous.md)
- [TASK-1502](TASK-1502-quickcheck-combinators-recursion-and-weights.md) (parser/check blocker removed by this task)
