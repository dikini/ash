# TASK-987: Ashgrove source payload local-state packet

## Status: ✅ Complete

## Description

Create the SPEC-074/PLAN-124 packet for fixing ashgrove source-root installs so ignored local state is not treated as source payload. This task is docs/planning only and must not claim Rust implementation.

## Specification Reference

- [SPEC-074](../../spec/SPEC-074-ASHGROVE-SOURCE-PAYLOAD-LOCAL-STATE-IGNORE.md)
- [PLAN-124](../PLAN-124-ASHGROVE-SOURCE-PAYLOAD-LOCAL-STATE-IGNORE.md)
- Amends [SPEC-073](../../spec/SPEC-073-ASHGROVE-INSTALL-UPDATE-CLEANUP-GIT-DEPLOYMENT.md) §7.1

## Dependencies

- ✅ TASK-986: SPEC-073 Implemented MVP closeout.
- ✅ Live investigation reproduced the local-state false dirtying path with `.agents/status/dashboard.json`.

## Requirements

### Functional Requirements

1. Create `docs/spec/SPEC-074-ASHGROVE-SOURCE-PAYLOAD-LOCAL-STATE-IGNORE.md`.
2. Create `docs/plan/PLAN-124-ASHGROVE-SOURCE-PAYLOAD-LOCAL-STATE-IGNORE.md`.
3. Create TASK-987 through TASK-990 task files.
4. Register SPEC-074 in `docs/spec/README.md`.
5. Register Phase 129 in `docs/plan/PLAN-INDEX.md`.
6. Add a SPEC-073 amendment note without changing SPEC-073 Implemented MVP status.
7. Add a `CHANGELOG.md` entry under `[Unreleased]`.

### Property Requirements

No Rust property tests are expected. The planning invariant is traceability: SPEC-074, PLAN-124, PLAN-INDEX, and TASK-987 through TASK-990 must cross-link without stale placeholder IDs.

## TDD Steps

### Step 1: Inspect live bug facts

Confirm the live implementation facts:

- `source_digest_skip_path` skips only top-level `.git` and `target`.
- `source_tree_digest` and `copy_source_tree_for_build` both use the same shallow predicate.
- ignored `.agents/status/dashboard.json` can change while the source root remains git-clean.

### Step 2: Create packet files

Create the SPEC, PLAN, and task files listed above.

### Step 3: Register packet

Patch SPEC-073, spec index, PLAN-INDEX, and CHANGELOG.

### Step 4: Verify docs packet

Run the commands in this task's Verification block.

## Dispatch

```yaml
agent: hermes
reasoning: high
toolsets: [terminal, file]
```

## Verification

```yaml
strictness: clean
commands:
  - git diff --check
  - python3 -c "from pathlib import Path; files=['docs/spec/SPEC-074-ASHGROVE-SOURCE-PAYLOAD-LOCAL-STATE-IGNORE.md','docs/plan/PLAN-124-ASHGROVE-SOURCE-PAYLOAD-LOCAL-STATE-IGNORE.md','docs/plan/tasks/TASK-987-ashgrove-source-payload-local-state-packet.md','docs/plan/tasks/TASK-988-ashgrove-source-payload-audit-gate.md','docs/plan/tasks/TASK-989-ashgrove-source-payload-ignore-implementation.md','docs/plan/tasks/TASK-990-ashgrove-source-payload-local-state-closeout.md']; missing=[p for p in files if not Path(p).exists()]; assert not missing, missing; idx=Path('docs/plan/PLAN-INDEX.md').read_text(); assert '## Phase 129:' in idx and all(f'TASK-{n}' in idx for n in range(987,991)); spec_index=Path('docs/spec/README.md').read_text(); assert 'SPEC-074' in spec_index; spec073=Path('docs/spec/SPEC-073-ASHGROVE-INSTALL-UPDATE-CLEANUP-GIT-DEPLOYMENT.md').read_text(); assert '[SPEC-074]' in spec073 and 'source-root payload/local-state' in spec073; changelog=Path('CHANGELOG.md').read_text(); assert changelog.count('TASK-987') >= 1; print('phase129 source-payload packet verified')"
checklist:
  - [x] Create SPEC-074.
  - [x] Create PLAN-124.
  - [x] Create TASK-987 through TASK-990.
  - [x] Register SPEC-074 and Phase 129.
  - [x] Update CHANGELOG.md.
```

## Dependencies for Next Task

This task outputs the planning packet. TASK-988 must run before TASK-989 Rust implementation starts.

## Notes

Do not mark SPEC-074 Implemented MVP from this task. Implementation evidence belongs to TASK-989 and TASK-990.
