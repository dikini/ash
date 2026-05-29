# TASK-968: Source install flow

## Status: ✅ Complete source-install-owned slice

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
  - cargo test -p ashgrove --test task_968_source_install -- --nocapture
  - cargo fmt --check
  - cargo check -p ashgrove
  - cargo clippy -p ashgrove --all-targets --all-features -- -D warnings
  - git diff --check
checklist:
  - [x] Build or stage Ash binaries from source according to the TASK-965 audit decision.
  - [x] Copy bundled stdlib and runtime support metadata into the toolchain layout.
  - [x] Record source URL/rev/build profile/target triple plus dirty/unidentified-source override state in install metadata.
  - [x] Reject dirty source installs unless `--allow-dirty-source` is provided and recorded.
  - [x] Reject source archives without commit metadata unless `--allow-unidentified-source` is provided and recorded.
  - [x] Prove installed `ash` uses the selected toolchain stdlib rather than workspace `std/src`.
  - [x] Prove identical source reinstall no-ops or rejects deterministically, and same-version/different-source builds follow the TASK-965 toolchain-id policy.
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

2026-05-28 follow-up slice: source installs now publish through the staged toolchain collision path, reject same-id manifest or source-metadata conflicts, keep identical reinstalls deterministic, and record source URL, source revision, build profile, target triple, dirty/unidentified override flags, reproducibility state, and install time. Prepared source-shaped directory installs remain covered for archive-shaped local inputs.

2026-05-29 review remediation: real local source-root installs build `ash` and `ashgrove` from an isolated cache copy with an external `CARGO_TARGET_DIR`, use `--locked` when the source root has `Cargo.lock`, leave clean source roots without `Cargo.lock` clean, fail closed when a git-like source cannot report `HEAD` or `git status --porcelain`, and add a dirty tree digest to both install metadata and dirty override toolchain IDs so distinct dirty payloads at the same HEAD do not collapse to the same install identity. Launcher dispatch now passes the selected installed stdlib root to `ash` through `ASH_STDLIB_ROOT`.

2026-05-29 completion slice: source-root installs now copy the source stdlib package
metadata from `std/Cargo.toml` into installed `lib/ash/std/ash.toml`, preserve the
bundled runtime stdlib support modules under `lib/ash/std/src/runtime/`, and fail
closed before publish when source stdlib metadata is missing. This completes the
TASK-968-owned source install layout behavior.

Honest deferral boundary: Phase 127 and SPEC-073 remain partial/draft because
source archive release metadata is owned by TASK-977 and concrete runtime-support
payload metadata is owned by TASK-978 after the TASK-976 audit gate. TASK-968 did
not define or overclaim those Phase 128 contracts.
