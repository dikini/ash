# TASK-2043: Remove Tracked Rust Target Artifacts

**Status:** Complete
**Phase:** [PLAN-205](../PLAN-205-ENGINE-ONLY-EXECUTION-CUTOVER.md)
**Depends on:** None
**Classification:** Repository maintenance; not semantic work

## Description

Remove Cargo build output from the Git index without deleting local caches. The observed tracked
inventory is three files under `crates/ash-bench/target/` and 582 files under
`crates/ash-fuzz/target/`. Replace the path-specific ignore rule with one repository-wide
`target/` policy so every current and future nested Cargo build-output directory is ignored.

## Requirements

- Remove every tracked path beneath a Cargo `target/` directory from the Git index while retaining
  the local directories as ignored build caches.
- Add one global `target/` ignore rule that applies at the repository root and at arbitrary
  nesting depth; do not introduce per-crate exceptions.
- Add a focused regression guard that rejects any tracked `target/` path and proves both root and
  nested target examples are ignored.
- Run the focused guard, documentation gate, diff check, and normal staged pre-commit gate before
  completion.

## Non-goals

- Changing Cargo packages, dependency versions, Rust source behavior, or build configuration.
- Deleting local build caches or adding an artifact-cleanup runtime step.
- Changing semantic coverage, traceability, execution authority, admission, runtime behavior, or
  documentation policy beyond the no-tracked-artifacts repository guard.

## TDD and verification steps

1. Add the focused Git-index/ignore-policy test and run it RED against the tracked
   `crates/ash-bench/target/` and `crates/ash-fuzz/target/` inventory.
2. Register the test in the ordinary pre-commit gate.
3. Replace the path-specific ignore entry with the global `target/` rule and remove both tracked
   build-output trees from the index only.
4. Run the focused test GREEN, `bash scripts/check-docs-gate.sh`, `git diff --check`, and the
   ordinary staged pre-commit gate.

## Completion evidence

- **RED:** `python3 -m unittest tools.docs.tests.test_no_tracked_rust_target_directories` reported
  two failures: 585 tracked target artifacts in the Git index and the missing nested global-ignore
  behavior for `crates/example/target/.rustc_info.json`.
- **GREEN:** the focused guard now records two test passes; `.gitignore` has one global `target/` rule,
  no Git-index target path remains, and all 585 artifacts were removed from the index only.
  The local Cargo caches remain on disk as ignored directories.
- **Hook registration:** `scripts/check-pre-commit-gate.sh` runs the focused no-tracked-target
  guard in the ordinary pre-commit workflow.
- **Staged changelog compatibility:** the staged changelog check uses a direct path-limited Git
  query, so large staged target deletions cannot trigger the former pipefail/SIGPIPE false
  `CHANGELOG not updated` failure. Its deterministic 5,000-path regression passes.
- **Semantic-task-gate compatibility:** in a staged snapshot, the gate recognizes only exact,
  unregistered TASK-2043 repository-maintenance metadata when no semantic Rust is staged.
  Arbitrary tasks, registered records, and snapshots with co-staged semantic Rust remain selected
  and fail closed. This transports task metadata only: it adds no execution, admission, or
  conformance authority and creates no broad documentation bypass.

## Completion checklist

- [x] No Git-index path remains beneath a `target/` directory.
- [x] The repository-wide `target/` rule ignores both root and nested Cargo output paths.
- [x] The focused regression test is run by the normal pre-commit gate.
- [x] Local target directories remain on disk as ignored Cargo caches.
- [x] RED and GREEN evidence, inventory, and index-only removal are recorded.
- [x] `CHANGELOG.md` and `PLAN-INDEX.md` are current.
