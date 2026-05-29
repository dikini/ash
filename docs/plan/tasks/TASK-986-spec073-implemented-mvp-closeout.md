# TASK-986: SPEC-073 Implemented MVP closeout

## Status: 📝 Planned

## Description

Close the Phase 128 follow-on by proving every SPEC-073 acceptance row has concrete evidence or an explicitly accepted non-MVP boundary. Only this task may promote SPEC-073 beyond Draft, and only after broad gates and independent review pass.

## Specification Reference

- [SPEC-073](../../spec/SPEC-073-ASHGROVE-INSTALL-UPDATE-CLEANUP-GIT-DEPLOYMENT.md) §20-§22
- [PLAN-123](../PLAN-123-SPEC-073-ASHGROVE-RELEASE-TRUST-REGISTRY-RUNTIME-SUPPORT-COMPLETION.md) §7

## Dependencies

- 📝 Depends on TASK-977 through TASK-985 completion.

## Requirements

### Functional Requirements

1. Create `docs/plan/audits/TASK-986-spec073-completion-closeout-evidence.md`.
2. Map A73-1 through A73-12 to current commands, tests, files, and evidence.
3. Reconcile SPEC-073, docs/spec/README.md, PLAN-122, PLAN-123, PLAN-INDEX, all TASK-975..986 files, audit artifacts, and CHANGELOG.md.
4. Preserve Phase 127 as historical partial closeout language.
5. Run broad gates and independent review before changing SPEC-073 status.

### Property Requirements

No Rust properties are expected unless TASK-985 discovers integration gaps. The closeout invariant is evidence completeness: no acceptance row may be accepted by prose alone.

## TDD Steps

### Step 1: Build acceptance matrix

Create the closeout evidence artifact and map every A73 row to concrete evidence.

### Step 2: Run broad gates

Run the verification commands below. If any broad gate fails, keep TASK-986 in progress and do not promote SPEC-073.

### Step 3: Run independent review

Dispatch a review subagent to inspect the status surfaces, acceptance evidence, and overclaim risks.

### Step 4: Promote or defer honestly

Promote SPEC-073 only if the evidence supports it. If a row remains intentionally out of MVP, document that boundary explicitly instead of using the Phase 127 deferred wording.

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
  - bash scripts/check-rust-format.sh
  - bash scripts/check-rust-clippy.sh
  - bash scripts/check-rust-tests.sh --workspace --all-targets
  - bash scripts/check-doc-tests.sh
  - git diff --check
  - python3 -c "from pathlib import Path; files=[Path('docs/spec/SPEC-073-ASHGROVE-INSTALL-UPDATE-CLEANUP-GIT-DEPLOYMENT.md'),Path('docs/spec/README.md'),Path('docs/plan/PLAN-122-ASHGROVE-INSTALL-UPDATE-CLEANUP-GIT-DEPLOYMENT.md'),Path('docs/plan/PLAN-123-SPEC-073-ASHGROVE-RELEASE-TRUST-REGISTRY-RUNTIME-SUPPORT-COMPLETION.md'),Path('docs/plan/PLAN-INDEX.md'),*sorted(Path('docs/plan/tasks').glob('TASK-97[5-9]-*.md')),*sorted(Path('docs/plan/tasks').glob('TASK-98[0-6]-*.md')),Path('docs/plan/audits/TASK-976-ashgrove-completion-acceptance-delta.md'),Path('docs/plan/audits/TASK-986-spec073-completion-closeout-evidence.md')]; missing=[str(p) for p in files if not p.exists()]; assert not missing, missing; spec=files[0].read_text(); closeout=Path('docs/plan/audits/TASK-986-spec073-completion-closeout-evidence.md').read_text(); assert all(f'A73-{n}' in spec and f'A73-{n}' in closeout for n in range(1,13)); print(f'checked {len(files)} SPEC-073/Phase128 status files')"
checklist:
  - [ ] A73-1 through A73-12 have concrete evidence or explicit accepted non-MVP boundaries.
  - [ ] TASK-974 deferred gaps have current successor evidence.
  - [ ] SPEC-073, docs/spec/README.md, PLAN-122, PLAN-123, PLAN-INDEX, tasks, audits, and CHANGELOG agree on status.
  - [ ] Historical Phase 127 partial language is preserved.
  - [ ] Broad gates pass on final diff.
  - [ ] Independent phase review completed and blockers remediated or status represented honestly.
```

## Dependencies for Next Task

This is the final task in Phase 128.

## Notes

Do not promote SPEC-073 if any row still depends on future trust, release, registry, cleanup, or runtime-support work.
