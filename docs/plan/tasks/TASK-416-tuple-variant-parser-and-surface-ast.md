# TASK-416: Tuple Variant Parser and Surface AST Substrate

## Status: 🟡 Ready

## Description

Implement the first code follow-on after TASK-413 by teaching the parser and surface/core source-AST
layers to represent tuple variants explicitly.

This task should stop at parser and source-AST substrate. It should not yet attempt full lowering,
typechecking, or interpreter support. The goal is to make the canonical tuple-variant source
contract real in the parser-facing implementation surfaces.

Canonical syntax already frozen by TASK-413:

```ash
type RuntimeError = RuntimeError(Int, String);
let err = RuntimeError(2, "missing config");
match err {
  RuntimeError(code, msg) => msg,
}
```

## Specification Reference

- [TASK-413: Canonical Tuple Variant Syntax and ADT Contract Alignment](TASK-413-canonical-tuple-variant-syntax.md)
- [SPEC-020: Algebraic Data Types](../../spec/SPEC-020-ADT-TYPES.md)
- [SPEC-002: Surface Syntax](../../spec/SPEC-002-SURFACE.md)
- [Parser-to-Core Lowering Contract](../../reference/parser-to-core-lowering-contract.md)
- [Surface-to-Parser Contract](../../reference/surface-to-parser-contract.md)

## Dependencies

- ✅ TASK-413 complete

## Requirements

### Functional Requirements

1. Extend the source-level ADT metadata model so enum variants can represent unit, record, and tuple payload shapes explicitly.
2. Extend the parser surface AST so constructor expressions can preserve tuple-constructor shape distinctly from record constructors.
3. Extend the parser surface AST so variant patterns can preserve tuple-pattern shape distinctly from record variant patterns.
4. Update parsing logic for:
   - tuple-variant declarations in type definitions
   - tuple-variant constructor expressions
   - tuple-variant patterns
5. Preserve existing record-variant and unit-variant behavior.
6. Add parser regression tests covering:
   - declaration parsing
   - constructor-expression parsing
   - tuple-pattern parsing
   - nested tuple-pattern parsing
   - rejection of malformed tuple payload syntax

### Non-Functional Requirements

1. Do not add positional projection syntax such as `.0` / `.1`.
2. Do not silently collapse tuple variants into record variants at the parser surface.
3. Keep this task parser/surface-AST scoped; later tasks own lowering/typeck/runtime behavior.
4. Update `CHANGELOG.md`.

## Files

- Modify: `crates/ash-core/src/ast.rs`
- Modify: `crates/ash-parser/src/surface.rs`
- Modify: `crates/ash-parser/src/parse_type_def.rs`
- Modify: `crates/ash-parser/src/parse_expr.rs`
- Modify: `crates/ash-parser/src/parse_pattern.rs`
- Modify: `crates/ash-parser/src/lower.rs` only if required to preserve the new parsed surface shape without full lowering semantics yet
- Add/Modify tests under: `crates/ash-parser/tests/`
- Modify: `CHANGELOG.md`

## TDD Steps

### Step 1: Write failing parser/surface-AST tests

Add tests demonstrating that tuple-variant declarations, constructor expressions, and patterns are parsed into distinct tuple-payload shapes rather than record-field shapes.

### Step 2: Implement source-AST and parser changes

Introduce the minimal payload-shape enums/fields needed to preserve unit vs record vs tuple forms.

### Step 3: Verify parser crate quality

Run at least:
- `cargo test -p ash-parser`
- `cargo clippy -p ash-parser --all-targets -- -D warnings`
- `cargo fmt --check`

## Completion Checklist

- [ ] source AST supports tuple-variant payload shape
- [ ] parser surface AST supports tuple constructors/patterns
- [ ] parser accepts canonical tuple-variant syntax
- [ ] parser tests added/updated
- [ ] `CHANGELOG.md` updated

## Notes

This task is intentionally the parser/surface-AST substrate only. Later tasks should own:
- lowering/internal payload metadata
- typechecking/exhaustiveness
- runtime/interpreter support
