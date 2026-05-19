# TASK-927: RuntimeKernel Host-Mode Audit and Carriers

## Status: 📝 Planned

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
  - false # TASK-920 must replace this with exact focused non-zero evidence before implementation starts
  - git diff --check
checklist:
  - [ ] Focused evidence command patched by TASK-920
  - [ ] Focused tests pass
  - [ ] Broad relevant gate passes
  - [ ] Docs/status/changelog updated if public behavior changed
```

## Dependencies for Next Task

This task outputs:
- RuntimeKernel audit and core identity carriers.

## Notes

- File targets to inspect or modify: `crates/ash-cli/src/main.rs`, `crates/ash-engine/src/`, `crates/ash-interp/src/`, `crates/ash-core/src/runtime.rs`.
- Keep PLAN-118 decision gates in sync with implementation reality.
- Do not broaden scope beyond SPEC-069/SPEC-070 without a docs patch and review.
