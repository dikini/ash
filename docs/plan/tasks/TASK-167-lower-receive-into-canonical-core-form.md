# TASK-167: Lower Receive Into Canonical Core Form

## Status: 📝 Planned

## Description

Lower surface `receive` into the canonical core representation frozen by the convergence docs.

This task completes the parser/lowering half of the `receive` contract before type-checking
and runtime alignment begins.

## Specification Reference

- SPEC-001: IR
- SPEC-013: Streams and Event Processing

## Reference Contract

- `docs/reference/parser-to-core-lowering-contract.md`

## Requirements

### Functional Requirements

1. Add or finalize the canonical core `receive` form
2. Lower surface `receive` into that core form without collapsing semantics
3. Add tests proving canonical lowered structure

## Files

- Modify: `crates/ash-core/src/ast.rs`
- Modify: `crates/ash-parser/src/lower.rs`
- Test: `crates/ash-parser/tests/receive_lowering.rs`
- Test: `crates/ash-core/src/ast.rs`
- Modify: `CHANGELOG.md`

## TDD Steps

### Step 1: Write the failing tests (Red)

Add tests proving surface `receive` lowers into the canonical core representation.

### Step 2: Verify RED

Run:

```bash
cargo test -p ash-parser receive_lowering -- --nocapture
```

Expected: fail because lowering currently collapses `receive`.

### Step 3: Implement the minimal fix (Green)

Add the canonical core `receive` form and lower into it.

### Step 4: Verify focused GREEN

Run:

```bash
cargo test -p ash-parser receive_lowering -- --nocapture
cargo test -p ash-core
```

Expected: pass.

### Step 5: Commit

```bash
git add crates/ash-core/src/ast.rs crates/ash-parser/src/lower.rs crates/ash-parser/tests/receive_lowering.rs CHANGELOG.md
git commit -m "fix: lower receive into canonical core form"
```

## Completion Checklist

- [ ] failing receive-lowering tests added
- [ ] failure verified
- [ ] canonical core receive lowering implemented
- [ ] focused verification passing
- [ ] `CHANGELOG.md` updated

## Non-goals

- No runtime `receive` execution changes
- No scheduler/runtime fairness implementation yet

## Dependencies

- Depends on: TASK-162, TASK-164
- Blocks: TASK-168, TASK-170
