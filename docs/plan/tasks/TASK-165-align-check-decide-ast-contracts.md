# TASK-165: Align Check and Decide AST Contracts

## Status: ✅ Complete

## Description

Align core and surface AST handling for canonical `check` and `decide` contracts.

This task brings parser-visible AST structures into line with the frozen workflow-form
contracts before policy lowering or runtime changes.

## Specification Reference

- SPEC-001: IR
- SPEC-002: Surface Language

## Reference Contract

- `docs/reference/surface-to-parser-contract.md`
- `docs/reference/parser-to-core-lowering-contract.md`

## Requirements

### Functional Requirements

1. Make `check` obligation-only in core and surface AST handling
2. Make `decide` always carry an explicit named policy
3. Add tests proving canonical AST shapes for both forms

## Files

- Modify: `crates/ash-core/src/ast.rs`
- Modify: `crates/ash-parser/src/surface.rs`
- Modify: `crates/ash-parser/src/parse_workflow.rs`
- Test: `crates/ash-core/src/ast.rs`
- Test: `crates/ash-parser/tests/workflow_contracts.rs`
- Modify: `CHANGELOG.md`

## TDD Steps

### Step 1: Write the failing tests (Red)

Add tests proving `check` and `decide` surface forms produce canonical AST shapes.

### Step 2: Verify RED

Run:

```bash
cargo test -p ash-parser workflow_contracts -- --nocapture
```

Expected: fail due to current contract mismatch.

### Step 3: Implement the minimal fix (Green)

Update core and surface AST definitions and parser handling so both forms match the canonical contract.

### Step 4: Verify focused GREEN

Run:

```bash
cargo test -p ash-parser workflow_contracts -- --nocapture
cargo test -p ash-core
```

Expected: pass.

### Step 5: Commit

```bash
git add crates/ash-core/src/ast.rs crates/ash-parser/src/surface.rs crates/ash-parser/src/parse_workflow.rs crates/ash-parser/tests/workflow_contracts.rs CHANGELOG.md
git commit -m "fix: align check and decide ast contracts"
```

## Completion Checklist

- [x] failing AST contract tests added
- [x] failure verified
- [x] canonical AST shapes implemented
- [x] focused verification passing
- [x] `CHANGELOG.md` updated

## Non-goals

- No policy lowering changes
- No runtime behavior changes

## Dependencies

- Depends on: TASK-161, TASK-162
- Blocks: TASK-166
