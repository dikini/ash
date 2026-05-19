# TASK-930: OODA Library Demotion Compatibility

## Status: 📝 Planned

## Description

Move alpha planning and implementation away from primitive OODA IR roots by documenting compatibility, adding library/template/lint surfaces, and preserving historical examples without making OODA privileged bytecode semantics.

## Specification Reference

- [SPEC-069](../../spec/SPEC-069-ALPHA-VISIBLE-TOWER-ALGEBRA-AND-DO-LOWERING.md)
- [SPEC-070](../../spec/SPEC-070-ALPHA-RUNTIME-KERNEL-AND-OS-SURFACE.md)
- [PLAN-118](../PLAN-118-DESIGN-040-041-ALPHA-IMPLEMENTATION-PACKET.md)

## Dependencies

- TASK-920 completion

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
    p = Path("crates/ash-cli/tests/alpha_ooda_library_demotion.rs")
    text = p.read_text()
    names = [
        "ooda_examples_are_library_or_template_calls_not_primitive_ir",
        "ooda_lint_points_to_visible_tower_algebra",
    ]
    missing = [name for name in names if f"fn {name}" not in text]
    assert not missing, missing
    print("TASK-930 focused test file and names exist")
    PY
  - cargo test -p ash-cli --test alpha_ooda_library_demotion -- --nocapture
  - git diff --check
checklist:
  - [ ] Focused evidence command patched by TASK-920
  - [ ] Focused tests pass
  - [ ] Broad relevant gate passes
  - [ ] Docs/status/changelog updated if public behavior changed
```

## Dependencies for Next Task

This task outputs:
- OODA compatibility migration notes.

## Notes

- File targets to inspect or modify: `docs/spec/SPEC-001-IR.md`, `docs/spec/SPEC-003-TYPE-SYSTEM.md`, `docs/spec/SPEC-004-SEMANTICS.md`, `docs/spec/SPEC-025-SMALL-STEP-OPERATIONAL-SEMANTICS.md`, `docs/spec/SPEC-041-ASH-LINT-LIBRARY.md`, `std/`.
- Keep PLAN-118 decision gates in sync with implementation reality.
- Do not broaden scope beyond SPEC-069/SPEC-070 without a docs patch and review.
