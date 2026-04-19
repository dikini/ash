# TASK-630: Positive end-to-end regex test

## Status: ✅ Complete

## Description
Prove the builtin regex path works end-to-end: module import, typechecking, evaluator dispatch, and runtime result. The positive coverage now lives in `crates/ash-engine/tests/builtin_fn_e2e_import.rs`, while the historical `regex_import_limitation.rs` target has been repurposed into honest positive/complementary regression coverage so existing verification commands remain stable.

## Specification Reference
- SPEC-BUILTIN-FN: Sections 5, 6, 7, 8
- PLAN-BUILTIN-FN: Track E / TASK-630

## Dependencies
- ✅ TASK-627: Regex builtin declarations
- ✅ TASK-628: Regex evaluator builtin dispatch

## Requirements
1. Ensure the repository contains a positive regex e2e test proving the builtin import path.
2. Create a temp Ash file using `use regex::{find}` or equivalent regex builtin import.
3. Verify module-load/import resolution succeeds.
4. Verify typechecking succeeds.
5. Verify runtime execution returns the expected regex result.
6. Update any stale docs/task references that still describe regex import as a known limitation.

## TDD Steps
### Step 1: Replace failing/limitation framing
- Keep or rewrite `regex_import_limitation.rs` only if its contents are honest.
- Ensure the canonical positive e2e coverage exists in `builtin_fn_e2e_import.rs` (or equivalent).

### Step 2: Verify builtin path
- Execute the regex call through the real engine path.
- Assert correct success result.

### Step 3: Verify documentation updates
- Remove stale limitation framing from in-scope docs/task surfaces.

## Verification Steps
- [x] positive regex e2e test exists
- [x] old limitation-only regex import test no longer represents current truth
- [x] engine parse/check/execute path succeeds for regex builtin import
- [x] related docs/task surfaces reflect the new positive state

## Notes
This is the D4 decision gate for the regex migration.
