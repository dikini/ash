# TASK-844: Type-computation summary audit gate

## Status: ✅ Complete

## Description

Audit live public summary/export/import/normalizer seams before any Rust changes.

## Specification Reference

- [SPEC-062: Module-Summary Export/Import for Type Computation](../../spec/SPEC-062-MODULE-SUMMARY-EXPORT-IMPORT-FOR-TYPE-COMPUTATION.md)
- [PLAN-110: Module-Summary Export/Import for Type Computation](../PLAN-110-MODULE-SUMMARY-EXPORT-IMPORT-FOR-TYPE-COMPUTATION.md)
- [DESIGN-034 §16.6](../../design/DESIGN-034-TOTAL-TYPE-COMPUTATION.md#166-spec-f-module-summary-exportimport-for-type-computation)

## Dependencies

- Depends on TASK-843 completion

## Requirements

### Functional Requirements

1. Create docs/plan/audits/TASK-844-type-computation-summary-audit.md.
2. Map exact live carriers and callsites in ash-core, ash-engine, ash-typeck, and ash-parser.
3. Record current public type-function rejection points and summary leakage fences.
4. Record import-order risks and existing dedup/cache key gaps.
5. Do not modify Rust implementation in this task.

### Non-Goals

- Do not implement associated recursive type-family computation (SPEC-G).
- Do not add proposition solving, type-function inversion, or proof search (SPEC-H and beyond).
- Do not move type-computation semantic ownership into parser or engine-private carriers.

## TDD / Execution Steps

### Step 1: RED / Inspect

- Re-read the SPEC-062 section owned by this task.
- Inspect exact live files named by PLAN-110 and TASK-844 before patching.
- For implementation tasks, write focused failing tests before code changes.

### Step 2: GREEN / Implement

- Apply the smallest scoped patch for TASK-844 only.
- Preserve SPEC-057/059/060/061 behavior unless this task explicitly changes it.
- Keep public/private summary closure and negative leakage assertions in scope.

### Step 3: Verify

Run:

```bash
git diff --check
cargo fmt --check
python3 - <<'PY'
import re, sys
from pathlib import Path
files = [
    Path('docs/plan/tasks/TASK-844-type-computation-summary-audit-gate.md'),
    Path('docs/plan/audits/TASK-844-type-computation-summary-audit.md'),
]
link = re.compile(r'(?<!\!)\[[^\]]+\]\(([^)]+)\)')
bad = []
for path in files:
    if not path.exists():
        bad.append(f'missing expected audit/link-check input: {path}')
        continue
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
if bad:
    print('\n'.join(bad))
    sys.exit(1)
PY
cargo check --workspace
```

### Step 4: Independent Verification

Dispatch a review/verification subagent with this task file, SPEC-062, and changed files. Do not mark TASK-844 complete until the subagent reports no blocking findings and the commands above pass.

## Completion Checklist

- [x] Requirements above are satisfied.
- [x] Focused tests exist and pass, or docs-only verification is recorded.
- [x] Negative leakage/private-opacity behavior is tested where applicable.
- [x] Status docs and CHANGELOG.md are updated if this task changes behavior or status.
- [x] Independent verification completed by parent-required TASK-844 audit review gate after artifact creation.

## Dispatch

```yaml
agent: hermes
reasoning: medium
max_turns: 15
toolsets: [terminal, file]
```

## Verification

TASK-844 docs-only verification was run after creating `docs/plan/audits/TASK-844-type-computation-summary-audit.md` and reconciling status docs.

Results recorded 2026-05-11:

- `git diff --check` — passed.
- `cargo fmt --check` — passed.
- TASK-844 markdown link-check Python snippet — passed.
- `cargo check --workspace` — passed.

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
        Path('docs/plan/tasks/TASK-844-type-computation-summary-audit-gate.md'),
        Path('docs/plan/audits/TASK-844-type-computation-summary-audit.md'),
    ]
    link = re.compile(r'(?<!\!)\[[^\]]+\]\(([^)]+)\)')
    bad = []
    for path in files:
        if not path.exists():
            bad.append(f'missing expected audit/link-check input: {path}')
            continue
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
    if bad:
        print('\n'.join(bad))
        sys.exit(1)
    PY
  - cargo check --workspace
checklist:
  - [ ] Implementation matches SPEC-062 and PLAN-110 scope
  - [ ] Focused tests for this task pass
  - [ ] Formatting and diff checks pass
  - [ ] CHANGELOG.md updated if task changes code/docs policy/status
```

## Dependencies for Next Task

This task outputs:
- This task outputs a live audit artifact consumed by TASK-845 through TASK-854.
