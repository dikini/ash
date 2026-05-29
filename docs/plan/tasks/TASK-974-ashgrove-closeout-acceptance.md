# TASK-974: Ashgrove closeout acceptance

## Status: ⚠️ Reported first-slice status

## Description

Close out SPEC-073 with acceptance matrix, broad verification, and independent review remediation.

## Specification Reference

- SPEC-073 §20
- PLAN-122 §8-§9

## Dependencies

- TASK-966 through TASK-973 completion.

## Requirements

### Functional Requirements

1. Create an acceptance matrix mapping A73-1 through A73-12 to concrete evidence.
2. Run focused task verification and repo-native broad gates.
3. Update SPEC-073/PLAN-122/PLAN-INDEX/spec index/CHANGELOG status surfaces honestly.
4. Run independent code/spec review and remediate blockers before completion.
5. Confirm no package registry, global install, release-channel resolver, mandatory signing, or independent stdlib update support is overclaimed.

### Non-goals

- Do not mark SPEC-073 Implemented MVP until every acceptance row has evidence or explicit deferral.
- Do not overclaim package signing, registry support, global installs, release-channel discovery, `ashd`, or manifest-aware bare `ash check`.

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
  - bash scripts/check-rust-format.sh
  - bash scripts/check-rust-clippy.sh
  - bash scripts/check-rust-tests.sh --workspace --all-targets
  - bash scripts/check-doc-tests.sh
  - git diff --check
checklist:
  - [ ] Create an acceptance matrix mapping A73-1 through A73-12 to concrete evidence.
  - [ ] Run focused task verification and repo-native broad gates.
  - [ ] Update SPEC-073/PLAN-122/PLAN-INDEX/spec index/CHANGELOG status surfaces honestly.
  - [ ] Run independent code/spec review and remediate blockers before completion.
  - [ ] Confirm no package registry, global install, release-channel resolver, mandatory signing, or independent stdlib update support is overclaimed.
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

Area: closeout/verification. Completion requires acceptance-row evidence, not prose.

2026-05-29 continuation evidence: TASK-973 added a public CLI regression proving selected/explicit stdlib roots are not overridden by the auto-discovered project `vendor/ash` dependency namespace, even when a locked vendored package is shaped like a stdlib module. This strengthens A73-10/A73-12 evidence but does not complete TASK-974 or promote SPEC-073 beyond Draft because launcher-selected installed `ash`, source builds, release packaging, trust preservation, and broad closeout gates remain deferred.

2026-05-29 TASK-967 completion evidence: `cargo test -p ashgrove --test task_967_layout -- --nocapture` now includes 20 passing focused tests, including real temp-root `ash`/`ashgrove` launcher shim installation, shim execution through typed metadata dispatch, explicit `ASH_TOOLCHAIN` override precedence, project pin precedence, user default fallback, stable user-local `.ashgrove-dispatcher` shim targets, transparent selected-tool exit-code preservation, fail-closed missing/incomplete toolchain diagnostics, selected-root symlink rejection, symlink-escape rejection, manifest tool-path traversal rejection, and hardened shim temp-file behavior under temporary XDG/home roots. This completes TASK-967's metadata/XDG/staging/launcher substrate, while SPEC-073 remains Draft for later Phase 127 rows including release packaging and packaged dispatcher lifecycle.

2026-05-29 TASK-969 completion evidence: `scripts/package-ash-toolchain.sh` now produces the first-slice repository Ash toolchain tarball with `bin/ash`, `bin/ashgrove`, bundled stdlib metadata/source, typed `manifest.toml`, typed `install-record.toml`, required standard-tool metadata, and `archive_schema_version = 1`. Focused TASK-969 coverage installs producer output under temporary XDG/home roots and preserves local tarball path, digest, and install time. This completes the local binary tarball producer/install acceptance row while SPEC-073 remains Draft for authenticated URL download/recording, release-index trust policy, packaged dispatcher lifecycle, source-archive release metadata, and broader closeout gates.

2026-05-29 TASK-972 completion evidence: `ash-engine` now derives direct fetched-cache dependency roots from ancestor lower-case `ash.toml` plus `ash.lock`, `$XDG_CACHE_HOME/ash/git/checkouts/<package>-<url-digest>/<commit>/`, and the locked git URL digest without crawling arbitrary directories or requiring dependency-root environment variables. Focused engine and CLI coverage proves `ash check src/main.ash` and explicit ordinary-file `ash run src/main.ash:main` import locked fetched-cache dependencies, fail closed when a checkout is missing or when git `HEAD` differs from the lock commit, and preserve selected/explicit stdlib precedence over stdlib-shaped fetched packages. TASK-972 is complete for the SPEC-073 alpha git lock/fetch/module-root slice; SPEC-073 remains Draft for packaged dispatcher lifecycle, source-archive release metadata, authenticated URL install policy, registry-scale package metadata, manifest rewrite trust preservation, mandatory trust/signing enforcement, and broad closeout gates.
