# TASK-975: SPEC-073 Ashgrove completion packet

## Status: ✅ Complete

## Description

Create the Phase 128 follow-on packet for SPEC-073 completion. This task registers PLAN-123, creates TASK-975 through TASK-986, updates the relevant status surfaces, and keeps SPEC-073 in Draft until implementation evidence exists.

## Specification Reference

- [SPEC-073](../../spec/SPEC-073-ASHGROVE-INSTALL-UPDATE-CLEANUP-GIT-DEPLOYMENT.md) §21-§22
- [PLAN-123](../PLAN-123-SPEC-073-ASHGROVE-RELEASE-TRUST-REGISTRY-RUNTIME-SUPPORT-COMPLETION.md)

## Dependencies

- ✅ TASK-974: Phase 127 closeout report identified the deferred SPEC-073 rows.

## Requirements

### Functional Requirements

1. Register Phase 128 in `docs/plan/PLAN-INDEX.md`.
2. Create `docs/plan/PLAN-123-SPEC-073-ASHGROVE-RELEASE-TRUST-REGISTRY-RUNTIME-SUPPORT-COMPLETION.md`.
3. Create TASK-975 through TASK-986 task files.
4. Amend SPEC-073 to point at PLAN-123/TASK-975..986 as follow-on completion work without promoting the spec.
5. Update `docs/spec/README.md` and `CHANGELOG.md` without claiming implementation.

### Property Requirements

No Rust property tests are expected for this docs packet. The invariant is documentation traceability: every Phase 128 task must be discoverable from PLAN-INDEX, PLAN-123, and its task file.

## TDD Steps

### Step 1: Inspect current Phase 127 closeout evidence

Read TASK-974 report, TASK-974 task file, and SPEC-073 status. Confirm that SPEC-073 remains Draft and that deferred rows are explicit.

### Step 2: Create packet files

Create PLAN-123 and TASK-975 through TASK-986 with exact dependencies, decision gates, and fail-closed placeholders where TASK-976 must replace verification.

### Step 3: Register packet

Update `PLAN-INDEX.md`, SPEC-073, `docs/spec/README.md`, and `CHANGELOG.md`.

### Step 4: Verify packet links and status

Run the commands in this task's `Verification` block.

## Dispatch

```yaml
agent: codex
reasoning: high
toolsets: [terminal, file]
```

## Verification

```yaml
strictness: clean
commands:
  - git diff --check
  - python3 -c "from pathlib import Path; files=['docs/spec/SPEC-073-ASHGROVE-INSTALL-UPDATE-CLEANUP-GIT-DEPLOYMENT.md','docs/spec/README.md','docs/plan/PLAN-INDEX.md','docs/plan/PLAN-123-SPEC-073-ASHGROVE-RELEASE-TRUST-REGISTRY-RUNTIME-SUPPORT-COMPLETION.md','CHANGELOG.md']+[f'docs/plan/tasks/TASK-{n}-{slug}.md' for n,slug in [(975,'spec073-ashgrove-completion-packet'),(976,'ashgrove-completion-acceptance-delta-and-audit-gate'),(977,'source-archive-release-metadata'),(978,'runtime-support-payload-metadata'),(979,'release-index-authenticated-tarball-url-policy'),(980,'packaged-dispatcher-lifecycle-policy'),(981,'registry-scale-package-metadata-substrate'),(982,'cleanup-lockfile-cache-reachability'),(983,'manifest-rewrite-trust-preservation'),(984,'mandatory-trust-signing-and-remote-git-fetch-policy'),(985,'ashgrove-release-deployment-acceptance-integration'),(986,'spec073-implemented-mvp-closeout')]]; missing=[p for p in files if not Path(p).exists()]; assert not missing, missing; idx=Path('docs/plan/PLAN-INDEX.md').read_text(); assert '## Phase 128:' in idx; assert all(f'TASK-{n}' in idx for n in range(975,987)); print('phase128 docs packet verified')"
checklist:
  - [x] Create PLAN-123.
  - [x] Create TASK-975 through TASK-986.
  - [x] Register Phase 128 in PLAN-INDEX and progress summary.
  - [x] Amend SPEC-073 to mention PLAN-123/TASK-975..986 as follow-on completion work.
  - [x] Update docs/spec/README.md without promoting SPEC-073 yet.
  - [x] Update CHANGELOG.md.
  - [x] Verify scoped packet files.
```

## Dependencies for Next Task

This task outputs the Phase 128 packet. TASK-976 must run before any Rust implementation task starts.

## Notes

This is docs/planning work only. It must not claim that release, trust, registry-ready metadata, runtime-support, cleanup reachability, or remote git policy is implemented.
