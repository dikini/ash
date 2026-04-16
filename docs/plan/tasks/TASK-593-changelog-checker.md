# TASK-593: Changelog Completeness Checker

## Status: 📝 Planned

## Description

Compare `PLAN-INDEX.md` tasks marked “✅ Complete” against `CHANGELOG.md` entries and flag missing changelog coverage.

## Specification Reference

- PLAN-090-SPEC-PROCESSOR.md — Track A
- AGENTS.md (changelog policy)

## Dependencies

- TASK-591 (plan-index parser)

## Requirements

1. Extract completed task IDs from `PLAN-INDEX.md`.
2. Verify each appears at least once in `CHANGELOG.md`.
3. Emit `ChangelogMissing` findings for gaps.

## TDD Steps

### Step 1: Write failing test

Mock index with TASK-001 Complete, mock changelog without TASK-001. Assert finding.

### Step 2: Implement

Create `apps/spec_processor/src/changelog.ash` with `check(index_text: String, changelog_text: String) -> List<SpecFinding>`.

### Step 3: Verify

Run against real repo. Result should match known gaps with zero false positives.

## Verification Steps

- [ ] Tests pass
- [ ] Real repo run accurate
- [ ] Codex verification: VERIFIED
