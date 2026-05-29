# TASK-969: Binary tarball install flow

## Status: ✅ Complete

## Description

Implement `ashgrove install --from tarball` and the conforming binary tarball producer/fixture path.

## Specification Reference

- SPEC-073 §7.2, §10
- PLAN-122 §7 / TASK-969

## Dependencies

- TASK-965 completion.
- TASK-966 command skeleton completion.
- TASK-967 metadata/XDG/staging substrate completion.

## Requirements

### Functional Requirements

1. Define a conforming release tarball shape and produce it through the TASK-965-selected script/command or test fixture helper.
2. Validate tarball directory shape before publish.
3. Validate `manifest.toml` and `install-record.toml` schema.
4. Validate archive version/toolchain id matches the target directory.
5. Validate executable presence/permissions for required binaries.
6. Validate stdlib manifest presence.
7. Reject unsafe archive entries including absolute paths, traversal, symlink/hardlink escapes, device files, and setuid/setgid bits.
8. Record local tarball path/digest/install time in install metadata. Authenticated URL download and URL provenance are deferred to the later release-index/download policy.

### Non-goals

- Do not require signature enforcement in the first slice.
- Do not publish a tarball whose declared version disagrees with the target directory.
- Do not accept unsafe extraction behavior.

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
  - cargo test -p ashgrove task_969 -- --nocapture
  - cargo fmt --check
  - cargo check -p ashgrove
  - git diff --check
checklist:
  - [x] Define a conforming release tarball shape and produce it through the TASK-965-selected script/command or test fixture helper.
  - [x] Validate tarball directory shape before publish.
  - [x] Validate `manifest.toml` and `install-record.toml` schema.
  - [x] Validate archive version/toolchain id matches the target directory.
  - [x] Validate executable presence/permissions for required binaries.
  - [x] Validate stdlib manifest presence.
  - [x] Reject unsafe archive entries including absolute paths, traversal, symlink/hardlink escapes, device files, and setuid/setgid bits.
  - [x] Record tarball path/digest/install time in install metadata.
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

Area: install/semantic. Binary install acceptance requires both producer and consumer evidence.

2026-05-28 follow-up slice: added focused coverage that rejects tarballs whose required `bin/ash` payload is present but lacks executable bits, and made fixture tarballs model executable `ash`/`ashgrove` binaries. TASK-969 remains partial because release tarball production, schema validation, archive-version policy, path/URL recording, and atomic publish are still not implemented.

2026-05-28 tarball validation slice: made fixture tarballs emit the conforming first-slice manifest/install-record shape, validated tarball root shape through typed `manifest.toml` and `install-record.toml` parsing before publish, required the root directory name and metadata toolchain id to match, required the bundled stdlib manifest, preserved executable required-binary validation, routed tarball installs through `ToolchainStage::publish`, rewrote tarball install metadata with local tarball path, digest, and install time, and added focused unsafe-entry coverage for symlink, hardlink, absolute path, parent traversal, and setuid mode rejection. TASK-969 remained partial because the repository `scripts/package-ash-toolchain.sh` release producer and authenticated URL download/recording path were still deferred.

2026-05-29 completion slice: added `scripts/package-ash-toolchain.sh`, which packages a coherent repository Ash toolchain root containing `bin/ash`, `bin/ashgrove`, bundled stdlib metadata/source, `manifest.toml`, `install-record.toml`, and required standard-tool metadata. Tarball manifests and install-record templates now carry `archive_schema_version = 1`, and tarball install validation rejects missing or unsupported archive schema versions before staged publish. Focused integration coverage feeds producer output directly into `ashgrove install --from tarball --path ... --switch` under temporary XDG/home roots and verifies the installed immutable toolchain shape plus local tarball path, digest, and install time recording. Authenticated URL download remains intentionally rejected/deferred by SPEC-073 and is not required for TASK-969 completion.
