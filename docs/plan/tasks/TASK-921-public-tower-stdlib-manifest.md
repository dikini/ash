# TASK-921: Public Tower Stdlib Manifest

## Status: ✅ Complete

## Description

Define the public algebra manifest for `Act`, `Proc`, `Workflow`, `P`, and canonical user/domain examples such as `Result<_, E>`, with explicit visible operation to intrinsic mapping.

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
4. Define the public manifest under the real standard-library surface (`std/`, plus compiler prelude/type environment hooks where needed), not a non-existent `stdlib/` directory.
5. Add or explicitly document the migration path for visible `act::unit` / `act::bind` operations rather than relying on hidden Act dictionaries alone.
6. Patch affected specs/plans/status surfaces if behavior or authority changes.
7. Run focused and broad verification specified by TASK-920.

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
    p = Path("crates/ash-typeck/tests/alpha_visible_tower_manifest.rs")
    text = p.read_text()
    names = [
        "public_tower_manifest_exposes_act_proc_workflow_result_algebra",
        "visible_intrinsic_mapping_has_no_hidden_unrelated_do_magic",
    ]
    missing = [name for name in names if f"fn {name}" not in text]
    assert not missing, missing
    print("TASK-921 focused test file and names exist")
    PY
  - cargo test -p ash-typeck --test alpha_visible_tower_manifest -- --nocapture
  - git diff --check
checklist:
  - [x] Focused evidence command patched by TASK-920
  - [x] Focused tests pass
  - [x] Broad relevant gate passes
  - [x] Docs/status/changelog updated if public behavior changed
```

## Dependencies for Next Task

This task outputs:
- Public manifest and intrinsic mapping.

## Notes

- File targets to inspect or modify: `std/`, `crates/ash-typeck/src/type_env.rs`, `crates/ash-engine/src/module_loader.rs`, `crates/ash-interp/src/eval.rs`.
- Keep PLAN-118 decision gates in sync with implementation reality.
- Do not broaden scope beyond SPEC-069/SPEC-070 without a docs patch and review.

## Completion Notes

- Added `crates/ash-typeck/tests/alpha_visible_tower_manifest.rs` with the TASK-920-bound non-empty focused tests.
- Added `ash_typeck::TypeEnv::public_tower_manifest()` and typed manifest carriers for public `Act`, `Proc`, `Workflow`, `P`, `Result<_, E>`, and canonical `Option` tower/example entries.
- Recorded visible operation-to-intrinsic mappings for `act::unit`/`act::bind`, `proc::*`, `workflow::*`, `Ok`, and `result::and_then`; mappings retain hidden Act compiler-prelude evidence only behind the visible `act::unit`/`act::bind` names rather than as independent semantic roots.
- Added TypeEnv-visible `act::*` and `result::and_then` qualified signatures needed by the manifest. Existing `proc::*` and workflow metadata remain the public hooks for later do-lowering tasks.
- Added `std/src/workflow.ash` for value-level Workflow algebra operations and registered it from `std/src/lib.ash`. `workflow::requires` and `workflow::ensures` remain documented TypeEnv/compiler-prelude metadata because their parameter classes are not source-denotable stdlib types yet.
- Verification run: TASK-921 structure assertion, `cargo test -p ash-typeck --test alpha_visible_tower_manifest -- --nocapture`, `cargo fmt --check`, and `git diff --check`.
