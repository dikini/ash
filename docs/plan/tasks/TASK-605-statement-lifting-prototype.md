# TASK-605: Statement Lifting and Pipe Operator Prototype

## Status: In Progress

## Description

Prototype the statement lifting and pipe operator (`|>`) design described in `DESIGN-028-STATEMENT-LIFTING.md`. This is an exploratory implementation to validate that effectful computations can appear inside expression argument positions within workflow bodies via ANF lifting, and that the pipe operator desugars correctly into sequential `let` bindings.

## Requirements

1. Add the `|>` token to the lexer (`ash-parser`).
2. Add pipe operator parsing in **workflow expression context only**.
3. Implement partial application support in the typechecker and runtime, so `filter(ends_with(".md"))` evaluates to a closure.
4. Implement the ANF lifting pass that extracts effectful sub-expressions from `let` RHS into synthetic `let` bindings.
5. Ensure `fn` bodies still reject capability calls with a clear error.
6. Write parser and lowering tests covering:
   - `read_dir(path) |> filter(ends_with(".md"))`
   - `filter(ends_with(".md"), read_dir(path))`
   - Nested effectful calls: `read_text(fetch_url(get_env("API")))`

## TDD Steps

1. Add `|>` token and parser test for simple pipe.
2. Implement ANF lifting pass with a test for direct argument lifting.
3. Add partial application to the type representation / runtime.
4. Integrate lifting into the lowering pipeline.
5. Add parser and end-to-end tests for pipe chains.
6. Add rejection test for capability calls inside `fn` bodies.

## Completion Checklist

- [ ] `|>` token exists in the lexer
- [ ] Pipe operator parses in workflow context only
- [ ] Partial application produces `Value::Closure` (or equivalent)
- [ ] ANF lifting pass extracts effectful calls into synthetic `let`s
- [ ] Lifting runs after surface → core lowering
- [ ] Parser/lowering tests pass for all example patterns
- [ ] `cargo check` and `cargo clippy` clean for modified crates
- [ ] Prototype findings documented in a short report

## Related Documents

- `docs/design/DESIGN-028-STATEMENT-LIFTING.md`
- `docs/notes/NOTE-001-WORKFLOW-COMPUTATION-TYPE.md`
- `docs/spec/SPEC-002-SURFACE.md`
