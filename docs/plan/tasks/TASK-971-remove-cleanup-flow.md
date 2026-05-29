# TASK-971: Remove cleanup flow

## Status: ✅ Complete for SPEC-073 alpha remove/cleanup policy

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
  - [x] Implement `ashgrove remove <toolchain-id>` with default/current-working-directory project pin/live-daemon/running-manager protection; broader configured known-project root protection remains deferred.
  - [x] Add or consume minimal `ash daemon` toolchain id/root state under `$XDG_STATE_HOME/ash/daemon/` for live protection.
  - [x] Define which protections `--force` may override and prove it cannot override live daemon or running-manager protection.
  - [x] Implement `ashgrove cleanup --project PATH --dry-run` as a non-destructive planner.
  - [x] Implement conservative cache/orphan/old-toolchain cleanup flags.
  - [x] Add deletion tests using isolated temp roots.
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

2026-05-28 follow-up slice: `ashgrove remove` now refuses toolchains referenced by current-project `ash.toml` pins unless `--force` is passed, refuses toolchains referenced by TOML daemon state under `$XDG_STATE_HOME/ash/daemon/` even with `--force`, and keeps running-manager protection non-overridable. `ashgrove cleanup --project PATH --dry-run` now plans without deletion, protects project pins, and never touches project `ash.toml` or `ash.lock`. `--cache` removes only known Ash-owned cache children under `$XDG_CACHE_HOME/ash`, `--orphans` removes invalid toolchain directories under the toolchain root, and `--old-toolchains` removes installed toolchains only after preserving default, project-pinned, live-daemon, and running-manager toolchains. Broader orphan analysis across lockfile/cache references and cache reachability beyond known Ash-owned cache roots remain deferred by the SPEC-073 alpha boundary.

2026-05-29 completion evidence: Focused regressions in `crates/ashgrove/tests/task_971_remove_cleanup.rs` prove `remove --force` overrides only default/current-project pin protections after explicit stdin confirmation, live-daemon and running-manager protections remain non-overridable, bare `cleanup --project PATH --dry-run` emits only the non-destructive planner and leaves `ash.toml`, `ash.lock`, and toolchains intact, cache/orphan dry-runs leave deletion candidates intact, cache cleanup preserves unknown Ash-cache children and cache siblings outside `$XDG_CACHE_HOME/ash`, orphan cleanup is constrained to invalid toolchain directories under the XDG toolchain root, and `--old-toolchains --project PATH` preserves the supplied project pin while removing only explicitly confirmed unprotected installed toolchains. Combined cleanup flags confirm old-toolchain deletion before any destructive cleanup action. TASK-971 is complete for the SPEC-073 alpha remove/cleanup scope; broader reachability remains a later acceptance/depth slice, not a blocker for this task.
