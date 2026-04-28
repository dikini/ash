# TASK-755: Comprehension Surface AST and Parser

## Status: 📝 Planned

## References

- [SPEC-055](../../spec/SPEC-055-MONAD-COMPREHENSION-SYNTAX.md) §§4, 11
- [SPEC-054](../../spec/SPEC-054-GENERALIZED-TYPED-DO-NOTATION.md)

## Objective

Add source-fidelity parser/surface support for bracket comprehensions without semantic lowering.

## Files

- Modify: `crates/ash-parser/src/surface.rs`
- Modify: `crates/ash-parser/src/parse_expr.rs`
- Test: `crates/ash-parser/tests/task_755_comprehension_parser.rs` or equivalent parser unit tests

## Requirements

1. Add a comprehension surface expression carrying result expression, qualifier list, optional `DoTarget`, and spans.
2. Add qualifier variants for `x <- expr`, `_ <- expr`, and `let x = expr`.
3. Parse `[result | qualifiers]: K` with comma-separated qualifiers. The `: K` suffix is comprehension-specific target syntax; do not assume a general expression-level type-ascription parser exists.
4. Require at least one qualifier.
5. Preserve existing list/index parsing behavior and parser-state restoration on malformed comprehensions.
6. Do not implement untyped lowering or typed elaboration in this task.

## TDD Steps

1. Add parser tests for explicit-target comprehensions.
2. Add parser tests for multiple qualifiers and mixed `let` / `<-` forms.
3. Add parser negative tests for empty qualifier lists, trailing separators, malformed target annotations, and bare boolean qualifier shapes only to the extent the parser can distinguish/recover them without accepting them as valid qualifiers.
4. Implement the smallest parser/surface changes to pass.
5. Verify no regressions in existing expression/list/index parsing tests.

## Verification Checklist

- [ ] Parser tests fail before implementation.
- [ ] Parser tests pass after implementation.
- [ ] Existing Phase 105 do/act parser tests still pass.
- [ ] `cargo fmt --check` passes.
- [ ] `cargo test -p ash-parser` focused parser tests pass.
- [ ] Independent review confirms parser-state and precedence behavior.
