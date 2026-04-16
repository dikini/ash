# TASK-592: Spec Cross-Reference Validator

## Status: 📝 Planned

## Description

Validate internal Markdown links in `docs/spec/SPEC-*.md` and detect broken cross-references to other specs or plans.

## Specification Reference

- PLAN-090-SPEC-PROCESSOR.md — Track A

## Dependencies

- TASK-590 (file collector)

## Requirements

1. Extract `[text](target.md)` and `[text](target.md#anchor)` links.
2. Verify that `target.md` exists in `docs/spec/` or `docs/plan/`.
3. Emit `SpecDrift` findings for broken links.
4. (MVP) Do not verify anchors; only file existence.

## TDD Steps

### Step 1: Write failing test

Mock spec file with a broken link. Assert finding emitted.

### Step 2: Implement

Create `apps/spec_processor/src/spec_links.ash` with `check_files(paths: List<String>) -> List<SpecFinding>`.

### Step 3: Verify

Run against all `docs/spec/*.md`. Count broken links.

## Verification Steps

- [ ] Mock test passes
- [ ] Real repo run completes without panic
- [ ] Codex verification: VERIFIED
