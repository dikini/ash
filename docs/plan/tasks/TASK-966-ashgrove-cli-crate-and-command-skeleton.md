# TASK-966: Ashgrove cli crate and command skeleton

## Status: ✅ Complete

## Description

Add the public `ashgrove` command skeleton and shared reporting/error substrate.

## Specification Reference

- SPEC-073 §5
- PLAN-122 §7 / TASK-966

## Dependencies

- TASK-965 completion; audit must choose implementation home and exact focused verification.

## Requirements

### Functional Requirements

1. Add `ashgrove` as the public toolchain/deployment manager command, preferably in a new `crates/ashgrove` workspace member unless TASK-965 chooses otherwise.
2. Implement subcommand parsing for install, update, default, list, current, remove, cleanup, fetch, lock, and vendor.
3. Reject bare version install/update until release-index/channel policy exists.
4. Route incomplete commands to fail-closed not-implemented diagnostics until owning tasks land.
5. Add CLI smoke tests using isolated temp directories and non-zero test assertions.

### Non-goals

- Do not make `ashgrove` a second language execution CLI.
- Do not implement install/update side effects in the skeleton task.
- Do not touch real user XDG directories in tests.

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
  - cargo test -p ashgrove task_966 -- --nocapture
  - cargo fmt --check
  - cargo check -p ashgrove
  - git diff --check
checklist:
  - [x] Add `ashgrove` as the public toolchain/deployment manager command, preferably in a new `crates/ashgrove` workspace member unless TASK-965 chooses otherwise.
  - [x] Implement subcommand parsing for install, update, default, list, current, remove, cleanup, fetch, lock, and vendor.
  - [x] Reject bare version install/update until release-index/channel policy exists.
  - [x] Route incomplete commands to fail-closed not-implemented diagnostics until owning tasks land.
  - [x] Add CLI smoke tests using isolated temp directories and non-zero test assertions.
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

Area: cli/substrate. Keep language execution in `ash`; `ashgrove` manages toolchains/dependencies.
