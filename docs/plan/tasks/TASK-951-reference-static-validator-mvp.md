# TASK-951: Reference static validator MVP

## Status: 📝 Planned

## Description

Implement the first repo-local static validator for reference frontmatter, links, paths, and internal IDs. Use the `ash-documentation-style-guide` skill for documentation tone and style.

## Specification Reference

- [DESIGN-042](../../design/DESIGN-042-REFERENCE-CORPUS-AND-DOCUMENTATION-GOVERNANCE.md)
- [SPEC-071](../../spec/SPEC-071-REFERENCE-CORPUS-METADATA-AND-MAINTENANCE.md)
- [PLAN-120](../PLAN-120-REFERENCE-CORPUS-ROLLOUT.md)
- PLAN-INDEX Phase 124

## Dependencies

- TASK-948 completion
- TASK-949 completion

## Requirements

### Functional Requirements

1. Create `tools/reference/check_frontmatter.py`.
2. Create `tools/reference/check_links.py` or equivalent scoped link/path checker.
3. Validate required SPEC-071 fields and enum values.
4. Validate repo-relative paths in `verified_against`.
5. Validate internal reference IDs for the pilot slice.
6. Add focused tests or self-check fixtures so the validator cannot pass with zero coverage.

### Non-goals

- Do not rewrite or move the whole `docs/` corpus.
- Do not create a dynamic wiki/service unless a later phase explicitly owns it.
- Do not duplicate independent semantics for AI-agent material.

## TDD / Work Steps

1. Re-read DESIGN-042, SPEC-071, and PLAN-120 before editing.
2. Make the smallest documentation/tooling change that satisfies this task.
3. Run the focused verification commands listed below.
4. Record any drift or intentionally deferred work instead of overclaiming.
5. Request independent review before marking complete.

## Dispatch

```yaml
agent: codex
reasoning: high
toolsets: [terminal, file]
```

Codex instructions:

- Work in a dedicated worktree.
- Do not spawn nested agents.
- Keep this task's scope narrow.
- Return exact files changed, commands run, and remaining blockers.

## Verification

```yaml
strictness: clean
commands:
  - git diff --check
  - python3 -m py_compile tools/reference/check_frontmatter.py
  - python3 tools/reference/check_frontmatter.py --pilot
checklist:
  - [ ] Documentation impact classified.
  - [ ] CHANGELOG.md updated if docs policy or release-facing status changed.
  - [ ] New/changed links are scoped-checked.
  - [ ] Reference metadata and authority links are honest for this task's maturity.
```

Because this task adds tooling, also run `python3 -m py_compile tools/reference/check_frontmatter.py` and any focused validator test command added by the task.
