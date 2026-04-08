# TASK-444: Parser And Lowering Lexical Block Normalization

## Status: ⏳ Planned

## Description

Implement the parser/lowering changes that make surface statement lists normalize into one canonical lexical-block form. Binding statements must capture the lowered remainder of the block as continuation, while non-binding statements must sequence against that lowered remainder without inventing an alternate scope model.

## Specification Reference

- [SPEC-002: Syntax](../../spec/SPEC-002-SYNTAX.md)
- [SPEC-004: Operational Semantics](../../spec/SPEC-004-SEMANTICS.md)
- [TASK-443: Surface Statement List Scoping Spec Amendment](TASK-443-surface-statement-list-scoping-spec-amendment.md)

## Dependencies

- ⏳ [TASK-443: Surface Statement List Scoping Spec Amendment](TASK-443-surface-statement-list-scoping-spec-amendment.md)

## Requirements

1. Parse or normalize newline-separated surface statements into one canonical workflow form.
2. Lower binding statements into nested `LET ... in cont`.
3. Lower non-binding statements into `SEQ stmt cont`.
4. Add focused parser/lowering tests proving the canonical normalized form for representative blocks.
5. Preserve existing explicit core `SEQ` semantics.

## TDD Steps

### Red

- Today a surface statement list can be interpreted in a way that leaves later statements outside the scope of earlier `let` bindings.

### Green

- Parser/lowering outputs one canonical lexical-block form for ordinary files and blocks.
- Regression tests lock that normalized shape in place.

## Completion Checklist

- [ ] Parser/lowering tests fail before implementation and pass after
- [ ] Binding statements lower to continuation-owned scope
- [ ] Non-binding statements still lower via `SEQ`
- [ ] `CHANGELOG.md` records the parser/lowering normalization
