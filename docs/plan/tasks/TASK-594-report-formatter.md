# TASK-594: Report Formatter

## Status: 📝 Planned

## Description

Aggregate structured findings into human-readable and JSON output formats, with a non-zero exit code when blocked by Tier 2 findings.

## Specification Reference

- PLAN-090-SPEC-PROCESSOR.md — Track A
- DESIGN-SPEC-PROCESSOR.md §4

## Dependencies

- None (uses ad-hoc string construction; refactors to `std::json` after TASK-597)

## Requirements

1. Define `Report` record with `findings`, `blocked`, `tier_0_count`, `tier_1_count`, `tier_2_count`.
2. Implement `format_human(r: Report) -> String`.
3. Implement `format_json(r: Report) -> String` (ad-hoc for MVP).
4. `blocked` is true iff any Tier 2 finding exists.

## TDD Steps

### Step 1: Write failing test

Create `Report` with a Tier 2 finding. Assert JSON contains `"tier": 2` and `blocked == true`.

### Step 2: Implement

Create `apps/spec_processor/src/report.ash`.

### Step 3: Verify

Validate JSON syntax and human readability.

## Verification Steps

- [ ] Tests pass
- [ ] JSON output is syntactically valid
- [ ] Codex verification: VERIFIED
