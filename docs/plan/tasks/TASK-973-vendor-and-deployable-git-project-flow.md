# TASK-973: Vendor and deployable git project flow

## Status: ✅ Complete for SPEC-073 alpha offline vendor/deployable git project flow

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
- `task_973_vendor_materializes_every_locked_package_to_default_vendor_root` proves the default `vendor/ash/` layout materializes every locked package from its exact XDG fetched checkout commit and writes package-specific provenance.
- `task_973_vendor_materializes_package_content_from_locked_cache_commit` proves vendored content follows the exact lockfile commit even after the manifest tag is moved.
- `task_973_vendor_explicit_output_records_and_checks_provenance` proves explicit `--output PATH` materializes package content, records provenance, and is validated by `vendor --check`.
- `vendor --check` remains read-only and validates provenance plus vendored file content against the locked cached checkout without fetching or writing.
- `task_973_vendor_check_fails_read_only_when_cache_or_vendor_content_is_missing` proves `vendor --check` fails on missing vendored content and missing fetched-cache evidence without recreating cache directories.
- `check_discovers_locked_vendored_dependency_without_dependency_root_env` proves offline `ash check src/main.ash` resolves a locked dependency from `vendor/ash/<package>/` with only lower-case `ash.toml` and `ash.lock`.
- `run_discovers_locked_vendored_dependency_without_dependency_root_env` proves explicit offline `ash run src/main.ash:main` resolves the same locked vendored dependency without dependency-root environment variables.
- `malformed_lock_package_name_fails_closed_without_resolving_vendor_escape` proves CLI discovery rejects traversal package names before resolving any escaped vendor directory.
- `unlocked_vendor_package_is_not_importable`, `run_does_not_import_unlocked_vendor_package`, `unlocked_top_level_module_inside_locked_package_is_not_importable`, `run_does_not_import_top_level_module_inside_locked_package`, and `explicit_vendor_package_root_does_not_expose_top_level_modules` prove vendored dependency discovery is gated by the first import segment matching a validated locked package name for both `ash check`, explicit ordinary-file `ash run`, and explicit dependency-root package inputs.
- `cli_uses_explicit_stdlib_root_when_vendor_dependency_has_stdlib_module_name` proves an explicit selected stdlib root is searched before the auto-discovered project `vendor/ash` dependency namespace, so a locked vendored package shaped like a stdlib module cannot override the selected stdlib while ordinary locked dependency imports still resolve through `vendor/ash/<package>/`.

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
  - RUSTC_WRAPPER= CARGO_NET_OFFLINE=true cargo test -p ashgrove --test task_973_vendor -- --nocapture
  - RUSTC_WRAPPER= CARGO_NET_OFFLINE=true cargo test -p ash-cli --test phase127_vendored_dependency_resolution -- --nocapture
  - RUSTC_WRAPPER= CARGO_NET_OFFLINE=true cargo test -p ash-engine task_968 -- --nocapture
  - RUSTC_WRAPPER= cargo test -p ashgrove --test task_972_manifest_lock_git -- --nocapture
  - RUSTC_WRAPPER= cargo fmt --check
  - RUSTC_WRAPPER= cargo check -p ashgrove
  - RUSTC_WRAPPER= cargo clippy -p ashgrove --all-targets --all-features -- -D warnings
  - git diff --check
checklist:
  - [x] Materialize locked dependencies into the default `vendor/ash/` directory or explicit `--output PATH`.
  - [x] Record vendor provenance linking each vendored package to a lockfile entry.
  - [x] Implement `ashgrove vendor --check` as read-only validation without writes or network fetches.
  - [x] Add an offline deployment smoke test using a locked git dependency for `ash check`.
  - [x] Verify selected toolchain stdlib remains separate from project dependencies.
  - [x] Verify explicit current CLI forms such as `ash check src/main.ash` and `ash run src/main.ash:main` work with vendored dependencies.
```

## Completion Notes

2026-05-29 completion evidence: TASK-973 is complete for the SPEC-073 alpha offline vendor/deployable git project flow. The verified boundary is locked local-git packages materialized by `ashgrove fetch`, copied into default `vendor/ash/` or an explicit `--output PATH`, provenance checked by read-only `vendor --check`, and consumed offline by `ash check src/main.ash` plus explicit ordinary-file `ash run src/main.ash:main` from the default project vendor layout without dependency-root environment variables or usable XDG fetched cache. SPEC-073 remains Draft for TASK-974 closeout and deferred rows including source-archive release metadata, authenticated URL install policy, packaged dispatcher lifecycle, registry-scale package metadata, broader cleanup reachability, and mandatory trust/signing enforcement.


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
