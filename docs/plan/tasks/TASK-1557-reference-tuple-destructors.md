# TASK-1557: Reference Tuple Destructors

## Status: ✅ Complete

## Description

Update `reference/language/types/tuples.md` with destructor examples.

## Specification Reference

- [SPEC-091: Let Destructors](../../spec/SPEC-091-LET-DESTRUCTORS.md)
- [PLAN-155: Let Destructors](../PLAN-155-LET-DESTRUCTORS.md)

## Content to Add

- Tuple destructors: `let (a, b) = tuple`
- Order dependence: `(a, b)` different from `(b, a)`
- Position-based binding: first variable = first element
- Length matching: must match tuple arity
- Error conditions: length mismatch, wrong type
- Comparison with element access syntax (`tuple.0`, `tuple.1`)
- Examples from stdlib

## Verification

- Documentation renders correctly
- Examples are accurate and tested
- Cross-references to other docs work
