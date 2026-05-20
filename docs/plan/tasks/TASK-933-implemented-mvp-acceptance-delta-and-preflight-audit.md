# TASK-933: Implemented-MVP acceptance delta and preflight audit

## Status: 📝 Planned

## Description

Create the exact closure matrix for SPEC-069/SPEC-070 Implemented MVP before changing Rust code. This audit gate maps every Phase 122 deferred row to concrete tests, files, and implementation seams, and fails closed if any row lacks a runnable evidence recipe.

## Specification Reference

- SPEC-069: Alpha Visible Tower Algebra and Generalized Do Lowering
- SPEC-070: Alpha Runtime Kernel and OS-Facing Execution Surface
- PLAN-119: Implemented MVP Closure

## Dependencies

- ✅ TASK-932: Phase 122 Partial MVP closeout and documented limitations

## Requirements

### Functional Requirements

1. Add a Phase 123 acceptance-delta section or artifact that lists A69-8, A69-12, A70-2, A70-4, A70-6/NI-4, A70-7, and A70-8.
2. For each row, name the exact planned test file, test name, implementation file, and expected RED failure mode.
3. Check that no Phase 122 task is reopened; this phase is a follow-on.
4. Update PLAN-119 if preflight discovers a missing prerequisite or wrong file target.

Property invariant: every deferred row has exactly one owning follow-on task and no row is silently dropped.

## TDD Steps

1. Read TASK-931/TASK-932 and SPEC-069/SPEC-070 limitation text.
2. Create/update the Phase 123 acceptance-delta artifact or PLAN-119 section.
3. Run a script asserting every deferred row appears in exactly one TASK-934 through TASK-940 file.
4. Ask Codex to review for invented APIs, missing acceptance rows, and task ordering.

## Dispatch

```yaml
agent: codex
reasoning: high
max_turns: 20
toolsets: [terminal, file]
```

Codex instructions:
- Work in a dedicated worktree.
- Do not spawn nested agents.
- Follow RED-GREEN-REFACTOR for code tasks.
- Keep the task scope narrow; do not implement later tasks early.
- Return exact files changed, focused commands run, and any remaining blockers.

## Verification

```yaml
strictness: clean
commands:
  - python3 - <<'PY'
from pathlib import Path
root=Path('docs/plan/tasks')
required=['A69-8','A69-12','A70-2','A70-4','A70-6','A70-7','A70-8','NI-4']
text='
'.join(p.read_text() for p in sorted(root.glob('TASK-93[4-9]-*.md'))+sorted(root.glob('TASK-940-*.md')))
missing=[r for r in required if r not in text]
assert not missing, missing
print('phase123 deferred-row owners present')
PY
  - git diff --check
checklist:
  - [ ] Focused RED test was observed failing for the intended reason, unless this is a docs/planning task.
  - [ ] Focused GREEN test passes and runs non-zero tests, unless this is a docs/planning task.
  - [ ] cargo fmt --check passes when Rust code changed.
  - [ ] git diff --check passes.
  - [ ] cargo check --workspace passes if shared carriers or public APIs changed.
  - [ ] cargo clippy --workspace --all-targets --all-features -- -D warnings passes before task closeout if code changed.
  - [ ] CHANGELOG.md updated if code/tooling/docs-policy/release-facing status changed.
  - [ ] Codex verification reports no blockers.
```

## Dependencies for Next Task

Produces Phase 123 evidence for downstream closeout and status reconciliation.

## Notes

Do not mark this task complete until its own focused evidence, status surfaces, and Codex verification are reconciled.
