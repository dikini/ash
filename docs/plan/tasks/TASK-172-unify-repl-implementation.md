# TASK-172: Unify REPL Implementation

## Status: 📝 Planned

## Description

Unify the REPL implementation so both entrypoints expose one canonical command surface and one
shared authority.

This task removes REPL duplication debt without adding new REPL features.

## Specification Reference

- SPEC-005: CLI
- SPEC-011: REPL
- SPEC-016: Output Capabilities

## Reference Contract

- `docs/reference/runtime-observable-behavior-contract.md`

## Requirements

### Functional Requirements

1. Make both REPL entrypoints delegate to one canonical implementation
2. Expose the same command surface and handling behavior from both entrypoints
3. Add tests proving the shared authority and canonical command handling

## Files

- Modify: `crates/ash-repl/src/lib.rs`
- Modify: `crates/ash-repl/src/main.rs`
- Modify: `crates/ash-cli/src/commands/repl.rs`
- Test: `crates/ash-repl/tests/repl_commands.rs`
- Test: `crates/ash-cli/tests/repl_command.rs`
- Modify: `CHANGELOG.md`

## TDD Steps

### Step 1: Write the failing tests (Red)

Add tests proving both entrypoints expose the same canonical command surface and command handling behavior.

### Step 2: Verify RED

Run:

```bash
cargo test -p ash-repl repl_commands -- --nocapture
cargo test -p ash-cli repl_command -- --nocapture
```

Expected: fail because the implementations differ today.

### Step 3: Implement the minimal fix (Green)

Choose one shared REPL authority and make both entrypoints delegate to it.

### Step 4: Verify focused GREEN

Run the same commands again.

Expected: pass.

### Step 5: Commit

```bash
git add crates/ash-repl/src/lib.rs crates/ash-repl/src/main.rs crates/ash-cli/src/commands/repl.rs crates/ash-repl/tests/repl_commands.rs crates/ash-cli/tests/repl_command.rs CHANGELOG.md
git commit -m "refactor: unify repl implementation"
```

## Completion Checklist

- [ ] failing REPL authority tests added
- [ ] failure verified
- [ ] shared REPL authority implemented
- [ ] focused verification passing
- [ ] `CHANGELOG.md` updated

## Non-goals

- No new REPL features beyond the canonical spec
- No output-format redesign

## Dependencies

- Depends on: TASK-159, TASK-163
- Blocks: TASK-173
