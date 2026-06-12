# TASK-1432: Phase 143 Closeout and Re-review

## Status: 📝 Planned

## Description

Finalize Phase 143 by reconciling all status surfaces, running focused and broad verification gates, and performing a final independent review specifically against the original Phase 142 blockers.

## Specification Reference

- PLAN-143: MCP Cross-Language Completion Remediation
- Original Phase 142 review findings
- `docs/plan/PLAN-INDEX.md`
- `CHANGELOG.md`

## Dependencies

- 📝 TASK-1427: Status and artifact hygiene
- 📝 TASK-1428: Tool wiring
- 📝 TASK-1429: Real syn parser
- 📝 TASK-1430: Config and fixtures
- 📝 TASK-1431: Evaluation report

## Requirements

### Functional Requirements

1. Mark TASK-1427 through TASK-1432 complete only after their checklists are satisfied.
2. Update PLAN-143 status and task table to complete.
3. Update PLAN-INDEX Phase 143 row from planned to complete.
4. Reconcile any remaining Phase 142 status notes so they point to completed Phase 143 remediation.
5. Run final review focused on the original blockers: tool registration, parser reality, status corpus, evaluation report, config fixtures, artifact cleanup.
6. Record exact verification evidence in this task file or a closeout note.

### Property Requirements

No proptest requirement; this is a closeout/review task.

## TDD Steps

### Step 1: Re-run blocker checklist

Create a checklist mapping each Phase 142 blocker to evidence:
- tool registry test,
- syn parser test,
- status corpus assertion,
- evaluation report path,
- config positive fixture test,
- cleanup proof.

### Step 2: Broad gates

Run focused gates first, then broad gates where feasible. Classify unrelated pre-existing failures honestly.

### Step 3: Independent review

Use `code-review`/`ash-phase-review` procedure or a Codex sub-agent. Do not finalize if review finds blockers.

## Dispatch

```yaml
agent: hermes
reasoning: high
max_turns: 18
toolsets: [terminal, file]
```

## Verification

```yaml
strictness: clean
commands:
  - git status --short
  - cargo fmt --check
  - cargo test -p ash-mcp
  - cargo clippy -p ash-mcp --all-targets -- -D warnings
  - cargo test -p ash-mcp-bench
  - cargo clippy -p ash-mcp-bench --all-targets -- -D warnings
  - cargo check --workspace
  - cargo clippy --workspace --all-targets --all-features -- -D warnings
  - cargo test --workspace --lib -- --test-threads=1
checklist:
  - [ ] All original Phase 142 blockers have evidence-backed fixes
  - [ ] Phase 143 task files checked complete with evidence
  - [ ] PLAN-143 and PLAN-INDEX agree
  - [ ] CHANGELOG.md has scoped entries under [Unreleased]
  - [ ] Independent re-review found no blocking issues
  - [ ] Worktree/branch cleanup complete after merge
```

## Dependencies for Next Task

This task closes Phase 143 and unblocks any future Phase 144 expansion of cross-language analysis.
