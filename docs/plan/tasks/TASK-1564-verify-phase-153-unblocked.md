# TASK-1564: Verify Phase 153 unblocked

## Status: 📝 Planned

## Description

Verify that Phase 153 (List Builtin to Stdlib) is unblocked after all parser fixes are applied.

## Specification Reference

- [SPEC-092: Parser Blocker Resolution](../../spec/SPEC-092-PARSER-BLOCKER-RESOLUTION.md)
- [PLAN-156: Parser Blocker Resolution](../PLAN-156-PARSER-BLOCKER-RESOLUTION.md)

## Verification Steps

1. Write `std/src/list.ash` with all list operations using `if`/`match` and variant patterns
2. Run `ash check std/src/list.ash` — must pass
3. Run `cargo test -p ash-cli --test stdlib_corpus_check` — must pass
4. Verify all list operations compile without `builtin fn` declarations

## Acceptance Criteria

- [ ] `std/src/list.ash` compiles with pure Ash implementations
- [ ] All list operations work without `builtin fn` declarations
- [ ] Stdlib corpus check passes
- [ ] Phase 153 can proceed with remaining tasks

## Closeout Checklist

- [ ] Verification complete
- [ ] Phase 153 unblocked
- [ ] PLAN-INDEX updated
- [ ] Committed to branch
