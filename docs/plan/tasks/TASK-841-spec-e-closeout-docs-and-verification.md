# TASK-841: Reconcile docs/status/changelog and record closeout verification evidence

## Status: 📋 Planned

## Description

Reconcile docs/status/changelog and record closeout verification evidence.

## Specification Reference

- [SPEC-061](../../spec/SPEC-061-DIRECT-STRUCTURAL-TYPE-FUNCTIONS.md)
- [PLAN-109](../PLAN-109-DIRECT-STRUCTURAL-TYPE-FUNCTIONS.md)
- [DESIGN-034](../../design/DESIGN-034-TOTAL-TYPE-COMPUTATION.md) §16.5
- [SPEC-060](../../spec/SPEC-060-NORMALIZER-DEFINITIONAL-EQUALITY-CORE.md)

## Dependencies

- ✅ Phase 112 / SPEC-060 complete.
- Depends on TASK-840 diagnostics and acceptance matrix completion.

## Dispatch

```
agent: hermes
provider: openai-codex
model: gpt-5.5
profile: default
reasoning: high
max_turns: 20
toolsets: [terminal, file]
```

## Objective

Reconcile docs/status/changelog and record closeout verification evidence.

## Requirements

1. Update SPEC-061 status if implemented.
2. Update PLAN-109, PLAN-INDEX, tasks, spec index, and CHANGELOG.
3. Record focused and broad verification commands, including positive behavior and negative leakage evidence.
4. Run `git diff --check`, scoped markdown-link checks, cargo fmt/check/clippy/test/doc gates assigned by the phase closeout.

## Files

- Modify/create exact files identified by [PLAN-109](../PLAN-109-DIRECT-STRUCTURAL-TYPE-FUNCTIONS.md) and the TASK-831 audit gate.
- Update `CHANGELOG.md` for completed implementation/tooling/docs-policy changes.

## TDD Steps

1. Write focused failing tests or docs/audit checks appropriate to task type.
2. Run the focused target and verify the expected failure or missing evidence.
3. Implement the minimal change for this task only.
4. Re-run the focused target and relevant non-regression tests.
5. Update docs/status evidence only after verification.

## Verification

```
strictness: clean
commands:
  - git diff --check
  - |
    python - <<'PY'
    from pathlib import Path
    import re
    root = Path('.')
    files = [Path('docs/spec/SPEC-061-DIRECT-STRUCTURAL-TYPE-FUNCTIONS.md'), Path('docs/plan/PLAN-109-DIRECT-STRUCTURAL-TYPE-FUNCTIONS.md'), Path('docs/spec/README.md'), Path('docs/plan/PLAN-INDEX.md')] + sorted(Path('docs/plan/tasks').glob('TASK-83*.md')) + sorted(Path('docs/plan/tasks').glob('TASK-84*.md'))
    missing = []
    for p in files:
        for href in re.findall(r'\[[^\]]+\]\(([^)]+)\)', p.read_text()):
            target = href.split('#', 1)[0]
            if target and '://' not in target and not target.startswith('mailto:') and not (p.parent / target).resolve().exists():
                missing.append((str(p), href))
    assert not missing, missing
    PY
  - cargo test -p ash-parser --test task_832_type_function_parser -- --nocapture
  - cargo test -p ash-core --test task_833_type_function_carriers -- --nocapture
  - cargo test -p ash-typeck --test task_834_type_function_lowering -- --nocapture
  - cargo test -p ash-typeck --test task_835_type_function_validation -- --nocapture
  - cargo test -p ash-typeck --test task_836_type_function_patterns -- --nocapture
  - cargo test -p ash-typeck --test task_837_type_function_recursion -- --nocapture
  - cargo test -p ash-typeck --test task_838_type_function_normalizer -- --nocapture
  - cargo test -p ash-engine --test task_839_type_function_module_boundary -- --nocapture
  - cargo test -p ash-typeck --test task_840_type_function_acceptance -- --nocapture
  - cargo fmt --check
  - cargo check --workspace
  - cargo clippy --workspace --all-targets --all-features -- -D warnings
  - cargo test --workspace
  - cargo doc --workspace --no-deps
checklist:
  - [ ] Update SPEC-061 status if implemented.
  - [ ] Update PLAN-109, PLAN-INDEX, tasks, spec index, and CHANGELOG.
  - [ ] Record focused and broad verification commands, including positive behavior and negative leakage evidence.
  - [ ] Run `git diff --check`, scoped markdown-link checks, cargo fmt/check/clippy/test/doc gates assigned by the phase closeout.
  - [ ] focused tests/evidence recorded in this task file
  - [ ] no SPEC-F/G/H scope creep
```


## Notes

Task type: Docs/Planning. Estimated effort: 4 hours. Keep the slice within SPEC-061 / Phase 113 boundaries.
