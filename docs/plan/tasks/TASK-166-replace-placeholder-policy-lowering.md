# TASK-166: Replace Placeholder Policy Lowering

## Status: ✅ Complete

## Description

Replace placeholder policy lowering with the canonical core policy representation.

This task removes dummy or debug-string lowering behavior and establishes meaningful core
policy lowering aligned with the frozen policy contract.

## Specification Reference

- SPEC-001: IR
- SPEC-006: Policy Definitions
- SPEC-007: Policy Combinators

## Reference Contract

- `docs/reference/parser-to-core-lowering-contract.md`

## Requirements

### Functional Requirements

1. Lower canonical policy forms into a meaningful core representation
2. Eliminate placeholder or dummy lowering behavior
3. Add tests proving the canonical lowered structure

## Files

- Modify: `crates/ash-parser/src/lower.rs`
- Modify: `crates/ash-core/src/ast.rs`
- Test: `crates/ash-parser/tests/policy_lowering.rs`
- Test: `crates/ash-core/src/ast.rs`
- Modify: `CHANGELOG.md`

## TDD Steps

### Step 1: Write the failing tests (Red)

Add tests proving canonical policy forms lower into meaningful core policy structures.

### Step 2: Verify RED

Run:

```bash
cargo test -p ash-parser policy_lowering -- --nocapture
```

Expected: fail against current placeholder lowering.

### Step 3: Implement the minimal fix (Green)

Introduce the canonical core policy representation and lower into it.

### Step 4: Verify focused GREEN

Run:

```bash
cargo test -p ash-parser policy_lowering -- --nocapture
cargo test -p ash-core
```

Expected: pass.

### Step 5: Commit

```bash
git add crates/ash-parser/src/lower.rs crates/ash-core/src/ast.rs crates/ash-parser/tests/policy_lowering.rs CHANGELOG.md
git commit -m "fix: replace placeholder policy lowering"
```

## Completion Checklist

- [x] failing policy-lowering tests added
- [x] failure verified
- [x] canonical core policy lowering implemented
- [x] focused verification passing
- [x] `CHANGELOG.md` updated

## Non-goals

- No runtime policy evaluator rewrite
- No type-checker updates yet

## Dependencies

- Depends on: TASK-162, TASK-165
- Blocks: TASK-168, TASK-171
