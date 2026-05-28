# TASK-973: Vendor and deployable git project flow

## Status: ⚠️ Partial second slice

## Description

Implement vendor/offline deployment for git-pinned Ash projects.

## Specification Reference

- SPEC-073 §15
- PLAN-122 §7 / TASK-973

## Dependencies

- TASK-965 completion.
- TASK-966 command skeleton completion.
- TASK-967 metadata/XDG/staging substrate completion.
- TASK-972 manifest/lock/fetch/module-root integration completion.

## Requirements

### Functional Requirements

1. Materialize locked dependencies into the default `vendor/ash/` directory or explicit `--output PATH`.
2. Record vendor provenance linking each vendored package to a lockfile entry.
3. Implement `ashgrove vendor --check` as read-only validation without writes or network fetches.
4. Add an offline deployment smoke test using a locked git dependency.
5. Verify selected toolchain stdlib remains separate from project dependencies.
6. Verify explicit current CLI forms such as `ash check src/main.ash` and `ash run src/main.ash:main` work with vendored dependencies.

### Current Slice Evidence

- `ashgrove vendor` now requires locked dependencies to have been materialized by `ashgrove fetch` and copies package content from `$XDG_CACHE_HOME/ash/git/checkouts/<package>-<url-digest>/<commit>/` into `vendor/ash/<package>/`.
- `task_973_vendor_materializes_package_content_from_locked_cache_commit` proves vendored content follows the exact lockfile commit even after the manifest tag is moved.
- `vendor --check` remains read-only and validates provenance plus vendored file content against the locked cached checkout without fetching or writing.
- `check_discovers_locked_vendored_dependency_without_dependency_root_env` proves offline `ash check src/main.ash` resolves a locked dependency from `vendor/ash/<package>/` with only lower-case `ash.toml` and `ash.lock`.
- `run_discovers_locked_vendored_dependency_without_dependency_root_env` proves explicit offline `ash run src/main.ash:main` resolves the same locked vendored dependency without dependency-root environment variables.
- `malformed_lock_package_name_fails_closed_without_resolving_vendor_escape` proves CLI discovery rejects traversal package names before resolving any escaped vendor directory.

### Non-goals

- Do not invent registry semantics.
- Do not require network access for vendor-check tests after materialization.
- Do not assume manifest-aware bare `ash check` or `ash run <entry>` until a later CLI spec adds it.

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
  - cargo test -p ashgrove task_973 -- --nocapture
  - cargo fmt --check
  - cargo check -p ashgrove
  - git diff --check
checklist:
  - [x] Materialize locked dependencies into the default `vendor/ash/` directory or explicit `--output PATH`.
  - [x] Record vendor provenance linking each vendored package to a lockfile entry.
  - [x] Implement `ashgrove vendor --check` as read-only validation without writes or network fetches.
  - [x] Add an offline deployment smoke test using a locked git dependency for `ash check`.
  - [ ] Verify selected toolchain stdlib remains separate from project dependencies.
  - [x] Verify explicit current CLI forms such as `ash check src/main.ash` and `ash run src/main.ash:main` work with vendored dependencies.
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

Area: deployment/semantic. Offline deployment must use lockfile/vendor evidence, not ambient caches.
