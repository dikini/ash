# TASK-874: Parser proposition surface

## Status: 🟡 Ready

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
- Modify: parser files bound by TASK-872, expected candidates include `parse_module.rs` and `parse_type_def.rs`
- Modify: `crates/ash-parser/src/lower.rs` only for raw pass-through where audited
- Test: exact ash-parser test target bound by TASK-872

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

- [ ] ash-parser focused tests pass and list non-zero matching tests.
- [ ] Surface spans cover operators and predicate names.
- [ ] No semantic identities are introduced in parser structs.

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
  - |
    python3 - <<'PY'
    raise SystemExit('TASK-872 must replace this intentional verification guard with exact non-zero focused test commands before implementation can be verified')
    PY
checklist:
  - "[ ] Task requirements are satisfied"
  - "[ ] Focused verification is recorded"
  - "[ ] Status docs and CHANGELOG.md are updated if release-facing docs changed"
```

## Dependencies for Next Task

This task outputs:
- Phase 116 artifact/surface owned by TASK-874 for downstream tasks.
