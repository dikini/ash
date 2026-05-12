# TASK-869: SPEC-G closeout docs and verification

## Status: 🟡 Ready

## Description

Reconcile SPEC-063/PLAN-111/Phase 115 status, examples/docs/changelog, and broad verification evidence.

## Specification Reference

- [SPEC-063: Associated Type-Family Computation](../../spec/SPEC-063-ASSOCIATED-TYPE-FAMILY-COMPUTATION.md)
- [PLAN-111: Associated Type-Family Computation](../PLAN-111-ASSOCIATED-TYPE-FAMILY-COMPUTATION.md)
- [DESIGN-034 §16.7](../../design/DESIGN-034-TOTAL-TYPE-COMPUTATION.md#167-spec-g-associated-type-family-computation)

## Dependencies

- Depends on TASK-868 completion

## Files / Ownership

- Modify: `docs/spec/SPEC-063-ASSOCIATED-TYPE-FAMILY-COMPUTATION.md`
- Modify: `docs/spec/README.md`
- Modify: `docs/plan/PLAN-111-ASSOCIATED-TYPE-FAMILY-COMPUTATION.md`
- Modify: `docs/plan/PLAN-INDEX.md`
- Modify: `docs/plan/tasks/TASK-857-*.md` through `docs/plan/tasks/TASK-870-*.md` as needed for honest task statuses, checklists, and completion evidence
- Modify: `CHANGELOG.md`

## Requirements

### Functional Requirements

1. Promote SPEC-063 and docs/spec/README.md status only after implementation evidence is complete.
2. Update PLAN-111, PLAN-INDEX, task files, and CHANGELOG coherently.
3. Run full workspace fmt/check/clippy/test/doc gates and scoped markdown checks.
4. Record any residual limitations honestly instead of overclaiming completion.

### Non-Goals

- Do not implement SPEC-H proposition solving, type-function inversion, proof search, or HKT/hole support.
- Do not move semantic ownership into `ash-parser` or `ash-engine`.
- Preserve existing SPEC-035 simple associated type behavior unless this task explicitly assigns a compatibility bridge.

## TDD / Execution Steps

### Step 1: Inspect status surfaces

- Re-read all TASK-857 through TASK-868 files, PLAN-111, PLAN-INDEX, spec README, and CHANGELOG.

### Step 2: Run broad verification

- Run closeout commands from PLAN-111 and record exact outputs/counts.

### Step 3: Reconcile docs

- Mark only verified tasks complete.
- Update spec status to Implemented MVP only when all implementation/acceptance gates pass.

### Independent Verification

Dispatch an independent review/verification subagent with this task file, SPEC-063, PLAN-111, and the changed files. Do not mark this task complete until findings are fixed and verification passes.

## Completion Checklist

- [ ] Requirements above are satisfied.
- [ ] Focused tests/evidence exist and pass, or docs-only verification is recorded.
- [ ] Negative leakage/non-interference behavior is covered for this task's surface.
- [ ] Status docs and CHANGELOG.md are updated if this task changes release-facing docs.
- [ ] Independent verification completed or scheduled by the closeout task.

## Completion Evidence

- Completion evidence must be recorded by the implementing agent before marking this task complete.

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
  - cargo clippy --workspace --all-targets --all-features -- -D warnings
  - cargo test --workspace
  - |
    cargo doc --workspace --no-deps 2>&1 | tee /tmp/ash-phase115-doc.log
  - |
    ! grep -i '^warning:' /tmp/ash-phase115-doc.log
  - |
    python3 - <<'PY'
    import re, sys
    from pathlib import Path
    files=[Path('docs/spec/SPEC-063-ASSOCIATED-TYPE-FAMILY-COMPUTATION.md'),Path('docs/plan/PLAN-111-ASSOCIATED-TYPE-FAMILY-COMPUTATION.md'),Path('docs/plan/PLAN-INDEX.md'),Path('docs/spec/README.md'),Path('CHANGELOG.md')]
    files += sorted(Path('docs/plan/tasks').glob('TASK-85[7-9]-*.md'))
    files += sorted(Path('docs/plan/tasks').glob('TASK-86[0-9]-*.md'))
    files += sorted(Path('docs/plan/tasks').glob('TASK-870-*.md'))
    link=re.compile(r'(?<!\!)\[[^\]]+\]\(([^)]+)\)')
    bad=[]
    for p in files:
        txt=p.read_text(); fence=False
        for n,line in enumerate(txt.splitlines(),1):
            if line.strip().startswith('```'):
                fence=not fence; continue
            if line.rstrip()!=line: bad.append(f'{p}:{n}: trailing whitespace')
            if fence: continue
            for m in link.finditer(line):
                target=m.group(1).split('#',1)[0]
                if not target or re.match(r'^[a-zA-Z][a-zA-Z0-9+.-]*:', target): continue
                if not (p.parent/target).exists(): bad.append(f'{p}:{n}: {target}')
    if bad:
        print('\n'.join(bad)); sys.exit(1)
    PY
checklist:
  - "[ ] Implementation matches SPEC-063 and PLAN-111 scope"
  - "[ ] Focused tests/evidence for this task pass"
  - "[ ] No SPEC-H/proof-search/type-function-inversion behavior added"
```

## Dependencies for Next Task

This task outputs:
- Honest phase closeout state ready for independent post-closeout review.
