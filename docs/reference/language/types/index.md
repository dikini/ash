# Types, Callables, Interfaces, and Implementations

[Language reference](../index.md) · [Status and coverage](../status.md) ·
[Source of truth](../source-of-truth.md)

## Page status

**Reviewed revision:** `423f603c`.

**Implementation:** partial. This chapter records accepted source type forms, checked nominal and
interface boundaries, and selected summary/entry paths. It does not establish that a declaration,
callable value, interface method, or implementation is generally executable.
**Evidence:** tested. See [AUDIT-206](../../../plan/audits/AUDIT-206-implementation-backed-language-reference.md#implementation-census)
rows LANG-008, LANG-009, and LANG-020.
**Parity:** below_spec. The described behavior follows the live parser, type checker, lowering,
and Engine boundaries rather than a target-language design.

## In this chapter

- [Data types, newtypes, callable types, and capability types](data-newtypes-and-callables.md)
  — ordinary declarations, nominal wrappers, current callable arrow spellings, and the narrow
  `capability Name` type route.
- [Generics, kinds, interfaces, and implementations](generics-kinds-interfaces-and-impls.md)
  — checked binder, evidence, arity, closed-world, and summary boundaries.
- [Type-level domains, functions, families, and propositions](type-level-domains-functions-families-and-propositions.md)
  — sealed marker domains, type-function normalization, associated-family boundaries,
  proposition syntax, and the parser-only `data kind` declaration.

## Boundary with type-level computation

`data kind`, `type fn`, sealed type domains/families, associated-family evaluation, and
propositions are not ordinary type-declaration semantics. They have a separate parser/static
route documented by [the type-level chapter](type-level-domains-functions-families-and-propositions.md).
This chapter may mention an associated-type declaration only to explain the shape of an interface;
it does not define type-level computation or create a second grammar for it.

## Current-example boundary

`dtype` has no active source spelling. Historical `Fn(...)` callable notation and the `-*>`,
`=>`, and `=*>` callable arrows are not alternatives to the forms documented here. In
particular, `=>` remains a match-arm delimiter, not a callable arrow. Top-level `capability`
declarations are likewise excluded: the current source form in this chapter is only the type
spelling `capability Name`.

## Related work

Effect rows, resource/role metadata, and the authority boundary are owned by TASK-2050. Handler
signatures and execution are owned by TASK-2051; entry admission and terminal behavior are owned
by TASK-2052. A type, row, interface summary, or entry parameter annotation does not itself grant
authority or install a runtime frame.
