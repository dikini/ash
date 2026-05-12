# TASK-857: SPEC-G spec/plan packet

## Status: ✅ Complete

## Description

Promote DESIGN-034 §16.7 into SPEC-063/PLAN-111, create Phase 115 task files, register status surfaces, reconcile Phase 114 progress-table drift, and update CHANGELOG.

## Specification Reference

- [SPEC-063: Associated Type-Family Computation](../../spec/SPEC-063-ASSOCIATED-TYPE-FAMILY-COMPUTATION.md)
- [PLAN-111: Associated Type-Family Computation](../PLAN-111-ASSOCIATED-TYPE-FAMILY-COMPUTATION.md)
- [DESIGN-034 §16.7](../../design/DESIGN-034-TOTAL-TYPE-COMPUTATION.md#167-spec-g-associated-type-family-computation)

## Dependencies

- ✅ TASK-856: Phase 114 review remediation (complete)

## Files / Ownership

- Create: `docs/spec/SPEC-063-ASSOCIATED-TYPE-FAMILY-COMPUTATION.md`
- Create: `docs/plan/PLAN-111-ASSOCIATED-TYPE-FAMILY-COMPUTATION.md`
- Create: `docs/plan/tasks/TASK-857-*.md` through `TASK-870-*.md`
- Modify: `docs/spec/README.md`
- Modify: `docs/plan/PLAN-INDEX.md`
- Modify: `CHANGELOG.md`

## Requirements

### Functional Requirements

1. Create SPEC-063 with normative associated type-family computation rules.
2. Create PLAN-111 with ordered task breakdown and verification gates.
3. Create TASK-857 through TASK-870 task files with Dispatch and Verification metadata.
4. Register SPEC-063 in docs/spec/README.md and Phase 115 in PLAN-INDEX.
5. Correct stale Phase 114 progress-summary drift so Phase 115 depends on a completed SPEC-F substrate.
6. Record changelog entry for the planning packet.

### Non-Goals

- Do not implement SPEC-H proposition solving, type-function inversion, proof search, or HKT/hole support.
- Do not move semantic ownership into `ash-parser` or `ash-engine`.
- Preserve existing SPEC-035 simple associated type behavior unless this task explicitly assigns a compatibility bridge.

## TDD / Execution Steps

### Step 1: Inspect

- Re-read DESIGN-034 §16.7 lines 1336-1381.
- Inspect SPEC-035 and SPEC-057 through SPEC-062 for authority boundaries.
- Inspect PLAN-INDEX progress tables before editing.

### Step 2: Write docs packet

- Create SPEC-063, PLAN-111, and TASK-857 through TASK-870.
- Keep all future implementation tasks at `🟡 Ready`, not `✅ Complete`.
- Mark only TASK-857 complete because this docs packet owns it.

### Step 3: Verify docs packet

- Run the scoped link/trailing-whitespace/task-range checks in this task.
- Run `git diff --check`, `cargo fmt --check`, and `cargo check --workspace`.

### Independent Verification

Dispatch an independent review/verification subagent with this task file, SPEC-063, PLAN-111, and the changed files. Do not mark this task complete until findings are fixed and verification passes.

## Completion Checklist

- [x] Requirements above are satisfied.
- [x] Focused tests/evidence exist and pass, or docs-only verification is recorded.
- [x] Negative leakage/non-interference behavior is covered for this task's surface.
- [x] Status docs and CHANGELOG.md are updated if this task changes release-facing docs.
- [x] Independent verification completed or scheduled by the closeout task.

## Completion Evidence

- Created SPEC-063, PLAN-111, and TASK-857 through TASK-870.
- Registered SPEC-063 in `docs/spec/README.md`.
- Registered Phase 115 in both PLAN-INDEX progress tables and appended the detailed Phase 115 section.
- Reconciled stale Phase 114 progress-summary rows from 4/14 in progress to 14/14 complete.
- Updated CHANGELOG.md under `[Unreleased]`.
- Independent review findings from the Phase 115 packet review were addressed, including parser declaration ownership, YAML metadata validity, closeout gates, recursive-decreases wording, acceptance non-inversion coverage, and track-hour consistency.

## Dispatch

```yaml
agent: hermes
reasoning: high
max_turns: 15
toolsets: [terminal, file]
```

## Verification

```yaml
strictness: clean
commands:
  - git diff --check
  - cargo fmt --check
  - |
    python3 - <<'PY'
    import re, sys
    from pathlib import Path
    files = [
        Path('CHANGELOG.md'),
        Path('docs/spec/README.md'),
        Path('docs/spec/SPEC-063-ASSOCIATED-TYPE-FAMILY-COMPUTATION.md'),
        Path('docs/plan/PLAN-111-ASSOCIATED-TYPE-FAMILY-COMPUTATION.md'),
        Path('docs/plan/PLAN-INDEX.md'),
    ]
    files += sorted(Path('docs/plan/tasks').glob('TASK-85[7-9]-*.md'))
    files += sorted(Path('docs/plan/tasks').glob('TASK-86[0-9]-*.md'))
    files += sorted(Path('docs/plan/tasks').glob('TASK-870-*.md'))
    link = re.compile(r'(?<!\!)\[[^\]]+\]\(([^)]+)\)')
    bad = []
    for path in files:
        text = path.read_text()
        in_fence = False
        for line_no, line in enumerate(text.splitlines(), 1):
            if line.strip().startswith('```'):
                in_fence = not in_fence
                continue
            if in_fence:
                continue
            for match in link.finditer(line):
                target = match.group(1).split('#', 1)[0]
                if not target or re.match(r'^[a-zA-Z][a-zA-Z0-9+.-]*:', target):
                    continue
                if not (path.parent / target).exists():
                    bad.append(f'{path}:{line_no}: {target}')
        for line_no, line in enumerate(text.splitlines(), 1):
            if line.rstrip() != line:
                bad.append(f'{path}:{line_no}: trailing whitespace')
    if bad:
        print('\n'.join(bad))
        sys.exit(1)
    PY
  - cargo check --workspace
checklist:
  - "[x] SPEC-063 and PLAN-111 created"
  - "[x] TASK-857 through TASK-870 created with dispatch/verification metadata"
  - "[x] PLAN-INDEX progress tables and detailed Phase 115 section updated"
  - "[x] docs/spec/README.md and CHANGELOG.md updated"
```

## Dependencies for Next Task

This task outputs:
- An implementation-grade Phase 115 packet ready for TASK-858 audit-first implementation.
