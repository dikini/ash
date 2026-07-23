---
id: ref.language.functions.implementation
title: Function Implementation Notes
kind: guide
audience: [human, agent]
authority: canonical-adjacent
status: current
stability: alpha
owner: implementation
last_verified: 2026-05-26
verified_against:
  git_commit: 0874763
  specs:
    - docs/spec/SPEC-027-PURE-FUNCTIONS.md
    - docs/spec/SPEC-031-FIRST-CLASS-FUNCTIONS.md
    - docs/spec/SPEC-071-REFERENCE-CORPUS-METADATA-AND-MAINTENANCE.md
  tasks:
    - docs/plan/tasks/TASK-954-functions-reference-chapter.md
    - docs/plan/tasks/TASK-961-callable-syntax-reference-docs.md
  code:
    - crates/ash-parser/src/parse_module.rs
    - crates/ash-parser/src/parse_expr.rs
    - crates/ash-parser/src/surface.rs
    - crates/ash-parser/src/lower.rs
    - crates/ash-core/src/ast.rs
  tests:
    []
  examples:
    []
related:
  depends_on:
    - ref.language.functions
  explains:
    []
  supersedes: []
  superseded_by: null
  historical_rationale:
    []
refresh_trigger:
  - SPEC-027 changes
  - SPEC-031 changes
  - function parser or typechecker changes
---
# Function Implementation Notes

## Summary

This page explains how the current alpha implementation represents function syntax. It is for implementers and agents, not for ordinary language learning.

## Parser surfaces

Module-level pure functions parse as `Definition::Function(FnDef)` in `crates/ash-parser/src/surface.rs`. The parser entry point is `parse_fn_definition` in `crates/ash-parser/src/parse_module.rs`.

`builtin fn` declarations parse as `Definition::BuiltinFn(BuiltinFnDef)`. They require a return type and a semicolon, and the parser rejects a body.

Anonymous functions and closure shorthand parse in `crates/ash-parser/src/parse_expr.rs`:

- `fn(params) [-> Type] { body }` becomes `Expr::FnDef`.
- `|params| -> expr` desugars immediately to `Expr::FnDef` and typechecks as pure `Type::Fn`.
- Named local functions inside blocks desugar to `BlockStmt::Let` with an `Expr::FnDef` value.
- `-*>`, `=>`, and `=*>` are fail-closed reserved arrows in callable-type and closure-literal contexts; they do not lower to callable representations yet.

## Core and lowering

Lowering maps surface function expressions to core `Expr::FnDef`. Function application uses `Expr::FnApply` for user-defined function values and closure values. `Expr::Call` remains the named-call path used for named functions, builtins, and distinguished runtime primitives.

## Module-level vs local functions

Module-level functions are module definitions. They are collected and imported by module infrastructure. They are not ordinary serialized runtime closure values.

Local functions are expression values. They may capture lexical scope and are subject to the closure boundaries described in SPEC-031.

## Current reference stance

The human-facing pages describe what a reader can write and how to reason about it. This implementation page records why some edges remain alpha-scoped, especially local closure runtime behavior, process/application serialization boundaries, and higher-order use across checked function boundaries.
