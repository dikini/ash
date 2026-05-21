# TASK-933: Implemented-MVP acceptance delta and preflight audit

## Status: ✅ Complete

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

1. Add `docs/plan/audits/TASK-933-phase123-acceptance-delta.md` listing A69-8, A69-12, A70-2, A70-4, A70-6/NI-4, A70-7, and A70-8.
2. For each row, name the exact planned test file, test name, implementation file, and expected RED failure mode.
3. Check that no Phase 122 task is reopened; this phase is a follow-on.
4. Update PLAN-119 if preflight discovers a missing prerequisite or wrong file target.
5. Replace presence-only row checks with an exactly-one-owner audit over the acceptance-delta artifact's canonical owner table for TASK-934 through TASK-940.

Property invariant: every deferred row has exactly one owning follow-on task in the acceptance-delta artifact and no row is silently dropped.

## TDD Steps

1. Read TASK-931/TASK-932 and SPEC-069/SPEC-070 limitation text.
2. Create/update the Phase 123 acceptance-delta artifact or PLAN-119 section.
3. Run the exactly-one-owner script from this task's Verification section and confirm every deferred row appears in exactly one TASK-934 through TASK-940 owner mapping in the acceptance-delta artifact.
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
import re
artifact = Path('docs/plan/audits/TASK-933-phase123-acceptance-delta.md')
text = artifact.read_text()
expected = {
    'A69-8': 'TASK-934',
    'A69-12': 'TASK-936',
    'A70-2': 'TASK-937',
    'A70-4': 'TASK-938',
    'A70-6': 'TASK-939',
    'NI-4': 'TASK-939',
    'A70-7': 'TASK-940',
    'A70-8': 'TASK-936',
}
rows = [line for line in text.splitlines() if line.startswith('| A') or line.startswith('| NI-4')]
owners = {}
for line in rows:
    cells = [cell.strip() for cell in line.strip('|').split('|')]
    if len(cells) < 3:
        continue
    row_id, owner = cells[0], cells[2]
    owners[row_id] = owner
assert owners == expected, (owners, expected)
for row_id, owner in expected.items():
    task_text = next(Path('docs/plan/tasks').glob(f'{owner}-*.md')).read_text()
    assert row_id in task_text, f'{row_id} missing from {owner}'
assert re.search(r'exactly one owning follow-on task', text, re.I), 'artifact must state owner invariant'
print('phase123 deferred rows have exactly one owner')
PY
  - git diff --check
checklist:
  - [x] Focused RED test was observed failing for the intended reason, unless this is a docs/planning task. Docs/planning task: exact-owner verifier is the focused evidence.
  - [x] Focused GREEN test passes and runs non-zero tests, unless this is a docs/planning task. Docs/planning task: exact-owner verifier passed with `phase123 deferred rows have exactly one owner`.
  - [x] cargo fmt --check passes when Rust code changed. No Rust code changed for TASK-933.
  - [x] git diff --check passes.
  - [x] cargo check --workspace passes if shared carriers or public APIs changed. No shared carriers or public APIs changed for TASK-933.
  - [x] cargo clippy --workspace --all-targets --all-features -- -D warnings passes before task closeout if code changed. No Rust code changed for TASK-933.
  - [x] CHANGELOG.md updated if code/tooling/docs-policy/release-facing status changed.
  - [x] Codex verification reports no blockers: remediation re-review returned APPROVE after checking canonical owner mapping, Phase 122/TASK-932 status consistency, Phase 123 non-promotion, SPEC-069/SPEC-070 Partial MVP status, and `git diff --check`.
```

## Dependencies for Next Task

Produces Phase 123 evidence for downstream closeout and status reconciliation.

## Notes

Do not mark this task complete until its own focused evidence, status surfaces, and Codex verification are reconciled.
