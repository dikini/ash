# TASK-992: Reference Slice 2 packet

## Status: ✅ Complete

## Description

Create the Phase 130 planning packet for Reference Slice 2. The packet defines the design, spec, plan, task breakdown, PLAN-INDEX registration, spec-index registration, and changelog entry required before expanding the `reference/` corpus.

## Specification Reference

- [DESIGN-043](../../design/DESIGN-043-REFERENCE-SLICE-2-RUNTIME-TOOLCHAIN-MAINTENANCE.md)
- [SPEC-075](../../spec/SPEC-075-REFERENCE-SLICE-2-RUNTIME-TOOLCHAIN-MAINTENANCE.md)
- [PLAN-125](../PLAN-125-REFERENCE-SLICE-2-RUNTIME-TOOLCHAIN-MAINTENANCE.md)
- PLAN-INDEX Phase 130

## Dependencies

- ✅ TASK-953: Reference corpus closeout and drift report
- ✅ TASK-954: Functions reference chapter expansion
- ✅ TASK-986: SPEC-073 implemented MVP closeout
- ✅ TASK-990: SPEC-074 source-payload/local-state closeout
- ✅ TASK-991: Focused ignored-lockfile source-install fix

## Requirements

1. Create `docs/design/DESIGN-043-REFERENCE-SLICE-2-RUNTIME-TOOLCHAIN-MAINTENANCE.md`.
2. Create `docs/spec/SPEC-075-REFERENCE-SLICE-2-RUNTIME-TOOLCHAIN-MAINTENANCE.md`.
3. Create `docs/plan/PLAN-125-REFERENCE-SLICE-2-RUNTIME-TOOLCHAIN-MAINTENANCE.md`.
4. Create TASK-992 through TASK-999 task files.
5. Register SPEC-075 in `docs/spec/README.md`.
6. Register Phase 130 in the current PLAN-INDEX progress table and the detailed phase section.
7. Update `CHANGELOG.md` under `[Unreleased]`.
8. Verify the packet structure, links, and docs hygiene.

## Work Steps

1. Inspect current reference-corpus status and recent phase-packet patterns.
2. Write the design note with subsystem/detail, reader-journey, and maintenance architecture.
3. Write SPEC-075 with scope, metadata/staleness rules, page inventory, and acceptance criteria.
4. Write PLAN-125 with ordered tasks and decision gates.
5. Write TASK-992 through TASK-999.
6. Update PLAN-INDEX, spec README, and CHANGELOG.
7. Run the verification commands below.

## Dispatch

```yaml
agent: hermes
reasoning: high
toolsets: [terminal, file]
```

## Verification

```yaml
strictness: clean
commands:
  - git diff --check
  - python3 -m py_compile tools/reference/check_frontmatter.py
  - python3 tools/reference/check_frontmatter.py --pilot
  - |
    python3 - <<'PY'
    from pathlib import Path
    files = [
        'docs/design/DESIGN-043-REFERENCE-SLICE-2-RUNTIME-TOOLCHAIN-MAINTENANCE.md',
        'docs/spec/SPEC-075-REFERENCE-SLICE-2-RUNTIME-TOOLCHAIN-MAINTENANCE.md',
        'docs/plan/PLAN-125-REFERENCE-SLICE-2-RUNTIME-TOOLCHAIN-MAINTENANCE.md',
    ] + [f'docs/plan/tasks/TASK-{n}-{slug}.md' for n, slug in [
        (992, 'reference-slice-2-packet'),
        (993, 'reference-maintenance-metadata-and-staleness'),
        (994, 'reference-getting-started-journey'),
        (995, 'reference-ashgrove-and-cli-procedures'),
        (996, 'reference-runtime-kernel-pages'),
        (997, 'reference-stdlib-tower-pages'),
        (998, 'reference-agent-cards-and-context-pack'),
        (999, 'reference-slice-2-closeout'),
    ]]
    missing = [p for p in files if not Path(p).exists()]
    assert not missing, missing
    idx = Path('docs/plan/PLAN-INDEX.md').read_text()
    assert '## Phase 130: Reference Slice 2 Runtime, Toolchain, and Maintenance Manual' in idx
    for n in range(992, 1000):
        assert f'TASK-{n}' in idx
    print('phase130 packet structure verified')
    PY
checklist:
  - [x] DESIGN-043 created.
  - [x] SPEC-075 created.
  - [x] PLAN-125 created.
  - [x] TASK-992 through TASK-999 created.
  - [x] PLAN-INDEX updated.
  - [x] docs/spec/README.md updated.
  - [x] CHANGELOG.md updated.
```

## Dependencies for Next Task

This task outputs the packet that TASK-993 through TASK-999 implement.
