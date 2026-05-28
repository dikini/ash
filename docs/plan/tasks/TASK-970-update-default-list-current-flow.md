# TASK-970: Update default list current flow

## Status: ⚠️ Partial local/source/tarball selector slice

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

### Implemented slice

- `ashgrove list` reports only installed toolchains with valid manifest/install metadata and marks the selector default.
- `ashgrove current` reads typed selector metadata and fails closed when the selected default or project pin is not an installed metadata-valid toolchain.
- `ashgrove default <toolchain-id>` validates an installed exact toolchain id before updating selector metadata, with an exact-id diagnostic for package-version-only selection when multiple installed immutable ids share that version.
- `ashgrove update --from source --path PATH --to TOOLCHAIN_ID` and `ashgrove update --from tarball --path PATH --to TOOLCHAIN_ID` install the local fixture-backed source/tarball payload only when the payload identity matches `--to`.
- `update --switch` changes the default, update without `--switch` preserves an existing default, and first update install initializes the default when none exists.
- Focused tests prove the old toolchain manifest and install record are unchanged after installing a new immutable source fixture.

### Remaining limitations

- Bare `ashgrove update VERSION` and network/release-index discovery remain intentionally rejected until an authenticated release-index policy exists.
- Source update still uses the TASK-968 fixture-backed source-shaped payload path, not a real cargo build from a checkout or source archive.
- Tarball update still uses local tarball files only; authenticated URL download and release signing/trust enforcement remain deferred.

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
  - [x] Implement `ashgrove list` and `ashgrove current` against selector and install metadata.
  - [x] Implement `ashgrove default <toolchain-id>` as selector update; stable launcher behavior remains a TASK-967/Phase 127 deferred boundary.
  - [x] Implement local/source/tarball `ashgrove update` as install-new-toolchain behavior using existing fixture-backed source/tarball paths.
  - [x] Prove update does not mutate the previously installed toolchain.
  - [x] Prove `update --switch` changes user default and update without `--switch` preserves user default.
  - [x] Prove first update install initializes default only when no default exists.
  - [x] Prove exact toolchain ids are required for selection when multiple installed toolchains share the same package version.
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

TASK-970 is not a full release-channel update implementation. The landed behavior is the local, metadata-valid update/default/list/current selector slice over already supported source-shaped fixtures, local tarballs, and existing installed toolchains.
