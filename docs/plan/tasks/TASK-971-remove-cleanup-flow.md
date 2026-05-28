# TASK-971: Remove cleanup flow

## Status: ⚠️ Partial first slice

## Description

Implement conservative remove and cleanup commands.

## Specification Reference

- SPEC-073 §8, §11
- PLAN-122 §7 / TASK-971

## Dependencies

- TASK-965 completion.
- TASK-966 command skeleton completion.
- TASK-967 metadata/XDG/staging substrate completion.
- TASK-970 selector/current flow completion.

## Requirements

### Functional Requirements

1. Implement `ashgrove remove <toolchain-id>` with default/current/project-selected/live-daemon/running-manager protection.
2. Add or consume minimal `ash daemon` toolchain id/root state under `$XDG_STATE_HOME/ash/daemon/` for live protection.
3. Define which protections `--force` may override and prove it cannot override live daemon or running-manager protection.
4. Implement `ashgrove cleanup --project PATH --dry-run` as a non-destructive planner.
5. Implement cache/orphan/old-toolchain cleanup flags.
6. Add deletion tests using isolated temp roots.

### Non-goals

- Do not delete project `ash.toml` or `ash.lock`.
- Do not delete live daemon or running-manager toolchains, even with `--force`.
- Do not crawl the user filesystem for known projects.

## Work Steps

1. Inspect the exact live files named by the task or audit output.
2. Write focused RED tests or docs assertions before changing behavior.
3. Implement or document the minimal target behavior.
4. Run focused verification.
5. Update status surfaces and `CHANGELOG.md` if files beyond tests are changed.
6. Request independent review before marking complete.

## Verification

```yaml
strictness: clean
commands:
  - cargo test -p ashgrove task_971 -- --nocapture
  - cargo test -p ash-cli task_971 -- --nocapture
  - cargo fmt --check
  - cargo check -p ashgrove
  - git diff --check
checklist:
  - [ ] Implement `ashgrove remove <toolchain-id>` with default/current/project-selected/live-daemon/running-manager protection.
  - [ ] Add or consume minimal `ash daemon` toolchain id/root state under `$XDG_STATE_HOME/ash/daemon/` for live protection.
  - [ ] Define which protections `--force` may override and prove it cannot override live daemon or running-manager protection.
  - [ ] Implement `ashgrove cleanup --project PATH --dry-run` as a non-destructive planner.
  - [ ] Implement cache/orphan/old-toolchain cleanup flags.
  - [ ] Add deletion tests using isolated temp roots.
```


## Dispatch

```yaml
agent: codex
reasoning: high
toolsets: [terminal, file]
```

## Dependencies for Next Task

This task contributes to PLAN-122 and SPEC-073 completion. Later tasks must preserve the alpha rules that toolchains are immutable, stdlib is bundled with the selected toolchain, lower-case `ash.toml` is the project manifest, and git dependencies resolve to exact commits in `ash.lock`.


## Notes

Area: lifecycle/semantic. Deletion safety outranks convenience.
