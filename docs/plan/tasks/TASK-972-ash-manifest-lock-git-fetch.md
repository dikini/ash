# TASK-972: Ash manifest lock git fetch

## Status: ⚠️ Partial second slice

## Description

Implement project `ash.toml`, `ash.lock`, git dependency resolution, fetch, lock checking, trust preservation, and module-loader dependency-root integration.

## Specification Reference

- SPEC-073 §12, §13, §14, §15, §18
- PLAN-122 §7 / TASK-972

## Dependencies

- TASK-965 completion.
- TASK-966 command skeleton completion.
- TASK-967 metadata/XDG/staging substrate completion.

## Requirements

### Functional Requirements

1. Parse lower-case `ash.toml` package/toolchain/dependency metadata needed by SPEC-073.
2. Reject ambiguous package/dependency/toolchain metadata split across `ash.toml` and legacy `.ash.toml`.
3. Resolve git tag dependencies to exact commits in `ash.lock`.
4. Expand accepted abbreviated revs to full commit hashes in `ash.lock` or reject them.
5. Reject unpinned dependencies outside an explicit development override.
6. Preserve reserved trust/signing fields in manifest/lockfile read-modify-write flows.
7. Implement `ashgrove fetch` and `ashgrove lock --check`.
8. Integrate locked dependency roots with `ash-cli`/`ash-engine` module resolution so `ash check` and `ash run` can import fetched dependencies.

### Current Slice Evidence

- `ashgrove fetch` now writes `ash.lock`, clones git dependencies into `$XDG_CACHE_HOME/ash/git/repos/<package>-<url-digest>.git`, and publishes detached checkouts under `$XDG_CACHE_HOME/ash/git/checkouts/<package>-<url-digest>/<commit>/`.
- `task_972_fetch_materializes_exact_lock_commit_in_xdg_cache` proves a moved manifest tag does not change the already-materialized cached dependency root; the checkout content comes from the exact lockfile commit.
- Lockfile commit values consumed by vendoring must be full 40-character hexadecimal commit hashes.
- `ash-cli` still does not discover `ash.toml`/`ash.lock` automatically, so the full `ash check`/`ash run` requirement remains partial.

### Non-goals

- Do not fetch the standard library as a third-party dependency.
- Do not treat tags as execution truth after lock resolution.
- Do not stop at metadata/fetch-only behavior without compiler/module-loader visibility.

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
  - cargo test -p ashgrove task_972 -- --nocapture
  - cargo test -p ash-engine task_972 -- --nocapture
  - cargo fmt --check
  - cargo check -p ashgrove
  - git diff --check
checklist:
  - [ ] Parse lower-case `ash.toml` package/toolchain/dependency metadata needed by SPEC-073.
  - [ ] Reject ambiguous package/dependency/toolchain metadata split across `ash.toml` and legacy `.ash.toml`.
  - [x] Resolve git tag dependencies to exact commits in `ash.lock`.
  - [ ] Expand accepted abbreviated revs to full commit hashes in `ash.lock` or reject them.
  - [ ] Reject unpinned dependencies outside an explicit development override.
  - [ ] Preserve reserved trust/signing fields in manifest/lockfile read-modify-write flows.
  - [x] Implement `ashgrove fetch` and `ashgrove lock --check`.
  - [ ] Integrate locked dependency roots with `ash-cli`/`ash-engine` module resolution so `ash check` and `ash run` can import fetched dependencies.
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

Area: git-deps/semantic. Fetch-only dependency management is not deployment support.
