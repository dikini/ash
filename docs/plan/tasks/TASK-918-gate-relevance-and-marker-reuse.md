# TASK-918: Gate Relevance and Marker Reuse

## Status: ✅ Complete

## Description

Optimize local commit/push verification gates so documentation-only changes do not run Rust/fuzz suites, and pre-push reuses a fresh successful pre-commit marker instead of repeating identical checks. The change must preserve conservative failure behavior for Rust/source/gate changes and keep gate skip decisions explicit in output.

## Specification Reference

- `scripts/check-pre-commit-gate.sh`
- `scripts/check-full-gate.sh`
- `scripts/gate-helpers.sh`
- Project hook policy in `.githooks/pre-commit` and `.githooks/pre-push`

## Dependencies

- ✅ Existing hook scripts and gate marker helpers
- ✅ CHANGELOG.md policy

## Requirements

### Functional Requirements

1. Add a gate classifier that distinguishes docs-only, Rust-relevant, fuzz-relevant, and gate-script-relevant change sets.
2. Docs-only pre-commit must run docs/changelog/link hygiene and skip Rust check/clippy/tests/fuzz/doctests with explicit output.
3. Rust/source/gate changes must continue to run the existing pre-commit Rust gate.
4. Full/pre-push gate must reuse a fresh pre-commit marker when `HEAD` and content hash match.
5. Full/pre-push gate must use docs-only gate for docs-only changes and skip all-target Rust tests/full fuzz with explicit output.
6. Unknown or gate-script changes must fail conservative by running the Rust/full gate path.
7. Add shell self-tests for classifier and marker-reuse/docs-only gate behavior.

### Property Requirements

No property tests required; this is shell tooling. Behavioral shell tests must cover positive and negative classifier/gate decisions.

## TDD Steps

### Step 1: Write Tests (Red)

**Files:**
- `scripts/check-gate-classifier-tests.sh`
- `scripts/check-gate-marker-tests.sh`

**Current State:**
- No classifier script exists.
- No marker match helper exists.
- Full gate always reruns pre-commit and full Rust/fuzz suites.

**Target State:**
- Classifier tests prove docs-only, Rust, fuzz, and gate-script classifications.
- Marker tests prove matching marker reuse and stale marker rejection.
- Tests fail before implementation.

### Step 2: Implement (Green)

**Files:**
- `scripts/gate-helpers.sh`
- `scripts/check-gate-classifier.sh`
- `scripts/check-docs-gate.sh`
- `scripts/check-pre-commit-gate.sh`
- `scripts/check-full-gate.sh`
- `scripts/check-gate-classifier-tests.sh`
- `scripts/check-gate-marker-tests.sh`

Implementation guidelines:
- Keep shell scripts `set -euo pipefail` safe.
- Print gate decisions explicitly.
- Fail conservative for unknown/gate-sensitive paths.
- Do not silently bypass checks.

### Step 3: Integration (Green)

Wire classifier self-tests into pre-commit so hook changes are tested before gate use.

### Step 4: Verify

Run focused shell verification and representative gate paths.

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
  - bash -n scripts/gate-helpers.sh scripts/check-gate-classifier.sh scripts/check-docs-gate.sh scripts/check-pre-commit-gate.sh scripts/check-full-gate.sh scripts/check-gate-classifier-tests.sh scripts/check-gate-marker-tests.sh
  - scripts/check-gate-classifier-tests.sh
  - scripts/check-gate-marker-tests.sh
  - scripts/check-pre-commit-gate.sh --no-marker
  - scripts/check-full-gate.sh --no-marker
  - git diff --check
checklist:
  - [x] Shell syntax clean
  - [x] Classifier tests pass
  - [x] Marker tests pass
  - [x] Pre-commit gate passes
  - [x] Full gate passes
  - [x] Changelog updated
```

## Dependencies for Next Task

This task outputs:
- Relevance-aware local gates.
- Fresh pre-commit marker reuse in full/pre-push gate.
- Test coverage for classifier and marker behavior.

Required by:
- Future CI/local-gate specialization work.

## Notes

- Preserve zero-tolerance baseline for source changes.
- Docs-only optimization is intentionally narrow: Markdown/docs/changelog-only changes should not trigger Rust/fuzz suites.
- Gate-script changes must be conservative because they can affect verification itself.
