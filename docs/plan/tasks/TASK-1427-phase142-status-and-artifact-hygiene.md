# TASK-1427: Phase 142 Status and Artifact Hygiene

## Status: 📝 Planned

## Description

Reconcile the Phase 142 documentation/status corpus and remove artifact hygiene drift discovered during post-merge review. This task does not implement cross-language behavior; it makes the current state honest before code remediation proceeds.

## Specification Reference

- PLAN-142: `docs/plan/PLAN-142-MCP-CROSS-LANGUAGE-INTEGRATION.md`
- PLAN-143: `docs/plan/PLAN-143-MCP-CROSS-LANGUAGE-COMPLETION-REMEDIATION.md`
- Review findings: Phase 142 on-main review, blocking source-of-truth drift and cleanup issues

## Dependencies

- ✅ Phase 142 merged to main
- ✅ Phase 142 post-merge review completed

## Requirements

### Functional Requirements

1. Update `docs/plan/PLAN-142-MCP-CROSS-LANGUAGE-INTEGRATION.md` so its header and task rows reflect the post-review state honestly: completed substrate pieces, reopened/remediated items owned by Phase 143.
2. Update `docs/plan/tasks/TASK-1420` through `TASK-1426` statuses/checklists so they do not still say `📋 Planned` when PLAN-INDEX says complete.
3. Remove the accidental tracked `crates/ash-mcp-bench benches/main.rs` artifact if it is confirmed unused.
4. Prune stale `.worktrees/phase-142-completion` metadata and delete stale branch metadata if safe.
5. Keep `CHANGELOG.md` under `[Unreleased]` structurally valid with one `Added` and one `Changed` subsection.

### Property Requirements

No proptest requirement; this is a docs/artifact hygiene task.

## TDD Steps

### Step 1: Status inventory (Red)

**Files:**
- `docs/plan/PLAN-142-MCP-CROSS-LANGUAGE-INTEGRATION.md`
- `docs/plan/tasks/TASK-1420-cross-lang-configuration.md`
- `docs/plan/tasks/TASK-1421-ash-to-rust-mapping.md`
- `docs/plan/tasks/TASK-1421-ash-rust-symbol-mapping.md`
- `docs/plan/tasks/TASK-1422-rust-to-ash-mapping.md`
- `docs/plan/tasks/TASK-1423-latency-optimization.md`
- `docs/plan/tasks/TASK-1424-enhanced-hover.md`
- `docs/plan/tasks/TASK-1425-verification-gates.md`
- `docs/plan/tasks/TASK-1426-phase-evaluation.md`

Write a script/assertion that fails if any Phase 142 owned task file still contains `**Status:** 📋 Planned` or if PLAN-142 still says `Planning (Not Started)`.

### Step 2: Reconcile docs (Green)

Patch Phase 142 docs to distinguish:
- completed substrate from Phase 142,
- reopened correctness/evidence gaps now owned by Phase 143,
- exact Phase 143 task links for each remediation.

### Step 3: Artifact cleanup

Remove accidental tracked path with a space only if `git ls-files` proves it is tracked and no code references it. Run `git worktree prune` for stale metadata.

## Dispatch

```yaml
agent: hermes
reasoning: medium
max_turns: 12
toolsets: [terminal, file]
```

## Verification

```yaml
strictness: clean
commands:
  - git status --short
  - git worktree list
  - python3 -c 'from pathlib import Path; files=[Path("docs/plan/PLAN-142-MCP-CROSS-LANGUAGE-INTEGRATION.md")]+list(Path("docs/plan/tasks").glob("TASK-142*.md")); bad=[str(p) for p in files if "📋 Planned" in p.read_text() or "Planning (Not Started)" in p.read_text()]; assert not bad, bad'
  - git diff --check
  - cargo fmt --check
checklist:
  - [ ] Phase 142 plan header no longer says not started
  - [ ] Phase 142 task files are status-consistent
  - [ ] Stale worktree metadata is pruned or explicitly recorded if still present
  - [ ] Accidental tracked path with a space is removed or justified
  - [ ] CHANGELOG.md updated if docs/artifacts changed
```

## Dependencies for Next Task

This task unblocks TASK-1428 by making the remediation packet honest before implementation starts.
