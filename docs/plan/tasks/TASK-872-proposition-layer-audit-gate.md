# TASK-872: Proposition layer audit gate

## Status: 🟡 Ready

## Description

Audit live parser/core/typeck/normalizer/engine seams before any Rust implementation and bind downstream TASK-873 through TASK-882 to exact files/tests/callsites.

## Specification Reference

- [SPEC-064: Constraint and Proposition Layer](../../spec/SPEC-064-CONSTRAINT-PROPOSITION-LAYER.md)
- [PLAN-112: Constraint and Proposition Layer](../PLAN-112-CONSTRAINT-PROPOSITION-LAYER.md)
- [DESIGN-034 §16.8](../../design/DESIGN-034-TOTAL-TYPE-COMPUTATION.md#168-spec-h-constraintproposition-layer)

## Dependencies

- Depends on TASK-871 completion

## Files / Ownership

- Create: `docs/plan/audits/TASK-872-proposition-layer-audit.md`
- Inspect: `crates/ash-parser/src/surface.rs`, `parse_module.rs`, `parse_type_def.rs`, `parse_expr.rs`, `lower.rs`
- Inspect: `crates/ash-core/src/type_ir.rs`, `semantic_summary.rs`, `ast.rs`, `workflow_contract.rs`
- Inspect: `crates/ash-typeck/src/type_env.rs`, `normalizer.rs`, `error.rs`, `diagnostic.rs`
- Inspect: `crates/ash-engine/src/lib.rs`, `module_loader.rs`
- Update or explicitly confirm downstream task bindings in TASK-873 through TASK-882.

## Requirements

### Functional Requirements

1. Create the audit artifact with exact live call graph and owner mapping.
2. Map current WhereBound/interface-bound carriers, equality APIs, disequality seams, normalizer outcomes, semantic-summary versions, parser proposition-syntax gaps, and workflow/runtime constraint non-overlap.
3. Produce a proposition solving/forcing matrix assigning each future site to TASK-873 through TASK-882.
4. Bind downstream tasks to exact source files, exact test targets, callsite/audit-row IDs, and zero-test-safe verification commands.
5. Replace the intentional failing verification guards in TASK-873 through TASK-882 with exact non-zero focused test commands before any implementation task starts.
6. State that no TASK-873+ Rust implementation starts until the binding table is complete and guard replacement is done.

### Non-Goals

- Do not implement type-function inversion, injectivity, unrestricted SMT/proof search, HKT, holes, or partial type-constructor application.
- Do not merge type-level propositions with runtime workflow/capability/provider constraints.
- Do not move semantic proposition ownership into `ash-parser` or `ash-engine`.
- Preserve SPEC-035/SPEC-063 associated-type behavior unless the task explicitly assigns compatibility coverage.

## TDD / Execution Steps

### Step 1

- Inspect live parser/core/typeck/normalizer/engine code.

### Step 2

- Write the audit file with tables: current carriers, gaps, callsites, forcing points, summary seams, diagnostics, non-interference risks.

### Step 3

- Include downstream binding table header: `| Task | Source files | Test targets | Callsite/audit-row IDs | Task-file action |`.

### Step 4

- Patch downstream task files immediately if the audit changes ownership or to replace the intentional failing verification guards.

### Independent Verification

Dispatch an independent review/verification subagent with this task file, SPEC-064, PLAN-112, changed files, and any task-owned audit/test evidence. Do not mark this task complete until findings are fixed and verification passes.

## Completion Checklist

- [ ] Audit artifact exists and has required tables.
- [ ] Every TASK-873 through TASK-882 has a non-empty binding row.
- [ ] No Rust source changes are made by the audit task.

## Dispatch

```yaml
agent: hermes
reasoning: high
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
checklist:
  - "[ ] Task requirements are satisfied"
  - "[ ] Focused verification is recorded"
  - "[ ] Status docs and CHANGELOG.md are updated if release-facing docs changed"
```

## Dependencies for Next Task

This task outputs:
- Phase 116 artifact/surface owned by TASK-872 for downstream tasks.
