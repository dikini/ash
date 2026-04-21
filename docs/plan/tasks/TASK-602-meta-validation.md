# TASK-602: Meta-Validation

## Status: ✅ Complete

## Description

The processor validates that its own source files and rules conform to the same standards it applies to the rest of the repository.

## Specification Reference

- PLAN-090-SPEC-PROCESSOR.md — Track C
- DESIGN-SPEC-PROCESSOR.md §2.2 (meta-stability)

## Dependencies

- TASK-590, TASK-592, TASK-601

## Requirements

1. Include `apps/spec_processor/` in the scanned file tree.
2. Ensure all processor `.ash` files parse successfully.
3. Validate `capability_boundary.ash` is well-formed.
4. Verify processor's own spec cross-references.

## TDD Steps

### Step 1: Write failing test

Introduce a deliberate broken link in processor docs. Assert processor flags it.

### Step 2: Implement

Modify `apps/spec_processor/src/main.ash` to include its own directory in `scan_tree`.

### Step 3: Verify

Broken link is detected as `SpecDrift`.

## Verification Steps

- [ ] Self-audit passes on clean state
- [ ] Deliberate defect is detected
- [ ] Codex verification: VERIFIED
