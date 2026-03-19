# TASK-164: Route Receive Through Main Parser

## Status: 📝 Planned

## Description

Route canonical `receive` syntax through the main workflow parser entrypoint.

This task ensures the main parser dispatches `receive` consistently with the stabilized
surface contract before any lowering or runtime changes.

## Specification Reference

- SPEC-002: Surface Language
- SPEC-013: Streams and Event Processing

## Reference Contract

- `docs/reference/surface-to-parser-contract.md`

## Requirements

### Functional Requirements

1. Accept canonical `receive` syntax from the main workflow parser entrypoint
2. Reuse the dedicated `receive` parser rather than parallel ad hoc handling
3. Add parser tests proving the main entrypoint accepts canonical forms

## Files

- Modify: `crates/ash-parser/src/parse_workflow.rs`
- Modify: `crates/ash-parser/src/parse_receive.rs`
- Test: `crates/ash-parser/tests/receive_parser.rs`
- Modify: `CHANGELOG.md`

## TDD Steps

### Step 1: Write the failing tests (Red)

Add parser tests proving the main workflow parser accepts canonical `receive` forms.

### Step 2: Verify RED

Run:

```bash
cargo test -p ash-parser receive_parser -- --nocapture
```

Expected: fail because the main parser does not dispatch correctly.

### Step 3: Implement the minimal fix (Green)

Wire `receive` into the main parser entrypoint without changing downstream semantics yet.

### Step 4: Verify focused GREEN

Run:

```bash
cargo test -p ash-parser receive_parser -- --nocapture
```

Expected: pass.

### Step 5: Verify broader GREEN

Run:

```bash
cargo test -p ash-parser
```

Expected: pass.

### Step 6: Commit

```bash
git add crates/ash-parser/src/parse_workflow.rs crates/ash-parser/src/parse_receive.rs crates/ash-parser/tests/receive_parser.rs CHANGELOG.md
git commit -m "fix: route receive through main parser"
```

## Completion Checklist

- [ ] failing parser tests added
- [ ] failure verified
- [ ] main parser dispatch fixed
- [ ] focused parser tests passing
- [ ] broader parser verification passing
- [ ] `CHANGELOG.md` updated

## Non-goals

- No lowering changes
- No interpreter/runtime changes

## Dependencies

- Depends on: TASK-161
- Blocks: TASK-167
