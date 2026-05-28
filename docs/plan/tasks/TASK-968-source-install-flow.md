# TASK-968: Source install flow

## Status: ⚠️ Partial first slice

## Description

Implement `ashgrove install --from source` for source checkouts/source archives.

## Specification Reference

- SPEC-073 §7.1, §10, §16
- PLAN-122 §7 / TASK-968

## Dependencies

- TASK-965 completion.
- TASK-966 command skeleton completion.
- TASK-967 metadata/XDG/staging substrate completion.

## Requirements

### Functional Requirements

1. Build or stage Ash binaries from source according to the TASK-965 audit decision.
2. Copy bundled stdlib and runtime support metadata into the toolchain layout.
3. Record source URL/rev/build profile/target triple plus dirty/unidentified-source override state in install metadata.
4. Reject dirty source installs unless `--allow-dirty-source` is provided and recorded.
5. Reject source archives without commit metadata unless `--allow-unidentified-source` is provided and recorded.
6. Prove installed `ash` uses the selected toolchain stdlib rather than workspace `std/src`.
7. Prove identical source reinstall no-ops or rejects deterministically, and same-version/different-source builds follow the TASK-965 toolchain-id policy.

### Non-goals

- Do not silently install from dirty or unidentified source.
- Do not publish incomplete toolchain directories.
- Do not accept workspace-only stdlib discovery as release-safe.

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
  - cargo test -p ashgrove task_968 -- --nocapture
  - cargo test -p ash-engine task_968 -- --nocapture
  - cargo fmt --check
  - cargo check -p ashgrove
  - git diff --check
checklist:
  - [ ] Build or stage Ash binaries from source according to the TASK-965 audit decision.
  - [ ] Copy bundled stdlib and runtime support metadata into the toolchain layout.
  - [ ] Record source URL/rev/build profile/target triple plus dirty/unidentified-source override state in install metadata.
  - [ ] Reject dirty source installs unless `--allow-dirty-source` is provided and recorded.
  - [ ] Reject source archives without commit metadata unless `--allow-unidentified-source` is provided and recorded.
  - [ ] Prove installed `ash` uses the selected toolchain stdlib rather than workspace `std/src`.
  - [ ] Prove identical source reinstall no-ops or rejects deterministically, and same-version/different-source builds follow the TASK-965 toolchain-id policy.
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

Area: install/semantic. Source install must be reproducible unless explicitly marked otherwise.
