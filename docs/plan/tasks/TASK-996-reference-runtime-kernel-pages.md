# TASK-996: Reference RuntimeKernel pages

## Status: 📝 Planned

## Description

Create RuntimeKernel concept and status pages that teach current Alpha execution behavior without requiring readers to inspect SPEC-069/SPEC-070 first.

## Specification Reference

- [DESIGN-043](../../design/DESIGN-043-REFERENCE-SLICE-2-RUNTIME-TOOLCHAIN-MAINTENANCE.md)
- [SPEC-075](../../spec/SPEC-075-REFERENCE-SLICE-2-RUNTIME-TOOLCHAIN-MAINTENANCE.md)
- [PLAN-125](../PLAN-125-REFERENCE-SLICE-2-RUNTIME-TOOLCHAIN-MAINTENANCE.md)
- PLAN-INDEX Phase 130
- [SPEC-069](../../spec/SPEC-069-ALPHA-VISIBLE-TOWER-ALGEBRA-AND-DO-LOWERING.md)
- [SPEC-070](../../spec/SPEC-070-ALPHA-RUNTIME-KERNEL-AND-OS-SURFACE.md)

## Dependencies

- 📝 TASK-993: Reference maintenance metadata and staleness substrate completion
- 📝 TASK-994: Reader journey link targets established

## Requirements

1. Create `reference/runtime/README.md`.
2. Create `reference/runtime/kernel.md`.
3. Create `reference/runtime/admission.md`.
4. Create `reference/runtime/artifacts.md`.
5. Create `reference/runtime/daemon.md`.
6. Create `reference/runtime/policy-profiles.md`.
7. Create or update `reference/status/runtime-kernel.md`.
8. Preserve SPEC-070 integrity and authority boundaries.
9. Update `reference/INDEX.md`, status pages, and known limitations as needed.

## Work Steps

1. Re-read SPEC-069/SPEC-070 and Phase 123 closeout evidence before writing.
2. Explain RuntimeKernel as the semantic execution host abstraction with one-shot and local daemon host modes.
3. State integrity caveats: file presence does not execute code; verified artifacts are source/check-summary based; reload affects future starts.
4. State authority caveats: provider/resource existence is not authority; admission grants authority before user body execution.
5. State non-goals: no remote/multi-user daemon API, no distributed scheduling, no production init integration, no hot-swapping running instances.
6. Keep concept pages separate from `reference/status/runtime-kernel.md` evidence tables.

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
  - |
    python3 - <<'PY'
    from pathlib import Path
    required = [
        'reference/runtime/README.md',
        'reference/runtime/kernel.md',
        'reference/runtime/admission.md',
        'reference/runtime/artifacts.md',
        'reference/runtime/daemon.md',
        'reference/runtime/policy-profiles.md',
        'reference/status/runtime-kernel.md',
    ]
    missing = [p for p in required if not Path(p).exists()]
    assert not missing, missing
    text = '\n'.join(Path(p).read_text() for p in required)
    for phrase in ['file presence', 'provider', 'admission', 'reload']:
        assert phrase in text.lower(), phrase
    PY
checklist:
  - [ ] Runtime pages created.
  - [ ] Integrity and authority caveats preserved.
  - [ ] Status/evidence separated from learner-facing concept pages.
  - [ ] Non-goals are explicit.
```

## Dependencies for Next Task

TASK-998 must create a RuntimeKernel agent card from these canonical pages after they exist.
