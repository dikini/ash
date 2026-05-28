# TASK-967: Toolchain metadata and xdg layout

## Status: ⚠️ Partial first slice

## Description

Implement metadata schemas, XDG path resolution, launcher dispatch, selectors, stdlib metadata, trust preservation, and atomic staging/publish helpers.

## Specification Reference

- SPEC-073 §6, §8, §9, §12, §16, §17
- PLAN-122 §7 / TASK-967

## Dependencies

- TASK-965 completion.
- TASK-966 command skeleton completion.

## Requirements

### Functional Requirements

1. Define typed toolchain manifest and install-record models.
2. Implement XDG data/config/cache/state path resolution with test overrides.
3. Implement stable launcher dispatch semantics for project pin, user default, and missing-toolchain diagnostics.
4. Implement user default selector metadata and known-project root metadata.
5. Generate or stage `lib/ash/std/ash.toml` from stdlib/release metadata, or fail explicitly if missing.
6. Preserve reserved trust/signing fields during metadata read-modify-write operations.
7. Implement staging directory publish/rollback helpers without mutating installed toolchains.
8. Implement first-slice toolchain-id parsing/formatting and deterministic already-installed/collision helpers.

### Non-goals

- Do not mutate installed toolchain contents after publish.
- Do not depend on global/system install roots.
- Do not silently discard unknown trust fields.

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
  - cargo test -p ashgrove task_967 -- --nocapture
  - cargo fmt --check
  - cargo check -p ashgrove
  - git diff --check
checklist:
  - [ ] Define typed toolchain manifest and install-record models.
  - [ ] Implement XDG data/config/cache/state path resolution with test overrides.
  - [ ] Implement stable launcher dispatch semantics for project pin, user default, and missing-toolchain diagnostics.
  - [ ] Implement user default selector metadata and known-project root metadata.
  - [ ] Generate or stage `lib/ash/std/ash.toml` from stdlib/release metadata, or fail explicitly if missing.
  - [ ] Preserve reserved trust/signing fields during metadata read-modify-write operations.
  - [ ] Implement staging directory publish/rollback helpers without mutating installed toolchains.
  - [ ] Implement first-slice toolchain-id parsing/formatting and deterministic already-installed/collision helpers.
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

Area: metadata/substrate. This task creates the path/metadata substrate consumed by install/update/remove/dependency tasks.
