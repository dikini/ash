# TASK-1860: Define missing-discharge failures

## Description

Define and test fail-closed behavior when no handler/provider can discharge an operation requirement.

## Requirements

- Admission diagnostics must name the missing operation and mention handler/provider discharge.
- CPS raise without a matching handler/provider remains `UnhandledEffect`.
- Tests must cover both admission and runtime missing-discharge paths.

## Completion criteria

- [x] Admission missing-discharge test passes.
- [x] CPS unhandled raise test passes.
- [x] Diagnostics name the operation identity.

## Evidence

- Admission missing-discharge diagnostics name `PosixFs::read`, mention handler/provider frames, and restate that rows do not grant authority. CPS raise without a matching frame returns `CpsError::UnhandledEffect`.

## Depends on

- TASK-1857; TASK-1858.
