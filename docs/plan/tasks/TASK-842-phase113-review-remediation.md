# TASK-842: Remediate independent post-closeout review findings for Phase 113

## Status: ✅ Complete

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
  - [x] Run independent review after closeout.
  - [x] Fix blocking findings and reopen any premature completion status.
  - [x] Rerun focused and broad verification after the final remediation patch.
  - [x] Reconcile docs/status/changelog after remediation.
  - [x] focused tests/evidence recorded in this task file
  - [x] no SPEC-F/G/H scope creep
```


## Notes

Task type: Review/Hardening. Estimated effort: 6 hours. Keep the slice within SPEC-061 / Phase 113 boundaries.

## Completion Evidence

Independent post-closeout review found two docs/status findings and no blocking semantic/code findings:

1. `PLAN-INDEX.md` Phase 113 summary rows still reported `13 | 1 | Planned` after TASK-841.
   - Fixed both current and legacy summary rows to `13 | 12 | Implementation Complete; Review Remediation Pending` while TASK-842 was still open.
2. `CHANGELOG.md` contained an older inline builtin-function signature that the scoped checker interpreted as a Markdown link to `<params>`.
   - Reworded the old TASK-615 changelog entry to avoid accidental Markdown-link syntax.

Follow-up independent reviews:

- Semantic/code re-review: PASS. No blocking SPEC-061 issues found for public/export/import leakage, cross-module normalization, no-sealed scrutinee, result-domain validation, residual/default semantics, structural recursion, open catch-all neutrality, or clippy-remediation regressions.
- Docs/status re-review after the first remediation: PLAN-INDEX rows and TASK-842 pending state passed; CHANGELOG link still failed due escaped-bracket parsing. Fixed by rewording the inline signature.

Verification after remediation:

- `git diff --check` — passed.
- Scoped Markdown link check including `CHANGELOG.md` — passed with `checked_docs=18 missing_links=0` after the final changelog rewording.
- Focused acceptance regression: `cargo test -p ash-typeck --test task_840_type_function_acceptance -- --nocapture` — 7 passed.
- TASK-841 broad gate remains the phase-level broad cargo evidence after the clippy remediation commit:
  - `cargo fmt --check` — passed.
  - `cargo check --workspace` — passed.
  - `cargo clippy --workspace --all-targets --all-features -- -D warnings` — passed.
  - `cargo test --workspace` — passed.
  - `cargo doc --workspace --no-deps` — passed with no `warning:` lines in `/tmp/ash-phase113-doc.log`.

No SPEC-F/G/H behavior was added during review remediation.

### Post-finalization verification remediation

A fresh verification subagent found one semantic blocker after TASK-842 was initially committed: nested sealed-domain constructor result fields were lowered with their expected domain but not validated against that field domain. Added regression coverage that rejects both `Cons<Int, Int>` and `Cons<Int, ys: OtherList>` in `TypeList` result positions, then fixed result-constructor lowering to validate every constrained constructor field.

Fresh remediation verification:

- `cargo test -p ash-typeck --test task_835_type_function_validation rejects_nested_result_constructor_field -- --nocapture` — 2 passed.
- `cargo test -p ash-typeck --test task_835_type_function_validation -- --nocapture` — 21 passed.
- `cargo test -p ash-typeck --test task_840_type_function_acceptance -- --nocapture` — 7 passed.
- `cargo test -p ash-typeck --test task_838_type_function_normalizer -- --nocapture` — 6 passed.
- `cargo test -p ash-engine --test task_839_type_function_module_boundary -- --nocapture` — 4 passed.
- `cargo check --workspace` — passed.
