# TASK-871: SPEC-H spec/plan packet

## Status: ✅ Complete

## Description

Promote DESIGN-034 §16.8 into SPEC-064/PLAN-112, create Phase 116 task files, register status surfaces, and update CHANGELOG.

## Specification Reference

- [SPEC-064: Constraint and Proposition Layer](../../spec/SPEC-064-CONSTRAINT-PROPOSITION-LAYER.md)
- [PLAN-112: Constraint and Proposition Layer](../PLAN-112-CONSTRAINT-PROPOSITION-LAYER.md)
- [DESIGN-034 §16.8](../../design/DESIGN-034-TOTAL-TYPE-COMPUTATION.md#168-spec-h-constraintproposition-layer)

## Dependencies

- ✅ TASK-870: Phase 115 review remediation (complete)

## Files / Ownership

- Create: `docs/spec/SPEC-064-CONSTRAINT-PROPOSITION-LAYER.md`
- Create: `docs/plan/PLAN-112-CONSTRAINT-PROPOSITION-LAYER.md`
- Create: `docs/plan/tasks/TASK-871-*.md` through `TASK-884-*.md`
- Modify: `docs/spec/README.md`
- Modify: `docs/plan/PLAN-INDEX.md`
- Modify: `CHANGELOG.md`

## Requirements

### Functional Requirements

1. Create SPEC-064 with normative constraint/proposition layer rules.
2. Create PLAN-112 with ordered task breakdown and verification gates.
3. Create TASK-871 through TASK-884 task files with Dispatch and Verification metadata.
4. Register SPEC-064 in docs/spec/README.md and Phase 116 in PLAN-INDEX.
5. Record changelog entry for the planning packet.

### Non-Goals

- Do not implement type-function inversion, injectivity, unrestricted SMT/proof search, HKT, holes, or partial type-constructor application.
- Do not merge type-level propositions with runtime workflow/capability/provider constraints.
- Do not move semantic proposition ownership into `ash-parser` or `ash-engine`.
- Preserve SPEC-035/SPEC-063 associated-type behavior unless the task explicitly assigns compatibility coverage.

## TDD / Execution Steps

### Step 1

- Re-read DESIGN-034 §16.8 lines 1381-1407.

### Step 2

- Inspect SPEC-057 through SPEC-063 and Phase 115 status surfaces.

### Step 3

- Create SPEC-064, PLAN-112, and TASK-871 through TASK-884.

### Step 4

- Keep future implementation tasks at 🟡 Ready; mark only TASK-871 complete.

### Step 5

- Run scoped link/trailing-whitespace/task-range checks.

### Independent Verification

Dispatch an independent review/verification subagent with this task file, SPEC-064, PLAN-112, changed files, and any task-owned audit/test evidence. Do not mark this task complete until findings are fixed and verification passes.

## Completion Checklist

- [x] Requirements above are satisfied.
- [x] Focused docs-only verification is recorded.
- [x] Status docs and CHANGELOG.md are updated.

## Completion Evidence

- Created SPEC-064, PLAN-112, and TASK-871 through TASK-884.
- Registered SPEC-064 in `docs/spec/README.md`.
- Registered Phase 116 in both PLAN-INDEX progress tables and appended the detailed Phase 116 section.
- Updated CHANGELOG.md under `[Unreleased]`.
- Independent review findings were addressed: disequality is now constructor-head disjointness rather than fully closed arguments; proposition-tail and `prop` declaration grammar are concrete and live-parser-shaped; TASK-882 cannot document-only defer required acceptance rows; core evidence ownership is boundary-scoped; Phase 116 proposition operands account for sealed-domain constructor apps; and TASK-873 through TASK-882 now contain intentional failing verification guards plus `cargo check --workspace` that TASK-872 must replace with exact non-zero commands.
- Focused re-review after those patches returned PASS.

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
  - cargo fmt --check
  - git diff --check
  - cargo check --workspace
  - |
    python3 - <<'PY'
    import re, sys
    from pathlib import Path
    files=[Path('CHANGELOG.md'),Path('docs/spec/README.md'),Path('docs/spec/SPEC-064-CONSTRAINT-PROPOSITION-LAYER.md'),Path('docs/plan/PLAN-112-CONSTRAINT-PROPOSITION-LAYER.md'),Path('docs/plan/PLAN-INDEX.md')]
    files += sorted(Path('docs/plan/tasks').glob('TASK-87[1-9]-*.md'))
    files += sorted(Path('docs/plan/tasks').glob('TASK-88[0-4]-*.md'))
    link=re.compile(r'(?<!\!)\[[^\]]+\]\(([^)]+)\)')
    bad=[]
    for path in files:
        if not path.exists():
            bad.append(f'{path}: missing'); continue
        text=path.read_text()
        if text.startswith('    #'):
            bad.append(f'{path}: starts as indented code block')
        in_fence=False
        for line_no,line in enumerate(text.splitlines(),1):
            if line.rstrip()!=line:
                bad.append(f'{path}:{line_no}: trailing whitespace')
            if line.strip().startswith('```'):
                in_fence=not in_fence; continue
            if in_fence:
                continue
            for m in link.finditer(line):
                target=m.group(1).split('#',1)[0]
                if not target or re.match(r'^[a-zA-Z][a-zA-Z0-9+.-]*:', target):
                    continue
                if not (path.parent/target).exists():
                    bad.append(f'{path}:{line_no}: broken link {target}')
    if bad:
        print('\n'.join(bad)); sys.exit(1)
    PY
checklist:
  - "[x] Task requirements are satisfied"
  - "[x] Focused verification is recorded"
  - "[x] Status docs and CHANGELOG.md are updated if release-facing docs changed"
```

## Dependencies for Next Task

This task outputs:
- Phase 116 artifact/surface owned by TASK-871 for downstream tasks.
