# TASK-976: Ashgrove completion acceptance delta and audit gate

## Status: ✅ Complete

## Description

Create the acceptance-delta audit that turns every TASK-974 deferred SPEC-073 row into exact implementation owners, file targets, focused RED tests, expected failures, and verification commands. This is the hard gate before TASK-977 through TASK-985 may edit Rust code.

## Specification Reference

- [SPEC-073](../../spec/SPEC-073-ASHGROVE-INSTALL-UPDATE-CLEANUP-GIT-DEPLOYMENT.md) §20-§22
- [PLAN-123](../PLAN-123-SPEC-073-ASHGROVE-RELEASE-TRUST-REGISTRY-RUNTIME-SUPPORT-COMPLETION.md) §4-§6

## Dependencies

- ✅ TASK-975: Phase 128 packet created.
- ✅ TASK-974: Phase 127 closeout report identified deferred gaps.

## Requirements

### Functional Requirements

1. Create `docs/plan/audits/TASK-976-ashgrove-completion-acceptance-delta.md`.
2. Map every deferred gap to exactly one owning implementation task.
3. For each owner, name the exact production files, test files, test names, and expected RED failure modes.
4. Replace the fail-closed placeholder verification blocks in TASK-977 through TASK-985 with executable focused non-zero commands.
5. Reconcile PLAN-123 if audit discoveries change task order, file targets, or dependencies.

### Property Requirements

No Rust properties are expected in the audit itself. The key invariant is ownership uniqueness: every deferred gap must have exactly one owner and no owner may claim implementation before focused tests exist.

## TDD Steps

### Step 1: Read closeout evidence

Inspect TASK-974 report and SPEC-073 acceptance rows A73-1 through A73-12.

### Step 2: Write acceptance-delta artifact

Create a table with columns: deferred gap, acceptance row, owner task, production files, test files, RED failure mode, required GREEN evidence, status surface updates.

### Step 3: Patch downstream task verification

Replace `false # TASK-976...` placeholders in TASK-977 through TASK-985 with focused commands.

### Step 4: Verify no placeholder remains

Run this task's verification commands.

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
  - test -f docs/plan/audits/TASK-976-ashgrove-completion-acceptance-delta.md
  - python3 -c "from pathlib import Path; text=Path('docs/plan/audits/TASK-976-ashgrove-completion-acceptance-delta.md').read_text(); expected={'source-archive-release-metadata':'TASK-977','runtime-support-payload-metadata':'TASK-978','authenticated-tarball-url-release-index':'TASK-979','packaged-dispatcher-lifecycle':'TASK-980','registry-scale-package-metadata':'TASK-981','cleanup-lockfile-cache-reachability':'TASK-982','manifest-rewrite-trust-preservation':'TASK-983','mandatory-trust-signing-enforcement':'TASK-984','remote-authenticated-git-fetch':'TASK-984','release-deployment-acceptance-integration':'TASK-985'}; [(_ for _ in ()).throw(AssertionError(k)) for k,v in expected.items() if k not in text or v not in text]; [(_ for _ in ()).throw(AssertionError(str(p))) for n in range(977,986) for p in [next(Path('docs/plan/tasks').glob(f'TASK-{n}-*.md'))] if 'false # TASK-976' in p.read_text() or 'placeholder' in p.read_text().lower()]; assert 'exactly one owning follow-on task' in text; print('phase128 deferred rows have exact owners and downstream verification is bound')"
checklist:
  - [x] Create acceptance-delta audit artifact.
  - [x] Map every TASK-974 deferred gap to exactly one owning follow-on task.
  - [x] Name exact implementation files, test files, test names, and expected RED failure modes.
  - [x] Confirm Phase 127 history is not reopened.
  - [x] Replace all downstream fail-closed placeholder verification blocks.
  - [x] Update PLAN-123 if audit discovers different file targets or sequencing.
```

## Dependencies for Next Task

TASK-977 through TASK-985 require this audit to replace their placeholder verification with focused commands.

## Notes

Do not implement release or trust behavior in this task. Its deliverable is executable planning evidence.
