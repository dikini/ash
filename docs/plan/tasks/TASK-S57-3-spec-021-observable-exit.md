# TASK-S57-3: Update SPEC-021 with Observable Exit Behavior

## Status: ⬜ Pending

## Description

Update SPEC-021 (Runtime Observable Behavior) with observable semantics for process exit, including what is guaranteed vs implementation-defined regarding spawned descendants.

## Background

Per architectural review, the observable behavior of process exit needs specification:
- What does an external observer see when `ash run` completes?
- What happens to spawned workflows after exit?
- What is testable/verifiable behavior?

## Requirements

Update SPEC-021 with:

1. **Observable exit**: External observer sees process exit with code from `main`
2. **Completion boundary**: `main` completion is the observable event
3. **Descendant opacity**: Spawned workflows are not externally observable after exit
4. **Implementation latitude**: Runtime may terminate, orphan, or continue descendants (implementation-defined)

## SPEC-021 Sections to Update

### Section 2.x: Process Exit Observables (NEW)

Add:

> **Observable Event**: Process termination with exit code
> 
> **Trigger**: Entry workflow (`main`) completion
> 
> **Exit Code Source**: 
> - 0 if `main` returns `Ok(())` with obligations discharged
> - N if `main` returns `Err(RuntimeError { exit_code: N, ... })`
> - 1 for bootstrap/verification errors
> 
> **Non-Observable**: The fate of spawned descendant workflows after process exit is not part of the observable contract.

### Section on Control Authority (reference)

Update references to control authority / completion observation being runtime-internal, not user-observable.

## Open Questions

### Q1: Testing Observable Behavior
- How to test that exit code comes from main, not descendants?
- Need test harness that can spawn and check exit codes
- **Approach:** Spawn workflow that spawns child, child exits with error, parent exits 0; verify exit code is 0

### Q2: Logging/Tracing
- Can descendants log after main completes?
- **Resolution:** Not part of observable contract; implementation-defined

### Q3: Signal Handling
- SIGTERM before main completes?
- **Resolution:** To be defined in future signal-handling spec

## Acceptance Criteria

- [ ] SPEC-021 defines observable exit event
- [ ] SPEC-021 defines exit code source
- [ ] SPEC-021 states descendant behavior is non-observable
- [ ] SPEC-021 provides testable assertions
- [ ] Cross-references to SPEC-004 and SPEC-005

## Related

- SPEC-004: Operational semantics
- SPEC-005: CLI specification
- MCE-001: Entry point design
- TASK-363: Runtime bootstrap (blocked on this)
- TASK-365: Exit code handling (blocked on this)
- TASK-368: Integration tests (blocked on this)

## Est. Hours: 3-4
