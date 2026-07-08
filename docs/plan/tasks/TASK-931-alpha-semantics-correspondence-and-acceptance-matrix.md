# TASK-931: Alpha Semantics Correspondence and Acceptance Matrix

## Status: ✅ Complete

## Description

Create final SPEC-069/SPEC-070 acceptance and non-interference evidence mapping A69-1 through A69-12 and A70-1 through A70-8 to concrete tests, docs, and runtime traces.

## Specification Reference

- [SPEC-069](../../spec/SPEC-069-ALPHA-VISIBLE-TOWER-ALGEBRA-AND-DO-LOWERING.md)
- [SPEC-070](../../spec/SPEC-070-ALPHA-RUNTIME-KERNEL-AND-OS-SURFACE.md)
- [PLAN-118](../PLAN-118-DESIGN-040-041-ALPHA-IMPLEMENTATION-PACKET.md)

## Dependencies

- TASK-921 through TASK-930 completion

## Requirements

### Functional Requirements

1. Use TASK-920 exact file/callsite/test bindings before implementation.
2. Add RED tests or evidence first.
3. Implement only the SPEC-069/SPEC-070 slice assigned to this task.
4. Patch affected specs/plans/status surfaces if behavior or authority changes.
5. Run focused and broad verification specified by TASK-920.

### Property Requirements

Property tests are required for Rust semantic tasks when TASK-920 identifies a stable strategy. Documentation-only or audit tasks must instead provide corpus consistency evidence.

## TDD Steps

### Step 1: Write failing tests or evidence

Use TASK-920-selected files and exact commands; avoid zero-test filters.

### Step 2: Implement or document the slice

Make the smallest change satisfying this task without pulling later tasks forward.

### Step 3: Integrate at public seams

Wire through parser/typeck/engine/runtime/CLI surfaces named by TASK-920.

### Step 4: Verify and record evidence

Run focused commands, broad relevant gate, docs diff/link checks, and update evidence/status surfaces.

## Dispatch

```
agent: hermes
reasoning: high
max_turns: 20
toolsets: [terminal, file]
```

## Verification

```
strictness: clean
commands:
  - |
    python3 - <<'PY'
    from pathlib import Path
    audit = Path("docs/plan/audits/TASK-931-alpha-acceptance-matrix.md")
    assert audit.is_file(), audit
    p = Path("crates/ash-typeck/tests/alpha_visible_computation_acceptance_matrix.rs")
    text = p.read_text()
    names = [
        "spec069_acceptance_cases_are_mapped_to_focused_tests",
        "spec070_runtime_acceptance_cases_are_mapped_to_focused_tests",
        "alpha_non_interference_matrix_covers_legacy_surfaces",
    ]
    missing = [name for name in names if f"fn {name}" not in text]
    assert not missing, missing
    print("TASK-931 focused acceptance artifact, test file, and names exist")
    PY
  - cargo test -p ash-typeck --test alpha_visible_computation_acceptance_matrix -- --nocapture
  - git diff --check
checklist:
  - [x] Focused evidence command patched by TASK-920
  - [x] Focused tests pass
  - [x] Broad relevant gate passes
  - [x] Docs/status/changelog updated if public behavior changed
```

## Completion Evidence

- RED: `cargo test -p ash-typeck --test alpha_visible_computation_acceptance_matrix -- --nocapture` ran 3 tests and failed before matrix completion. Expected failures included missing `A69-1` and `A70-1` rows in the audit stub; one overly literal SPEC-070 backtick assertion was corrected before GREEN.
- `python3 - <<'PY' ... PY`: focused artifact/name assertion prints `TASK-931 focused acceptance artifact, test file, and names exist`.
- `cargo test -p ash-typeck --test alpha_visible_computation_acceptance_matrix -- --nocapture`: 3 focused tests pass, proving SPEC-069 A69-1 through A69-12, SPEC-070 A70-1 through A70-8, and the non-interference rows map to concrete paths, exact test names, task/docs references, and limitations in `docs/plan/audits/TASK-931-alpha-acceptance-matrix.md`.
- `git diff --check` passes.
- `RUSTC_WRAPPER= cargo fmt --check` passes.
- `RUSTC_WRAPPER= cargo check -p ash-typeck` passes.

## Implementation Notes

- Added `docs/plan/audits/TASK-931-alpha-acceptance-matrix.md` as the acceptance/non-interference matrix artifact.
- Added `crates/ash-typeck/tests/alpha_visible_computation_acceptance_matrix.rs` as a non-zero aggregator test suite that reads the audit/spec/task files and asserts concrete row IDs, file paths, test names, and required limitation strings.
- TASK-931 is evidence/audit only. It does not change SPEC-069/SPEC-070 semantics, RuntimeKernel behavior, authority/admission logic, or daemon execution.
- Do not mark TASK-932 complete; TASK-932 remains responsible for closeout review, broad gates, and remediation.

## Dependencies for Next Task

This task outputs:
- Acceptance/non-interference matrix artifact.

## Notes

- File targets to inspect or modify: `docs/plan/audits/TASK-931-alpha-acceptance-matrix.md`, `docs/spec/SPEC-069-ALPHA-VISIBLE-TOWER-ALGEBRA-AND-DO-LOWERING.md`, `docs/spec/SPEC-070-ALPHA-RUNTIME-KERNEL-AND-OS-SURFACE.md`.
- Keep PLAN-118 decision gates in sync with implementation reality.
- Do not broaden scope beyond SPEC-069/SPEC-070 without a docs patch and review.
