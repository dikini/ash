# TASK-873: Core proposition carriers

## Status: 🟡 Ready

## Description

Add core canonical proposition, boundary evidence/refutation/deferred-reason, predicate identity, and V5 semantic-summary carriers without adding solving logic. Solver-private normalized traces remain in `ash-typeck` unless TASK-872 proves they cross a stable boundary.

## Specification Reference

- [SPEC-064: Constraint and Proposition Layer](../../spec/SPEC-064-CONSTRAINT-PROPOSITION-LAYER.md)
- [PLAN-112: Constraint and Proposition Layer](../PLAN-112-CONSTRAINT-PROPOSITION-LAYER.md)
- [DESIGN-034 §16.8](../../design/DESIGN-034-TOTAL-TYPE-COMPUTATION.md#168-spec-h-constraintproposition-layer)

## Dependencies

- Depends on TASK-872 completion

## Files / Ownership

- Modify: `crates/ash-core/src/type_ir.rs` or a new core module chosen by TASK-872
- Modify: `crates/ash-core/src/semantic_summary.rs`
- Modify: `crates/ash-core/src/lib.rs` if exports are added
- Test: exact ash-core test target bound by TASK-872

## Requirements

### Functional Requirements

1. Add typed carriers for equality, disequality, interface-bound, and named-predicate propositions, including a proposition operand carrier that represents sealed-domain constructor applications such as `Cons<A, T>` without nominal-constructor encoding.
2. Add typed outcome/evidence/refutation/deferred carriers only for boundary facts that cross crate/module/cache/summary/stable diagnostic boundaries; leave solver-private normalized traces to `ash-typeck` unless TASK-872 assigns them to core.
3. Add predicate identity/source-anchor carriers.
4. Add V5 semantic-summary version gates for proposition facts.
5. Preserve live `CanonicalTypeExpr` limitations honestly: if `CanonicalTypeExpr` is not extended, add a separate proposition-term carrier rather than pretending it can carry domain constructor apps.
6. Reject older summaries carrying proposition facts before partial registration.
7. Do not encode proposition facts as strings or debug output.

### Non-Goals

- Do not implement type-function inversion, injectivity, unrestricted SMT/proof search, HKT, holes, or partial type-constructor application.
- Do not merge type-level propositions with runtime workflow/capability/provider constraints.
- Do not move semantic proposition ownership into `ash-parser` or `ash-engine`.
- Preserve SPEC-035/SPEC-063 associated-type behavior unless the task explicitly assigns compatibility coverage.

## TDD / Execution Steps

### Step 1

- Write failing ash-core serde/equality/hash/version tests for every carrier.

### Step 2

- Implement minimal carriers and exports.

### Step 3

- Add V5 summary validation tests, including malformed V4-with-proposition-facts rejection.

### Step 4

- Verify no parser/typeck/engine semantics are introduced in ash-core.

### Independent Verification

Dispatch an independent review/verification subagent with this task file, SPEC-064, PLAN-112, changed files, and any task-owned audit/test evidence. Do not mark this task complete until findings are fixed and verification passes.

## Completion Checklist

- [ ] ash-core focused tests pass.
- [ ] Carriers derive Debug, Clone, PartialEq/Eq/Hash/Serialize/Deserialize as appropriate.
- [ ] Summary cache/version tests cover proposition facts.

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
- Phase 116 artifact/surface owned by TASK-873 for downstream tasks.
