# TASK-970: Update default list current flow

## Status: ⚠️ Partial first slice

## Description

Implement update, default, list, and current toolchain selection flows.

## Specification Reference

- SPEC-073 §9, §10
- PLAN-122 §7 / TASK-970

## Dependencies

- TASK-965 completion.
- TASK-966 command skeleton completion.
- TASK-967 metadata/XDG/staging substrate completion.
- TASK-968 and/or TASK-969 install flow available per TASK-965 update-source decision.

## Requirements

### Functional Requirements

1. Implement `ashgrove list` and `ashgrove current` against selector and install metadata.
2. Implement `ashgrove default <toolchain-id>` as selector update plus launcher behavior.
3. Implement `ashgrove update` as install-new-toolchain behavior using source or tarball input.
4. Prove update does not mutate the previously installed toolchain.
5. Prove `update --switch` changes user default and update without `--switch` preserves user default.
6. Prove first install initializes default only when no default exists.
7. Prove exact toolchain ids are required for selection when multiple installed toolchains share the same package version.

### Non-goals

- Do not update third-party project dependencies from toolchain update commands.
- Do not rewrite project manifests during toolchain update.
- Do not overwrite an installed immutable toolchain in place.

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
  - cargo test -p ashgrove task_970 -- --nocapture
  - cargo fmt --check
  - cargo check -p ashgrove
  - git diff --check
checklist:
  - [ ] Implement `ashgrove list` and `ashgrove current` against selector and install metadata.
  - [ ] Implement `ashgrove default <toolchain-id>` as selector update plus launcher behavior.
  - [ ] Implement `ashgrove update` as install-new-toolchain behavior using source or tarball input.
  - [ ] Prove update does not mutate the previously installed toolchain.
  - [ ] Prove `update --switch` changes user default and update without `--switch` preserves user default.
  - [ ] Prove first install initializes default only when no default exists.
  - [ ] Prove exact toolchain ids are required for selection when multiple installed toolchains share the same package version.
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

Area: lifecycle/semantic. Toolchain update and project dependency update stay separate.
