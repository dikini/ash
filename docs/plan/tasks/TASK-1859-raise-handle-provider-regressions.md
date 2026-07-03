# TASK-1859: Add raise/handle/provider regressions

## Description

Add current target regressions for executable raise/handle/provider semantics.

## Requirements

- Cover handled raises returning through resume.
- Cover provider-backed raises returning through the provider handler.
- Cover nested handlers and exact operation identity matching.

## Completion criteria

- [x] Tests pass for handler dispatch.
- [x] Tests pass for provider dispatch.
- [x] Tests prove exact operation identity matching remains required.

## Evidence

- Added `crates/ash-interp/tests/task_1858_1859_handler_provider_semantics.rs` for handled raise/resume, provider-frame dispatch, handler/provider shadowing, and unhandled raise. Existing `task_1616b_cps_ir_correctness_fixes` exact-operation regression remains green.

## Depends on

- TASK-1858.
