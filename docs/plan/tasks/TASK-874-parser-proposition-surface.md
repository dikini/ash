# TASK-874: Parser proposition surface

## Status: ✅ Complete

## Description

Add raw parser carriers for audited proposition clauses and explicit named predicate declarations while keeping semantic resolution out of ash-parser.

## Specification Reference

- [SPEC-064: Constraint and Proposition Layer](../../spec/SPEC-064-CONSTRAINT-PROPOSITION-LAYER.md)
- [PLAN-112: Constraint and Proposition Layer](../PLAN-112-CONSTRAINT-PROPOSITION-LAYER.md)
- [DESIGN-034 §16.8](../../design/DESIGN-034-TOTAL-TYPE-COMPUTATION.md#168-spec-h-constraintproposition-layer)

## Dependencies

- Depends on TASK-873 completion

## Files / Ownership

- Modify: `crates/ash-parser/src/surface.rs`
- Modify: `crates/ash-parser/src/parse_module.rs`
- Modify: `crates/ash-parser/src/parse_type_def.rs`
- Modify: `crates/ash-parser/src/parse_expr.rs` only for shared expression/type-token helpers or unsupported-surface diagnostics
- Modify: `crates/ash-parser/src/lower.rs` only for raw pass-through; do not lower type-level propositions into workflow/runtime contracts
- Test: `crates/ash-parser/tests/task_874_proposition_surface.rs`
- Audit rows: H-AUD-PARSE-01, H-AUD-PARSE-02, H-AUD-PARSE-03, H-AUD-PARSE-04, H-AUD-PARSE-05, H-FORCE-02, H-RISK-03

## TASK-872 Binding Notes

- Parser owns raw proposition syntax and spans only: equality, disequality, interface-bound, named-predicate clauses, and `visibility? prop Name<params>;` declarations.
- Preserve legacy impl/interface `where T: Interface` carriers; do not reinterpret capability `where` constraints or fn runtime contracts as type-level propositions.
- Semantic identities, predicate registration, and proposition lowering belong to `ash-typeck::TypeEnv` in later tasks.

## Requirements

### Functional Requirements

1. Add raw proposition clause surface types preserving spans.
2. Parse proposition tails with concrete `where` grammar for `type fn`, `fn`, and `builtin fn` signatures unless TASK-872 records a scoped deferral that patches SPEC-064/PLAN-112 first.
3. Parse explicit named predicate declarations using `visibility? prop Name<params>;` grammar.
4. Preserve legacy impl/interface `where T: Interface` behavior; enable generalized impl/interface proposition tails only if TASK-872 proves the parser migration is safe.
5. Reject proposition clauses at unsupported surfaces with explicit parse/deferred-feature diagnostics.
6. Do not resolve interface names, predicate names, or type expressions semantically in the parser.

### Non-Goals

- Do not implement type-function inversion, injectivity, unrestricted SMT/proof search, HKT, holes, or partial type-constructor application.
- Do not merge type-level propositions with runtime workflow/capability/provider constraints.
- Do not move semantic proposition ownership into `ash-parser` or `ash-engine`.
- Preserve SPEC-035/SPEC-063 associated-type behavior unless the task explicitly assigns compatibility coverage.

## TDD / Execution Steps

### Step 1

- Write parser tests for accepted equality, disequality, interface-bound, and named-predicate clauses.

### Step 2

- Write parser tests for unsupported surfaces and malformed clauses.

### Step 3

- Implement raw carriers and parsing.

### Step 4

- Verify legacy `where T: Interface` impl parsing still matches existing AST expectations.

### Independent Verification

Dispatch an independent review/verification subagent with this task file, SPEC-064, PLAN-112, changed files, and any task-owned audit/test evidence. Do not mark this task complete until findings are fixed and verification passes.

## Completion Checklist

- [x] ash-parser focused tests pass and list non-zero matching tests.
- [x] Surface spans cover operators and predicate names.
- [x] No semantic identities are introduced in parser structs.

## Completion Notes

- Added raw proposition-tail parser surface for `type fn`, `fn`, and `builtin fn` signatures, including equality, disequality, interface-bound, and named-predicate clauses.
- Added explicit `visibility? prop Name<params>;` proposition-predicate declarations without semantic name resolution.
- Preserved legacy `impl`/`interface` `where T: Interface` behavior and kept propositions out of runtime contracts/lowering.
- Added explicit rejection coverage for unsupported top-level and inline-module proposition clauses, including stray `where`, equality/disequality, interface-bound, and standalone named-predicate-shaped clauses.
- Verification: `cargo fmt --check`, `git diff --check`, non-zero focused test listing, `cargo test -p ash-parser --test task_874_proposition_surface` (6/6 passed), `cargo check --workspace`, and independent review approval.

## Dispatch

```yaml
agent: hermes
reasoning: medium
max_turns: 15
toolsets: [terminal, file]
```

## Verification

```yaml
strictness: clean
commands:
  - cargo fmt --check
  - git diff --check
  - cargo check --workspace
  - test -f crates/ash-parser/tests/task_874_proposition_surface.rs
  - cargo test -p ash-parser --test task_874_proposition_surface -- --list | grep -q task_874_
  - cargo test -p ash-parser --test task_874_proposition_surface
checklist:
  - "[x] Task requirements are satisfied"
  - "[x] Focused verification is recorded"
  - "[x] Status docs and CHANGELOG.md are updated if release-facing docs changed"
```

## Dependencies for Next Task

This task outputs:
- Phase 116 artifact/surface owned by TASK-874 for downstream tasks.
