# TASK-941: SPEC-069/SPEC-070 Implemented MVP closeout

## Status: 📝 Planned

## Description

Run final verification and reconcile every status surface so SPEC-069/SPEC-070 are promoted to Implemented MVP only if all formerly partial rows have concrete evidence and independent review reports no blockers.

## Specification Reference

- SPEC-069 all acceptance rows
- SPEC-070 all acceptance rows
- PLAN-119
- PLAN-INDEX Phase 123

## Dependencies

- TASK-934 completion
- TASK-935 completion
- TASK-936 completion
- TASK-937 completion
- TASK-938 completion
- TASK-939 completion
- TASK-940 completion

## Requirements

### Functional Requirements

1. Update SPEC-069 and SPEC-070 status to Implemented MVP only after evidence passes.
2. Update `docs/spec/README.md`, PLAN-119, PLAN-INDEX, task files, acceptance matrix/audit docs, and CHANGELOG.
3. Remove or reclassify limitation language for A69-8, A69-12, A70-2, A70-4, A70-6/NI-4, A70-7, A70-8.
4. Preserve historical Phase 122 Partial MVP language where it describes past state.
5. Run broad gates and final Codex phase audit.

Property invariant: no current-status document may claim Partial MVP after promotion, and no historical document may be silently rewritten to erase Phase 122 history.

## TDD Steps

1. Re-read TASK-934 through TASK-940 evidence.
2. Update all status surfaces and CHANGELOG.
3. Run broad gates from PLAN-119.
4. Ask Codex to audit Phase 123 against SPEC-069/SPEC-070 and status honesty.

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
  - cargo fmt --check
  - git diff --check
  - cargo check --workspace
  - cargo clippy --workspace --all-targets --all-features -- -D warnings
  - scripts/check-rust-tests.sh --workspace
  - cargo doc --workspace --no-deps 2>&1 | tee /tmp/phase123-doc.log && ! grep -i "^warning:" /tmp/phase123-doc.log
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
