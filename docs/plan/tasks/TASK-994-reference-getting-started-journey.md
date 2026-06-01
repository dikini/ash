# TASK-994: Reference getting-started journey

## Status: ✅ Complete

## Description

Create the basic reader journey for Alpha Ash. These pages should help readers orient, install, update, run, use daemon mode, clean up, and find next references while linking to subsystem detail pages for exact behavior.

## Specification Reference

- [DESIGN-043](../../design/DESIGN-043-REFERENCE-SLICE-2-RUNTIME-TOOLCHAIN-MAINTENANCE.md)
- [SPEC-075](../../spec/SPEC-075-REFERENCE-SLICE-2-RUNTIME-TOOLCHAIN-MAINTENANCE.md)
- [PLAN-125](../PLAN-125-REFERENCE-SLICE-2-RUNTIME-TOOLCHAIN-MAINTENANCE.md)
- PLAN-INDEX Phase 130

## Dependencies

- ✅ TASK-992: Reference Slice 2 packet
- ✅ TASK-993: Reference maintenance metadata and staleness substrate completion

## Requirements

1. Create `reference/getting-started/README.md`.
2. Create `reference/getting-started/what-is-ash.md`.
3. Create `reference/getting-started/install.md`.
4. Create `reference/getting-started/update.md`.
5. Create `reference/getting-started/run-a-program.md`.
6. Create `reference/getting-started/run-as-daemon.md`.
7. Create `reference/getting-started/cleanup.md`.
8. Create `reference/getting-started/next-steps.md`.
9. Update `reference/INDEX.md` and `reference/README.md` to surface the journey.
10. Keep journey pages thin and cross-linked; do not duplicate full subsystem policy.

## Work Steps

1. Apply the metadata rules from TASK-993 to every new page.
2. Use current Ash identity: Transform with Pure, Effect with Act/Proc, Orchestrate with Workflow.
3. Write supported Alpha basics only.
4. Link install/update/cleanup pages to `reference/tools/ashgrove/*` detail pages.
5. Link run/daemon pages to `reference/runtime/*` and `reference/tools/cli.md` detail pages.
6. Mark future practical advice as future scope rather than inventing unsupported deployment guidance.

## Dispatch

```yaml
agent: codex
reasoning: medium
toolsets: [terminal, file]
```

## Verification

```yaml
strictness: clean
commands:
  - git diff --check
  - python3 tools/reference/check_frontmatter.py
  - python3 tools/reference/check_frontmatter.py --pilot
  - |
    python3 - <<'PY'
    from pathlib import Path
    required = [
        'reference/getting-started/README.md',
        'reference/getting-started/what-is-ash.md',
        'reference/getting-started/install.md',
        'reference/getting-started/update.md',
        'reference/getting-started/run-a-program.md',
        'reference/getting-started/run-as-daemon.md',
        'reference/getting-started/cleanup.md',
        'reference/getting-started/next-steps.md',
    ]
    missing = [p for p in required if not Path(p).exists()]
    assert not missing, missing
    for rel in required:
        text = Path(rel).read_text()
        assert 'verified_against:' in text, rel
        assert 'refresh_trigger:' in text, rel
    PY
checklist:
  - [x] All getting-started pages created.
  - [x] Pages cross-link to subsystem details.
  - [x] Pages avoid duplicating full subsystem policy.
  - [x] Unsupported practical/deployment advice is not overclaimed.
```

## Dependencies for Next Task

TASK-995 through TASK-997 supply the detailed subsystem pages linked by this journey.

## Completion Notes

TASK-994 created the `reference/getting-started/` reader journey and surfaced it from the reference root/index. Because TASK-995 and TASK-996 had not yet created the toolchain and runtime detail pages, this task added only minimal frontmatter-valid draft targets under `reference/tools/` and `reference/runtime/` to keep journey links valid without overclaiming complete subsystem coverage.
