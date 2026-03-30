# TASK-S57-2: Update SPEC-005 with Exit-Immediately CLI Policy

## Status: ⬜ Pending

## Description

Update SPEC-005 (CLI Specification) with the exit-immediately process policy: the OS process exits when the entry workflow (`main`) completes, regardless of spawned descendant workflows.

## Background

Per architectural review, the process exit policy needs explicit specification:
- Does the process wait for all spawned workflows to complete?
- Or does it exit immediately when `main` returns?

**Decision:** Exit immediately when `main` completes. Spawned descendants are not part of process-liveness semantics.

## Requirements

Update SPEC-005 with:

1. **Process lifecycle**: `ash run <file>` creates process, runs `main`, exits on `main` completion
2. **Exit code source**: Derived from `main`'s `Result<(), RuntimeError>`
3. **Descendant policy**: Spawned workflows do not extend process lifetime
4. **Supervisor termination**: System supervisor exits after `main` completion, propagating exit code

## SPEC-005 Sections to Update

### Section 2.x: Process Exit Policy (NEW or update existing)

Add explicit statement:

> The `ash run <file>` command creates an OS process that executes the entry workflow (`main`) and exits immediately upon `main`'s completion. The process exit code is derived from `main`'s return value. The fate of spawned descendant workflows after process exit is outside the CLI contract (implementation-defined).

### Section on Command Syntax (update)

Clarify:
```
ash run <file> [-- <args>...]
```

- `<file>`: Path to `.ash` file containing entry workflow
- `--`: Separator between ash CLI args and program args
- `<args>`: Arguments passed to program via `Args` capability

**Note:** `ash file.ash` (without `run` subcommand) is not supported in minimal core; explicit `ash run` required.

### Section on Exit Codes (update)

Current: SPEC-005 Section 4 "Exit Codes"

Add:
- Exit code 0: `main` returned `Ok(())` with obligations discharged
- Exit code N: `main` returned `Err(RuntimeError { exit_code: N, ... })`
- Exit code 1: Bootstrap or verification error
- Note: Descendant workflow failures do not affect exit code; descendant fate after exit is not part of CLI contract

## Open Questions

### Q1: Orphaned Workflows
- What happens to spawned descendants when process exits?
- **Resolution:** Outside CLI contract; implementation-defined

### Q2: Detach Support
- Should there be `spawn detached` for workflows that outlive main?
- Or is this out of scope for minimal core?

### Q3: Cleanup Guarantees
- Does runtime guarantee any cleanup of spawned workflows?
- **Resolution:** Not part of CLI/observable contract

## Acceptance Criteria

- [ ] SPEC-005 states exit-immediately policy
- [ ] SPEC-005 defines exit code derivation from `main`
- [ ] SPEC-005 clarifies descendant workflow policy
- [ ] SPEC-005 updates exit code documentation
- [ ] Cross-reference to SPEC-004 (completion semantics)

## Related

- SPEC-004: Operational semantics
- SPEC-021: Runtime observable behavior
- MCE-001: Entry point design
- TASK-365: Exit code handling (blocked on this)
- TASK-366: CLI run command (blocked on this)
- TASK-367: CLI error reporting (blocked on this)

## Est. Hours: 2-3
