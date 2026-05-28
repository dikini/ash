# TASK-964: Ashgrove install policy packet

## Status: ✅ Complete

## Description

Create the SPEC-073/PLAN-122/task packet for Ash install, update, cleanup, removal, and git deployment policy.

## Specification Reference

- SPEC-073 §1-§22
- PLAN-122 §1-§10

## Dependencies

- None; this task creates the Phase 127 packet.

## Requirements

### Functional Requirements

1. Create `docs/spec/SPEC-073-ASHGROVE-INSTALL-UPDATE-CLEANUP-GIT-DEPLOYMENT.md`.
2. Create `docs/plan/PLAN-122-ASHGROVE-INSTALL-UPDATE-CLEANUP-GIT-DEPLOYMENT.md`.
3. Create TASK-964 through TASK-974 task files.
4. Update `docs/spec/README.md`, `docs/plan/PLAN-INDEX.md`, and `CHANGELOG.md`.
5. Verify every new packet file exists and scoped Markdown links resolve.

### Non-goals

- Do not implement Rust install tooling in this docs-packet task.
- Do not add a package registry, release-channel resolver, global install, or mandatory signing requirement.

## Work Steps

1. Inspect the exact live files named by the task or audit output.
2. Write focused RED tests or docs assertions before changing behavior.
3. Implement or document the minimal target behavior.
4. Run focused verification.
5. Update status surfaces and `CHANGELOG.md` if files beyond tests are changed.
6. Request independent review before marking complete.

## Verification

```yaml
strictness: clean
commands:
  - git diff --check
  - python3 - <<'PY'
from pathlib import Path
import re
required = [
    'docs/spec/SPEC-073-ASHGROVE-INSTALL-UPDATE-CLEANUP-GIT-DEPLOYMENT.md',
    'docs/plan/PLAN-122-ASHGROVE-INSTALL-UPDATE-CLEANUP-GIT-DEPLOYMENT.md',
    'docs/plan/tasks/TASK-964-ashgrove-install-policy-packet.md',
    'docs/plan/tasks/TASK-965-ashgrove-live-install-audit-gate.md',
    'docs/plan/tasks/TASK-966-ashgrove-cli-crate-and-command-skeleton.md',
    'docs/plan/tasks/TASK-967-toolchain-metadata-and-xdg-layout.md',
    'docs/plan/tasks/TASK-968-source-install-flow.md',
    'docs/plan/tasks/TASK-969-binary-tarball-install-flow.md',
    'docs/plan/tasks/TASK-970-update-default-list-current-flow.md',
    'docs/plan/tasks/TASK-971-remove-cleanup-flow.md',
    'docs/plan/tasks/TASK-972-ash-manifest-lock-git-fetch.md',
    'docs/plan/tasks/TASK-973-vendor-and-deployable-git-project-flow.md',
    'docs/plan/tasks/TASK-974-ashgrove-closeout-acceptance.md',
    'docs/spec/README.md',
    'docs/plan/PLAN-INDEX.md',
    'CHANGELOG.md',
]
for rel in required:
    assert Path(rel).exists(), rel
link_re = re.compile(r'\[[^\]]+\]\(([^)]+)\)')
for rel in required:
    path = Path(rel)
    text = path.read_text()
    for target in link_re.findall(text):
        if '://' in target or target.startswith('#') or target.startswith('mailto:'):
            continue
        target_path = target.split('#', 1)[0]
        if not target_path:
            continue
        assert (path.parent / target_path).exists(), f'{rel}: broken link {target}'
PY
checklist:
  - [x] Create `docs/spec/SPEC-073-ASHGROVE-INSTALL-UPDATE-CLEANUP-GIT-DEPLOYMENT.md`.
  - [x] Create `docs/plan/PLAN-122-ASHGROVE-INSTALL-UPDATE-CLEANUP-GIT-DEPLOYMENT.md`.
  - [x] Create TASK-964 through TASK-974 task files.
  - [x] Update `docs/spec/README.md`, `docs/plan/PLAN-INDEX.md`, and `CHANGELOG.md`.
  - [x] Verify every new packet file exists and scoped Markdown links resolve.
```


## Dispatch

```yaml
agent: codex
reasoning: high
toolsets: [terminal, file]
```

## Dependencies for Next Task

This task contributes to PLAN-122 and SPEC-073 completion. Later tasks must preserve the alpha rules that toolchains are immutable, stdlib is bundled with the selected toolchain, lower-case `ash.toml` is the project manifest, and git dependencies resolve to exact commits in `ash.lock`.


## Notes

Area: docs/planning. This task freezes the initial packet only; TASK-965 owns live-code audit before implementation.
