# TASK-988: Ashgrove source payload audit gate

## Status: ✅ Complete

## Description

Audit every ashgrove source-payload membership seam before code changes. Freeze the exact source-root payload walker strategy, source-archive non-interference boundary, install-record metadata choice, and focused verification commands for TASK-989.

## Specification Reference

- [SPEC-074](../../spec/SPEC-074-ASHGROVE-SOURCE-PAYLOAD-LOCAL-STATE-IGNORE.md) §5-§10
- [PLAN-124](../PLAN-124-ASHGROVE-SOURCE-PAYLOAD-LOCAL-STATE-IGNORE.md) §3-§7

## Dependencies

- ✅ TASK-987: SPEC-074/PLAN-124 packet exists.

## Requirements

### Functional Requirements

1. Inspect all `source_tree_digest`, `copy_source_tree_for_build`, `source_digest_skip_path`, `install_from_source_root`, and source archive callsites.
2. Decide whether git source roots use git CLI membership or an ignore-compatible Rust crate; constrained built-in local-state policy is acceptable only for non-git source roots.
3. Confirm source archive behavior remains separate from source-root payload behavior.
4. Decide whether install records gain `source_payload_digest_policy` and/or `source_payload_digest` fields in TASK-989.
5. Create or update an audit artifact under `docs/plan/audits/TASK-988-ashgrove-source-payload-audit-gate.md`.
6. Patch TASK-989 and TASK-990 verification blocks with exact focused commands and remove all placeholder `false` commands.
7. Classify live git source roots, live non-git source roots, source-shaped archives, and non-source-root source archives before TASK-989 starts.
8. Freeze source-update parity evidence and fake-cargo observation plumbing.

### Property Requirements

No proptest is required. The audit invariant is single ownership: every source payload file-selection consumer must be mapped to either live source-root payload policy or source-archive payload policy, and every downstream verification block must be executable without placeholder `false` commands.

## TDD Steps

### Step 1: Trace source-root call graph

Inspect `crates/ashgrove/src/lib.rs` and record every function that reads, copies, digests, or stages source-root files.

### Step 2: Trace source-archive call graph

Record every source-archive digest/metadata path, including source-shaped archives that satisfy `is_source_root`, and identify tests that must remain green after TASK-989.

### Step 3: Choose implementation strategy

Document the selected membership strategy and its failure behavior. Git CLI membership is selected for the first implementation: `git ls-files --cached --others --exclude-standard -z`, parsed as null-delimited relative paths, with nonzero exit treated as fail-closed for git-like roots.

### Step 4: Freeze focused tests

Name the exact test file and test functions TASK-989 must add. Required scenarios:

1. ignored `.agents/status/dashboard.json` mutates during source install and install succeeds;
2. nested `target/` is excluded from digest and isolated build copy;
3. nonignored source file mutates during build and install fails before publish;
4. nonignored dirty source root still rejects without `--allow-dirty-source`;
5. source-archive release metadata behavior remains fail-closed;
6. source-archive digest policy is not replaced by source-root ignore policy;
7. `ashgrove update --from source` uses the same payload policy as source install.

### Step 5: Patch downstream verification

Replace every TASK-989/TASK-990 placeholder command with exact non-zero commands and add verification that no `false # TASK-988` placeholders remain.

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
  - python3 -c "from pathlib import Path; audit=Path('docs/plan/audits/TASK-988-ashgrove-source-payload-audit-gate.md'); assert audit.exists(), audit; text=audit.read_text(); required=['source_tree_digest','copy_source_tree_for_build','source_digest_skip_path','Source-shaped archive','git ls-files --cached --others --exclude-standard -z','task_989_update_from_source_uses_same_payload_policy_as_install','source_payload_digest_policy']; missing=[s for s in required if s not in text]; assert not missing, missing; task989=Path('docs/plan/tasks/TASK-989-ashgrove-source-payload-ignore-implementation.md').read_text(); task990=Path('docs/plan/tasks/TASK-990-ashgrove-source-payload-local-state-closeout.md').read_text(); assert 'false # TASK-988' not in task989 and 'false # TASK-988' not in task990; assert 'task_989_update_from_source_uses_same_payload_policy_as_install' in task989; print('TASK-988 audit artifact and downstream verification verified')"
checklist:
  - [x] Audit artifact created.
  - [x] Source-root and source-archive policies separated, including source-shaped archives.
  - [x] TASK-989 focused tests named, including update parity and source-archive digest-policy noninterference.
  - [x] TASK-989/TASK-990 verification commands patched with no placeholder `false` commands.
```

## Dependencies for Next Task

TASK-989 may start now that this audit artifact exists, source classification is frozen, focused test names are concrete, and downstream verification placeholders are removed.

## Notes

This is a hard gate because source-payload membership affects reproducibility and install identity.
