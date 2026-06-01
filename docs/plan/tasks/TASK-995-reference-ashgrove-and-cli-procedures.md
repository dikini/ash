# TASK-995: Reference Ashgrove and CLI procedures

## Status: ✅ Complete

## Description

Create the `ash` and `ashgrove` tool reference pages for current Alpha command and procedure behavior, including install, update, list/current/default, remove/cleanup, project dependencies, vendor/deploy, trust/signing, and source-payload policy.

## Specification Reference

- [DESIGN-043](../../design/DESIGN-043-REFERENCE-SLICE-2-RUNTIME-TOOLCHAIN-MAINTENANCE.md)
- [SPEC-075](../../spec/SPEC-075-REFERENCE-SLICE-2-RUNTIME-TOOLCHAIN-MAINTENANCE.md)
- [PLAN-125](../PLAN-125-REFERENCE-SLICE-2-RUNTIME-TOOLCHAIN-MAINTENANCE.md)
- PLAN-INDEX Phase 130
- [SPEC-073](../../spec/SPEC-073-ASHGROVE-INSTALL-UPDATE-CLEANUP-GIT-DEPLOYMENT.md)
- [SPEC-074](../../spec/SPEC-074-ASHGROVE-SOURCE-PAYLOAD-LOCAL-STATE-IGNORE.md)

## Dependencies

- ✅ TASK-993: Reference maintenance metadata and staleness substrate
- ✅ TASK-994: Reader journey link targets established

## Requirements

1. Create `reference/tools/README.md`.
2. Create `reference/tools/cli.md`.
3. Create `reference/tools/ashgrove.md`.
4. Create Ashgrove procedure pages under `reference/tools/ashgrove/` for install, update, list/current/default, remove/cleanup, project dependencies, vendor/deploy, trust/signing, and source payload.
5. Create or update `reference/status/ashgrove.md`.
6. Explicitly preserve SPEC-073/SPEC-074 non-goals and fail-closed boundaries.
7. Use command examples only after checking the live command surface or mark them illustrative/reference-only.
8. Update `reference/INDEX.md`, `reference/status/README.md`, and example/status surfaces as needed.

## Work Steps

1. Inspect `ash` and `ashgrove` command help and relevant crate READMEs before writing command examples.
2. Map each procedure page to SPEC-073/SPEC-074 tasks and code/test evidence.
3. Keep `reference/tools/cli.md` as a command map; put detailed Ashgrove policy in the Ashgrove pages.
4. Add explicit non-goal warnings: no hosted registry, no global/system installs, no OS package-manager integration, no arbitrary SemVer solver, no broad source-ignore glob CLI.
5. Classify examples honestly.

## Dispatch

```yaml
agent: codex
reasoning: high
toolsets: [terminal, file]
```

## Verification

```yaml
strictness: clean
commands:
  - git diff --check
  - python3 tools/reference/check_frontmatter.py
  - python3 tools/reference/check_frontmatter.py --pilot
  - cargo run -p ash-cli -- --help
  - cargo run -p ashgrove -- --help
  - |
    python3 - <<'PY'
    from pathlib import Path
    required = [
        'reference/tools/README.md',
        'reference/tools/cli.md',
        'reference/tools/ashgrove.md',
        'reference/tools/ashgrove/install.md',
        'reference/tools/ashgrove/update.md',
        'reference/tools/ashgrove/list-current-default.md',
        'reference/tools/ashgrove/remove-cleanup.md',
        'reference/tools/ashgrove/project-dependencies.md',
        'reference/tools/ashgrove/vendor-deploy.md',
        'reference/tools/ashgrove/trust-and-signing.md',
        'reference/tools/ashgrove/source-payload.md',
        'reference/status/ashgrove.md',
    ]
    missing = [p for p in required if not Path(p).exists()]
    assert not missing, missing
    ashgrove_text = '\n'.join(Path(p).read_text() for p in required if 'ashgrove' in p)
    for phrase in ['hosted registry', 'global/system', 'SemVer', 'source payload']:
        assert phrase in ashgrove_text, phrase
    PY
checklist:
- [x] Tool pages created.
- [x] Live command surface checked or examples marked non-executable.
- [x] SPEC-073/SPEC-074 non-goals preserved.
- [x] Source-payload/local-state policy documented without overclaiming.
```

## Dependencies for Next Task

TASK-998 must create agent cards from these canonical pages after they exist.

## Completion Notes

Completed on 2026-06-01. Replaced the TASK-994 Ashgrove/CLI draft placeholders with command-map and Ashgrove procedure pages under `reference/tools/`, added `reference/status/ashgrove.md`, updated reference indexes/status links, and reconciled PLAN-125/PLAN-INDEX.

Live command surfaces were checked with `cargo run -p ash-cli -- --help`, `cargo run -p ashgrove -- --help`, and focused subcommand help for documented command forms. Initial help execution hit the sandboxed `sccache` wrapper; reruns used `RUSTC_WRAPPER=` to expose the same live clap command surfaces.
