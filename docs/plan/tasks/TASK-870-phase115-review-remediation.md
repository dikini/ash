# TASK-870: Phase 115 review remediation

## Status: 🟡 Ready

## Description

Reserve a mandatory independent review remediation slice for Phase 115 after closeout, with all findings addressed before final completion.

## Specification Reference

- [SPEC-063: Associated Type-Family Computation](../../spec/SPEC-063-ASSOCIATED-TYPE-FAMILY-COMPUTATION.md)
- [PLAN-111: Associated Type-Family Computation](../PLAN-111-ASSOCIATED-TYPE-FAMILY-COMPUTATION.md)
- [DESIGN-034 §16.7](../../design/DESIGN-034-TOTAL-TYPE-COMPUTATION.md#167-spec-g-associated-type-family-computation)

## Dependencies

- Depends on TASK-869 completion

## Files / Ownership

- Modify: files identified by independent review
- Expected review surfaces include: `docs/spec/SPEC-063-ASSOCIATED-TYPE-FAMILY-COMPUTATION.md`, `docs/plan/PLAN-111-ASSOCIATED-TYPE-FAMILY-COMPUTATION.md`, `docs/plan/PLAN-INDEX.md`, `docs/spec/README.md`, `CHANGELOG.md`, `docs/plan/tasks/TASK-857-*.md` through `TASK-870-*.md`, `docs/plan/audits/TASK-858-associated-family-computation-audit.md`, and `docs/plan/audits/TASK-868-associated-family-acceptance-matrix.md`, plus all Phase 115 code/test files changed during implementation.

## Requirements

### Functional Requirements

1. Run independent review across SPEC-063, PLAN-111, TASK-857 through TASK-869, changed code, acceptance artifact, and verification evidence.
2. Treat blocking, important, minor, and non-blocking findings as work to address unless explicitly documented as out-of-scope by spec authority.
3. Patch docs/code/tests and rerun focused plus broad gates after the final change.
4. Update status surfaces and changelog with actual remediation evidence.

### Non-Goals

- Do not implement SPEC-H proposition solving, type-function inversion, proof search, or HKT/hole support.
- Do not move semantic ownership into `ash-parser` or `ash-engine`.
- Preserve existing SPEC-035 simple associated type behavior unless this task explicitly assigns a compatibility bridge.

## TDD / Execution Steps

### Step 1: Run review

- Delegate independent review with axes: spec conformance, task/order drift, live-code feasibility, non-inversion, summary opacity, diagnostics, verification honesty.

### Step 2: Address findings

- Patch every finding, including non-blocking clarity items, unless impossible or out of scope; document any scoped exception.

### Step 3: Reverify

- Rerun broad closeout gates after the final remediation patch and record evidence.

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
- Post-review remediated Phase 115 packet and implementation ready for final status.
