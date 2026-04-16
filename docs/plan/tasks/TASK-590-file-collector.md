# TASK-590: Spec Processor File Collector

## Status: 📝 Planned

## Description

Implement the repository file discovery stage for the spec processor: gather all spec, plan, example, and changelog files from the Ash repository tree.

## Specification Reference

- PLAN-090-SPEC-PROCESSOR.md — Track A
- DESIGN-SPEC-PROCESSOR.md §3, §6

## Dependencies

- `std::io::dir` and `std::io::path` must be available in the runtime.

## Requirements

1. Recursively traverse a given root directory.
2. Categorise files into: `spec_files`, `plan_files`, `example_files`, `changelog_files`.
3. Return a `FileTree` record.

## TDD Steps

### Step 1: Write failing test

Create `apps/spec_processor/tests/test_collect.ash` (or Rust harness) that asserts `scan_tree` returns non-empty lists for a mock fixture directory.

### Step 2: Implement

Create `apps/spec_processor/src/collect.ash` with:

```ash
pub record FileTree {
    spec_files: List<String>,
    plan_files: List<String>,
    example_files: List<String>,
    changelog_files: List<String>,
}

pub fn scan_tree(root: String) -> FileTree { ... }
```

### Step 3: Verify

Run tests. Expected: PASS.

## Verification Steps

- [ ] Tests pass
- [ ] `cargo fmt --check` clean
- [ ] Codex verification: VERIFIED
