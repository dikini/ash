# TASK-443: Surface Statement List Scoping Spec Amendment

**Status:** ✅ Complete
**Phase:** 68 - Surface Binding Scope Conformance
**Dependencies:** None

## Problem Statement

Newline-separated statement lists currently have no single declared lowering into `LET` versus `SEQ`. This creates an ambiguity in the surface-to-core contract: it is not normatively specified how a surface block like:

```ash
let items = [1, 2, 3]
let first = items[0]
done
```

should be lowered into the core language.

## Missing Normative Rule

The Ash language specification currently lacks a canonical surface-to-core lowering rule for statement lists that establishes lexical scoping. Specifically:

1. **SPEC-002 (Syntax):** Does not define the lowering transformation from surface statement lists to core `LET`/`SEQ` forms.
2. **SPEC-003 (Type System):** Does not document the type-environment consequences of statement list lowering.
3. **SPEC-004 (Semantics) & SPEC-025 (Operational Semantics):** Do not explicitly reference the pre-lowered form and assume statement lists are already in canonical form.

## Expected Resolution

The normative rule should be: surface statement lists lower to nested `LET ... in cont` where binding statements capture the lowered remainder as continuation, and non-binding statements lower via `SEQ stmt cont`.

This establishes that earlier bindings are lexically visible in later statements of the same block.

## Amendments Required

- Add one surface-to-core lowering rule for statement lists in SPEC-002
- Add one type-environment consequence note in SPEC-003
- Add explicit cross-references in SPEC-004 and SPEC-025 stating that surface statement lists must already be lowered to canonical LET/SEQ forms

## Verification

After amendments, all four specs should present one coherent lexical-scope story.

## Implementation Notes

The spec amendments have been completed to establish that:
- Surface statement lists lower canonically into nested `LET ... in cont` structures
- Binding statements capture the lowered remainder as continuation
- Non-binding statements lower via `SEQ stmt cont`
- Earlier bindings are lexically visible in later statements of the same block

This provides the normative foundation for parser, lowering, type checking, and runtime conformance work in subsequent tasks.
