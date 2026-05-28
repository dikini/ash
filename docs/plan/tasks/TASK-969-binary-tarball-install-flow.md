# TASK-969: Binary tarball install flow

## Status: ⚠️ Partial first slice

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
8. Record tarball path/URL/digest/install time in install metadata.

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
  - [ ] Define a conforming release tarball shape and produce it through the TASK-965-selected script/command or test fixture helper.
  - [ ] Validate tarball directory shape before publish.
  - [ ] Validate `manifest.toml` and `install-record.toml` schema.
  - [ ] Validate archive version/toolchain id matches the target directory.
  - [ ] Validate executable presence/permissions for required binaries.
  - [ ] Validate stdlib manifest presence.
  - [ ] Reject unsafe archive entries including absolute paths, traversal, symlink/hardlink escapes, device files, and setuid/setgid bits.
  - [ ] Record tarball path/URL/digest/install time in install metadata.
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
