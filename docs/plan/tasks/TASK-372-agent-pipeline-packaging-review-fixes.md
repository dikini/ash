# TASK-372: Agent Pipeline Packaging Review Fixes

## Status: Complete

## Description

Fix the packaged `tools/agent-pipeline` deployment so its runtime paths, systemd sandbox, CLI queue semantics, and helper scripts are internally consistent and portable across clone locations.

## Requirements

1. Packaged deployment must use explicit environment configuration for workspace root and state dir.
2. The systemd sandbox must permit the repo edits required by the `impl` stage while keeping runtime state under `tools/agent-pipeline/.agents`.
3. `queue --from-spec` must validate the source file before creating any task state.
4. Helper installation/integration scripts must not assume `~/Projects/ash`.
5. Regression coverage must prevent the reviewed failures from recurring.

## TDD Steps

1. Add a failing CLI test proving missing `--from-spec` input leaves no queued bundle.
2. Add failing tests for portable helper/service path generation as needed.
3. Implement the minimum code and script changes to satisfy the tests.
4. Run focused `tools/agent-pipeline` lint and test verification.

## Completion Checklist

- [x] Packaged runtime env is explicit and consistent
- [x] Service sandbox matches actual repo/state write needs
- [x] `queue --from-spec` is non-mutating on invalid input
- [x] Hardcoded clone-location paths removed
- [x] Tests and lint pass
- [x] `CHANGELOG.md` updated
