# TASK-924: Act/Proc/Workflow Opaque Carrier Alignment

## Status: ✅ Complete

## Description

Align Act/Proc/Workflow runtime carriers with visible tower algebra while preserving opaque environments, explicit tower lifts, and workflow governance/failure boundaries.

## Specification Reference

- [SPEC-069](../../spec/SPEC-069-ALPHA-VISIBLE-TOWER-ALGEBRA-AND-DO-LOWERING.md)
- [SPEC-070](../../spec/SPEC-070-ALPHA-RUNTIME-KERNEL-AND-OS-SURFACE.md)
- [PLAN-118](../PLAN-118-DESIGN-040-041-ALPHA-IMPLEMENTATION-PACKET.md)

## Dependencies

- TASK-920 completion
- TASK-921 completion for the public tower manifest and intrinsic mapping before opaque-carrier alignment closeout.

## Requirements

### Functional Requirements

1. Use TASK-920 exact file/callsite/test bindings before implementation.
2. Add RED tests or evidence first.
3. Implement only the SPEC-069/SPEC-070 slice assigned to this task.
4. Resolve the `ActEnv` opacity boundary explicitly: either make it non-denotable/compiler-internal despite current public registration, or patch visibility/API so hidden runtime environment state cannot be ordinary user data.
5. Preserve explicit tower lifts; do not accept direct `Act` in `Proc` or `Proc` in `Workflow` as an implicit lift.
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
    p = Path("crates/ash-typeck/tests/alpha_tower_opaque_carriers.rs")
    text = p.read_text()
    names = [
        "proc_requires_explicit_from_act_lift",
        "workflow_requires_explicit_from_proc_or_from_act_lift",
        "act_env_and_process_identity_remain_non_denotable",
    ]
    missing = [name for name in names if f"fn {name}" not in text]
    assert not missing, missing
    print("TASK-924 focused test file and names exist")
    PY
  - cargo test -p ash-typeck --test alpha_tower_opaque_carriers -- --nocapture
  - git diff --check
checklist:
  - [x] Focused evidence command patched by TASK-920
  - [x] Focused tests pass
  - [x] Broad relevant gate passes
  - [x] Docs/status/changelog updated if public behavior changed
```

## Completion Notes

- Added `crates/ash-typeck/tests/alpha_tower_opaque_carriers.rs` with the TASK-920-required focused tests for explicit tower lifts and opaque carrier non-denotability.
- Removed `ActEnv` from `TypeEnv::with_builtin_types()` and from `std/src/act.ash` so hidden Act runtime environment state is not an ordinary Ash source type. Public `Act<T>`, `Proc<T>`, `Workflow<T>`, and opaque `P<T>` handle typing remain available.
- Preserved existing explicit lift behavior: direct `Act` in `do:Proc` and direct `Proc`/`Act` in `do:Workflow` fail with lift hints, while `proc::from_act`, `workflow::from_proc`, and `workflow::from_act` remain accepted.
- Hardened the existing workflow builtin signature test to compare fresh type-variable structure instead of brittle global `TypeVar` IDs.

## Verification Evidence

- `python3 - <<'PY' ...` focused test-name structure assertion from this task.
- `cargo test -p ash-typeck --test alpha_tower_opaque_carriers -- --nocapture`
- `cargo test -p ash-typeck --test task_749_typed_do --test task_772_workflow_do --test alpha_visible_tower_manifest --test alpha_generalized_do_full_bind_lowering --test alpha_monad_evidence_method_body_lowering --test task_719_proc_from_act_types --test task_771_workflow_type_stdlib_intrinsics -- --nocapture`
- `cargo test -p ash-typeck --lib task689d_act_env_type_expr_is_not_source_denotable -- --nocapture`
- `cargo test -p ash-typeck --lib test_type_env_with_builtin_types -- --nocapture`
- `target/debug/ash check std/src/act.ash`
- `cargo fmt --check`
- `git diff --check`
- `RUSTC_WRAPPER= cargo clippy -p ash-typeck --test alpha_tower_opaque_carriers -- -D warnings`

## Dependencies for Next Task

This task outputs:
- Aligned tower runtime/typing boundary.

## Notes

- File targets to inspect or modify: `crates/ash-core/src/runtime.rs`, `crates/ash-typeck/src/type_env.rs`, `crates/ash-interp/src/eval.rs`, `crates/ash-typeck/src/check_expr.rs`.
- Keep PLAN-118 decision gates in sync with implementation reality.
- Do not broaden scope beyond SPEC-069/SPEC-070 without a docs patch and review.
