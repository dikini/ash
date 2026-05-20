# TASK-927: RuntimeKernel Host-Mode Audit and Carriers

## Status: ✅ Complete

## Description

Audit current CLI/runtime/provider/engine seams and introduce core host-mode, root, definition, artifact, workflow-instance, and process-tree identity carriers for SPEC-070.

## Specification Reference

- [SPEC-069](../../spec/SPEC-069-ALPHA-VISIBLE-TOWER-ALGEBRA-AND-DO-LOWERING.md)
- [SPEC-070](../../spec/SPEC-070-ALPHA-RUNTIME-KERNEL-AND-OS-SURFACE.md)
- [PLAN-118](../PLAN-118-DESIGN-040-041-ALPHA-IMPLEMENTATION-PACKET.md)

## Dependencies

- TASK-920 completion
- TASK-925/TASK-926 completion for final TCIR/AMIR/bytecode artifact-version verification; before then, TASK-927 may define interim source/check-summary identity carriers only if the deferral is explicit in the task evidence.

## Requirements

### Functional Requirements

1. Use TASK-920 exact file/callsite/test bindings before implementation.
2. Add RED tests or evidence first.
3. Implement only the SPEC-069/SPEC-070 slice assigned to this task.
4. Define whether existing `Engine` becomes the `RuntimeKernel`, wraps a new kernel, or is embedded under one; entry, ordinary run, trace, and daemon host modes must share the same semantic lifecycle.
5. Define concrete root, definition, artifact, workflow-instance, process-tree, cache-key, and profile/config identity carriers rather than name-only placeholders.
6. Inventory existing runtime/admission/process/resource/capability carriers in `ash-core::runtime`, `ash-engine::WorkflowAdmission*`, and `ash-interp` context/runtime state; reuse or explicitly supersede them before adding new `RuntimeKernel` identity types, and document any aliasing or migration path.
7. Patch affected specs/plans/status surfaces if behavior or authority changes.
8. Run focused and broad verification specified by TASK-920.

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
    p = Path("crates/ash-core/tests/alpha_runtime_kernel_carriers.rs")
    text = p.read_text()
    names = [
        "runtime_kernel_ids_cover_root_definition_artifact_instance_and_host_mode",
        "runtime_kernel_host_modes_share_definition_and_artifact_identity",
    ]
    missing = [name for name in names if f"fn {name}" not in text]
    assert not missing, missing
    print("TASK-927 focused test file and names exist")
    PY
  - cargo test -p ash-core --test alpha_runtime_kernel_carriers -- --nocapture
  - RUSTC_WRAPPER= cargo clippy -p ash-core --test alpha_runtime_kernel_carriers -- -D warnings
  - cargo fmt --check
  - git diff --check
checklist:
  - [x] Focused evidence command patched by TASK-920
  - [x] Focused tests pass
  - [x] Focused clippy passes
  - [x] Formatting and diff whitespace checks pass
  - [x] Docs/status/changelog updated if public behavior changed
```

## Completion Notes

- Added `ash-core::runtime_kernel` with explicit host-mode, runtime-root, profile/config, artifact cache-key, workflow-definition, workflow-artifact, workflow-instance, process-tree, provider-registry, admission-grant, and kernel identity carriers.
- Defined the TASK-927 relationship as `RuntimeEngineRelationship::ExistingAshEngineEmbedded`: the future `RuntimeKernel` owns host/root/admission identity while the current `ash-engine::Engine` remains embedded below it for checking/execution until TASK-928 routes `ash run`.
- Added `RuntimeKernelCarrierInventory::task_927()` and module-level audit notes documenting reuse of existing `ash-core::runtime` IDs, `ash-engine::WorkflowAdmission*`, and `ash-interp::RuntimeState`/`Context` carriers.
- Preserved the authority boundary: provider registry identity is host inventory only; explicit `AdmissionIdentity` grants carry admission authority.
- Deferred CLI routing, daemon/control-plane behavior, reload semantics, and actual start/reload execution to TASK-928/TASK-929.

## Evidence

- RED: `cargo test -p ash-core --test alpha_runtime_kernel_carriers -- --nocapture` failed before implementation with unresolved import `ash_core::runtime_kernel`.
- `python3` focused test-name assertion: passed; printed `TASK-927 focused test file and names exist`.
- `cargo test -p ash-core --test alpha_runtime_kernel_carriers -- --nocapture`: passed, 2 passed / 0 failed.
- `RUSTC_WRAPPER= cargo clippy -p ash-core --test alpha_runtime_kernel_carriers -- -D warnings`: passed.
- `cargo fmt --check`: passed after formatting.
- `git diff --check`: passed.

## Dependencies for Next Task

This task outputs:
- RuntimeKernel audit and core identity carriers.

## Notes

- File targets to inspect or modify: `crates/ash-cli/src/main.rs`, `crates/ash-engine/src/`, `crates/ash-interp/src/`, `crates/ash-core/src/runtime.rs`.
- Keep PLAN-118 decision gates in sync with implementation reality.
- Do not broaden scope beyond SPEC-069/SPEC-070 without a docs patch and review.
