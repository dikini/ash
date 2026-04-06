# TASK-413: Canonical Tuple Variant Syntax and ADT Contract Alignment

## Status: ✅ Complete

## Description

Promote `TYPES-001` from an open syntax exploration into a narrow contract-first documentation/spec
slice by fixing one canonical tuple-variant syntax for Ash and aligning the ADT corpus around that
choice.

This task is documentation/spec work first. It should not begin Rust implementation until the
source contract is frozen in the docs corpus.

The canonical syntax chosen by `TYPES-001` is explicit parenthesized tuple payload syntax:

```ash
type RuntimeError = RuntimeError(Int, String);
type Box<T> = Box(T);
match err {
  RuntimeError(code, msg) => msg,
}
```

This task should align the ADT and surface-language docs around that single choice and remove the
remaining sense that tuple variants are still undecided.

## Specification Reference

- [TYPES-001: Canonical Tuple Variant Syntax for ADTs](../../ideas/type-system/TYPES-001-tuple-variants.md)
- [SPEC-020: Algebraic Data Types](../../spec/SPEC-020-ADT-TYPES.md)
- [SPEC-002: Surface Syntax](../../spec/SPEC-002-SURFACE.md)
- [SPEC-003: Type System](../../spec/SPEC-003-TYPE-SYSTEM.md)
- [SPEC-004: Operational Semantics](../../spec/SPEC-004-SEMANTICS.md)
- [Type-to-Runtime Contract](../../reference/type-to-runtime-contract.md)

## Dependencies

- ✅ `TYPES-001` now selects one canonical source syntax
- ✅ Existing ADT source/type/runtime docs already distinguish source contracts from internal elaboration

## Requirements

### Functional Requirements

1. Record explicit parenthesized tuple payload syntax as the canonical tuple-variant source form.
2. Update `SPEC-020` so ADT source definitions recognize three constructor families:
   - unit variants
   - record variants
   - tuple variants
3. Update `SPEC-020` examples to show tuple-variant declarations, constructor expressions, and tuple-variant patterns.
4. Update `SPEC-002` grammar/prose so parser-facing syntax includes tuple-variant declarations and matching tuple-variant constructor/pattern forms.
5. Update `SPEC-003` and/or the type/runtime boundary references so tuple-variant constructor typing and pattern typing are described as positional source contracts rather than named-field-only contracts.
6. Update `SPEC-004` only as needed to acknowledge that tuple-variant source forms exist even if runtime elaboration remains implementation-defined.
7. Update `docs/ideas/README.md` and `docs/ideas/IMPLEMENTABILITY-REPORT.md` so `TYPES-001` is shown as promoted/candidate rather than an unresolved syntax brainstorm.
8. Update `CHANGELOG.md`.

### Non-Functional Requirements

1. Keep this task scoped to docs/spec alignment; do not implement parser/runtime changes here.
2. Do not introduce positional field projection syntax such as `.0` or `.1`.
3. Preserve the distinction between source contract and internal elaboration.
4. Keep record variants and tuple variants as distinct source forms.

## Deliverables

1. `TYPES-001` updated to candidate status with one chosen canonical syntax.
2. Normative docs aligned around that choice.
3. Planning/docs corpus updated so later implementation can proceed from one contract.

## TDD Evidence

### Red

Before this task:

- `TYPES-001` presented multiple syntax options;
- the normative corpus still reads as record-variant-first;
- tuple-variant constructor/pattern contracts are not frozen in one place.

### Green

This task is complete when:

- docs/specs consistently present tuple variants with one canonical parenthesized syntax;
- downstream implementation planning no longer needs to reopen the syntax choice.

## Files

- Modify: `docs/ideas/type-system/TYPES-001-tuple-variants.md`
- Modify: `docs/spec/SPEC-020-ADT-TYPES.md`
- Modify: `docs/spec/SPEC-002-SURFACE.md`
- Modify: `docs/spec/SPEC-003-TYPE-SYSTEM.md`
- Modify: `docs/spec/SPEC-004-SEMANTICS.md`
- Modify: `docs/reference/type-to-runtime-contract.md`
- Modify: `docs/ideas/README.md`
- Modify: `docs/ideas/IMPLEMENTABILITY-REPORT.md`
- Modify: `docs/plan/PLAN-INDEX.md`
- Modify: `CHANGELOG.md`

## Completion Checklist

- [x] `TYPES-001` updated with one canonical syntax decision
- [x] `SPEC-020` aligned to tuple-variant contract
- [x] `SPEC-002` parser-facing syntax aligned
- [x] type/runtime contract docs updated
- [x] ideas/reporting corpus updated
- [x] `PLAN-INDEX.md` updated
- [x] `CHANGELOG.md` updated

## Completion Notes

- Canonical tuple-variant source syntax is now frozen across the ADT, surface, typing, runtime,
  and boundary-reference corpus as `Constructor(T1, T2, ...)` / `Constructor(v1, v2, ...)` /
  `Constructor(p1, p2, ...)`.
- The normative docs now distinguish unit, record, and tuple variants at source level while
  preserving implementation freedom for internal elaboration.
- Some concrete `RuntimeError` examples outside the core TASK-413 corpus still reflect the earlier
  stdlib-visible record-shaped contract. TASK-413 freezes the general tuple-variant source
  contract; follow-on runtime-stdlib reconciliation remains separate work.
- This task intentionally does not implement parser, typechecker, or runtime code.

## Notes

This task is intentionally the documentation/spec precursor to any parser/typechecker/runtime
implementation work. It should freeze the source contract first.
