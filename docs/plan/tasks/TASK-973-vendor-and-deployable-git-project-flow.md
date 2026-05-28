# TASK-973: Vendor and deployable git project flow

## Status: ⚠️ Partial first slice

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
  - [ ] Materialize locked dependencies into the default `vendor/ash/` directory or explicit `--output PATH`.
  - [ ] Record vendor provenance linking each vendored package to a lockfile entry.
  - [ ] Implement `ashgrove vendor --check` as read-only validation without writes or network fetches.
  - [ ] Add an offline deployment smoke test using a locked git dependency.
  - [ ] Verify selected toolchain stdlib remains separate from project dependencies.
  - [ ] Verify explicit current CLI forms such as `ash check src/main.ash` and `ash run src/main.ash:main` work with vendored dependencies.
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
