# TASK-842: Remediate independent post-closeout review findings for Phase 113

## Status: 📋 Planned

## Description

Remediate independent post-closeout review findings for Phase 113.

## Specification Reference

- [SPEC-061](../../spec/SPEC-061-DIRECT-STRUCTURAL-TYPE-FUNCTIONS.md)
- [PLAN-109](../PLAN-109-DIRECT-STRUCTURAL-TYPE-FUNCTIONS.md)
- [DESIGN-034](../../design/DESIGN-034-TOTAL-TYPE-COMPUTATION.md) §16.5
- [SPEC-060](../../spec/SPEC-060-NORMALIZER-DEFINITIONAL-EQUALITY-CORE.md)

## Dependencies

- ✅ Phase 112 / SPEC-060 complete.
- Depends on TASK-841 closeout completion.
- Depends on independent post-closeout review completion with findings classified.

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

Remediate independent post-closeout review findings for Phase 113.

## Requirements

1. Run independent review after closeout.
2. Fix blocking findings and reopen any premature completion status.
3. Rerun focused and broad verification after the final remediation patch.
4. Reconcile docs/status/changelog after remediation.

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
  - finding-specific focused command(s) named by the independent review report
checklist:
  - [ ] Run independent review after closeout.
  - [ ] Fix blocking findings and reopen any premature completion status.
  - [ ] Rerun focused and broad verification after the final remediation patch.
  - [ ] Reconcile docs/status/changelog after remediation.
  - [ ] focused tests/evidence recorded in this task file
  - [ ] no SPEC-F/G/H scope creep
```


## Notes

Task type: Review/Hardening. Estimated effort: 6 hours. Keep the slice within SPEC-061 / Phase 113 boundaries.
