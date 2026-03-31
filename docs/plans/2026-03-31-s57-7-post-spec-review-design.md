# TASK-S57-7 Post-SPEC Review Design

## Context

TASK-S57-7 is the final Phase 57A validation pass over the downstream 57B implementation tasks. After S57-1 through S57-6, the implementation task files must no longer rely on stale import syntax, outdated capability typing forms, unresolved entry-workflow assumptions, or blocked-status wording that the completed specs have now settled.

## Decision

- `TASK-S57-7` will become the canonical post-spec review record for Phase 57B task alignment.
- The review will audit all Phase 57B tasks (`TASK-359` through `TASK-369`) against completed S57-1 through S57-6 specification updates.
- Only downstream task files that still contain stale assumptions or invalid examples will be edited.
- Tasks that already align with the updated specs will be listed in the S57-7 review record as reviewed with no content change required.
- The review will focus on normative drift caused by:
  - stdlib import and namespace rules from S57-4
  - usage-site capability typing and explicit invocation from S57-5
  - entry workflow typing from S57-6
  - task status and validation-gate drift after the completion of 57A

## Rationale

This keeps S57-7 small and high-signal. The review task should remove known-invalid downstream assumptions without generating noisy edits across every 57B task file. Centralizing the audit summary in the S57-7 task file also gives future implementers a single validation gate to consult before starting 57B work.

## Scope

This design updates:

- `docs/plan/tasks/TASK-S57-7-post-spec-review.md`
- the stale 57B task files that still conflict with S57-1 through S57-6
- `docs/plan/PLAN-INDEX.md`
- `CHANGELOG.md`

It does not implement any runtime, CLI, stdlib, or type-checker behavior. It only validates and corrects planning/task artifacts so later implementation tasks begin from aligned assumptions.
