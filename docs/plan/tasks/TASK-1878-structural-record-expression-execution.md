# TASK-1878: Structural Record Expression Execution

**Status:** Complete
**Plan:** [PLAN-187](../PLAN-187-SURFACE-RECORD-EXPRESSIONS.md)

## Description

Implement structural record expressions as ordinary surface expressions for the function-first target
language.

## Requirements

- Parse bare `{ field: expr, ... }` in expression position as a structural record expression.
- Lower/evaluate structural record expressions to `Value::Record`, evaluating field expressions in
  the existing expression evaluator paths.
- Preserve nominal constructor behavior for `Name { field: expr }`.
- Typecheck record field projection through the existing record type machinery.
- Cover both parser shape and end-to-end engine/CLI execution.

## TDD Steps

1. RED: Add parser coverage for `{ name: "Ada", age: 41 }` as a record expression.
2. RED: Add engine/CLI-facing regression for a function-first `fn main` binding a record expression
   and returning `person.age`.
3. GREEN: Add surface/core AST support, parser branch, lowering, typecheck traversal, and interpreter
   evaluation for record expressions.
4. REGRESSION: Re-run focused parser, engine, CLI, formatting, clippy, and docs gates.

## Completion Checklist

- [x] RED parser evidence captured.
- [x] RED engine/CLI evidence captured.
- [x] GREEN evidence captured.
- [x] Focused regressions pass.
- [x] Specs, indexes, and CHANGELOG updated.

## Evidence

- Initial CLI RED: `cargo run -q -p ash-cli -- check` failed on the target fixture with `parse error: Parsing Error: ContextError`.
- Engine RED: `cargo test -p ash-engine --test task_1878_surface_record_expressions` failed with the same parse boundary before implementation.
- GREEN: `cargo test -p ash-engine --test task_1878_surface_record_expressions` passed after structural record expression parsing, type checking, lowering, and evaluation were implemented.
- Parser regression: `cargo test -p ash-parser parse_structural_record_expression` passed.
- CLI regression: `cargo run -q -p ash-cli -- check`, `cargo run -q -p ash-cli -- run --dry-run`, and `cargo run -q -p ash-cli -- run` passed for the structural record fixture, with execution returning `41`.
