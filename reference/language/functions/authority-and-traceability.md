---
id: ref.language.functions.authority
title: Function Authority and Traceability
kind: guide
audience: [human, agent]
authority: canonical-adjacent
status: current
stability: alpha
owner: reference-corpus
last_verified: 2026-05-23
verified_against:
  git_commit: 414549f
  specs:
    - docs/spec/SPEC-027-PURE-FUNCTIONS.md
    - docs/spec/SPEC-031-FIRST-CLASS-FUNCTIONS.md
    - docs/spec/SPEC-071-REFERENCE-CORPUS-METADATA-AND-MAINTENANCE.md
  tasks:
    - docs/plan/tasks/TASK-954-functions-reference-chapter.md
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
    - ref.language.functions.implementation
  supersedes: []
  superseded_by: null
  historical_rationale:
    - docs/spec/SPEC-027-PURE-FUNCTIONS.md
    - docs/spec/SPEC-031-FIRST-CLASS-FUNCTIONS.md
refresh_trigger:
  - SPEC-027 changes
  - SPEC-031 changes
  - function parser or typechecker changes
---
# Function Authority and Traceability

## Summary

This chapter is a curated reference projection. The specs and code below remain the authority for implementation work, but ordinary readers should be able to learn function syntax from the chapter pages without replaying design history.

## Primary authority

| Source | Role |
| --- | --- |
| `docs/spec/SPEC-027-PURE-FUNCTIONS.md` | Pure function definition, type, purity, expression, and operational contract. |
| `docs/spec/SPEC-031-FIRST-CLASS-FUNCTIONS.md` | Local closures, anonymous functions, closure shorthand, and module-vs-local distinction. |
| `crates/ash-parser/src/parse_module.rs` | Current parser for module-level `fn` and `builtin fn`. |
| `crates/ash-parser/src/parse_expr.rs` | Current parser for anonymous functions, closure shorthand, calls, and local named functions. |
| `crates/ash-parser/src/surface.rs` | Surface AST carriers for `FnDef` and `BuiltinFnDef`. |
| `crates/ash-parser/src/lower.rs` | Lowering behavior for function expressions and builtins. |
| `crates/ash-core/src/ast.rs` | Core expression and builtin carriers. |

## Reference examples

The examples in this chapter are reference snippets. They are written to show syntax and semantics directly. They are not a claim that every snippet is part of the executable example corpus.

When a snippet needs stronger evidence, add a file under `examples/` and classify it in `reference/examples/README.md` before using it as executable evidence.

## Drift watchlist

- If `extern fn` lands, add a dedicated section instead of folding it into `builtin fn`.
- If local closures become serializable or process-safe, update the local/anonymous and boundary pages together.
- If parameter inference changes, update declarations, local closures, and the agent card.
- If function contracts become a hard daily-use surface, split contract syntax into its own page.

## Maintenance rule

Any task that changes `FnDef`, `BuiltinFnDef`, `Expr::FnDef`, `Expr::FnApply`, pure-function checking, or module-level function import/export should refresh this chapter or record a drift finding in `reference/status/drift-report.md`.
