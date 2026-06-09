# TASK-1378: Add module-size audit and split policy

## Status: ✅ Complete

## Description

Create a repeatable Rust file-size audit and freeze the module-size policy used by the rest of Phase 137. This task should not move production code; it establishes measurement, budgets, and baseline evidence.

## Specification Reference

- [PLAN-137: Rust Module Size and Discoverability Refactor](../PLAN-137-RUST-MODULE-SIZE-REFACTOR.md)
- Rust split rules from `rust-skills`: `proj-mod-by-feature`, `proj-pub-crate-internal`, `proj-pub-use-reexport`, `own-borrow-over-clone`, `anti-over-abstraction`, `lint-rustfmt-check`.

## Dependencies

- ✅ Phase 136 complete on `main` at `975ccea8`.

## Deferral / Planned-Feature Reconciliation

None. This is a behavior-preserving refactor task; any discovered semantic redesign belongs in a separate future phase.

## Requirements

### Functional Requirements

1. Create `tools/dev/rust_file_size_report.py` that scans workspace `.rs` files by Cargo package using `cargo metadata`.
2. Exclude `.git/`, `target/`, and `.worktrees/`.
3. Report per crate: total `.rs` files, count above 500 lines, count above 10KB, largest file by lines, largest file by bytes.
4. Support machine-readable JSON and Markdown output.
5. Add `docs/audit/RUST-FILE-SIZE-AUDIT.md` with the Phase 137 baseline from `975ccea8`.
6. Add a policy section to `PLAN-137` describing preferred file/module budgets and exception rules.
7. Update `CHANGELOG.md` with the planning/audit packet entry.

### Size / Discoverability Requirements

1. Prefer feature-owned modules below 500 lines.
2. Preserve stable public API paths with compatibility reexports where existing callsites require them.
3. Keep helper modules `pub(crate)` / `pub(super)` unless a public API already existed.
4. Do not introduce new production `unwrap` / `expect` paths during extraction.
5. Keep module names discoverable from the feature name an agent would search for.

## TDD / Refactor Steps

### Step 1: Write the audit script first

**File:** `tools/dev/rust_file_size_report.py`

The script must be deterministic and must use Cargo metadata for crate ownership rather than guessing from path prefixes.

### Step 2: Run the script against current `main`

```bash
python3 tools/dev/rust_file_size_report.py --markdown > /tmp/rust-size.md
python3 tools/dev/rust_file_size_report.py --json > /tmp/rust-size.json
```

Expected: non-empty output listing all workspace crates.

### Step 3: Write the audit document

**File:** `docs/audit/RUST-FILE-SIZE-AUDIT.md`

Include the baseline table and the top 20 files by line count.

### Step 4: Verify policy wording

Confirm the plan records both soft and hard budgets and does not overclaim that all files can be split in one task.

### Step 5: Codex verification

Ask Codex to review script determinism, package attribution, ignored directories, threshold logic, and whether the policy supports later implementation tasks.


## Dispatch

```
agent: hermes
reasoning: medium
max_turns: 20
toolsets: [terminal, file]
```

## Verification

```
strictness: clean
commands:
  - python3 tools/dev/rust_file_size_report.py --markdown > /tmp/phase137-size-audit.md
  - python3 tools/dev/rust_file_size_report.py --json > /tmp/phase137-size-audit.json
  - python3 tools/dev/rust_file_size_report.py --tests-only > /tmp/phase137-tests-only.md
  - python3 tools/dev/rust_file_size_report.py --fail-on-regression
  - CARGO_BUILD_RUSTC_WRAPPER= RUSTC_WRAPPER= cargo fmt --check
  - git diff --check
checklist:
  - [x] Audit script produces Markdown and JSON output
  - [x] Audit script supports tests-only and Phase 137 baseline regression guard output
  - [x] Baseline audit document records current counts and top outliers
  - [x] PLAN-137 budget/policy is consistent with task files
  - [x] CHANGELOG.md records the planning/audit packet
  - [x] Formatting and diff checks pass
  - [x] Codex final review reports no blocking issues
```

## Implementation Evidence

- Added `tools/dev/rust_file_size_report.py` using `cargo metadata --format-version 1 --no-deps` for workspace package attribution, with Markdown/JSON output, tests-only filtering, and a Phase 137 regression guard that checks oversized-file counts plus largest line/byte outliers without failing merely because module splitting adds files.
- Added `docs/audit/RUST-FILE-SIZE-AUDIT.md` with the Phase 137 baseline: 18 workspace crates, 663 Rust files, 165 files over 500 lines, and 284 files over 10KB.
- Confirmed the largest baseline outlier is `crates/ash-typeck/src/type_env.rs` at 20,935 lines / 807.1KB.

### Verification run

```bash
python3 tools/dev/rust_file_size_report.py --markdown > /tmp/phase137-size-audit.md
python3 tools/dev/rust_file_size_report.py --json > /tmp/phase137-size-audit.json
python3 tools/dev/rust_file_size_report.py --tests-only > /tmp/phase137-tests-only.md
python3 tools/dev/rust_file_size_report.py --fail-on-regression
CARGO_BUILD_RUSTC_WRAPPER= RUSTC_WRAPPER= cargo fmt --check
git diff --check
```

Result: all six commands exited 0; the regression guard reported no Phase 137 baseline regressions and the remaining commands produced no errors.

### Codex review

Codex initially found two guard-quality blockers: raw Rust file count would fail valid module-splitting work, and byte regression was checked against the largest-by-lines file rather than the true largest-by-bytes file. Both were fixed and re-reviewed. Final Codex review reported no blocking issues: "The audit script, baseline document, plan updates, and changelog entry are consistent with the stated TASK-1378 requirements."

## Dependencies for Next Task

This task outputs:
- `tools/dev/rust_file_size_report.py`
- `docs/audit/RUST-FILE-SIZE-AUDIT.md`
- Frozen Phase 137 budget/policy for downstream tasks.

Required by:
- TASK-1387: Module-size closeout and final audit.

## Notes

- This task should be committed independently.
- If any split exposes a genuine semantic bug, stop and create a follow-on bug task rather than hiding behavior changes in the refactor.
- End with Codex review for code quality, semantic preservation, style, and size-budget compliance.
