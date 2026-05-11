# TASK-856: Phase 114 review remediation

## Status: ✅ Complete

## Description

Run independent post-closeout review and remediate findings without broadening SPEC-062 scope.

## Specification Reference

- [SPEC-062: Module-Summary Export/Import for Type Computation](../../spec/SPEC-062-MODULE-SUMMARY-EXPORT-IMPORT-FOR-TYPE-COMPUTATION.md)
- [PLAN-110: Module-Summary Export/Import for Type Computation](../PLAN-110-MODULE-SUMMARY-EXPORT-IMPORT-FOR-TYPE-COMPUTATION.md)
- [DESIGN-034 §16.6](../../design/DESIGN-034-TOTAL-TYPE-COMPUTATION.md#166-spec-f-module-summary-exportimport-for-type-computation)

## Dependencies

- Depends on TASK-855 completion

## Requirements

### Functional Requirements

1. Dispatch independent semantic/code and docs/status review subagents.
2. Fix blocker/important findings only; record non-blocking follow-ups in notes if needed.
3. Rerun focused and broad verification after any code change.
4. Commit a final remediation slice and leave branch clean.

### Non-Goals

- Do not implement associated recursive type-family computation (SPEC-G).
- Do not add proposition solving, type-function inversion, or proof search (SPEC-H and beyond).
- Do not move type-computation semantic ownership into parser or engine-private carriers.

## TDD / Execution Steps

### Step 1: RED / Inspect

- Re-read the SPEC-062 section owned by this task.
- Inspect exact live files named by PLAN-110 and TASK-856 before patching.
- For implementation tasks, write focused failing tests before code changes.

### Step 2: GREEN / Implement

- Apply the smallest scoped patch for TASK-856 only.
- Preserve SPEC-057/059/060/061 behavior unless this task explicitly changes it.
- Keep public/private summary closure and negative leakage assertions in scope.

### Step 3: Verify

Run:

```bash
git diff --check
cargo fmt --check
cargo check --workspace
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace
cargo doc --workspace --no-deps
```

### Step 4: Independent Verification

Dispatch a review/verification subagent with this task file, SPEC-062, and changed files. Do not mark TASK-856 complete until the subagent reports no blocking findings and the commands above pass.

## Completion Checklist

- [x] Requirements above are satisfied.
- [x] Focused tests exist and pass, or docs-only verification is recorded.
- [x] Negative leakage/private-opacity behavior is tested where applicable.
- [x] Status docs and CHANGELOG.md are updated if this task changes behavior or status.
- [x] Independent verification completed.

## Completion Evidence

- Remediated independent review findings by separating source-visible imported type-function heads from semantic-summary dependency helper payloads.
- Added import-side canonical signature validation for imported type-function parameter/return types before normalizer registration, including unknown nominal identity and arity regressions.
- Hid ordinary type dependency source names from selected type-function imports while preserving dependency metadata under internal names, and rewrote transported summary references to those internal names.
- Refreshed summary dedup/cache integrity by updating merged-summary keys after mutation, preserving same-head type-function aliases as distinct selected summary exports, and including all current semantic-summary surfaces in `semantic_cache_key`.
- Added/updated focused engine and typeck coverage for selected aliases, same-head aliased re-exports, re-export chains, deterministic glob visible-head ordering, repeated selected imports, ordinary dependency hiding, malformed imported signatures, and negative helper-name leakage.
- Independent final review reported no blocking code findings after remediation; status/changelog/comment updates were reconciled here.
- Verification run after remediation:
  - `cargo test -p ash-engine --test task_849_type_computation_summary_transport -- --nocapture` — 10 passed.
  - `cargo test -p ash-engine --test task_850_summary_dedup_cache -- --nocapture` — 2 passed.
  - `cargo test -p ash-engine --test task_853_type_computation_import_order -- --nocapture` — 5 passed.
  - `cargo test -p ash-engine --test task_854_type_computation_summary_acceptance -- --nocapture` — 3 passed.
  - `cargo test -p ash-typeck --test task_851_imported_type_function_normalizer -- --nocapture` — 12 passed.
  - `cargo test -p ash-core --test task_850_summary_versioning_cache -- --nocapture` — 5 passed.
  - `cargo test --workspace` — passed.
  - `git diff --check && cargo fmt --check && cargo check --workspace && cargo clippy --workspace --all-targets --all-features -- -D warnings && cargo doc --workspace --no-deps` — passed.

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
  - git diff --check
  - cargo fmt --check
  - cargo check --workspace
  - cargo clippy --workspace --all-targets --all-features -- -D warnings
  - cargo test --workspace
  - cargo doc --workspace --no-deps
checklist:
  - [ ] Implementation matches SPEC-062 and PLAN-110 scope
  - [ ] Focused tests for this task pass
  - [ ] Formatting and diff checks pass
  - [ ] CHANGELOG.md updated if task changes code/docs policy/status
```

## Dependencies for Next Task

This task outputs:
- Final phase hardening before merge readiness.
