# TASK-850: Summary versioning and cache invalidation

## Status: ✅ Complete

## Description

Make computation-summary versioning, dedup, and cache invalidation inputs explicit and tested.

## Specification Reference

- [SPEC-062: Module-Summary Export/Import for Type Computation](../../spec/SPEC-062-MODULE-SUMMARY-EXPORT-IMPORT-FOR-TYPE-COMPUTATION.md)
- [PLAN-110: Module-Summary Export/Import for Type Computation](../PLAN-110-MODULE-SUMMARY-EXPORT-IMPORT-FOR-TYPE-COMPUTATION.md)
- [DESIGN-034 §16.6](../../design/DESIGN-034-TOTAL-TYPE-COMPUTATION.md#166-spec-f-module-summary-exportimport-for-type-computation)

## Dependencies

- Depends on TASK-849 completion

## Requirements

### Functional Requirements

1. Harden core/engine version and content keying so V1/V2 summaries with non-empty computation facts are not accepted as computation-aware summaries.
2. Include summary version, type-function summaries/equations, dependency refs, and sealed-domain summaries in imported summary keys.
3. Document in-memory cache boundaries and future persistent cache digest inputs.
4. Add tests proving summaries with identical ordinary types but different computation facts do not dedup together.
5. Leave TypeEnv batch no-partial-registration behavior and structured unsupported-version diagnostics to TASK-851/TASK-852.

### Non-Goals

- Do not implement associated recursive type-family computation (SPEC-G).
- Do not add proposition solving, type-function inversion, or proof search (SPEC-H and beyond).
- Do not move type-computation semantic ownership into parser or engine-private carriers.

## TDD / Execution Steps

### Step 1: RED / Inspect

- Re-read the SPEC-062 section owned by this task.
- Inspect exact live files named by PLAN-110 and TASK-850 before patching.
- For implementation tasks, write focused failing tests before code changes.

### Step 2: GREEN / Implement

- Apply the smallest scoped patch for TASK-850 only.
- Preserve SPEC-057/059/060/061 behavior unless this task explicitly changes it.
- Keep public/private summary closure and negative leakage assertions in scope.

### Step 3: Verify

Run:

```bash
cargo test -p ash-core --test task_850_summary_versioning_cache -- --nocapture
cargo test -p ash-engine --test task_850_summary_dedup_cache -- --nocapture
cargo fmt --check
git diff --check
cargo check --workspace
```

### Step 4: Independent Verification

Dispatch a review/verification subagent with this task file, SPEC-062, and changed files. Do not mark TASK-850 complete until the subagent reports no blocking findings and the commands above pass.

## Completion Checklist

- [x] Requirements above are satisfied.
- [x] Focused tests exist and pass, or docs-only verification is recorded.
- [x] Negative leakage/private-opacity behavior is tested where applicable.
- [x] Status docs and CHANGELOG.md are updated if this task changes behavior or status.
- [x] Independent verification completed.

## Dispatch

```yaml
agent: hermes
reasoning: medium
max_turns: 15
toolsets: [terminal, file]
```

## Verification

```yaml
strictness: clean
commands:
  - cargo test -p ash-core --test task_850_summary_versioning_cache -- --nocapture
  - cargo test -p ash-engine --test task_850_summary_dedup_cache -- --nocapture
  - cargo fmt --check
  - git diff --check
  - cargo check --workspace
checklist:
  - [x] Implementation matches SPEC-062 and PLAN-110 scope
  - [x] Focused tests for this task pass
  - [x] Formatting and diff checks pass
  - [x] CHANGELOG.md updated if task changes code/docs policy/status
```

## Dependencies for Next Task

This task outputs:
- Hardens summary transport before broad import-order tests.
