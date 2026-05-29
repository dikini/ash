# TASK-967: Toolchain metadata and xdg layout

## Status: ✅ Complete

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
3. Implement stable launcher dispatch semantics for explicit override, project pin, user default, and missing-toolchain diagnostics.
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
  - [x] Define typed toolchain manifest and install-record models.
  - [x] Implement XDG data/config/cache/state path resolution with test overrides.
  - [x] Implement stable launcher dispatch semantics for explicit override, project pin, user default, and missing-toolchain diagnostics.
  - [x] Install real user-local `ash` and `ashgrove` launcher shims under the configured home-local bin root using isolated temporary roots in tests.
  - [x] Route installed shims through a stable user-local `.ashgrove-dispatcher` copy instead of embedding the transient `current_exe()` path.
  - [x] Implement user default selector metadata and known-project root metadata.
  - [x] Generate or stage `lib/ash/std/ash.toml` from stdlib/release metadata, or fail explicitly if missing.
  - [x] Preserve reserved trust/signing fields during metadata read-modify-write operations.
  - [x] Implement staging directory publish/rollback helpers without mutating installed toolchains.
  - [x] Implement first-slice toolchain-id parsing/formatting and deterministic already-installed/collision helpers.
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

The metadata/staging slice covers staged publish with implicit temporary-directory cleanup on failure or drop. A later install hardening slice may add an explicit user-facing rollback command if needed.

2026-05-29 completion evidence: `crates/ashgrove::resolve_launcher_dispatch` validates installed toolchain metadata and resolves bundled tools in explicit override, project pin, then user-default order. `crates/ashgrove::install_launcher_shims` installs real stable `ash` and `ashgrove` launcher scripts under the configured home-local bin root, while install/update flows refresh a stable user-local `.ashgrove-dispatcher` copy instead of embedding the transient `std::env::current_exe()` path in shims. The hidden `ashgrove __launcher-dispatch` entrypoint transparently executes the selected immutable tool binary, using Unix `exec` when available and preserving child exit codes on non-Unix. Focused public tests in `crates/ashgrove/tests/task_967_layout.rs` cover project pin precedence, user default fallback, `ASH_TOOLCHAIN` explicit override through a real shim, distinct selected-tool exit-code preservation without wrapper stderr, stable dispatcher-copy shim targets, fail-closed missing/incomplete selected toolchains, selected-root symlink rejection, symlink escape rejection, manifest tool-path traversal rejection, and hardened shim temp-file behavior under fully temporary XDG/home roots. TASK-967 is complete for the metadata/XDG/staging/launcher substrate; SPEC-073 remains Draft because later Phase 127 rows still defer packaged dispatcher lifecycle, authenticated URL installs, and broader closeout acceptance.
