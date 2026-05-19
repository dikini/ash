# TASK-928: ash run RuntimeKernel Mode

## Status: 📝 Planned

## Description

Route one-shot `ash run` through the RuntimeKernel abstraction while preserving current CLI behavior, deterministic exit classes, admission distinction, and report emission.

## Specification Reference

- [SPEC-069](../../spec/SPEC-069-ALPHA-VISIBLE-TOWER-ALGEBRA-AND-DO-LOWERING.md)
- [SPEC-070](../../spec/SPEC-070-ALPHA-RUNTIME-KERNEL-AND-OS-SURFACE.md)
- [PLAN-118](../PLAN-118-DESIGN-040-041-ALPHA-IMPLEMENTATION-PACKET.md)

## Dependencies

- TASK-920 completion
- TASK-927 RuntimeKernel carrier decision
- TASK-925/TASK-926 completion for final compiled artifact/version verification, unless this task explicitly records an interim source/check-summary substrate.

## Requirements

### Functional Requirements

1. Use TASK-920 exact file/callsite/test bindings before implementation.
2. Add RED tests or evidence first.
3. Implement only the SPEC-069/SPEC-070 slice assigned to this task.
4. Route ordinary-file, source, entry-bootstrap, dry-run, and trace execution branches through the same RuntimeKernel admission/report lifecycle without requiring daemon state.
5. Implement or explicitly defer SPEC-070 `FILE[:WORKFLOW]` selector parsing and workflow definition selection; if deferred, patch SPEC-070/PLAN-118 acceptance wording before completing this task.
6. Prove provider/resource registration is not authority: undeclared or unadmitted capability use must fail before user body execution where admission fails. Registered host providers that are not present in admitted grants must also be unable to execute through fallback dispatch paths such as `act_env.capability_ctx.execute`; either project capability contexts to admitted providers only or reject invocation unless an admitted grant/binding exists, with diagnostics distinguishing admission rejection from body-time authority-boundary failure.
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
- One-shot RuntimeKernel path and compatibility evidence.

## Notes

- File targets to inspect or modify: `crates/ash-cli/src/main.rs`, `crates/ash-engine/src/`, `crates/ash-interp/src/`, `crates/ash-cli/tests/`.
- Keep PLAN-118 decision gates in sync with implementation reality.
- Do not broaden scope beyond SPEC-069/SPEC-070 without a docs patch and review.
