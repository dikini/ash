# TASK-841: Reconcile docs/status/changelog and record closeout verification evidence

## Status: ✅ Complete

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
  - [x] Update SPEC-061 status if implemented.
  - [x] Update PLAN-109, PLAN-INDEX, tasks, spec index, and CHANGELOG.
  - [x] Record focused and broad verification commands, including positive behavior and negative leakage evidence.
  - [x] Run `git diff --check`, scoped markdown-link checks, cargo fmt/check/clippy/test/doc gates assigned by the phase closeout.
  - [x] focused tests/evidence recorded in this task file
  - [x] no SPEC-F/G/H scope creep
```


## Notes

Task type: Docs/Planning. Estimated effort: 4 hours. Keep the slice within SPEC-061 / Phase 113 boundaries.

## Completion Evidence

- Promoted `SPEC-061` from Draft to Implemented MVP and updated `docs/spec/README.md` accordingly.
- Reconciled Phase 113 status surfaces:
  - `PLAN-109`: TASK-841 marked complete; phase status set to implementation complete with TASK-842 post-closeout review/remediation still pending.
  - `PLAN-INDEX`: TASK-841 marked complete; phase status mirrors PLAN-109.
  - `CHANGELOG.md`: added TASK-841 closeout/clippy-remediation evidence.
- Positive behavior evidence recorded by focused Phase 113 suites:
  - parser raw `type fn` syntax/spans/rejections: `cargo test -p ash-parser --test task_832_type_function_parser -- --nocapture` — 6 passed.
  - core carriers/source anchors/serde and boxed pattern carrier regression: `cargo test -p ash-core --test task_833_type_function_carriers -- --nocapture` — 5 passed.
  - lowering/registration: `cargo test -p ash-typeck --test task_834_type_function_lowering -- --nocapture` — 6 passed.
  - validation diagnostics: `cargo test -p ash-typeck --test task_835_type_function_validation -- --nocapture` — 19 passed.
  - residual coverage/defaults: `cargo test -p ash-typeck --test task_836_type_function_patterns -- --nocapture` — 10 passed.
  - structural recursion: `cargo test -p ash-typeck --test task_837_type_function_recursion -- --nocapture` — 11 passed.
  - source-backed normalizer: `cargo test -p ash-typeck --test task_838_type_function_normalizer -- --nocapture` — 6 passed.
  - engine/public-boundary negative leakage: `cargo test -p ash-engine --test task_839_type_function_module_boundary -- --nocapture` — 4 passed.
  - full acceptance matrix: `cargo test -p ash-typeck --test task_840_type_function_acceptance -- --nocapture` — 7 passed.
- Negative leakage/non-interference evidence:
  - public ordinary alias/callable/workflow signature leakage of local computation heads is rejected by TASK-839.
  - imported semantic summaries structurally omit local type-function heads/equations before SPEC-F.
  - cross-module type-function normalization remains unavailable before SPEC-F.
- Scoped markdown-link check: `checked_docs=17 missing_links=0`.
- Broad workspace closeout gate passed after clippy remediation:
  - `cargo fmt --check` — passed.
  - `cargo check --workspace` — passed.
  - `cargo clippy --workspace --all-targets --all-features -- -D warnings` — passed.
  - `cargo test --workspace` — passed.
  - `cargo doc --workspace --no-deps` — passed.
  - doc warning audit: `grep -i '^warning:' /tmp/ash-phase113-doc.log` found no warnings.
- Broad gate remediation performed in this task:
  - boxed large `TypeFunctionPattern::DomainConstructor` identity fields to satisfy `clippy::large-enum-variant`.
  - boxed source-equation selection result payload in the normalizer.
  - collapsed a nested pattern-resolution `if let` and introduced `TypeFunctionResultLoweringContext` to satisfy `too_many_arguments` without weakening semantics.
  - rewrote a `map(...).unwrap_or_else(...)` engine boundary helper into `map_or_else`.
- No SPEC-F/G/H behavior was added; public/export/import semantics remain rejected or fenced until later specs.
