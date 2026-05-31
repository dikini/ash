# TASK-988: Ashgrove source payload audit gate

## Status: 🟡 Ready

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
6. Patch TASK-989 and TASK-990 verification blocks if this audit chooses different exact focused commands.

### Property Requirements

No proptest is required. The audit invariant is single ownership: every source payload file-selection consumer must be mapped to either source-root payload policy or source-archive payload policy.

## TDD Steps

### Step 1: Trace source-root call graph

Inspect `crates/ashgrove/src/lib.rs` and record every function that reads, copies, digests, or stages source-root files.

### Step 2: Trace source-archive call graph

Record every source-archive digest/metadata path and identify tests that must remain green after TASK-989.

### Step 3: Choose implementation strategy

Document the selected membership strategy and its failure behavior. If git CLI membership is selected, specify exact commands and null-delimited parsing rules.

### Step 4: Freeze focused tests

Name the exact test file and test functions TASK-989 must add. Required scenarios:

1. ignored `.agents/status/dashboard.json` mutates during source install and install succeeds;
2. nested `target/` is excluded from digest and isolated build copy;
3. nonignored source file mutates during build and install fails before publish;
4. nonignored dirty source root still rejects without `--allow-dirty-source`;
5. source-archive release metadata behavior remains fail-closed.

### Step 5: Patch downstream verification if needed

Replace any remaining TASK-989/TASK-990 placeholder commands with exact non-zero commands.

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
  - python3 -c "from pathlib import Path; audit=Path('docs/plan/audits/TASK-988-ashgrove-source-payload-audit-gate.md'); assert audit.exists(), audit; text=audit.read_text(); required=['source_tree_digest','copy_source_tree_for_build','source_digest_skip_path','source archive','TASK-989']; missing=[s for s in required if s not in text]; assert not missing, missing; print('TASK-988 audit artifact verified')"
checklist:
  - [ ] Audit artifact created.
  - [ ] Source-root and source-archive policies separated.
  - [ ] TASK-989 focused tests named.
  - [ ] TASK-989/TASK-990 verification commands patched if needed.
```

## Dependencies for Next Task

TASK-989 may start only after this audit artifact exists and its focused test names are concrete.

## Notes

This is a hard gate because source-payload membership affects reproducibility and install identity.
