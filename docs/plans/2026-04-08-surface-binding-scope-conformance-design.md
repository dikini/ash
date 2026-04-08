# Surface Binding Scope Conformance Design

## Goal

Remove the current ambiguity around newline-separated surface statements and variable scope by making the surface-to-core lowering rule normative and aligning parser, lowering, type checking, IR, and interpreter behavior to that one rule.

## Problem

The normative core semantics already distinguish between:

- `LET pat = expr in cont`, which extends the environment only for `cont`
- `SEQ w1 w2`, which sequences workflows without threading a new variable environment into `w2`

What is missing is one normative rule for how a surface block or file consisting of newline-separated statements lowers into canonical core IR. Without that rule, readers can infer two incompatible interpretations:

1. lexical block scope, where earlier `let` bindings remain visible to later statements in the same block
2. independent statement sequencing, where each statement is isolated unless an explicit continuation is present

This ambiguity leaks into type checking and execution because they can both appear locally consistent while disagreeing on the meaning of the same source file.

## Chosen Design

Use lexical-block lowering as the single canonical interpretation.

Normative rule:

- a surface statement list lowers right-associatively into one canonical workflow
- a binding statement lowers to `LET pat = expr in cont`, where `cont` is the lowered remainder of the enclosing block
- a non-binding statement lowers via `SEQ stmt cont`
- a binding introduced by a statement is in scope from that statement to the end of the enclosing block or file

Example:

```ash
let items = [1, 2, 3]
let first = items[0]
emit first
```

lowers canonically to:

```text
LET items = [1,2,3] in
  LET first = items[0] in
    SEQ (emit first) DONE
```

## Why This Option

This is the least controversial normalization because it:

- preserves the existing core meaning of `LET` and `SEQ`
- adds one missing surface-to-core contract instead of changing the operational model
- matches ordinary lexical-scope expectations for examples and user-written workflows
- gives the type checker and interpreter one shared environment story

## Scope

This phase should include:

- primary spec amendments in `docs/spec`
- parser/surface AST updates needed to represent block lowering faithfully
- lowering changes that make lexical block scope canonical
- IR and type-system alignment so the same source block implies the same scope in checking and execution
- interpreter updates needed to execute the canonical lowered form faithfully
- focused conformance tests and examples to prevent regression

This phase should not reopen unrelated semantics such as `Par`, imports, packaging, or broader module resolution.

## Success Criteria

- `docs/spec` contains one explicit normative surface-to-core lowering rule for statement lists and binding scope
- there is no remaining ambiguity between lexical block scope and isolated sequencing for newline-separated statements
- `ash check`, `ash run`, and `ash trace` agree on variable scope for ordinary file workflows
- unbound names are rejected consistently according to the same canonical lowering story
